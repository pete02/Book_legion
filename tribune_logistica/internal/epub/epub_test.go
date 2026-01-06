package epub

import (
	"archive/zip"
	"os"
	"path/filepath"
	"testing"
)

func createTestEpub(t *testing.T, files map[string]string) string {
	t.Helper()

	dir := t.TempDir()
	epubPath := filepath.Join(dir, "test.epub")

	f, err := os.Create(epubPath)
	if err != nil {
		t.Fatalf("failed to create epub file: %v", err)
	}
	defer f.Close()

	zw := zip.NewWriter(f)

	for name, content := range files {
		w, err := zw.Create(name)
		if err != nil {
			t.Fatalf("failed to create zip entry %s: %v", name, err)
		}
		_, err = w.Write([]byte(content))
		if err != nil {
			t.Fatalf("failed to write zip entry %s: %v", name, err)
		}
	}

	if err := zw.Close(); err != nil {
		t.Fatalf("failed to close zip writer: %v", err)
	}

	return epubPath
}

func TestExtractChapter_HappyPath(t *testing.T) {
	epubPath := createTestEpub(t, map[string]string{
		"OEBPS/chapter1.xhtml": "<html>Chapter 1</html>",
	})

	e := &Epub{
		Path: epubPath,
		Spine: []SpineItem{
			{
				Index: 0,
				ID:    "chap1",
				Href:  "OEBPS/chapter1.xhtml",
			},
		},
	}

	data, err := e.ExtractChapter(0)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	got := string(data)
	want := "<html>Chapter 1</html>"

	if got != want {
		t.Fatalf("content mismatch:\nwant: %q\ngot:  %q", want, got)
	}
}

func TestExtractChapter_UnhappyPaths(t *testing.T) {
	validEpubPath := createTestEpub(t, map[string]string{
		"OEBPS/chapter1.xhtml": "ok",
	})

	tests := []struct {
		name      string
		epub      *Epub
		index     int
		expectErr bool
	}{
		{
			name: "negative index",
			epub: &Epub{
				Path: validEpubPath,
				Spine: []SpineItem{
					{Index: 0, Href: "OEBPS/chapter1.xhtml"},
				},
			},
			index:     -1,
			expectErr: true,
		},
		{
			name: "index out of range",
			epub: &Epub{
				Path: validEpubPath,
				Spine: []SpineItem{
					{Index: 0, Href: "OEBPS/chapter1.xhtml"},
				},
			},
			index:     1,
			expectErr: true,
		},
		{
			name: "empty spine",
			epub: &Epub{
				Path:  validEpubPath,
				Spine: nil,
			},
			index:     0,
			expectErr: true,
		},
		{
			name: "epub file does not exist",
			epub: &Epub{
				Path: "/does/not/exist.epub",
				Spine: []SpineItem{
					{Index: 0, Href: "chapter.xhtml"},
				},
			},
			index:     0,
			expectErr: true,
		},
		{
			name: "href not found in zip",
			epub: &Epub{
				Path: validEpubPath,
				Spine: []SpineItem{
					{Index: 0, Href: "OEBPS/missing.xhtml"},
				},
			},
			index:     0,
			expectErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			data, err := tt.epub.ExtractChapter(tt.index)

			if tt.expectErr {
				if err == nil {
					t.Fatalf("expected error, got nil (data=%q)", string(data))
				}
				return
			}

			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
		})
	}
}
