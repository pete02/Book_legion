package epub

import (
	"archive/zip"
	"bytes"
	"errors"
	"fmt"
	"io"
	"path"
	"regexp"
	"strings"
	"unicode"

	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/storage"
	"github.com/book_legion-tribune_logistica/internal/types"
	"golang.org/x/net/html"
)

type Epub struct {
	Path           string
	Spine          []SpineItem
	Nav            []PrettySpineItem
	extractChapter func(navIndex int) ([]byte, error)
	maxChunkMap    func(policy ChunkPolicy) map[int]int
}

func New(path string) (Epub, error) {
	spine, err := LoadSpine(path)
	if err != nil {
		return Epub{}, err
	}
	epub := Epub{
		Path:  path,
		Spine: spine,
	}
	nav, err := epub.LoadPrettySpine()
	if err != nil {
		return epub, err
	}

	epub.Nav = nav

	return epub, nil
}

func Load(db storage.Storage, bookID string) (Epub, error) {
	book, err := library.LoadBook(db, bookID)
	if err != nil {
		return Epub{}, err
	}
	return New(book.FilePath)
}

func (e *Epub) MaxChunkIndex(navIndex int, policy ChunkPolicy) (int, error) {
	chapterBytes, err := e.ExtractChapter(navIndex)
	if err != nil {
		return 0, err
	}

	doc, err := html.Parse(bytes.NewReader(chapterBytes))
	if err != nil {
		return 0, fmt.Errorf("failed to parse chapter HTML: %w", err)
	}

	linear := LinearizeChapter(doc)
	chunks := TextChunk(linear, policy)

	if len(chunks) == 0 {
		return 0, fmt.Errorf("no chunks generated for chapter %d", navIndex)
	}

	return chunks[len(chunks)-1].Index, nil
}

func (e *Epub) MaxChunkMap(policy ChunkPolicy) map[int]int {
	if e.maxChunkMap != nil {
		return e.maxChunkMap(policy)
	} else {
		return e.realMaxChunkMap(policy)
	}
}

func (e *Epub) realMaxChunkMap(policy ChunkPolicy) map[int]int {
	chunkmap := map[int]int{}
	for index := range e.Nav {
		i, err := e.MaxChunkIndex(index, policy)

		if err == nil {
			chunkmap[index] = i
		}
	}

	return chunkmap
}

func (e *Epub) BookProgress(u types.UserCursor, policy ChunkPolicy) (float32, error) {
	chunkMap := e.MaxChunkMap(policy)
	if len(chunkMap) == 0 {
		return 0.0, nil
	}

	totalChunks := 0
	for _, maxChunk := range chunkMap {
		totalChunks += maxChunk
	}

	if totalChunks == 0 {
		return 0.0, nil
	}

	completedChunks := 0
	for i := 0; i < u.Cursor.Chapter; i++ {
		if max, ok := chunkMap[i]; ok {
			completedChunks += max
		}
	}
	completedChunks += u.Cursor.Chunk

	progress := float32(completedChunks) / float32(totalChunks)
	if progress > 1.0 {
		return 1.0, nil
	}

	return progress, nil
}

func (e *Epub) ChapterProgress(u types.UserCursor, policy ChunkPolicy) (float32, error) {
	max, err := e.MaxChunkIndex(u.Cursor.Chapter, policy)
	if err != nil {
		return 0.0, err
	}

	if max == 0 {
		return 0.0, nil
	}

	progress := float32(u.Cursor.Chunk) / float32(max)
	if progress > 1.0 {
		return 1.0, nil
	}

	return progress, nil
}

func (e *Epub) ExtractChapter(navIndex int) ([]byte, error) {
	if e.extractChapter != nil {
		return e.extractChapter(navIndex)
	}
	return e.extractChapterFromFile(navIndex)
}

func (e *Epub) extractChapterFromFile(navIndex int) ([]byte, error) {
	if e.Nav == nil {
		return nil, fmt.Errorf("Nav cannot be Nil")
	}

	if navIndex < 0 || navIndex >= len(e.Nav) {
		return nil, fmt.Errorf("Nav index %d out of range", navIndex)
	}

	nav := e.Nav[navIndex]

	zr, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, err
	}
	defer zr.Close()

	for _, f := range zr.File {
		if path.Clean(f.Name) != path.Clean(nav.Href) {
			continue
		}

		rc, err := f.Open()
		if err != nil {
			return nil, err
		}
		defer rc.Close()

		data, err := io.ReadAll(rc)
		if err != nil {
			return nil, err
		}

		doc, err := html.Parse(bytes.NewReader(data))
		if err != nil {
			return nil, fmt.Errorf("failed to parse chapter HTML: %w", err)
		}

		body := findBodyNode(doc)
		if body == nil {
			return nil, fmt.Errorf("no <body> found in chapter %s", nav.Href)
		}

		removeWhitespaceTextNodes(body)

		var buf bytes.Buffer
		for c := body.FirstChild; c != nil; c = c.NextSibling {
			if err := html.Render(&buf, c); err != nil {
				return nil, err
			}
		}

		return buf.Bytes(), nil
	}

	return nil, fmt.Errorf("extract chapter error: chapter href not found in epub: %s\n", nav.Href)
}

