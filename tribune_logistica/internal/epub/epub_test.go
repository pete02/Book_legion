package epub

import (
	"archive/zip"
	"bytes"
	"fmt"
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
		Path:  epubPath,
		Spine: nil,
		Nav: []PrettySpineItem{
			{
				Index:  0,
				Number: 1,
				Title:  "test",
				Href:   "OEBPS/chapter1.xhtml",
			},
		},
	}

	data, err := e.GetChapter(0)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	got := string(data)
	want := "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\t\t<html xmlns=\"http://www.w3.org/1999/xhtml\">\n\t\t\t<head>\n\t\t\t\t<title>Chapter 1</title>\n\t\t\t</head>\n\t\t\t<body>\n\t\t\t\t<p>Chapter 1</p>\n\t\t\t</body>\n\t\t</html>"

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
						Href:   "OEBPS/chapter1.xhtml",
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
				Nav:   nil,
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
						Href:   "chapter.xhtml",
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
						Href:   "OEBPS/missing.xhtml",
					},
				},
			},
			index:     0,
			expectErr: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			data, err := tt.epub.GetChapter(tt.index)

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

func TestEpub_GetCover(t *testing.T) {
	coverData := []byte{0x89, 0x50, 0x4E, 0x47} // PNG signature (dummy)
	otherData := []byte("not the cover")

	// EPUB with OPF metadata declaring cover
	files := map[string]string{
		"OEBPS/cover.png": string(coverData),
		"META-INF/container.xml": `<?xml version="1.0" encoding="UTF-8"?>
			<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
			<rootfiles>
				<rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml" />
			</rootfiles>
			</container>`,
		"OEBPS/content.opf": `<?xml version="1.0" encoding="UTF-8"?>
        <package xmlns="http://www.idpf.org/2007/opf" version="3.0">
          <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
            <meta name="cover" content="cover"/>
          </metadata>
          <manifest>
            <item id="cover" href="cover.png" media-type="image/png"/>
          </manifest>
          <spine></spine>
        </package>`,

		"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
		"OEBPS/chapter2.xhtml": "<html><body>Chapter 2 content</body></html>",
		"OEBPS/other.jpg":      string(otherData),
	}

	path := createTestEpub(t, files)

	epub := Epub{Path: path}

	tests := []struct {
		name     string
		wantData []byte
		wantName string
		wantErr  bool
	}{
		{
			name:     "find cover via metadata",
			wantData: coverData,
			wantName: "image/png",
			wantErr:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gotData, gotName, err := epub.GetCover()

			if (err != nil) != tt.wantErr {
				t.Fatalf("GetCover() error = %v, wantErr %v", err, tt.wantErr)
			}
			if tt.wantErr {
				return
			}
			if !bytes.Equal(gotData, tt.wantData) {
				t.Errorf("data mismatch: got %v, want %v", gotData, tt.wantData)
			}
			if gotName != tt.wantName {
				t.Errorf("name mismatch: got %v, want %v", gotName, tt.wantName)
			}
		})
	}

	// Test missing cover metadata (should error)
	filesNoCover := map[string]string{
		"OEBPS/content.opf": `<?xml version="1.0" encoding="UTF-8"?>
        <package xmlns="http://www.idpf.org/2007/opf" version="3.0">
          <metadata></metadata>
          <manifest></manifest>
          <spine></spine>
        </package>`,

		"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
	}

	noCoverPath := createTestEpub(t, filesNoCover)
	epubNoCover := Epub{Path: noCoverPath}

	t.Run("no cover present", func(t *testing.T) {
		if _, _, err := epubNoCover.GetCover(); err == nil {
			t.Errorf("expected error when cover metadata missing")
		}
	})
}

func TestEpub_GetCSS(t *testing.T) {
	// in-memory EPUB with multiple CSS files
	files := map[string]string{
		"OEBPS/style1.css":     "body { color: red; }",
		"OEBPS/style2.CSS":     "p { margin: 0; }",
		"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
	}

	data := createTestEpub(t, files)

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
				Href:   "OEBPS/chapter1.xhtml",
			},
		},
	}

	t.Run("concatenate all CSS files", func(t *testing.T) {
		got, err := epub.GetCSS()
		if err != nil {
			t.Fatalf("GetCSS() error = %v", err)
		}
		want := "body { color: red; }\n" + "p { margin: 0; }\n"
		if string(got) != want {
			t.Errorf("GetCSS() = %q, want %q", string(got), want)
		}
	})

	t.Run("no CSS files present", func(t *testing.T) {
		filesNoCSS := map[string]string{
			"OEBPS/chapter1.xhtml": "<html><body>Chapter 1 content</body></html>",
		}
		dataNoCSS := createTestEpub(t, filesNoCSS)
		epubNoCSS := Epub{Path: dataNoCSS}

		_, err := epubNoCSS.GetCSS()
		if err == nil {
			t.Errorf("GetCSS() expected error, got nil")
		}
	})
}

