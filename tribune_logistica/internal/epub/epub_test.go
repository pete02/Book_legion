package epub

import (
	"archive/zip"
	"bytes"
	"errors"
	"os"
	"path/filepath"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/types"
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
		"OEBPS/chapter1.xhtml": `
		<?xml version="1.0" encoding="utf-8"?>
		<html xmlns="http://www.w3.org/1999/xhtml">
			<head>
				<title>Chapter 1</title>
			</head>
			<body>
				<p>Chapter 1</p>
			</body>
		</html>
	`,
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
		Nav: []PrettySpineItem{
			{
				Index:  0,
				Number: 1,
				Title:  "test",
			},
		},
	}

	data, err := e.ExtractChapter(0)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	got := string(data)
	want := "<p>Chapter 1</p>"

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
				Nav: []PrettySpineItem{
					{
						Index:  0,
						Number: 1,
						Title:  "test",
					},
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
				Nav: []PrettySpineItem{
					{
						Index:  0,
						Number: 1,
						Title:  "test",
					},
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
				Nav: []PrettySpineItem{
					{
						Index:  0,
						Number: 1,
						Title:  "test",
					},
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
		Nav: []PrettySpineItem{
			{
				Index:  0,
				Number: 1,
				Title:  "test",
			}, {
				Index:  1,
				Number: 2,
				Title:  "test2",
			},
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
		Nav: []PrettySpineItem{
			{
				Index:  0,
				Number: 1,
				Title:  "test",
			}, {
				Index:  1,
				Number: 2,
				Title:  "test2",
			},
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
		Nav: []PrettySpineItem{
			{
				Index:  0,
				Number: 1,
				Title:  "test",
			},
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
		Nav: []PrettySpineItem{
			{
				Index:  0,
				Number: 1,
				Title:  "test",
			},
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
		chunks := TextChunk(linear, policy)

		if maxIdx != chunks[len(chunks)-1].Index {
			t.Errorf("MaxChunkIndex() = %d, want last chunk index %d", maxIdx, chunks[len(chunks)-1].Index)
		}
	})
}

func make_cursor(chapter int, chunk int) types.UserCursor {
	return types.NewUserCursor("u1", "b1", chapter, chunk)
}

func TestChapterProgress(t *testing.T) {
	policy := ChunkPolicy{TargetSize: 12, MaxSize: 20}

	// HTML wrapper is important because MaxChunkIndex parses HTML
	chapterHTML := []byte(`
		<html>
			<body>
				<p>Hello world. This is a test. Split nicely.</p>
			</body>
		</html>
	`)

	epub := &Epub{
		Nav: []PrettySpineItem{{}},
		extractChapter: func(navIndex int) ([]byte, error) {
			return chapterHTML, nil
		},
	}

	tests := []struct {
		name    string
		cursor  types.UserCursor
		want    float32
		wantErr bool
	}{
		{
			name:    "chunk 0 of 2",
			cursor:  make_cursor(0, 0),
			want:    0.0,
			wantErr: false,
		},
		{
			name:    "chunk 1 of 2",
			cursor:  make_cursor(0, 1),
			want:    0.5,
			wantErr: false,
		},
		{
			name:    "chunk 2 of 2",
			cursor:  make_cursor(0, 2),
			want:    1.0,
			wantErr: false,
		},
		{
			name:    "Too large",
			cursor:  make_cursor(0, 3),
			want:    1.0,
			wantErr: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := epub.ChapterProgress(tt.cursor, policy)
			if tt.wantErr {
				if err == nil {
					t.Fatalf("expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}

			if got != tt.want {
				t.Fatalf("got %f, want %f", got, tt.want)
			}
		})
	}
}

func TestBookProgress(t *testing.T) {
	policy := ChunkPolicy{TargetSize: 12, MaxSize: 20}

	epub := &Epub{
		Nav: []PrettySpineItem{{}, {}, {}}, // 3 chapters
	}

	epub.maxChunkMap = func(policy ChunkPolicy) map[int]int {
		return map[int]int{
			0: 2, // chapter 0 has 3 chunks (0,1,2)
			1: 1, // chapter 1 has 2 chunks (0,1)
			2: 3, // chapter 2 has 4 chunks (0,1,2,3)
		}
	}

	tests := []struct {
		name   string
		cursor types.UserCursor
		want   float32
	}{
		{
			name: "start of book",
			cursor: types.UserCursor{
				Cursor: types.Cursor{Chapter: 0, Chunk: 0},
			},
			want: 0.0,
		},
		{
			name: "middle of first chapter",
			cursor: types.UserCursor{
				Cursor: types.Cursor{Chapter: 0, Chunk: 1},
			},
			want: float32(1) / float32(2+1+3),
		},
		{
			name: "end of first chapter",
			cursor: types.UserCursor{
				Cursor: types.Cursor{Chapter: 0, Chunk: 2},
			},
			want: float32(2) / float32(2+1+3),
		},
		{
			name: "start of second chapter",
			cursor: types.UserCursor{
				Cursor: types.Cursor{Chapter: 1, Chunk: 0},
			},
			want: float32(2) / float32(2+1+3),
		},
		{
			name: "middle of second chapter",
			cursor: types.UserCursor{
				Cursor: types.Cursor{Chapter: 1, Chunk: 1},
			},
			want: float32(2+1) / float32(2+1+3),
		},
		{
			name: "last chunk of last chapter",
			cursor: types.UserCursor{
				Cursor: types.Cursor{Chapter: 2, Chunk: 4},
			},
			want: 1.0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := epub.BookProgress(tt.cursor, policy)
			if err != nil {
				t.Fatalf("unexpected error: %v", err)
			}
			if got != tt.want {
				t.Fatalf("got %f, want %f", got, tt.want)
			}
		})
	}
}

func TestCalculateCursorPlace_Success(t *testing.T) {
	e := &Epub{}

	// Mock chapter content
	e.extractChapter = func(navIndex int) ([]byte, error) {
		return []byte(`
			<div>
				<p>Chapter 1</p>
				<p>Tala looked around the room, ignoring the man.</p>
				<p>A waist-high stone wall stood in a circle halfway between her and the smooth granite of the outer walls.</p>
			</div>
		`), nil
	}

	policy := ChunkPolicy{
		TargetSize:     80,
		MinSize:        40,
		MaxSize:        120,
		MinSnippetSize: 30,
	}

	exampleHTML := `<p>Tala looked around the room, ignoring the man.</p>`

	cursor, err := e.CalculateCursorPlace(0, exampleHTML, policy)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}

	if cursor.Chapter != 0 {
		t.Errorf("expected chapter 0, got %d", cursor.Chapter)
	}

	if cursor.Chunk < 0 {
		t.Errorf("expected valid chunk index, got %d", cursor.Chunk)
	}
}

func TestCalculateCursorPlace_SnippetTooShort(t *testing.T) {
	e := &Epub{}

	e.extractChapter = func(navIndex int) ([]byte, error) {
		return []byte(`<p>This is a chapter</p>`), nil
	}

	policy := ChunkPolicy{
		TargetSize:     80,
		MinSize:        40,
		MaxSize:        120,
		MinSnippetSize: 20,
	}

	exampleHTML := `<p>Hi</p>`

	_, err := e.CalculateCursorPlace(0, exampleHTML, policy)
	if err == nil {
		t.Fatal("expected error, got nil")
	}

	if !errors.Is(err, errors.New("snippet too short to uniquely locate cursor")) &&
		err.Error() != "snippet too short to uniquely locate cursor" {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestCalculateCursorPlace_SnippetNotFound(t *testing.T) {
	e := &Epub{}

	e.extractChapter = func(navIndex int) ([]byte, error) {
		return []byte(`
			<p>This is the first paragraph.</p>
			<p>This is the second paragraph.</p>
		`), nil
	}

	policy := ChunkPolicy{
		TargetSize:     80,
		MinSize:        40,
		MaxSize:        120,
		MinSnippetSize: 20,
	}

	exampleHTML := `<p>This text does not exist in the chapter.</p>`

	_, err := e.CalculateCursorPlace(0, exampleHTML, policy)
	if err == nil {
		t.Fatal("expected error, got nil")
	}

	if err.Error() != "example text not found in chapter" {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestCalculateCursorPlace_ChunkBoundary(t *testing.T) {
	e := &Epub{}

	e.extractChapter = func(navIndex int) ([]byte, error) {
		return []byte(`
			<p>This is sentence one. This is sentence two.</p>
			<p>This is sentence three. This is sentence four.</p>
		`), nil
	}

	policy := ChunkPolicy{
		TargetSize:     30, // force multiple chunks
		MinSize:        20,
		MaxSize:        40,
		MinSnippetSize: 22,
	}

	exampleHTML := `<p>This is sentence three.</p>`

	cursor, err := e.CalculateCursorPlace(0, exampleHTML, policy)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}

	if cursor.Chunk == 0 {
		t.Errorf("expected cursor to land in later chunk, got chunk %d", cursor.Chunk)
	}
}
