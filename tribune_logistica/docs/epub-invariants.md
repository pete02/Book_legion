# EPUB Invariants for Tribune Logistica

This document describes the structural and content invariants that an EPUB file must satisfy for successful processing by `tribune_logistica/internal/epub`. Books that violate these invariants will fail validation.

## Archive Structure

### Required Files

1. **`META-INF/container.xml`**
   - Must exist at the root of the EPUB archive
   - Must be valid XML
   - Must contain at least one `<rootfile>` element with:
     - `media-type="application/oebps-package+xml"` (preferred)
     - OR a `full-path` attribute ending with `.opf`
   - The referenced OPF file path must be valid and accessible

2. **OPF Package Document** (referenced by container.xml)
   - Must be valid XML
   - Must contain a `<spine>` element with at least one `<itemref>`
   - Must contain a `<manifest>` element

### Optional but Recommended Files

- **Navigation Document** (EPUB3): A manifest item with `properties="nav"`
- **NCX Document** (EPUB2): A manifest item with `media-type="application/x-dtbncx+xml"`
- **Cover Image**: Referenced via `properties="cover-image"` (EPUB3), `<meta name="cover">` (EPUB2), or guide reference

## Spine Requirements

### Linear Content

- **Only linear chapters are processed**: Items with `linear="no"` are skipped
- **At least one linear item required**: An empty spine (after filtering non-linear items) causes failure
- **All spine references must exist**: Every `idref` in `<itemref>` must have a corresponding `<item>` in the manifest
- **All spine files must be accessible**: Every file referenced by a spine item must exist in the archive

### Spine Item Structure

Each spine item must have:
- `idref`: Reference to a manifest item ID
- `linear`: Either "yes" or omitted (defaults to linear); "no" excludes the item

## Navigation Requirements

### EPUB3 Navigation (Preferred)

The EPUB3 nav document must contain:
- A `<nav>` element with `epub:type="toc"` (or `type="toc"`)
- Within the nav element, an `<ol>` (ordered list)
- Within the `<ol>`, `<li>` elements containing `<a>` tags with `href` attributes

**Structure tolerance**: The parser is lenient and accepts:
- Unescaped characters (e.g., `&` instead of `&amp;`)
- Unclosed tags (e.g., `<br>`, `<meta>`)
- Nested elements between `<li>` and `<a>` (e.g., `<li><span><a>...</a></span></li>`)

### EPUB2 NCX Fallback

If no EPUB3 nav is found, the system falls back to NCX:
- Must have a manifest item with `media-type="application/x-dtbncx+xml"`
- OR a `<spine toc="idref">` pointing to an NCX item
- NCX must contain `<navMap>` elements with `<navPoint>` children

### Navigation Content Requirements

Each navigation entry must have:
- **Valid href**: A non-empty path to a chapter file (fragment-only links are skipped)
- **Label**: A text label for the chapter (can be empty, but not recommended)

**Failure condition**: If no usable TOC entries are found (all hrefs are empty or missing), the book fails validation.

## Chapter Content Requirements

### Chapter File Format

- Chapters must be valid HTML that can be parsed by `golang.org/x/net/html`
- The parser is lenient and accepts malformed HTML that would fail strict XML parsing

### Resource Link Rewriting

Chapter content may contain resources that get rewritten to API endpoints:

**Supported resource types:**
- `<img src="...">` - Images
- `<source src="...">` - Media sources
- `<link href="..." rel="stylesheet">` - Stylesheets
- `<image xlink:href="...">` - Inline SVG images

**Resources that are NOT rewritten (left unchanged):**
- Empty `href`/`src` attributes
- Fragment-only links (e.g., `#section1`)
- Data URIs (e.g., `data:image/png;base64,...`)
- Mailto links (e.g., `mailto:email@example.com`)
- Absolute/external URLs (e.g., `https://example.com/image.png`)

### Path Resolution Security

- Relative paths in resources are resolved against the chapter's directory
- **Path traversal is blocked**: Attempts to escape the EPUB root (e.g., `../`) cause the link to be dropped
- Only paths within the EPUB root are served

## Cover Image Requirements

The system searches for cover images in this priority order:

