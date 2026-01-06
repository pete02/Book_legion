package epub

import (
	"archive/zip"
	"bytes"
	"io"
	"os"
	"path/filepath"
	"testing"
)

func createMinimalEPUB(t *testing.T, files map[string]string) string {
	t.Helper()

	tmpFile := t.TempDir() + "/test.epub"

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
		_, _ = io.Copy(w, bytes.NewBufferString(content))
	}
	if err := zw.Close(); err != nil {
		t.Fatal(err)
	}
	return tmpFile
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

	epubPath := createMinimalEPUB(t, files)

	spine, err := LoadSpine(epubPath)
	if err != nil {
		t.Fatalf("LoadSpine failed: %v", err)
	}

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
	epubPath := createMinimalEPUB(t, map[string]string{})
	_, err := LoadSpine(epubPath)
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

	epubPath := createMinimalEPUB(t, files)
	_, err := LoadSpine(epubPath)
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

	epubPath := createMinimalEPUB(t, files)
	_, err := LoadSpine(epubPath)
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

	epubPath := createMinimalEPUB(t, files)
	spine, err := LoadSpine(epubPath)
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

	epubPath := createMinimalEPUB(t, files)
	spine, err := LoadSpine(epubPath)
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
