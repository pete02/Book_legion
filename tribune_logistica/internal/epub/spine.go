package epub

import (
	"archive/zip"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"path"
	"strings"
)

type SpineItem struct {
	Index int    // 0-based, stable, internal
	ID    string // manifest ID
	Href  string // resolved path
}

func LoadSpine(epubPath string) ([]SpineItem, error) {
	zr, err := zip.OpenReader(epubPath)
	if err != nil {
		return nil, fmt.Errorf("open epub: %w", err)
	}
	defer zr.Close()

	// Build file lookup
	files := make(map[string]*zip.File, len(zr.File))
	for _, f := range zr.File {
		files[f.Name] = f
	}

	// Step 1: locate OPF via container.xml
	opfPath, err := FindOPFPath(zr)
	if err != nil {
		return nil, err
	}

	// Step 2: parse OPF
	opf, err := ParseOPF(files, opfPath)
	if err != nil {
		return nil, err
	}

	// Step 3: build manifest lookup
	manifest := make(map[string]manifestItem, len(opf.Manifest.Items))
	for _, it := range opf.Manifest.Items {
		manifest[it.ID] = manifestItem{
			Href:      it.Href,
			MediaType: it.MediaType,
		}
	}

	// Step 4: resolve spine
	opfDir := path.Dir(opfPath)
	var spine []SpineItem

	for _, ref := range opf.Spine.Itemrefs {
		if ref.Linear == "no" {
			continue
		}

		mi, ok := manifest[ref.IDRef]
		if !ok {
			return nil, fmt.Errorf("spine idref %q not found in manifest", ref.IDRef)
		}

		fullPath := path.Join(opfDir, mi.Href)
		if _, ok := files[fullPath]; !ok {
			return nil, fmt.Errorf("spine file %q not found in epub", fullPath)
		}

		item := SpineItem{
			Index: len(spine),
			ID:    ref.IDRef,
			Href:  fullPath,
		}
		spine = append(spine, item)
	}

	if len(spine) == 0 {
		return nil, errors.New("epub spine is empty")
	}

	return spine, nil
}

/* ---------- Internal helpers ---------- */
func FindOPFPath(zr *zip.ReadCloser) (string, error) {
	containerData, err := readZipFile(zr, "META-INF/container.xml")
	if err == nil {
		type RootFile struct {
			FullPath  string `xml:"full-path,attr"`
			MediaType string `xml:"media-type,attr"`
		}
		type RootFiles struct {
			RootFiles []RootFile `xml:"rootfile"`
		}
		type Container struct {
			RootFiles RootFiles `xml:"rootfiles"`
		}
		var container Container
		if err := xml.Unmarshal(containerData, &container); err == nil {
			for _, rf := range container.RootFiles.RootFiles {
				if rf.MediaType == "application/oebps-package+xml" || strings.HasSuffix(rf.FullPath, ".opf") {
					return rf.FullPath, nil
				}
			}
		}
	}

	// Fallback
	for _, f := range zr.File {
		if strings.HasSuffix(strings.ToLower(f.Name), ".opf") {
			return f.Name, nil
		}
	}
	return "", fmt.Errorf("OPF file not found")
}

func ParseOPF(files map[string]*zip.File, opfPath string) (*opfPackage, error) {
	f, ok := files[opfPath]
	if !ok {
		return nil, fmt.Errorf("opf file %q not found", opfPath)
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

	var pkg opfPackage
	if err := xml.Unmarshal(data, &pkg); err != nil {
		return nil, fmt.Errorf("parse opf: %w", err)
	}

	return &pkg, nil
}

func BuildFileMap(zr *zip.ReadCloser) map[string]*zip.File {
	files := make(map[string]*zip.File, len(zr.File))
	for _, f := range zr.File {
		files[f.Name] = f
	}
	return files
}

func ReadFromMap(files map[string]*zip.File, name string) ([]byte, error) {
	f, ok := files[name]
	if !ok {
		return nil, fmt.Errorf("file not found in epub: %s", name)
	}
	rc, err := f.Open()
	if err != nil {
		return nil, err
	}
	defer rc.Close()
	return io.ReadAll(rc)
}

/* ---------- XML structures ---------- */

type containerXML struct {
	Rootfiles []struct {
		FullPath string `xml:"full-path,attr"`
	} `xml:"rootfiles>rootfile"`
}

type opfPackage struct {
	Metadata struct {
		Metas []struct {
			Name     string `xml:"name,attr"`
			Content  string `xml:"content,attr"`
			Property string `xml:"property,attr"`
			CharData string `xml:",chardata"`
		} `xml:"meta"`
	} `xml:"metadata"`

	Manifest struct {
		Items []struct {
			ID         string `xml:"id,attr"`
			Href       string `xml:"href,attr"`
			MediaType  string `xml:"media-type,attr"`
			Properties string `xml:"properties,attr"` // EPUB3 cover-image
		} `xml:"item"`
	} `xml:"manifest"`

	Spine struct {
		Itemrefs []struct {
			IDRef  string `xml:"idref,attr"`
			Linear string `xml:"linear,attr"`
		} `xml:"itemref"`
	} `xml:"spine"`

	Guide struct {
		References []struct {
			Type string `xml:"type,attr"`
			Href string `xml:"href,attr"`
		} `xml:"reference"`
	} `xml:"guide"`
}
type manifestItem struct {
	Href      string
	MediaType string
}
