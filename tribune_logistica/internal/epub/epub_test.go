package epub

import (
	"archive/zip"
	"bytes"
	"os"
	"path/filepath"
	"testing"

	"golang.org/x/net/html"
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

func TestEpub_ExtractChunk(t *testing.T) {

	// in-memory EPUB contents
	files := map[string]string{
		"chapter1.xhtml": "<html><body>Hello world. This is chapter 1.</body></html>",
		"chapter2.xhtml": "<html><body>Second chapter content here!</body></html>",
	}

	path := createTestEpub(t, files)

	epub := Epub{
		Path: path,
		Spine: []SpineItem{
			{Index: 0, ID: "c1", Href: "chapter1.xhtml"},
			{Index: 1, ID: "c2", Href: "chapter2.xhtml"},
		},
	}

	policy := ChunkPolicy{TargetSize: 50, MaxSize: 60}

	tests := []struct {
		name       string
		spineIndex int
		chunkIndex int
		want       string
		wantErr    bool
	}{
		{
			name:       "first chunk of chapter 1",
			spineIndex: 0,
			chunkIndex: 0,
			want:       "Hello world. This is chapter 1.",
			wantErr:    false,
		},
		{
			name:       "first chunk of chapter 2",
			spineIndex: 1,
			chunkIndex: 0,
			want:       "Second chapter content here!",
			wantErr:    false,
		},
		{
			name:       "spine index out of range",
			spineIndex: 2,
			chunkIndex: 0,
			want:       "",
			wantErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := epub.ExtractChunk(tt.spineIndex, tt.chunkIndex, policy)
			if (err != nil) != tt.wantErr {
				t.Fatalf("ExtractChunk() error = %v, wantErr %v", err, tt.wantErr)
			}
			if got != tt.want {
				t.Errorf("ExtractChunk() = %q, want %q", got, tt.want)
			}
		})
	}
}

func TestEpub_ExtractCover(t *testing.T) {
	// dummy image bytes
	coverData := []byte{0x89, 0x50, 0x4E, 0x47} // just the PNG signature
	otherData := []byte("not the cover")

	// in-memory EPUB with a cover image
	files := map[string]string{
		"OEBPS/cover.png":      string(coverData),
		"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
		"OEBPS/chapter2.xhtml": "<html><body>Chapter 2 content</body></html>",
		"OEBPS/other.jpg":      string(otherData),
	}

	path := createTestEpub(t, files)

	epub := Epub{
		Path: path,
		Spine: []SpineItem{
			{Index: 0, ID: "c1", Href: "OEBPS/chapter1.xhtml"},
			{Index: 1, ID: "c2", Href: "OEBPS/chapter2.xhtml"},
		},
	}

	tests := []struct {
		name     string
		wantData []byte
		wantName string
		wantErr  bool
	}{
		{
			name:     "find PNG cover",
			wantData: coverData,
			wantName: "OEBPS/cover.png",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gotData, gotName, err := epub.ExtractCover()
			if (err != nil) != tt.wantErr {
				t.Fatalf("ExtractCover() error = %v, wantErr %v", err, tt.wantErr)
			}
			if !bytes.Equal(gotData, tt.wantData) {
				t.Errorf("ExtractCover() data mismatch, got %v, want %v", gotData, tt.wantData)
			}
			if gotName != tt.wantName {
				t.Errorf("ExtractCover() name = %v, want %v", gotName, tt.wantName)
			}
		})
	}

	// Test missing cover
	filesNoCover := map[string]string{
		"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
	}
	noCoverPath := createTestEpub(t, filesNoCover)

	epubNoCover := Epub{
		Path: noCoverPath,
	}
	t.Run("no cover present", func(t *testing.T) {
		_, _, err := epubNoCover.ExtractCover()
		if err == nil {
			t.Errorf("ExtractCover() expected error, got nil")
		}
	})
}

func TestEpub_ExtractCSS(t *testing.T) {
	// in-memory EPUB with multiple CSS files
	files := map[string]string{
		"OEBPS/style1.css":     "body { color: red; }",
		"OEBPS/style2.CSS":     "p { margin: 0; }",
		"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
	}

	data := createMinimalEPUB(t, files)

	epub := Epub{
		Path: data,
		Spine: []SpineItem{
			{Index: 0, ID: "c1", Href: "OEBPS/chapter1.xhtml"},
		},
	}

	t.Run("concatenate all CSS files", func(t *testing.T) {
		got, err := epub.ExtractCSS()
		if err != nil {
			t.Fatalf("ExtractCSS() error = %v", err)
		}
		want := "body { color: red; }\n" + "p { margin: 0; }\n"
		if string(got) != want {
			t.Errorf("ExtractCSS() = %q, want %q", string(got), want)
		}
	})

	t.Run("no CSS files present", func(t *testing.T) {
		filesNoCSS := map[string]string{
			"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
		}
		dataNoCSS := createMinimalEPUB(t, filesNoCSS)
		epubNoCSS := Epub{Path: dataNoCSS}

		_, err := epubNoCSS.ExtractCSS()
		if err == nil {
			t.Errorf("ExtractCSS() expected error, got nil")
		}
	})
}

func TestEpub_MaxChunkIndex(t *testing.T) {
	files := map[string]string{
		"chapter1.xhtml": "<html><body>Hello world. This is a test chapter. It has multiple sentences.</body></html>",
	}
	data := createMinimalEPUB(t, files)

	epub := Epub{
		Path: data,
		Spine: []SpineItem{
			{Index: 0, ID: "c1", Href: "chapter1.xhtml"},
		},
	}

	policy := ChunkPolicy{TargetSize: 20, MaxSize: 25}

	t.Run("last chunk index is correct", func(t *testing.T) {
		maxIdx, err := epub.MaxChunkIndex(0, policy)
		if err != nil {
			t.Fatalf("MaxChunkIndex() error = %v", err)
		}

		chapterBytes, _ := epub.ExtractChapter(0)
		doc, _ := html.Parse(bytes.NewReader(chapterBytes))
		linear := LinearizeChapter(doc)
		chunks := ChunkText(linear, policy)

		if maxIdx != chunks[len(chunks)-1].Index {
			t.Errorf("MaxChunkIndex() = %d, want last chunk index %d", maxIdx, chunks[len(chunks)-1].Index)
		}
	})
}
