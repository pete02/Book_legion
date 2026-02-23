package epub

import (
	"archive/zip"
	"bytes"
	"io"
	"os"
	"path/filepath"
	"testing"
)

func createMinimalEPUBWithNav(t *testing.T, files map[string]string) string {
	t.Helper()

	tmpFile := filepath.Join(t.TempDir(), "test.epub")

	f, err := os.Create(tmpFile)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	zw := zip.NewWriter(f)
	for name, content := range files {
		w, err := zw.Create(name)
		if err != nil {
			t.Fatal(err)
		}
		if _, err := io.Copy(w, bytes.NewBufferString(content)); err != nil {
			t.Fatal(err)
		}
	}
	if err := zw.Close(); err != nil {
		t.Fatal(err)
	}
	return tmpFile
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
  </manifest>
  <spine>
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

	epubPath := createMinimalEPUBWithNav(t, files)

	// load mechanical spine
	e, err := New(epubPath)
	if err != nil {
		t.Fatalf("failed to create Epub: %v", err)
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
