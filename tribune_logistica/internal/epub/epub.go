package epub

import (
	"archive/zip"
	"bytes"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"path"
	"sort"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/storage"
)

type Epub struct {
	Path  string
	Spine []SpineItem
	Nav   []PrettySpineItem
}

func New(path string) (Epub, error) {
	epub := Epub{
		Path:  path,
		Spine: []SpineItem{},
	}

	return epub, nil
}

func Load(db storage.Storage, bookID string) (Epub, error) {
	book, err := library.LoadBook(db, bookID)
	if err != nil {
		return Epub{}, err
	}
	return New(book.FilePath)
}

func (e *Epub) GetFile(filepath string) ([]byte, error) {
	r, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, fmt.Errorf("opening epub %q: %w", e.Path, err)
	}
	defer r.Close()

	for _, f := range r.File {
		if f.Name == filepath {
			rc, err := f.Open()
			if err != nil {
				return nil, fmt.Errorf("opening %q in epub: %w", filepath, err)
			}
			defer rc.Close()

			data, err := io.ReadAll(rc)
			if err != nil {
				return nil, fmt.Errorf("reading %q: %w", filepath, err)
			}
			return data, nil
		}
	}

	return nil, fmt.Errorf("file %q not found in epub", filepath)
}

func (e *Epub) GetCover() ([]byte, string, error) {
	opfPath, err := e.findOPFPath()
	if err != nil {
		return nil, "", err
	}
	pkg, err := e.parseOPF()
	if err != nil {
		return nil, "", err
	}
	opfDir := path.Dir(opfPath)

	// 1. EPUB3: properties="cover-image"
	for _, it := range pkg.Manifest.Items {
		for _, p := range strings.Fields(it.Properties) {
			if p == "cover-image" {
				data, err := e.GetFile(path.Join(opfDir, it.Href))
				if err != nil {
					return nil, "", fmt.Errorf("reading cover image: %w", err)
				}
				return data, it.MediaType, nil
			}
		}
	}

	// 2. EPUB2: <meta name="cover" content="manifest-id"/>
	for _, meta := range pkg.Metadata.Metas {
		if meta.Name != "cover" || meta.Content == "" {
			continue
		}
		for _, it := range pkg.Manifest.Items {
			if it.ID == meta.Content {
				data, err := e.GetFile(path.Join(opfDir, it.Href))
				if err != nil {
					return nil, "", fmt.Errorf("reading cover image: %w", err)
				}
				return data, it.MediaType, nil
			}
		}
		return nil, "", fmt.Errorf("cover meta content %q not found in manifest", meta.Content)
	}

	// 3. Guide fallback: <reference type="cover" href="..."/>
	for _, ref := range pkg.Guide.References {
		if ref.Type == "cover" {
			fullPath := path.Join(opfDir, ref.Href)
			data, err := e.GetFile(fullPath)
			if err != nil {
				return nil, "", fmt.Errorf("reading cover image: %w", err)
			}
			return data, mediaTypeFromExt(fullPath), nil
		}
	}

	return nil, "", errors.New("no cover image found")
}

func (e *Epub) GetCSS() ([]byte, error) {
	zr, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, fmt.Errorf("open epub: %w", err)
	}
	defer zr.Close()

	var names []string
	for _, f := range zr.File {
		if strings.EqualFold(path.Ext(f.Name), ".css") {
			names = append(names, f.Name)
		}
	}

	if len(names) == 0 {
		return nil, errors.New("no CSS files found in epub")
	}

	sort.Strings(names)

	var buf bytes.Buffer
	for _, name := range names {
		data, err := e.GetFile(name)
		if err != nil {
			return nil, fmt.Errorf("reading css %q: %w", name, err)
		}
		buf.Write(data)
		buf.WriteByte('\n')
	}

	return buf.Bytes(), nil
}

func (e *Epub) GetChapter(index int) ([]byte, error) {
	if index < 0 || index >= len(e.Nav) {
		return nil, fmt.Errorf("chapter index out of bounds")
	}
	data, err := e.GetFile(e.Nav[index].Href)
	if err != nil {
		return nil, err
	}
	return bytes.TrimSpace(data), nil
}

