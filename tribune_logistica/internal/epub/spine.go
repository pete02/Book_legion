package epub

import (
	"archive/zip"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"path"
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
	opfPath, err := findOPFPath(files)
	if err != nil {
		return nil, err
	}

	// Step 2: parse OPF
	opf, err := parseOPF(files, opfPath)
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

func findOPFPath(files map[string]*zip.File) (string, error) {
	f, ok := files["META-INF/container.xml"]
	if !ok {
		return "", errors.New("META-INF/container.xml not found")
	}

	rc, err := f.Open()
	if err != nil {
		return "", err
	}
	defer rc.Close()

	var c containerXML
	if err := xml.NewDecoder(rc).Decode(&c); err != nil {
		return "", fmt.Errorf("parse container.xml: %w", err)
	}

	if len(c.Rootfiles) == 0 {
		return "", errors.New("no rootfile entry in container.xml")
	}

	return c.Rootfiles[0].FullPath, nil
}

func parseOPF(files map[string]*zip.File, opfPath string) (*opfPackage, error) {
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

/* ---------- XML structures ---------- */

type containerXML struct {
	Rootfiles []struct {
		FullPath string `xml:"full-path,attr"`
	} `xml:"rootfiles>rootfile"`
}

type opfPackage struct {
	Manifest struct {
		Items []struct {
			ID        string `xml:"id,attr"`
			Href      string `xml:"href,attr"`
			MediaType string `xml:"media-type,attr"`
		} `xml:"item"`
	} `xml:"manifest"`

	Spine struct {
		Itemrefs []struct {
			IDRef  string `xml:"idref,attr"`
			Linear string `xml:"linear,attr"`
		} `xml:"itemref"`
	} `xml:"spine"`
}

type manifestItem struct {
	Href      string
	MediaType string
}
