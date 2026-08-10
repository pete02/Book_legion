package epub

import (
	"archive/zip"
	"bytes"
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strings"
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

	data, err := e.GetFile("OEBPS/chapter1.xhtml")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	got := string(data)
	want := "\n\t\t<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\t\t<html xmlns=\"http://www.w3.org/1999/xhtml\">\n\t\t\t<head>\n\t\t\t\t<title>Chapter 1</title>\n\t\t\t</head>\n\t\t\t<body>\n\t\t\t\t<p>Chapter 1</p>\n\t\t\t</body>\n\t\t</html>\n\t"

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
		want := "body { color: red; }\np { margin: 0; }\n"
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

	want := "<!--?xml version=\"1.0\" encoding=\"UTF-8\"?--><html xmlns=\"http://www.w3.org/1999/xhtml\"><head>\n\t\t<title>Chapter 1</title>\n\t\t<script type=\"text/javascript\" src=\"js/kobo.js\">\n\t\t<style type=\"text/css\" id=\"kobostylehacks\">div#book-inner p { font-size: 1.0em; }</style>\n\t</head>\n\t<body>\n\t\t<p>Chapter 1 content</p>\n\t</body>\n</html></script></head><body></body></html>"
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
	epub, err := New(epubPath, "test-book-id")
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
	epub, err := New(epubPath, "test-book-id")
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
	epub, err := New(epubPath, "test-book-id")
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
	epub, err := New(epubPath, "test-book-id")
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
	epub, err := New(epubPath, "test-book-id")
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
	epub, err := New(epubPath, "test-book-id")
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
	e, err := New(epubPath, "test-book-id")
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

// findAttr walks parsed HTML and returns the value of attrKey for every
// element matching tag, in document order.
func findAttr(t *testing.T, data []byte, tag, attrKey string) []string {
	t.Helper()

	doc, err := html.Parse(strings.NewReader(string(data)))
	if err != nil {
		t.Fatalf("failed to parse rewritten HTML: %v", err)
	}

	var out []string
	var walk func(*html.Node)
	walk = func(n *html.Node) {
		if n.Type == html.ElementNode && n.Data == tag {
			for _, a := range n.Attr {
				if a.Key == attrKey {
					out = append(out, a.Val)
				}
			}
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			walk(c)
		}
	}
	walk(doc)
	return out
}

// parseResourceURL parses a rewritten resource URL and returns the decoded
// "file" and "token" query params, failing the test if the URL or its
// query string is malformed.
func parseResourceURL(t *testing.T, raw string) (filePath, token string) {
	t.Helper()

	u, err := url.Parse(raw)
	if err != nil {
		t.Fatalf("rewritten URL is not parseable: %v (%q)", err, raw)
	}
	q := u.Query()
	return q.Get("file"), q.Get("token")
}

func TestResolveEpubPath(t *testing.T) {
	tests := []struct {
		name     string
		baseDir  string
		ref      string
		wantPath string
		wantOK   bool
	}{
		{
			name:     "parent-relative resolves up one level",
			baseDir:  "OEBPS/text",
			ref:      "../images/cover.jpg",
			wantPath: "OEBPS/images/cover.jpg",
			wantOK:   true,
		},
		{
			name:     "sibling file in same dir",
			baseDir:  "OEBPS",
			ref:      "page_styles.css",
			wantPath: "OEBPS/page_styles.css",
			wantOK:   true,
		},
		{
			name:     "parent-relative resolves to root",
			baseDir:  "OEBPS",
			ref:      "../page_styles.css",
			wantPath: "page_styles.css",
			wantOK:   true,
		},
		{
			name:     "fragment is stripped before resolving",
			baseDir:  "OEBPS",
			ref:      "style.css#footnote",
			wantPath: "OEBPS/style.css",
			wantOK:   true,
		},
		{
			name:    "traversal above root is rejected",
			baseDir: "OEBPS",
			ref:     "../../secret.txt",
			wantOK:  false,
		},
		{
			name:    "traversal above root from root dir is rejected",
			baseDir: ".",
			ref:     "../secret.txt",
			wantOK:  false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok := resolveEpubPath(tt.baseDir, tt.ref)
			if ok != tt.wantOK {
				t.Fatalf("ok = %v, want %v (resolved: %q)", ok, tt.wantOK, got)
			}
			if ok && got != tt.wantPath {
				t.Fatalf("resolved = %q, want %q", got, tt.wantPath)
			}
		})
	}
}

func TestIsRewritableResource(t *testing.T) {
	tests := []struct {
		name string
		val  string
		want bool
	}{
		{"empty string", "", false},
		{"fragment only", "#anchor", false},
		{"data URI", "data:image/png;base64,AAAA", false},
		{"mailto link", "mailto:test@example.com", false},
		{"absolute external URL", "https://example.com/image.png", false},
		{"relative parent path", "../images/cover.jpg", true},
		{"relative sibling path", "chapter2.xhtml", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := isRewritableResource(tt.val); got != tt.want {
				t.Fatalf("isRewritableResource(%q) = %v, want %v", tt.val, got, tt.want)
			}
		})
	}
}

