package epub

import (
	"archive/zip"
	"bytes"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"log"
	"net/url"
	"path"
	"runtime"
	"sort"
	"strings"
	"sync"

	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/storage"
	"github.com/book_legion-tribune_logistica/internal/types"
	"golang.org/x/net/html"
)

type Epub struct {
	ID    string
	Path  string
	Spine []SpineItem
	Nav   []PrettySpineItem

	// internal is a lazily-shared cache that lets repeated GetFile/GetChapter
	// calls avoid reopening and re-scanning the zip archive every time, and
	// guards concurrent lazy population of Nav. It is only ever set by New,
	// via a pointer indirection (never embedded by value) so that Epub itself
	// stays safe to return/copy by value, as New and Load already do. A bare
	// Epub{} struct literal (internal == nil) still works correctly — it
	// just falls back to the original, uncached per-call behavior below.
	internal *epubCache
}

// epubCache holds everything that needs to be shared across calls on the
// same *Epub, and the locks that guard it. Keeping this behind a pointer
// (rather than embedding sync.Mutex directly in Epub) is what keeps Epub
// itself free of copy-lock issues.
type epubCache struct {
	mu     sync.Mutex
	rc     *zip.ReadCloser
	byName map[string]*zip.File

	navMu sync.Mutex // guards lazy population of Epub.Nav in GetChapter
}

func New(path string, bookId string) (Epub, error) {
	epub := Epub{
		ID:       bookId,
		Path:     path,
		Spine:    []SpineItem{},
		internal: &epubCache{},
	}

	return epub, nil
}

func Load(db *storage.SQLStorage, bookID string) (Epub, error) {
	book, err := library.LoadBookAuth(db, bookID)
	if err != nil {
		return Epub{}, err
	}
	return New(book.FilePath, bookID)
}

func (e *Epub) ExtractChunk(cursor types.UserCursor, end int) (string, error) {

	chunkI := types.ChunkIdentifier{
		ID:          e.ID,
		Chapter:     cursor.Cursor.Chapter,
		StartOffset: cursor.Cursor.Index,
		EndOffset:   cursor.Cursor.Index + end,
	}

	chapterHtml, err := e.GetChapter(cursor.Cursor.Chapter)
	if err != nil {
		return "", err
	}

	chunk, err := types.BuildTextChunk(string(chapterHtml), chunkI, types.DefaultChunkConfig())
	if err != nil {
		return "", err
	}

	return chunk.Data, nil
}