func TestExtractChapter_SelfClosingScript(t *testing.T) {
	epubPath := createTestEpub(t, map[string]string{
		"OEBPS/chapter1.xhtml": `<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
	<head>
		<title>Chapter 1</title>
		<script type="text/javascript" src="js/kobo.js"/>
		<style type="text/css" id="kobostylehacks">div#book-inner p { font-size: 1.0em; }</style>
	</head>
	<body>
		<p>Chapter 1 content</p>
	</body>
</html>`,
	})

	e := &Epub{
		Path: epubPath,
		Nav: []PrettySpineItem{
			{
				Index:  0,
				Number: 1,
				Title:  "Chapter 1",
				Href:   "OEBPS/chapter1.xhtml",
			},
		},
	}

	data, err := e.GetChapter(0)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	got := string(data)
	if got == "" {
		t.Fatal("extracted chapter body is empty: self-closing <script/> likely caused HTML parser to consume <body> as script text")
	}

	want := "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n\t<head>\n\t\t<title>Chapter 1</title>\n\t\t<script type=\"text/javascript\" src=\"js/kobo.js\"/>\n\t\t<style type=\"text/css\" id=\"kobostylehacks\">div#book-inner p { font-size: 1.0em; }</style>\n\t</head>\n\t<body>\n\t\t<p>Chapter 1 content</p>\n\t</body>\n</html>"
	if got != want {
		t.Fatalf("content mismatch:\nwant: %q\ngot:  %q", want, got)
	}
}

func TestLoadSpine_SimpleTwoChapters(t *testing.T) {
	files := map[string]string{
		"META-INF/container.xml": `
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>`,

		"OEBPS/content.opf": `
<package version="3.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="chap1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="chap2" href="text/ch2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chap1"/>
    <itemref idref="chap2"/>
  </spine>
</package>`,

		"OEBPS/text/ch1.xhtml": "<html><body>Chapter 1</body></html>",
		"OEBPS/text/ch2.xhtml": "<html><body>Chapter 2</body></html>",
	}

	epubPath := createTestEpub(t, files)
	epub, err := New(epubPath)
	if err != nil {
		t.Fatalf("failed to load epub: %v", err)
	}

	spine, err := epub.LoadSpine()
	if err != nil {
		t.Fatalf("LoadSpine failed: %v", err)
	}
	fmt.Println(spine)
	if len(spine) != 2 {
		t.Fatalf("expected 2 spine items, got %d", len(spine))
	}

	// Verify order, ID, href, number
	expectedIDs := []string{"chap1", "chap2"}

	for i, item := range spine {
		if item.ID != expectedIDs[i] {
			t.Errorf("item %d: expected ID %q, got %q", i, expectedIDs[i], item.ID)
		}
		expectedFiles := []string{"ch1.xhtml", "ch2.xhtml"}
		if filepath.Base(item.Href) != expectedFiles[i] {
			t.Errorf("item %d: unexpected Href %q", i, item.Href)
		}

	}
}

func TestLoadSpine_MissingContainer(t *testing.T) {
	epubPath := createTestEpub(t, map[string]string{})
	epub, err := New(epubPath)
	if err != nil {
		t.Fatal("failed to load epub")
	}

	_, err = epub.LoadSpine()
	if err == nil {
		t.Fatal("expected error due to missing container.xml")
	}
}

func TestLoadSpine_SpineIDNotInManifest(t *testing.T) {
	files := map[string]string{
		"META-INF/container.xml": `
<container version="1.0">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf"/>
  </rootfiles>
</container>`,
		"OEBPS/content.opf": `
<package version="3.0">
  <manifest>
    <item id="chap1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chap2"/>
  </spine>
</package>`,
		"OEBPS/text/ch1.xhtml": "<html></html>",
	}

	epubPath := createTestEpub(t, files)
	epub, err := New(epubPath)
	if err != nil {
		t.Fatal("failed to load epub")
	}
	_, err = epub.LoadSpine()
	if err == nil {
		t.Fatal("expected error due to spine ID not in manifest")
	}
}

func TestLoadSpine_EmptySpine(t *testing.T) {
	files := map[string]string{
		"META-INF/container.xml": `
<container version="1.0">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf"/>
  </rootfiles>
</container>`,
		"OEBPS/content.opf": `
<package version="3.0">
  <manifest>
    <item id="chap1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
  </spine>
</package>`,
		"OEBPS/text/ch1.xhtml": "<html></html>",
	}

	epubPath := createTestEpub(t, files)
	epub, err := New(epubPath)
	if err != nil {
		t.Fatal("failed to load epub")
	}
	_, err = epub.LoadSpine()
	if err == nil {
		t.Fatal("expected error due to empty spine")
	}
}