func (e *Epub) ExtractChunk(navIndex, chunkIndex int, policy ChunkPolicy) (string, error) {
	chapterBytes, err := e.ExtractChapter(navIndex)
	if err != nil {
		return "", err
	}

	doc, err := html.Parse(bytes.NewReader(chapterBytes))
	if err != nil {
		return "", err
	}
	linear := LinearizeChapter(doc)
	chunks := TextChunk(linear, policy)

	if chunkIndex < 0 || chunkIndex >= len(chunks) {
		return "", fmt.Errorf("chunk index %d out of range (0-%d)", chunkIndex, len(chunks)-1)
	}

	chunkStrings := PrettyChunks(chunks, linear)

	return chunkStrings[chunkIndex], nil
}

func (e *Epub) CalculateCursorPlace(navIndex int, example string, policy ChunkPolicy) (types.Cursor, error) {
	// --- 1. Extract chapter and linearize ---
	chapterBytes, err := e.ExtractChapter(navIndex)
	if err != nil {
		return types.Cursor{}, err
	}

	doc, err := html.Parse(bytes.NewReader(chapterBytes))
	if err != nil {
		return types.Cursor{}, err
	}
	linear := LinearizeChapter(doc)

	// --- 2. Linearize example text ---
	docExample, err := html.Parse(bytes.NewReader([]byte(example)))
	if err != nil {
		return types.Cursor{}, err
	}
	linearExample := LinearizeChapter(docExample)
	linearExample.FullText = trimTrailingPunctuation(linearExample.FullText)

	if len(linearExample.FullText) < policy.MinSnippetSize {
		return types.Cursor{}, errors.New("snippet too short to uniquely locate cursor")
	}

	// --- 3. Normalize both texts and build mapping ---
	normalizedChapter, chapterMap := buildNormalizedMapping(linear.FullText)
	normalizedExample, _ := buildNormalizedMapping(linearExample.FullText) // mapping not needed for example

	// --- 4. Find normalized offset ---
	normOffset := strings.Index(normalizedChapter, normalizedExample)
	if normOffset == -1 {
		return types.Cursor{}, errors.New("example text not found in chapter")
	}

	// --- 5. Map normalized offset back to original text ---
	if normOffset >= len(chapterMap) {
		return types.Cursor{}, errors.New("offset mapping failed")
	}
	origOffset := chapterMap[normOffset]

	// --- 6. Find target chunk ---
	chunks := TextChunk(linear, policy)
	var targetChunk Chunk
	for _, c := range chunks {
		if origOffset >= c.Start && origOffset < c.End {
			targetChunk = c
			break
		}
	}

	return types.Cursor{Chapter: navIndex, Chunk: targetChunk.Index}, nil
}

func (e *Epub) ExtractCover() ([]byte, string, error) {
	zr, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, "", err
	}
	defer zr.Close()

	files := BuildFileMap(zr)

	opfPath, err := FindOPFPath(zr)
	if err != nil {
		return nil, "", err
	}

	pkg, err := ParseOPF(files, opfPath)
	if err != nil {
		return nil, "", err
	}

	baseDir := path.Dir(opfPath)

	loadImage := func(href string) ([]byte, string, error) {
		href = strings.SplitN(href, "#", 2)[0]
		coverPath := path.Join(baseDir, href)
		data, err := ReadFromMap(files, coverPath)
		if err != nil {
			return nil, "", err
		}
		return data, coverPath, nil
	}

	isImageType := func(mediaType string) bool {
		return strings.HasPrefix(mediaType, "image/")
	}

	// Strategy 1: EPUB3 properties="cover-image"
	for _, item := range pkg.Manifest.Items {
		if strings.Contains(item.Properties, "cover-image") && isImageType(item.MediaType) {
			if data, p, err := loadImage(item.Href); err == nil {
				return data, p, nil
			}
		}
	}

	// Strategy 2: EPUB2 <meta name="cover" content="item-id">
	for _, meta := range pkg.Metadata.Metas {
		if strings.EqualFold(meta.Name, "cover") && meta.Content != "" {
			for _, item := range pkg.Manifest.Items {
				if item.ID == meta.Content {
					if isImageType(item.MediaType) {
						if data, p, err := loadImage(item.Href); err == nil {
							return data, p, nil
						}
					} else if strings.Contains(item.MediaType, "html") {
						if data, p, err := extractImageFromHTML(files, path.Join(baseDir, item.Href)); err == nil {
							return data, p, nil
						}
					}
				}
			}
		}
	}

	// Strategy 3: <guide type="cover">
	for _, ref := range pkg.Guide.References {
		if strings.EqualFold(ref.Type, "cover") {
			if data, p, err := extractImageFromHTML(files, path.Join(baseDir, ref.Href)); err == nil {
				return data, p, nil
			}
		}
	}

	// Strategy 4: Heuristic — href contains "cover"
	for _, item := range pkg.Manifest.Items {
		if isImageType(item.MediaType) && strings.Contains(strings.ToLower(item.Href), "cover") {
			if data, p, err := loadImage(item.Href); err == nil {
				return data, p, nil
			}
		}
	}

	return nil, "", fmt.Errorf("cover image not found")
}