func (e *Epub) FindNearestAllowedSplit(cursor types.UserCursor) (types.UserCursor, error) {

	chapterHTML, err := e.GetChapter(cursor.Cursor.Chapter)
	if err != nil {
		return cursor, err
	}

	nearestSplit := types.NearestFollowingSplit(string(chapterHTML), cursor.Cursor.Index, 10)

	cursor.Cursor.Index = nearestSplit.Offset
	return cursor, nil
}
func (e *Epub) GetFile(filepath string) ([]byte, error) {
	if e.internal == nil {
		return e.getFileUncached(filepath)
	}

	f, err := e.internal.lookup(e.Path, filepath)
	if err != nil {
		return nil, err
	}

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

// getFileUncached is the original, always-correct implementation: it opens
// the archive fresh on every call. Used whenever no cache is available
// (e.g. an Epub built via a bare struct literal rather than New/Load).
func (e *Epub) getFileUncached(filepath string) ([]byte, error) {
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

// listFileNames returns every entry name in the archive, using the shared
// cache when available (see lookup) instead of opening the zip again.
func (e *Epub) listFileNames() ([]string, error) {
	if e.internal == nil {
		zr, err := zip.OpenReader(e.Path)
		if err != nil {
			return nil, err
		}
		defer zr.Close()
		names := make([]string, 0, len(zr.File))
		for _, f := range zr.File {
			names = append(names, f.Name)
		}
		return names, nil
	}

	e.internal.mu.Lock()
	defer e.internal.mu.Unlock()
	if err := e.internal.ensureOpenLocked(e.Path); err != nil {
		return nil, err
	}
	names := make([]string, 0, len(e.internal.byName))
	for name := range e.internal.byName {
		names = append(names, name)
	}
	return names, nil
}

// lookup returns the *zip.File for name, opening and indexing the archive
// on first use (and caching it for the lifetime of this *epubCache). The
// open zip.ReadCloser is closed via a finalizer when the cache is garbage
// collected, so callers don't need to add an explicit Close step for this
// to be safe — it degrades to "closed a little later than ideal" rather
// than leaking indefinitely.
//
// Trade-off vs. the original always-reopen behavior: if the underlying file
// on disk is replaced or modified after the first successful lookup, later
// calls on this same Epub instance will keep serving the original archive's
// contents. Epub files in this system are static uploaded artifacts, not
// edited in place, so this should not matter in practice.
func (c *epubCache) lookup(epubPath, name string) (*zip.File, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if err := c.ensureOpenLocked(epubPath); err != nil {
		return nil, err
	}

	f, ok := c.byName[name]
	if !ok {
		return nil, fmt.Errorf("file %q not found in epub", name)
	}
	return f, nil
}

// ensureOpenLocked opens the archive and builds the name index on first
// call. Caller must hold c.mu.
func (c *epubCache) ensureOpenLocked(epubPath string) error {
	if c.rc != nil {
		return nil
	}

	rc, err := zip.OpenReader(epubPath)
	if err != nil {
		return fmt.Errorf("opening epub %q: %w", epubPath, err)
	}
	c.rc = rc
	c.byName = make(map[string]*zip.File, len(rc.File))
	for _, f := range rc.File {
		c.byName[f.Name] = f
	}
	runtime.SetFinalizer(c, func(cc *epubCache) {
		if cc.rc != nil {
			cc.rc.Close()
		}
	})
	return nil
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
				log.Printf("Found cover image: %s", it.Href)
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
		found := false
		for _, it := range pkg.Manifest.Items {
			if it.ID == meta.Content {
				found = true
				log.Printf("Found cover image: %s", it.Href)
				data, err := e.GetFile(path.Join(opfDir, it.Href))
				if err != nil {
					return nil, "", fmt.Errorf("reading cover image: %w", err)
				}
				return data, it.MediaType, nil
			}
		}
		if !found {
			// Malformed reference: don't give up here, the guide fallback
			// below may still resolve a valid cover.
			log.Printf("cover meta content %q not found in manifest; trying guide fallback", meta.Content)
		}
		break
	}

	// 3. Guide fallback: <reference type="cover" href="..."/>
	for _, ref := range pkg.Guide.References {
		if ref.Type == "cover" {
			fullPath := path.Join(opfDir, ref.Href)
			data, err := e.GetFile(fullPath)
			if err != nil {
				return nil, "", fmt.Errorf("reading cover image: %w", err)
			}
			log.Printf("Found cover image: %s", fullPath)
			return data, mediaTypeFromExt(fullPath), nil
		}
	}

	return nil, "", errors.New("no cover image found")
}

func (e *Epub) GetCSS() ([]byte, error) {
	allNames, err := e.listFileNames()
	if err != nil {
		return nil, fmt.Errorf("open epub: %w", err)
	}

	var names []string
	for _, name := range allNames {
		if strings.EqualFold(path.Ext(name), ".css") {
			names = append(names, name)
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
	nav, err := e.loadNav()
	if err != nil {
		return nil, fmt.Errorf("failed to get table of contents: %w", err)
	}

	if index < 0 || index >= len(nav) {
		return nil, fmt.Errorf("chapter index out of bounds")
	}
	chapterHref := nav[index].Href
	data, err := e.GetFile(chapterHref)
	if err != nil {
		return nil, err
	}

	rewritten, err := rewriteResourceLinks(data, chapterHref, e.ID)
	if err != nil {
		return nil, fmt.Errorf("failed to rewrite resource links: %w", err)
	}

	return rewritten, nil
}

// loadNav returns e.Nav, computing and caching it on first use. When a
// cache is available (Epub built via New/Load), this is guarded by a mutex
// so concurrent GetChapter calls on the same Epub can't race on populating
// Nav. Without a cache (bare struct literal), it falls back to the
// original, unsynchronized lazy-load.
func (e *Epub) loadNav() ([]PrettySpineItem, error) {
	if e.internal == nil {
		if len(e.Nav) == 0 {
			nav, err := e.GetToc()
			if err != nil {
				return nil, err
			}
			e.Nav = nav
		}
		return e.Nav, nil
	}

	e.internal.navMu.Lock()
	defer e.internal.navMu.Unlock()

	if len(e.Nav) == 0 {
		nav, err := e.GetToc()
		if err != nil {
			return nil, err
		}
		e.Nav = nav
	}
	return e.Nav, nil
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

type NavHtml struct {
	Body struct {
		Nav []struct {
			Type string `xml:"type,attr"`
			Ol   NavOl  `xml:"ol"`
		} `xml:"nav"`
	} `xml:"body"`
}

type NavOl struct {
	Li []NavLi `xml:"li"`
}

type NavLi struct {
	A  *NavA  `xml:"a"`
	Ol *NavOl `xml:"ol"` // nested sub-lists, if any
}

type NavA struct {
	Href string `xml:"href,attr"`
	Text string `xml:",chardata"`
}

// findTocNav searches the whole document for a <nav> element whose
// [epub:]type attribute includes the token "toc" (epub:type can in
// principle carry multiple space-separated tokens, mirroring how
// manifest item "properties" tokens are already handled elsewhere in this
// file). Matching is done on the attribute's local name so both a bare
// type="toc" and a namespaced epub:type="toc" (however the parser happens
// to have preserved the prefix) are accepted.
func findTocNav(n *html.Node) *html.Node {
	if n.Type == html.ElementNode && n.Data == "nav" && hasEpubType(n, "toc") {
		return n
	}
	for c := n.FirstChild; c != nil; c = c.NextSibling {
		if found := findTocNav(c); found != nil {
			return found
		}
	}
	return nil
}

func hasEpubType(n *html.Node, want string) bool {
	for _, a := range n.Attr {
		k := strings.ToLower(a.Key)
		if k != "type" && !strings.HasSuffix(k, ":type") {
			continue
		}
		for _, tok := range strings.Fields(a.Val) {
			if strings.EqualFold(tok, want) {
				return true
			}
		}
	}
	return false
}

// findDescendant returns the first element named tag found anywhere in n's
// subtree (document order, n itself excluded), or nil.
func findDescendant(n *html.Node, tag string) *html.Node {
	for c := n.FirstChild; c != nil; c = c.NextSibling {
		if c.Type == html.ElementNode && c.Data == tag {
			return c
		}
		if found := findDescendant(c, tag); found != nil {
			return found
		}
	}
	return nil
}

// directChildren returns n's immediate element children named tag, in
// document order.
func directChildren(n *html.Node, tag string) []*html.Node {
	var out []*html.Node
	for c := n.FirstChild; c != nil; c = c.NextSibling {
		if c.Type == html.ElementNode && c.Data == tag {
			out = append(out, c)
		}
	}
	return out
}

// attrVal returns n's attribute value for key (case-insensitive), if set.
func attrVal(n *html.Node, key string) (string, bool) {
	for _, a := range n.Attr {
		if strings.EqualFold(a.Key, key) {
			return a.Val, true
		}
	}
	return "", false
}

// textContent concatenates all text nodes in n's subtree.
func textContent(n *html.Node) string {
	var buf strings.Builder
	var walk func(*html.Node)
	walk = func(n *html.Node) {
		if n.Type == html.TextNode {
			buf.WriteString(n.Data)
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			walk(c)
		}
	}
	walk(n)
	return buf.String()
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

	navDir := path.Dir(navPath)
	var flat []struct {
		Href  string
		Label string
	}

	if strings.EqualFold(path.Ext(navPath), ".ncx") {
		// EPUB2: navMap>navPoint
		var nav NavToc
		if err := xml.Unmarshal(navData, &nav); err != nil {
			return nil, fmt.Errorf("unmarshal nav toc (ncx): %w", err)
		}

		var flattenNcx func([]NavPoint)
		flattenNcx = func(points []NavPoint) {
			for _, np := range points {
				flat = append(flat, struct {
					Href  string
					Label string
				}{Href: np.Content.Src, Label: np.Label})
				if len(np.Children) > 0 {
					flattenNcx(np.Children)
				}
			}
		}
		flattenNcx(nav.NavPoints)
	} else {
		// EPUB3: <nav epub:type="toc"><ol><li><a>...
		//
		// Parsed with the same lenient HTML5 parser used for chapter content
		// (golang.org/x/net/html) rather than encoding/xml. Real-world
		// EPUB3 nav documents are frequently "good enough for a browser"
		// HTML5 — unescaped '&', an unclosed <br> or <meta>, etc. — and
		// previously failed to unmarshal as strict XML, taking the whole
		// book down even though nothing was actually wrong with it from a
		// reader's point of view. This also incidentally fixes TOC entries
		// being silently dropped when <a> wasn't a *direct* child of <li>
		// (e.g. <li><span><a href="...">...</a></span></li>), since
		// findDescendant searches the whole subtree, not just immediate
		// children.
		doc, err := html.Parse(bytes.NewReader(navData))
		if err != nil {
			return nil, fmt.Errorf("parse nav toc (html): %w", err)
		}

		tocNav := findTocNav(doc)
		if tocNav == nil {
			return nil, errors.New("no nav with epub:type=\"toc\" found")
		}

		tocOl := findDescendant(tocNav, "ol")
		if tocOl == nil {
			return nil, errors.New("nav with epub:type=\"toc\" contains no <ol>")
		}

		var flattenHtml func(*html.Node)
		flattenHtml = func(ol *html.Node) {
			for _, li := range directChildren(ol, "li") {
				if a := findDescendant(li, "a"); a != nil {
					if href, ok := attrVal(a, "href"); ok {
						flat = append(flat, struct {
							Href  string
							Label string
						}{Href: href, Label: strings.TrimSpace(textContent(a))})
					}
				}
				// Nested sub-lists, if any.
				if nested := findDescendant(li, "ol"); nested != nil {
					flattenHtml(nested)
				}
			}
		}
		flattenHtml(tocOl)
	}

	pretty := make([]PrettySpineItem, 0, len(flat))
	for _, item := range flat {
		href := strings.SplitN(item.Href, "#", 2)[0] // strip fragment
		if href == "" {
			continue
		}

		// Index/Number are derived from the item's position in the final,
		// filtered slice (not its position in the raw flattened TOC), so
		// they stay aligned with GetChapter's slice indexing even when
		// fragment-only entries above were skipped.
		i := len(pretty)
		pretty = append(pretty, PrettySpineItem{
			Index:  i,
			Number: i + 1,
			Title:  item.Label,
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
	return "", fmt.Errorf("container.xml not found: %v", err)
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
		// Malformed reference: don't give up, a plain NCX item found by
		// media-type below may still resolve the nav document.
		log.Printf("spine toc idref %q not found in manifest; trying media-type fallback", pkg.Spine.TOC)
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

func rewriteResourceLinks(data []byte, chapterHref, bookID string) ([]byte, error) {
	doc, err := html.Parse(bytes.NewReader(data))
	if err != nil {
		return nil, fmt.Errorf("failed to parse chapter HTML: %w", err)
	}

	baseDir := path.Dir(chapterHref) // "OEBPS/chapter.xhtml" -> "OEBPS"

	var walk func(*html.Node)
	walk = func(n *html.Node) {
		if n.Type == html.ElementNode {
			switch n.Data {
			case "img", "source":
				rewriteAttr(n, "src", baseDir, bookID)
			case "link":
				if isStylesheet(n) {
					rewriteAttr(n, "href", baseDir, bookID)
				}
			case "image": // inline SVG <image xlink:href="...">
				rewriteAttr(n, "xlink:href", baseDir, bookID)
			}
			// Note: plain <a href="chapter2.xhtml"> nav links are
			// deliberately left untouched here — those need to route to
			// your chapter-navigation logic, not the file-serving endpoint.
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			walk(c)
		}
	}
	walk(doc)

	var buf bytes.Buffer
	if err := html.Render(&buf, doc); err != nil {
		return nil, fmt.Errorf("failed to render rewritten chapter HTML: %w", err)
	}
	return buf.Bytes(), nil
}

func isStylesheet(n *html.Node) bool {
	for _, a := range n.Attr {
		if a.Key == "rel" && strings.EqualFold(strings.TrimSpace(a.Val), "stylesheet") {
			return true
		}
	}
	return false
}

const chapterTokenPlaceholder = "TOKEN_PLACEHOLDER"

func rewriteAttr(n *html.Node, key, baseDir, bookID string) {
	for i, a := range n.Attr {
		if a.Key != key {
			continue
		}
		if !isRewritableResource(a.Val) {
			return
		}
		resolved, ok := resolveEpubPath(baseDir, a.Val)
		if !ok {
			// Resolved outside the epub root — drop rather than risk
			// serving/linking something unexpected.
			n.Attr[i].Val = ""
			return
		}
		n.Attr[i].Val = fmt.Sprintf("/api/v1/books/%s/file?file=%s",
			url.PathEscape(bookID),
			url.QueryEscape(resolved),
		)
		return
	}
}

// isRewritableResource filters out values we should never touch: empty,
// fragment-only, data URIs, mailto links, or absolute/external URLs.
func isRewritableResource(val string) bool {
	if val == "" || strings.HasPrefix(val, "#") {
		return false
	}
	if strings.HasPrefix(val, "data:") || strings.HasPrefix(val, "mailto:") {
		return false
	}
	if u, err := url.Parse(val); err == nil && u.IsAbs() {
		return false
	}
	return true
}

// resolveEpubPath resolves ref against the directory of the file that
// referenced it, and guards against the result escaping the epub root.
func resolveEpubPath(baseDir, ref string) (string, bool) {
	if i := strings.IndexByte(ref, '#'); i >= 0 {
		ref = ref[:i] // drop any fragment, e.g. "style.css#foo"
	}

	cleaned := path.Clean(path.Join(baseDir, ref))

	if cleaned == ".." || strings.HasPrefix(cleaned, "../") {
		return "", false // tried to climb above the epub root
	}

	return cleaned, true
}