func TestLoadSpine_NonLinearItemSkipped(t *testing.T) {
	files := map[string]string{
		"META-INF/container.xml": `
<container version="1.0">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf"/>
  </rootfiles>
</container>`,
		"OEBPS/content.opf": `
<package version="3.0">
  <manifest>
    <item id="chap1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="chap2" href="text/ch2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chap1"/>
    <itemref idref="chap2" linear="no"/>
  </spine>
</package>`,
		"OEBPS/text/ch1.xhtml": "<html></html>",
		"OEBPS/text/ch2.xhtml": "<html></html>",
	}

	epubPath := createTestEpub(t, files)
	epub, err := New(epubPath)
	if err != nil {
		t.Fatal("failed to load epub")
	}
	spine, err := epub.LoadSpine()
	if err != nil {
		t.Fatalf("LoadSpine failed: %v", err)
	}

	if len(spine) != 1 {
		t.Fatalf("expected 1 spine item (non-linear skipped), got %d", len(spine))
	}
	if spine[0].ID != "chap1" {
		t.Errorf("expected first item ID 'chap1', got %q", spine[0].ID)
	}
}

func TestLoadSpine_OPFInSubdirectory(t *testing.T) {
	files := map[string]string{
		"META-INF/container.xml": `
<container version="1.0">
  <rootfiles>
    <rootfile full-path="OPS/EPUB/content.opf"/>
  </rootfiles>
</container>`,
		"OPS/EPUB/content.opf": `
<package version="3.0">
  <manifest>
    <item id="c1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="c1"/>
  </spine>
</package>`,
		"OPS/EPUB/text/ch1.xhtml": "<html></html>",
	}

	epubPath := createTestEpub(t, files)
	epub, err := New(epubPath)
	if err != nil {
		t.Fatal("failed to load epub")
	}
	spine, err := epub.LoadSpine()
	if err != nil {
		t.Fatalf("LoadSpine failed: %v", err)
	}

	if len(spine) != 1 {
		t.Fatalf("expected 1 spine item, got %d", len(spine))
	}
	expectedPath := filepath.Join("OPS", "EPUB", "text", "ch1.xhtml")
	if spine[0].Href != expectedPath {
		t.Errorf("expected Href %q, got %q", expectedPath, spine[0].Href)
	}
}

func TestLoadPrettySpine_SimpleChapters(t *testing.T) {
	files := map[string]string{
		"META-INF/container.xml": `
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>`,

		"OEBPS/content.opf": `
<package version="3.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="chap1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="chap2" href="text/ch2.xhtml" media-type="application/xhtml+xml"/>
	<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="chap1"/>
    <itemref idref="chap2"/>
  </spine>
</package>`,

		"OEBPS/text/ch1.xhtml": "<html><body>Chapter 1</body></html>",
		"OEBPS/text/ch2.xhtml": "<html><body>Chapter 2</body></html>",

		// nav.toc in NCX format
		"OEBPS/toc.ncx": `
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="uid"/>
    <meta name="dtb:depth" content="1"/>
  </head>
  <docTitle><text>Test Book</text></docTitle>
  <navMap>
    <navPoint id="np1" playOrder="0">
      <navLabel><text>Chapter One</text></navLabel>
      <content src="text/ch1.xhtml"/>
    </navPoint>
    <navPoint id="np2" playOrder="1">
      <navLabel><text>Chapter Two</text></navLabel>
      <content src="text/ch2.xhtml"/>
    </navPoint>
  </navMap>
</ncx>`}

	epubPath := createTestEpub(t, files)

	// load mechanical spine
	e, err := New(epubPath)
	if err != nil {
		t.Fatalf("failed to create Epub: %v", err)
	}
	e.Nav, err = e.GetToc()
	if err != nil {
		t.Fatalf("failed to get TOC: %v", err)
	}

	// verify pretty spine
	if len(e.Nav) != 2 {
		t.Fatalf("expected 2 pretty spine items, got %d", len(e.Nav))
	}

	expectedTitles := []string{"Chapter One", "Chapter Two"}
	expectedNumbers := []int{1, 2}

	for i, item := range e.Nav {
		if item.Title != expectedTitles[i] {
			t.Errorf("item %d: expected title %q, got %q, Nav: %v", i, expectedTitles[i], item.Title, e.Nav)
		}
		if item.Number != expectedNumbers[i] {
			t.Errorf("item %d: expected number %d, got %d, Nav: %v", i, expectedNumbers[i], item.Number, e.Nav)
		}
	}
}
