package epub

import (
	"archive/zip"
	"bytes"
	"fmt"
	"io"

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

func (e *Epub) ExtractChapter(spineIndex int) ([]byte, error) {
	if spineIndex < 0 || spineIndex >= len(e.Spine) {
		return nil, fmt.Errorf("spine index %d out of range", spineIndex)
	}

	item := e.Spine[spineIndex]

	zr, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, err
	}
	defer zr.Close()

	for _, f := range zr.File {
		if f.Name == item.Href {
			rc, err := f.Open()
			if err != nil {
				return nil, err
			}
			defer rc.Close()

			return io.ReadAll(rc)
		}
	}

	return nil, fmt.Errorf("chapter href not found in epub: %s", item.Href)
}

func (e *Epub) ExtractChunk(spineIndex, chunkIndex int, policy ChunkPolicy) (string, error) {
	// 1. Get raw chapter bytes
	chapterBytes, err := e.ExtractChapter(spineIndex)
	if err != nil {
		return "", err
	}

	// 2. Parse HTML
	doc, err := html.Parse(bytes.NewReader(chapterBytes))
	if err != nil {
		return "", err
	}

	// 3. Linearize chapter (body content only)
	linear := LinearizeChapter(doc)

	// 4. Generate chunks
	chunks := ChunkText(linear, policy)

	// 5. Check chunkIndex bounds
	if chunkIndex < 0 || chunkIndex >= len(chunks) {
		return "", fmt.Errorf("chunk index %d out of range (0-%d)", chunkIndex, len(chunks)-1)
	}

	chunkStrings := PrettyChunks(chunks, linear)

	return chunkStrings[chunkIndex], nil
}