func (e *Epub) LoadSpine() ([]SpineItem, error) {
	// Step 1: locate OPF via container.xml
	opfPath, err := e.findOPFPath()
	if err != nil {
		return nil, err
	}

	// Step 2: parse OPF
	opf, err := e.parseOPF()
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
		if _, err := e.GetFile(fullPath); err != nil {
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

func (e *Epub) GetToc() ([]PrettySpineItem, error) {
	navPath, err := e.findNavPath()
	if err != nil {
		return nil, err
	}

	navData, err := e.GetFile(navPath)
	if err != nil {
		return nil, fmt.Errorf("loading nav file %q: %w", navPath, err)
	}

	var nav NavToc
	if err := xml.Unmarshal(navData, &nav); err != nil {
		return nil, fmt.Errorf("unmarshal nav toc: %w", err)
	}

	// flatten nav points recursively, preserving document order
	var flat []NavPoint
	var flatten func([]NavPoint)
	flatten = func(points []NavPoint) {
		for _, np := range points {
			flat = append(flat, np)
			if len(np.Children) > 0 {
				flatten(np.Children)
			}
		}
	}
	flatten(nav.NavPoints)

	navDir := path.Dir(navPath)
	pretty := make([]PrettySpineItem, 0, len(flat))

	for i, np := range flat {
		href := strings.SplitN(np.Content.Src, "#", 2)[0] // strip fragment
		if href == "" {
			continue
		}

		pretty = append(pretty, PrettySpineItem{
			Index:  i,
			Number: i + 1,
			Title:  np.Label,
			Href:   path.Join(navDir, href),
		})
	}

	if len(pretty) == 0 {
		return nil, errors.New("nav toc contained no usable entries")
	}

	return pretty, nil
}

func (e *Epub) findOPFPath() (string, error) {
	containerData, err := e.GetFile("META-INF/container.xml")
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
	return "", fmt.Errorf("container.xml not found")
}

func (e *Epub) findNavPath() (string, error) {
	opfPath, err := e.findOPFPath()
	if err != nil {
		return "", err
	}
	pkg, err := e.parseOPF()
	if err != nil {
		return "", err
	}

	opfDir := path.Dir(opfPath)

	// EPUB3: manifest item with properties="nav"
	for _, it := range pkg.Manifest.Items {
		for _, p := range strings.Fields(it.Properties) {
			if p == "nav" {
				return path.Join(opfDir, it.Href), nil
			}
		}
	}

	// EPUB2 fallback: <spine toc="idref"> points at the NCX manifest item
	if pkg.Spine.TOC != "" {
		for _, it := range pkg.Manifest.Items {
			if it.ID == pkg.Spine.TOC {
				return path.Join(opfDir, it.Href), nil
			}
		}
		return "", fmt.Errorf("spine toc idref %q not found in manifest", pkg.Spine.TOC)
	}

	for _, it := range pkg.Manifest.Items {
		if it.MediaType == "application/x-dtbncx+xml" {
			return path.Join(opfDir, it.Href), nil
		}
	}

	return "", errors.New("no navigation document found (no EPUB3 nav item, no EPUB2 toc.ncx reference)")
}

func (e *Epub) parseOPF() (*opfPackage, error) {
	opfPath, err := e.findOPFPath()
	if err != nil {
		return nil, err
	}
	f, err := e.GetFile(opfPath)
	if err != nil {
		return nil, fmt.Errorf("opf file %q not found", opfPath)
	}

	var pkg opfPackage
	if err := xml.Unmarshal(f, &pkg); err != nil {
		return nil, fmt.Errorf("parse opf: %w", err)
	}

	return &pkg, nil
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
			Properties string `xml:"properties,attr"` // EPUB3 cover-image / nav
		} `xml:"item"`
	} `xml:"manifest"`

	Spine struct {
		TOC      string `xml:"toc,attr"` // EPUB2: manifest id of the NCX file
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

type SpineItem struct {
	Index int    // 0-based, stable, internal
	ID    string // manifest ID
	Href  string // resolved path
}

type PrettySpineItem struct {
	Index  int    `json:"index"`
	Number int    `json:"number"`
	Href   string `json:"href"`
	Title  string `json:"title"`
}
type NavToc struct {
	NavPoints []NavPoint `xml:"navMap>navPoint"`
}
type Content struct {
	Src string `xml:"src,attr"`
}

type NavPoint struct {
	ID        string     `xml:"id,attr"`
	PlayOrder string     `xml:"playOrder,attr"`
	Label     string     `xml:"navLabel>text"`
	Content   Content    `xml:"content"`
	Children  []NavPoint `xml:"navPoint"`
}

func mediaTypeFromExt(p string) string {
	switch strings.ToLower(path.Ext(p)) {
	case ".png":
		return "image/png"
	case ".jpg", ".jpeg":
		return "image/jpeg"
	case ".gif":
		return "image/gif"
	case ".svg":
		return "image/svg+xml"
	case ".webp":
		return "image/webp"
	default:
		return "application/octet-stream"
	}
}
