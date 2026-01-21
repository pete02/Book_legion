package epub

import (
	"archive/zip"
	"bytes"
	"errors"
	"fmt"
	"io"
	"strings"

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
	if navIndex < 0 || navIndex >= len(e.Spine) {
		return nil, fmt.Errorf("spine index %d out of range", navIndex)
	}

	nav := e.Nav[navIndex]
	item := e.Spine[nav.Index]

	zr, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, err
	}
	defer zr.Close()

	for _, f := range zr.File {
		if f.Name != item.Href {
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
			return nil, fmt.Errorf("no <body> found in chapter %s", item.Href)
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

	return nil, fmt.Errorf("chapter href not found in epub: %s", item.Href)
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

func (e *Epub) CalculateCursorPlace(navIndex int, examle string, policy ChunkPolicy) (types.Cursor, error) {
	chapterBytes, err := e.ExtractChapter(navIndex)
	if err != nil {
		return types.Cursor{}, err
	}

	doc, err := html.Parse(bytes.NewReader(chapterBytes))
	if err != nil {
		return types.Cursor{}, err
	}
	linear := LinearizeChapter(doc)

	doc, err = html.Parse(bytes.NewReader([]byte(examle)))
	if err != nil {
		return types.Cursor{}, err
	}
	linearExample := LinearizeChapter(doc)
	linearExample.FullText = trimTrailingPunctuation(linearExample.FullText)

	if len(linearExample.FullText) < policy.MinSnippetSize {
		return types.Cursor{}, errors.New("snippet too short to uniquely locate cursor")
	}

	offset := strings.Index(linear.FullText, linearExample.FullText)
	if offset == -1 {
		return types.Cursor{}, errors.New("example text not found in chapter")
	}
	chunks := TextChunk(linear, policy)
	var targetChunk Chunk
	for _, c := range chunks {
		if offset >= c.Start && offset < c.End {
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

	for _, f := range zr.File {
		lower := strings.ToLower(f.Name)
		if strings.Contains(lower, "cover") &&
			(strings.HasSuffix(lower, ".jpg") || strings.HasSuffix(lower, ".jpeg") ||
				strings.HasSuffix(lower, ".png") || strings.HasSuffix(lower, ".gif")) {
			rc, err := f.Open()
			if err != nil {
				return nil, "", err
			}
			defer rc.Close()
			data, err := io.ReadAll(rc)
			if err != nil {
				return nil, "", err
			}
			return data, f.Name, nil
		}
	}

	return nil, "", fmt.Errorf("cover image not found")
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
