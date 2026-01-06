package epub

import (
	"archive/zip"
	"fmt"
	"io"
)

type CoverImage struct {
	Data     []byte
	MimeType string
	Href     string // resolved path inside epub (for debugging)
}

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