func TestRewriteResourceLinks(t *testing.T) {
	const bookID = "book-123"

	html := `<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
	<title>Chapter 1</title>
	<link rel="stylesheet" type="text/css" href="../page_styles.css"/>
	<link rel="icon" href="favicon.ico"/>
</head>
<body>
	<p>Some text</p>
	<img src="../images/cover.jpg" alt="cover"/>
	<img src="data:image/png;base64,AAAA" alt="inline"/>
	<img src="https://example.com/remote.png" alt="remote"/>
	<a href="chapter2.xhtml">Next chapter</a>
	<a href="#footnote1">footnote</a>
</body>
</html>`

	out, err := rewriteResourceLinks([]byte(html), "OEBPS/chapter.xhtml", bookID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	t.Run("stylesheet link is rewritten", func(t *testing.T) {
		hrefs := findAttr(t, out, "link", "href")
		if len(hrefs) != 2 {
			t.Fatalf("expected 2 <link> hrefs, got %d: %v", len(hrefs), hrefs)
		}

		// First link (rel=stylesheet) should be rewritten.
		file, token := parseResourceURL(t, hrefs[0])
		if file != "page_styles.css" {
			t.Errorf("file = %q, want %q", file, "page_styles.css")
		}
		if token != chapterTokenPlaceholder {
			t.Errorf("token = %q, want placeholder %q", token, chapterTokenPlaceholder)
		}
		if !strings.HasPrefix(hrefs[0], "/api/v1/books/"+bookID+"/file?token=TOKEN_PLACEHOLDER&file=page_styles.css") {
			t.Errorf("rewritten href doesn't point at expected endpoint: %q", hrefs[0])
		}

		// Second link (rel=icon) should be untouched.
		if hrefs[1] != "favicon.ico" {
			t.Errorf("non-stylesheet link was modified: %q", hrefs[1])
		}
	})

	t.Run("relative image src is rewritten", func(t *testing.T) {
		srcs := findAttr(t, out, "img", "src")
		if len(srcs) != 3 {
			t.Fatalf("expected 3 <img> srcs, got %d: %v", len(srcs), srcs)
		}

		file, token := parseResourceURL(t, srcs[0])
		if file != "images/cover.jpg" {
			t.Errorf("file = %q, want %q", file, "images/cover.jpg")
		}
		if token != chapterTokenPlaceholder {
			t.Errorf("token = %q, want placeholder %q", token, chapterTokenPlaceholder)
		}
	})

	t.Run("data URI image is left untouched", func(t *testing.T) {
		srcs := findAttr(t, out, "img", "src")
		if !strings.HasPrefix(srcs[1], "data:image/png") {
			t.Errorf("data URI was modified: %q", srcs[1])
		}
	})

	t.Run("absolute external image is left untouched", func(t *testing.T) {
		srcs := findAttr(t, out, "img", "src")
		if srcs[2] != "https://example.com/remote.png" {
			t.Errorf("external URL was modified: %q", srcs[2])
		}
	})

	t.Run("chapter navigation links are left untouched", func(t *testing.T) {
		hrefs := findAttr(t, out, "a", "href")
		if len(hrefs) != 2 {
			t.Fatalf("expected 2 <a> hrefs, got %d: %v", len(hrefs), hrefs)
		}
		if hrefs[0] != "chapter2.xhtml" {
			t.Errorf("chapter nav link was modified: %q", hrefs[0])
		}
		if hrefs[1] != "#footnote1" {
			t.Errorf("fragment-only link was modified: %q", hrefs[1])
		}
	})
}

func TestRewriteResourceLinks_PathTraversalIsBlocked(t *testing.T) {
	html := `<html><body>
		<img src="../../../etc/passwd" alt="evil"/>
	</body></html>`

	out, err := rewriteResourceLinks([]byte(html), "OEBPS/text", "book-123")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	srcs := findAttr(t, out, "img", "src")
	if len(srcs) != 1 {
		t.Fatalf("expected 1 <img> src, got %d", len(srcs))
	}
	if srcs[0] != "" {
		t.Errorf("path traversal attempt was not blocked, src = %q", srcs[0])
	}
}

func TestGetChapter_RewritesResourceLinks(t *testing.T) {
	epubPath := createTestEpub(t, map[string]string{
		"OEBPS/chapter1.xhtml": `<?xml version="1.0" encoding="utf-8"?>
			<html xmlns="http://www.w3.org/1999/xhtml">
				<head>
					<title>Chapter 1</title>
					<link rel="stylesheet" type="text/css" href="../page_styles.css"/>
				</head>
				<body>
					<p>Chapter 1</p>
					<img src="../images/cover.jpg" alt="cover"/>
				</body>
			</html>`,
	})

	e := &Epub{
		Path:  epubPath,
		ID:    "book-123",
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

	imgSrcs := findAttr(t, data, "img", "src")
	if len(imgSrcs) != 1 {
		t.Fatalf("expected 1 <img> src, got %d: %v", len(imgSrcs), imgSrcs)
	}
	file, token := parseResourceURL(t, imgSrcs[0])
	if file != "images/cover.jpg" {
		t.Errorf("file = %q, want %q", file, "images/cover.jpg")
	}
	if token != chapterTokenPlaceholder {
		t.Errorf("token = %q, want placeholder", token)
	}

	linkHrefs := findAttr(t, data, "link", "href")
	if len(linkHrefs) != 1 {
		t.Fatalf("expected 1 <link> href, got %d: %v", len(linkHrefs), linkHrefs)
	}
	file, _ = parseResourceURL(t, linkHrefs[0])
	if file != "page_styles.css" {
		t.Errorf("file = %q, want %q", file, "page_styles.css")
	}
}

func TestGetChapter_IndexOutOfBounds(t *testing.T) {
	e := &Epub{
		Nav: []PrettySpineItem{
			{Index: 0, Number: 1, Title: "test", Href: "OEBPS/chapter1.xhtml"},
		},
	}

	if _, err := e.GetChapter(-1); err == nil {
		t.Fatal("expected error for negative index, got nil")
	}
	if _, err := e.GetChapter(5); err == nil {
		t.Fatal("expected error for out-of-range index, got nil")
	}
}
