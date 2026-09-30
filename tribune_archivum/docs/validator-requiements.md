# Onboarder Validator Requirements (profile v1)

## 1. Purpose and principles

The onboarder admits a book to the library only if it is in the **canonical form** defined here. The reader may assume this form and implements nothing beyond it.

1. **Pipeline: `fix -> validate`.** The fixer accepts messy input and rewrites it into canonical form. The validator is strict and accepts only canonical form. A book is admitted only if the validator passes the fixer's output.
2. **One way to do everything.** Every requirement below names exactly one mechanism (one cover mechanism, one nav mechanism, and so on). The fixer may understand many input variants. The validator and reader understand one.
3. **Not the EPUB spec.** This is a private profile. It is stricter than EPUB in places and ignores parts of EPUB the reader does not use.
4. **Content is stable.** The fixer never reorders or rewrites content documents beyond the listed sanitizing and well-formedness repairs, because reading positions (chapter + offset cursors) and annotations depend on stable text.
5. **Originals are kept.** The original upload is stored untouched. The canonical copy is stored separately along with a fix log and a `profile_version`.
6. **Idempotent.** `fix(fix(x)) == fix(x)` and `validate(fix(x))` either passes or rejects cleanly, for every input.

### Outcomes per requirement

| Outcome | Meaning |
|---|---|
| **FIX** | Fixer repairs or normalizes it and logs the change. The validator requires the fixed form. |
| **STRIP** | Fixer removes the offending construct and logs it. |
| **REJECT** | Cannot be safely fixed. The book is refused with a clear, user-facing reason. |

Anything the validator checks must be either satisfied by the fixer or a REJECT. There is no "warning, accept as is" tier in the canonical form. If you want to tolerate something, the fixer must normalize it.

---

## 2. Requirements

### 2.1 Archive (Z)

| ID | Requirement | On violation |
|---|---|---|
| Z1 | File is a readable zip. | REJECT |
| Z2 | No entry name is absolute, escapes the zip, or contains backslashes or NUL bytes. | FIX (normalize separators); REJECT if it escapes the root |
| Z3 | No symlinks, no encrypted entries. | REJECT |
| Z4 | Size limits: total uncompressed <= 500 MB, per entry <= 100 MB, entry count <= 10,000, compression ratio <= 100:1 per entry. (Tune the numbers to your needs.) | REJECT |
| Z5 | `mimetype` is the first entry, stored uncompressed, content exactly `application/epub+zip`. | FIX (repack) |
| Z6 | No DRM: no `META-INF/rights.xml`; `META-INF/encryption.xml` is absent, or lists only font obfuscation algorithms. | REJECT for DRM; FIX (de-obfuscate fonts, then remove the file) otherwise |
| Z7 | All entry names are valid UTF-8 and use only a safe character set (no control characters). | FIX or REJECT |

### 2.2 Container and package document (C, O)

| ID | Requirement | On violation |
|---|---|---|
| C1 | `META-INF/container.xml` exists and is well-formed XML. | REJECT |
| C2 | It contains **exactly one** `<rootfile>`, with `media-type="application/oebps-package+xml"` and a `full-path` that ends in `.opf`. | FIX (pick the first matching rootfile, drop the others) |
| C3 | `full-path` is a normalized relative path (no leading `/`, no `..`, no percent-encoding) and the file exists. | FIX / REJECT if missing |
| O1 | The OPF is well-formed XML with root element `package`. | REJECT |
| O2 | Required metadata is present and non-empty: `dc:identifier`, `dc:title`, `dc:language`. | FIX (UUID / filename-derived title / `und`) |
| O3 | `package/@unique-identifier` resolves to a `dc:identifier`. | FIX |
| O4 | `package/@version` is `3.0`. | FIX (set it, add `dcterms:modified`) |
| O5 | Exactly one `<meta property="dcterms:modified">` with an ISO timestamp. | FIX |
| O6 | The OPF has no `<guide>` element. | STRIP (after using it to locate a cover) |

Note: the OPF may live anywhere in the zip, since moving it would require rewriting references. The onboarder records `opf_path` in the DB, and the reader uses that instead of parsing `container.xml` on every request. (`container.xml` is still required, because other tools and a re-export will want it.)

### 2.3 Manifest (M)

| ID | Requirement | On violation |
|---|---|---|
| M1 | Manifest items each have a non-empty, unique `id` and an `href`. | FIX (generate ids, drop items with no href) |
| M2 | Every `href` is a normalized relative path: percent-decoded, no fragment, no scheme, no `..` escaping the zip root, matching the zip entry name **exactly** (case included). | FIX (fix case, decode); drop item if unresolvable |
| M3 | Every manifest item's file exists in the zip. | FIX (drop the item and any spine refs to it) |
| M4 | `media-type` is present and is one of the **allowed types** (section 3). | FIX (sniff by content, then extension); STRIP items of disallowed types and their references |
| M5 | No manifest item has a remote `href` (`http:`, `https:`, `//`). | STRIP |
| M6 | Files in the zip not listed in the manifest (other than `mimetype`, `META-INF/*`, and the OPF itself) are not reachable by the reader. | STRIP (delete them) or add to manifest if referenced by a content document |