func extractImageFromHTML(files map[string]*zip.File, htmlPath string) ([]byte, string, error) {
	htmlPath = strings.SplitN(htmlPath, "#", 2)[0]
	htmlData, err := ReadFromMap(files, htmlPath)
	if err != nil {
		return nil, "", err
	}

	re := regexp.MustCompile(`(?i)<img[^>]+src=["']([^"']+)["']`)
	matches := re.FindSubmatch(htmlData)
	if matches == nil {
		return nil, "", fmt.Errorf("no img found in %s", htmlPath)
	}

	imgPath := path.Join(path.Dir(htmlPath), string(matches[1]))
	data, err := ReadFromMap(files, imgPath)
	if err != nil {
		return nil, "", err
	}
	return data, imgPath, nil
}

// readZipFile reads a file from the zip by exact path.
func readZipFile(zr *zip.ReadCloser, name string) ([]byte, error) {
	for _, f := range zr.File {
		if f.Name == name {
			rc, err := f.Open()
			if err != nil {
				return nil, err
			}
			defer rc.Close()
			return io.ReadAll(rc)
		}
	}
	return nil, fmt.Errorf("file not found in epub: %s", name)
}

func (e *Epub) ExtractCSS() ([]byte, error) {
	zr, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, err
	}
	defer zr.Close()

	var allCSS bytes.Buffer

	for _, f := range zr.File {
		if strings.HasSuffix(strings.ToLower(f.Name), ".css") {
			rc, err := f.Open()
			if err != nil {
				return nil, fmt.Errorf("failed to open %s: %w", f.Name, err)
			}

			data, err := io.ReadAll(rc)
			rc.Close()
			if err != nil {
				return nil, fmt.Errorf("failed to read %s: %w", f.Name, err)
			}

			// Append with newline separator to avoid accidental concatenation issues
			allCSS.Write(data)
			allCSS.WriteString("\n")
		}
	}

	if allCSS.Len() == 0 {
		return nil, fmt.Errorf("no CSS files found in EPUB")
	}

	return allCSS.Bytes(), nil
}

func findBodyNode(n *html.Node) *html.Node {
	if n.Type == html.ElementNode && n.Data == "body" {
		return n
	}
	for c := n.FirstChild; c != nil; c = c.NextSibling {
		if found := findBodyNode(c); found != nil {
			return found
		}
	}
	return nil
}

func removeWhitespaceTextNodes(n *html.Node) {
	for c := n.FirstChild; c != nil; {
		next := c.NextSibling

		if c.Type == html.TextNode && strings.TrimSpace(c.Data) == "" {
			n.RemoveChild(c)
		} else {
			removeWhitespaceTextNodes(c)
		}

		c = next
	}
}

func trimTrailingPunctuation(s string) string {
	return strings.TrimRightFunc(s, func(r rune) bool {
		return r == ' ' || r == '.' || r == '!' || r == '?'
	})
}

type NormalizedIndex struct {
	NormPos int // position in normalized string
	OrigPos int // position in original string
}

var htmlEntityRe = regexp.MustCompile(`&#\d+;|&[a-zA-Z]+;`)

func normalizeEntities(s string) string {
	return htmlEntityRe.ReplaceAllStringFunc(s, func(entity string) string {
		switch entity {
		case "&#39;", "&apos;":
			return "'"
		case "&quot;":
			return `"`
		case "&amp;":
			return "&"
		default:
			return "" // remove unknown entities
		}
	})
}

func buildNormalizedMapping(orig string) (string, []int) {
	orig = normalizeEntities(orig)
	var norm strings.Builder
	var mapping []int

	for i, r := range orig {
		if r >= 'a' && r <= 'z' || r >= 'A' && r <= 'Z' || r >= '0' && r <= '9' {
			norm.WriteRune(unicode.ToLower(r))
			mapping = append(mapping, i)
		}
		// optionally include space if you want normalized spacing
	}
	return norm.String(), mapping
}
