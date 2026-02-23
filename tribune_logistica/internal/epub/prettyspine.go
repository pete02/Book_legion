package epub

import (
	"archive/zip"
	"encoding/xml"
	"fmt"
	"io"
	"path/filepath"
	"strconv"
	"strings"
)

type PrettySpineItem struct {
	Index  int    `json:"index"`
	Number int    `json:"number"`
	Href   string `json: href`
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

func (e *Epub) LoadPrettySpine() ([]PrettySpineItem, error) {
	// 1. open EPUB as zip
	r, err := zip.OpenReader(e.Path)
	if err != nil {
		return nil, err
	}
	defer r.Close()
	var toc_folder = ""
	var navData []byte
	// 2. locate nav.toc in ZIP
	for _, f := range r.File {
		if filepath.Base(f.Name) == "toc.ncx" {
			rc, err := f.Open()
			if err != nil {
				return nil, err
			}
			defer rc.Close()
			navData, err = io.ReadAll(rc)
			toc_folder = filepath.Dir(f.Name)
			if err != nil {
				return nil, err
			}
			break
		}
	}

	if navData == nil {
		return nil, fmt.Errorf("Error in loading ToC: toc.ncx not found")
	}

	// 3. parse XML
	var nav NavToc
	if err := xml.Unmarshal(navData, &nav); err != nil {
		return nil, fmt.Errorf("unmarshal error: %v", err)
	}

	// 4. flatten nav points recursively
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

	// 5. map TOC to mechanical spine
	var pretty []PrettySpineItem
	number := 1
	for _, np := range flat {
		href := strings.Split(np.Content.Src, "#")[0] // ignore fragment
		playOrderInt, err := strconv.Atoi(np.PlayOrder)
		if err != nil {
			// handle invalid number
			playOrderInt = 0 // or skip
		}

		item := PrettySpineItem{
			Index:  playOrderInt,
			Number: playOrderInt + 1,
			Title:  np.Label,
			Href:   toc_folder + "/" + href,
		}
		pretty = append(pretty, item)
		number++
	}
	return pretty, nil
}