### 2.4 Spine (S)

| ID | Requirement | On violation |
|---|---|---|
| S1 | Spine has at least one `itemref`. | REJECT |
| S2 | Every `itemref/@idref` matches a manifest item. | FIX (drop dangling refs) |
| S3 | Every spine item has media type `application/xhtml+xml`. | FIX (convert `text/html` by renaming/re-serializing); STRIP non-content items from the spine |
| S4 | No `linear="no"` attributes. All items are linear. | FIX (remove the attribute; log it) |
| S5 | No duplicate idrefs in the spine. | FIX (keep first) |
| S6 | `spine/@page-progression-direction` is absent or `ltr`. | REJECT for `rtl` if unsupported, otherwise FIX |
| S7 | No `rendition:layout` of `pre-paginated` (fixed layout), no `rendition:*` overrides. | REJECT for fixed layout |

**The spine is the chapter list.** The reader's "chapter N" is spine item N. Chapter indices and cursors never depend on the nav document.

### 2.5 Navigation (N)

Nav is used **only for titles and the table-of-contents UI**. It never determines chapter order or count.

| ID | Requirement | On violation |
|---|---|---|
| N1 | Exactly one manifest item has `id="nav"`, and that same item has `properties="nav"` and `media-type="application/xhtml+xml"`. | FIX |
| N2 | The nav item is **not** in the spine. | FIX (remove from the spine) |
| N3 | The nav file is well-formed XML and contains exactly one `<nav epub:type="toc">` (other `<nav>` elements such as landmarks and page-list are removed). | FIX |
| N4 | The toc `<nav>` contains a single `<ol>` with nested `<li><a href>` entries (and optionally a nested `<ol>`). Every `<li>` has an `<a>` (no `<span>`-only headings). | FIX (convert spans, flatten odd structures) |
| N5 | Every `<a href>` is relative to the nav file, and its target (before `#`) is a spine item. | FIX (drop entries whose target is not in the spine) |
| N6 | Every label is non-empty after trimming. | FIX (derive from target's first heading, else `Chapter N`) |
| N7 | Nav has at least one entry. If the source has no TOC at all, the fixer **synthesizes** one from the spine (first heading of each spine document, else `Chapter N`). | FIX |
| N8 | The NCX is removed: no manifest item with `application/x-dtbncx+xml`, no `spine/@toc`. | FIX (convert NCX to nav first, then STRIP) |

A book with an NCX only is the most common case. The fixer converts NCX `navPoint` trees into the nav `ol`/`li` structure, then removes the NCX.

### 2.6 Cover (V)

| ID | Requirement | On violation |
|---|---|---|
| V1 | Exactly one manifest item has `id="cover"`, and it is an image of type `image/jpeg`, `image/png`, or `image/webp`. | FIX |
| V2 | That item also has `properties="cover-image"`. | FIX |
| V3 | No other item has `properties="cover-image"`, no `<meta name="cover">` (removed after use). | FIX / STRIP |
| V4 | If the source has no cover, the fixer **generates a placeholder** (title text on a plain background) and adds it as `id="cover"`. | FIX |
| V5 | Cover dimensions can be decoded, and width and height are each within 100 to 10,000 px. | FIX (re-encode or placeholder) |

The fixer locates the real cover in this order: `properties="cover-image"`, `<meta name="cover">`, `<guide><reference type="cover">` (follow to the image if it points at an XHTML wrapper), then the first image in the first spine item, then placeholder. If another item already uses `id="cover"`, it is renamed first and all references to it are updated.

The cover is also extracted to the library store (thumbnail plus full size) at onboarding. The library grid never opens the EPUB.

### 2.7 Content documents (X)

| ID | Requirement | On violation |
|---|---|---|
| X1 | Every XHTML document is well-formed XML, UTF-8 encoded, no BOM, with an XML declaration or none (consistently). | FIX (transcode; repair via HTML5 parse + XHTML serialize) |
| X2 | Root element is `html` in the XHTML namespace, with a `body`. | FIX |
| X3 | Every document contains some text or an image (not an empty body). | WARN in log only; fine to admit |
| X4 | No `<script>`, `<iframe>`, `<object>`, `<embed>`, `<applet>`, `<form>`, `<meta http-equiv>`, `<base>`. | STRIP |
| X5 | No event-handler attributes (`on*`), no `javascript:` URLs, no `srcdoc`. | STRIP |
| X6 | No inline `<style>` or `style` attribute containing `@import`, `url()` with a remote or `javascript:` target, `expression(`, or `behavior:`. | STRIP |
| X7 | SVG, if present, is inline or an allowed image, and contains no `<script>`, `<foreignObject>`, event handlers, or external references. | STRIP |
| X8 | No `epub:type`-based or `epub:switch` constructs the reader does not handle. `epub:switch`/`epub:case` are replaced by their default branch. | FIX |
| X9 | MathML: allowed only if the reader supports it; otherwise treat like any unsupported element. | decide per reader (recommend: REJECT, or STRIP to alt text) |
| X10 | `<audio>` and `<video>` are not supported. | STRIP (keep fallback text) |

### 2.8 Resource references (R)

Covers `img/@src`, `source/@src`, `link/@href`, `image/@xlink:href`, CSS `url()`, and `@font-face`.

| ID | Requirement | On violation |
|---|---|---|
| R1 | All references are **relative** paths, or fragment-only (`#id`), or `data:` URIs for images of allowed types. | see below |
| R2 | No reference has a scheme (`http:`, `https:`, `ftp:`, `file:`, and so on) or protocol-relative form. | STRIP the reference (drop the `<img>` or `<link>`); keep any alt text |
| R3 | Every local reference resolves (relative to the referencing file) to an existing manifest item, without escaping the zip root. | STRIP dangling references |
| R4 | Referenced resources are of allowed types (section 3). | STRIP or convert |
| R5 | Every resource referenced by a content document is in the manifest. | FIX (add it) or STRIP the reference |
| R6 | Internal links `<a href="other.xhtml#frag">` point to a spine item. | FIX (keep the text, remove `href` if the target does not exist) |
| R7 | External links (`http:`, `https:`, `mailto:`) on `<a>` are allowed, and the reader must render them with `rel="noopener noreferrer"` and open them in a new tab. | no change |

### 2.9 CSS (Y)

Because the reader concatenates all CSS files (`GetCSS`) and applies them to the page, book CSS can restyle the app itself. The contract should be:

| ID | Requirement | On violation |
|---|---|---|
| Y1 | CSS files are UTF-8, parse without error, and have type `text/css`. | FIX (drop unparseable rules) |
| Y2 | No `@import`, no remote `url()`, no `expression()`, `behavior:`, or `-moz-binding`. | STRIP |
| Y3 | `url()` references are relative and resolve to allowed manifest resources (fonts/images). | STRIP dangling |
| Y4 | Only `@font-face`, `@media`, `@page`, and `@supports` at-rules are kept. | STRIP others |
| Y5 | **Scoping (recommended):** the fixer prefixes every selector with a fixed container selector (for example `.book-content`), and maps `html`/`body` selectors to that container, so book CSS cannot style the app chrome. Alternatively, the reader renders each chapter in a sandboxed shadow root or iframe. Pick one and make it a contract. | FIX |

### 2.10 Library metadata (L)

The onboarder extracts these into the DB at admission. The library never re-reads the EPUB.

| ID | Field | Rule |
|---|---|---|
| L1 | `title` | From `dc:title` (first), trimmed, non-empty (fixer guarantees). |
| L2 | `authors` | All `dc:creator` values, in order. May be empty. |
| L3 | `language` | `dc:language`, normalized to BCP 47 (`und` if unknown). |
| L4 | `identifier` | The `unique-identifier` value. |
| L5 | `series` / `series_index` | From `belongs-to-collection` or `calibre:series`. Optional. |
| L6 | `file_hash` | SHA-256 of the **original** upload. Used for duplicate detection together with `identifier`. |
| L7 | `chapter_count` | Number of spine items. |
| L8 | `profile_version` | Integer, set to the version of this document. |
| L9 | `opf_path` | Path of the OPF inside the zip. |

---

## 3. Allowed resource types

| Category | Allowed types | Notes |
|---|---|---|
| Content | `application/xhtml+xml` | Only in the spine (and the nav). |
| Styles | `text/css` | |
| Images | `image/jpeg`, `image/png`, `image/gif`, `image/webp`, `image/svg+xml` | SVG is sanitized per X7. Consider rasterizing or rejecting SVG if you prefer. |
| Fonts | `font/woff2`, `font/woff`, `font/ttf`, `font/otf` | Legacy types (`application/vnd.ms-opentype`, `application/x-font-ttf`) are normalized. |
| Everything else | (none) | STRIP from manifest and zip. Includes audio, video, JS, and embedded PDFs. |

The fixer determines type by **content sniffing first, extension second**, and rewrites the manifest `media-type` accordingly. The validator checks that the manifest type equals the sniffed type.

---

## 4. Reader contract (what the reader may assume)

For any admitted book:

1. `opf_path` (from the DB) points to a valid OPF. There is no need to read `container.xml`.
2. **Chapters = spine items**, in order, all linear, all XHTML, all present. `chapter_index` is an index into the spine.
3. The nav document is found by `id="nav"`, has one toc `<ol>`, every link targets a spine item, and every label is non-empty. Navigation entries are not chapters. The reader maps each entry to a spine index (plus optional fragment).
4. The cover is the manifest item `id="cover"`, always present, always JPEG/PNG/WebP. Normally the reader uses the pre-extracted cover from the library store instead.
5. Every content document is well-formed UTF-8 XML and contains no scripts, frames, forms, event handlers, or remote references.
6. Every resource reference is relative and resolves to a manifest item. The reader's link rewriting can rely on this and does not need a fallback for "outside the root".
7. CSS is sanitized and scoped, and the reader loads it as is.
8. Paths match the zip entry names exactly, so `GetFile` can use exact string lookup.

---

## 5. Suggested simplifications in the current reader code

These follow from the contract above. Nothing here is required for the validator itself.

- **`GetChapter` should index the spine, not the nav.** Today it uses `e.Nav[index].Href`. TOC entries are not chapters: several entries can point into the same file, entries can have fragments, and some files have no entry. With S-rules guaranteeing a clean spine, use `e.Spine[index]`.
- **Delete `findNavPath`'s fallbacks and the NCX branch in `GetToc`.** Find the nav by `id="nav"`. Remove `NavToc`, `NavPoint`, `Content`, and the `.ncx` branch. Look only for `<nav epub:type="toc">`.
- **Delete the multi-strategy `GetCover`.** Read the `id="cover"` item, or better, serve the pre-extracted cover from the library store.
- **Stop reparsing on every call.** `findOPFPath` and `parseOPF` each reopen the zip and are called repeatedly (`GetCover`, `LoadSpine`, `findNavPath`). Parse once at load time, or at onboarding, and store the result (for example, the spine list and TOC as JSON in the DB). The reader then only needs `GetFile`.
- **Open the zip once per request** (or cache the `zip.Reader`) instead of once per `GetFile`. `GetChapter` plus its resources currently opens the archive many times.
- **Drop the `.opf` suffix and `media-type` fallback in `findOPFPath`.** C2 guarantees exactly one rootfile with the right type.
- **Keep `rewriteResourceLinks`, but simplify.** R1 to R3 mean `isRewritableResource` only needs to skip `#`, `data:`, and external `<a>` links. The "drop if it escapes the root" branch becomes a defensive assertion rather than a real case. Because content is now well-formed XHTML, you may also choose to parse with an XML parser and serialize as XHTML, which keeps the reader and sanitizer on one parser.
- **Consider moving link rewriting into the onboarder.** If resource links point at a stable URL scheme, the reader can serve chapters verbatim. The catch is that the stored book then bakes in the URL scheme, which ties it to the route layout and `bookID`. Keeping the rewrite at read time is the more flexible option.
- **Drop the `linear="no"` skip in `LoadSpine`.** S4 guarantees there are none.
- **Serve files by manifest membership.** The `/file?file=` endpoint should serve only paths that are in the manifest (and only allowed types), not any zip entry, as a second line of defense behind M6.

---

## 6. Testing the contract

1. **Corpus tests.** Keep a directory of real-world EPUBs, including deliberately broken ones: EPUB 2 NCX-only, no TOC, no cover, multiple rootfiles, wrong media types, case-mismatched paths, backslash paths, BOMs, Latin-1 content, zip-slip, zip bomb, DRM, fixed layout, scripts and remote resources.
2. **Properties.** For every corpus file: `validate(fix(x))` passes or rejects with a stable reason code, and `fix(fix(x))` is byte-identical to `fix(x)` (use deterministic zip output: fixed timestamps, sorted entries).
3. **Validator unit tests.** One minimal canonical book per requirement ID, each with one violation, to prove the validator catches it. Use the IDs (Z1, N5, and so on) as stable reason codes in logs and user-facing errors.
4. **Reader tests.** Only need canonical books. If a reader test needs a non-canonical book, the contract is missing a requirement.
5. **Track rejections.** Log the reason code for every rejected upload. The most common REJECT codes tell you which fixer to write next.

---

## 7. Versioning

- Bump `profile_version` whenever a requirement is added, removed, or tightened.
- On bump: re-run `fix -> validate` on every stored **original**, write the new canonical copy, and only then switch the reader over. Books that now fail are flagged (not deleted) and keep their previous canonical copy until resolved.
- The reader supports exactly one `profile_version` at a time. The migration, not the reader, carries the compatibility burden.