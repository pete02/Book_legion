package epub

import (
	"archive/zip"
	"bytes"
	"fmt"
	"io"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/storage"
	"golang.org/x/net/html"
)

type Epub struct {
	Path  string
	Spine []SpineItem
	Nav   []PrettySpineItem
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
	chunks := ChunkText(linear, policy)

	if len(chunks) == 0 {
		return 0, fmt.Errorf("no chunks generated for chapter %d", navIndex)
	}

	return chunks[len(chunks)-1].Index, nil
}

func (e *Epub) MaxChunkMap(policy ChunkPolicy) map[int]int {
	chunkmap := map[int]int{}
	for index, _ := range e.Nav {
		i, err := e.MaxChunkIndex(index, policy)

		if err == nil {
			chunkmap[index] = i
		}
	}

	return chunkmap
}

func (e *Epub) ExtractChapter(navIndex int) ([]byte, error) {
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
	chunks := ChunkText(linear, policy)

	if chunkIndex < 0 || chunkIndex >= len(chunks) {
		return "", fmt.Errorf("chunk index %d out of range (0-%d)", chunkIndex, len(chunks)-1)
	}

	chunkStrings := PrettyChunks(chunks, linear)

	return chunkStrings[chunkIndex], nil
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