1. **EPUB3**: Manifest item with `properties="cover-image"`
2. **EPUB2**: `<meta name="cover" content="manifest-id">` referencing a manifest item
3. **Guide fallback**: `<guide><reference type="cover" href="...">`

**Supported image formats:**
- PNG (`.png`)
- JPEG (`.jpg`, `.jpeg`)
- GIF (`.gif`)
- SVG (`.svg`)
- WebP (`.webp`)

**Failure condition**: If no cover image is found after all three methods, `GetCover()` returns an error.

## CSS Requirements

The `GetCSS()` method:
- Extracts all files with `.css` extension from the archive
- Sorts them alphabetically by filename
- Concatenates them with newlines

**Failure condition**: If no CSS files are found, an error is returned.

## File Access Requirements

### General File Access

- All file paths must exist within the EPUB archive
- File paths are case-sensitive
- Paths are resolved relative to the EPUB root

### Common File Paths

The system expects these standard EPUB paths:
- `META-INF/container.xml` - Archive container
- `[OPF_PATH]` - Package document (varies)
- `[NAV_PATH]` - Navigation document (varies)
- `[CHAPTER_PATHS]` - Chapter files (varies)

## Encoding and Character Requirements

### XML Encoding

- OPF and NCX documents must be valid XML
- XML encoding declarations should match the actual encoding
- The system uses Go's `encoding/xml` package for parsing

### HTML Encoding

- Chapter HTML is parsed with `golang.org/x/net/html` (HTML5 parser)
- Accepts malformed HTML that browsers would render
- No strict encoding requirements beyond valid HTML5 parsing

## Version Compatibility

### EPUB 2.0

Supported via NCX navigation:
- Requires `application/x-dtbncx+xml` NCX file
- Uses `<spine toc="idref">` for NCX reference
- Cover via `<meta name="cover">`

### EPUB 3.0

Preferred format:
- Requires `<nav epub:type="toc">` document
- Uses `properties="nav"` in manifest
- Cover via `properties="cover-image"`

## Validation Checklist

Before accepting a book into the system, validate:

- [ ] `META-INF/container.xml` exists and is valid XML
- [ ] Container references a valid `.opf` file
- [ ] OPF file is valid XML
- [ ] OPF contains a `<spine>` with at least one linear `<itemref>`
- [ ] All spine `idref` values have corresponding manifest items
- [ ] All spine files exist in the archive
- [ ] Navigation document exists (EPUB3 nav or EPUB2 NCX)
- [ ] Navigation contains at least one usable TOC entry with valid href
- [ ] All referenced chapter files exist and are parseable HTML
- [ ] (Optional) Cover image exists if cover display is required
- [ ] (Optional) CSS files exist if CSS extraction is required

## Error Conditions

The following conditions cause EPUB processing to fail:

1. **Archive errors**:
   - Not a valid ZIP archive
   - `META-INF/container.xml` missing or invalid
   - OPF file missing or invalid XML

2. **Spine errors**:
   - Empty spine (no linear items)
   - Spine references missing manifest items
   - Spine files missing from archive

3. **Navigation errors**:
   - No EPUB3 nav or EPUB2 NCX found
   - Nav document missing or unparseable
   - No usable TOC entries (all hrefs empty/missing)

4. **Chapter errors**:
   - Chapter files missing from archive
   - Chapter files unparseable as HTML

5. **Resource errors**:
   - Cover image missing (if required)
   - CSS files missing (if required)

## Implementation Notes

### Caching Behavior

The `epubCache` structure caches the opened ZIP archive for performance:
- First access opens and indexes the archive
- Subsequent accesses use the cached index
- The cache is closed via garbage collection finalizer
- **Important**: If the EPUB file is modified on disk after first access, the cache will serve stale data

### Concurrent Access

- `epubCache.mu` guards archive opening and indexing
- `epubCache.navMu` guards lazy navigation population
- Multiple concurrent `GetChapter()` calls on the same `Epub` instance are safe

### Path Security

- `resolveEpubPath()` prevents path traversal attacks
- Paths resolving to `..` or starting with `../` are rejected
- Only paths within the EPUB root are accessible
