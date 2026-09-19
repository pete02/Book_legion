# Tribune Logistica EPUB Contract

This document defines the contract that an EPUB must satisfy before Tribune Archivum may hand it to Tribune Logistica.

The contract is derived from the requirements and assumptions of `tribune_logistica/internal/epub`. It intentionally distinguishes **consumer requirements** from implementation details of the current Logistica implementation.

The purpose of this contract is not to define EPUB validity according to the EPUB specification. It defines the properties required for an EPUB to be safely and deterministically consumed by Tribune Logistica.

---

## 1. Contract Philosophy

Tribune Archivum is responsible for transforming arbitrary EPUB input into an EPUB that satisfies this contract.

Tribune Logistica must be able to assume that a book satisfying this contract is structurally usable.

The boundary is:

```text
                 Arbitrary EPUB
                       │
                       ▼
                Tribune Archivum
                       │
                       ▼
                 EPUB Contract
                    Validator
                       │
                  ┌────┴────┐
                  │         │
                 PASS      FAIL
                  │         │
                  ▼         ▼
              Logistica   Reject
```

Archivum must never rely on Logistica to repair, reconstruct, or guess missing structural information.

A validator failure must prevent the book from being passed to Logistica.

---

# 2. Requirement Classification

Requirements in this document use the following classifications.

### HARD REQUIREMENT

A property required for Logistica to process the book correctly.

Archivum must guarantee it.

### APPLICATION POLICY

A property required by the current Book Legion/Logistica application, but not inherently required for EPUB structural validity.

These requirements should be kept separate from the core EPUB contract where possible.

### IMPLEMENTATION DETAIL

A behavior of the current Logistica implementation that does not necessarily need to become a property of the EPUB produced by Archivum.

Implementation details should not become Archivum invariants merely because the current implementation happens to behave that way.

### DECISION REQUIRED

A currently observed behavior whose status as a true requirement has not yet been established.

It must be resolved before becoming a hard Archivum invariant.

---

# 3. EPUB Container

## 3.1 `META-INF/container.xml`

**Classification: HARD REQUIREMENT**

The EPUB archive must contain:

```text
META-INF/container.xml
```

The file must:

* be readable from the archive;
* be valid XML;
* contain at least one usable `<rootfile>`;
* identify the OPF package document.

The preferred rootfile is one whose:

```text
media-type="application/oebps-package+xml"
```

Alternatively, Logistica may accept a rootfile whose `full-path` ends in `.opf`.

Archivum should preferably normalize the output to a single unambiguous OPF rootfile.

---

# 4. Package Document

## 4.1 OPF Location

**Classification: HARD REQUIREMENT**

The OPF package document referenced by `container.xml` must:

* exist in the archive;
* be accessible using the path specified by the container;
* be valid XML;
* be parseable by the XML parser used by Logistica.

---

## 4.2 Manifest

**Classification: HARD REQUIREMENT**

The OPF must contain a manifest.

Every manifest item used by Logistica must have sufficient information to identify its resource, including:

* unique `id`;
* `href`;
* `media-type`.

Manifest IDs must be unique.

Referenced resources must resolve relative to the OPF's location.

---

## 4.3 Spine

**Classification: HARD REQUIREMENT**

The OPF must contain a spine with at least one usable linear item.

A spine item is usable when:

* it contains an `idref`;
* the `idref` resolves to exactly one manifest item;
* the referenced manifest resource exists in the archive;
* the resource can be consumed by Logistica.

Spine items with:

```xml
linear="no"
```

are not part of the primary reading order.

An omitted `linear` attribute is treated as linear.

After excluding non-linear items, at least one linear item must remain.

---

# 5. Reading Order

## 5.1 Spine → Manifest Consistency

**Classification: HARD REQUIREMENT**

Every linear spine `idref` must resolve to a manifest item.

A dangling `idref` is a validation failure.

---

## 5.2 Manifest → Archive Consistency

**Classification: HARD REQUIREMENT**

Every manifest resource required by the reading order must exist in the EPUB archive.

Resource paths must resolve relative to the OPF location.

---

## 5.3 Reading Order Determinism

**Classification: HARD REQUIREMENT**

The order of linear spine items must define a deterministic reading order.

Archivum must not produce ambiguous or duplicate reading-order entries.

---

# 6. Navigation

Navigation is required because Logistica exposes chapter-level navigation to the reader.

## 6.1 Usable Navigation

**Classification: HARD REQUIREMENT**

The resulting EPUB must provide a usable navigation structure containing at least one chapter entry.

Logistica currently supports two navigation mechanisms:

1. EPUB 3 Navigation Document
2. EPUB 2 NCX

Archivum may accept either as input but should normalize them where practical.

---

## 6.2 EPUB 3 Navigation

**Classification: HARD REQUIREMENT when EPUB 3 navigation is used**

A navigation document must contain a table of contents that Logistica can identify.

The relevant navigation structure must contain:

* a TOC `<nav>`;
* an ordered list;
* navigation entries containing links;
* usable `href` values.

Logistica currently accepts several tolerant HTML structures.

Archivum does not need to preserve malformed navigation merely because Logistica can currently tolerate it.

---

## 6.3 EPUB 2 NCX

**Classification: HARD REQUIREMENT when NCX navigation is used**

An NCX navigation document must:

* exist in the archive;
* be valid XML;
* contain a usable `<navMap>`;
* contain navigation points;
* provide usable chapter targets.

The NCX may be identified through the manifest or the spine's `toc` attribute.

---

## 6.4 Navigation Targets

**Classification: HARD REQUIREMENT**

Each navigation entry used by Logistica must have:

* a non-empty target;
* a target that resolves within the EPUB;
* a target document that exists;
* a target document that Logistica can consume.

Fragment-only links may be ignored when they do not identify a chapter/document.

If navigation contains no usable entries, validation fails.

Navigation labels may be empty from a structural perspective, although Archivum should preserve or construct useful labels whenever possible.

---

# 7. Chapter Documents

## 7.1 Chapter Existence

**Classification: HARD REQUIREMENT**

Every document appearing in the linear reading order must exist in the archive.

---

## 7.2 Chapter Parsing

**Classification: HARD REQUIREMENT**

Every linear chapter document must be parseable by the HTML parser used by Logistica:

```text
golang.org/x/net/html
```

The validator must therefore test parsing using the same parser or an equivalent parser with compatible behavior.

Strict XML validity is not required for HTML/XHTML chapter documents if Logistica itself consumes them using the HTML parser.

Malformed HTML that the parser can safely recover from is therefore not automatically a validation failure.

---

## 7.3 Chapter Resource Resolution

**Classification: HARD REQUIREMENT**

Resources referenced from chapter documents must resolve according to the chapter document's directory.

This applies to resources consumed by Logistica, including:

* images;
* stylesheets;
* media resources where applicable;
* inline SVG image references.

---

# 8. Resource Paths

## 8.1 EPUB Root Boundary

**Classification: HARD REQUIREMENT**

No resource reference consumed by Logistica may resolve outside the EPUB archive root.

Path traversal such as:

```text
../
../../
```

must not allow access outside the EPUB.

Archivum must produce paths that resolve unambiguously within the EPUB.

---

## 8.2 External Resources

**Classification: CURRENT IMPLEMENTATION DETAIL**

The current Logistica implementation leaves certain references unchanged, including:

* absolute URLs;
* `https://...`;
* `mailto:...`;
* data URIs;
* fragment-only references.

This behavior should not automatically become an Archivum invariant.

Archivum should only need to guarantee correct handling of resources that Logistica is expected to serve from the EPUB.

---

## 8.3 Resource Rewriting

**Classification: IMPLEMENTATION DETAIL**

Logistica currently rewrites certain resource references into API endpoints.

This is a responsibility of Logistica's serving layer and is not, by itself, an EPUB invariant.

Archivum must instead guarantee that the underlying resource references are resolvable and structurally coherent.

---

# 9. File Paths

## 9.1 Case Sensitivity

**Classification: HARD REQUIREMENT**

Resource resolution must be deterministic and must respect the case-sensitive semantics of the EPUB archive.

Archivum must not rely on case-insensitive filesystem behavior.

---

## 9.2 Path Normalization

**Classification: HARD REQUIREMENT**

Paths must be normalized sufficiently that:

* equivalent relative paths resolve consistently;
* traversal outside the EPUB root is impossible;
* references resolve to the intended archive entry;
* path interpretation does not depend on the host filesystem.

---

# 10. Encoding

## 10.1 XML

**Classification: HARD REQUIREMENT**

OPF and NCX documents must be parseable by Go's XML parser as used by Logistica.

The XML encoding declaration must be compatible with the actual document encoding.

Archivum should normalize XML output to a predictable encoding where practical.

---

## 10.2 HTML

**Classification: HARD REQUIREMENT**

Chapter documents must be parseable by Logistica's HTML parser.

Strict XML conformance is not required.

---

# 11. Cover

## 11.1 Cover Availability

**Classification: APPLICATION POLICY**

The current `GetCover()` implementation expects a cover image and returns an error when none can be found.

However, the absence of a cover does not necessarily make an EPUB structurally unusable.

As the application itself needs the covers, the validation should check that we can find a usable cover


---

## 11.2 Cover Discovery

The current implementation searches for covers in this order:

1. EPUB 3 `properties="cover-image"`
2. EPUB 2 `<meta name="cover">`
3. EPUB guide cover reference

If cover presence becomes an application requirement, Archivum should normalize valid cover information so that Logistica can discover it deterministically.

---

## 11.3 Supported Cover Formats

The current implementation recognizes:

* PNG;
* JPEG;
* GIF;
* SVG;
* WebP.

**Classification: HARD REQUIREMENT**

---

# 12. CSS

## 12.1 CSS Availability

**Classification: DECISION REQUIRED**

The current `GetCSS()` implementation returns an error when no CSS files are found.

This does not necessarily mean that a book without external CSS is unusable.

Determine whether Logistica requires at least one external CSS file.

If not, no-CSS EPUBs should remain valid.

---

## 12.2 CSS Ordering

The current implementation:

1. collects CSS files;
2. sorts them alphabetically;
3. concatenates them.

**Classification: IMPLEMENTATION DETAIL**

This must not become an Archivum invariant unless Logistica's behavior demonstrably depends upon that ordering.

If CSS ordering is semantically important, that dependency should instead be documented explicitly.

---

# 13. Cross-Structure Consistency

These checks are particularly important because an EPUB may contain individually valid components that do not agree with each other.

## 13.1 Container → OPF

**HARD REQUIREMENT**

The OPF identified by `container.xml` must exist and be parseable.

---

## 13.2 Spine → Manifest

**HARD REQUIREMENT**

Every linear spine reference must resolve to exactly one manifest item.

---

## 13.3 Manifest → Archive

**HARD REQUIREMENT**

Every resource required by the reading order must exist at the path declared by its manifest item.

---

## 13.4 Navigation → Documents

**HARD REQUIREMENT**

Navigation targets must resolve to documents that exist and can be consumed by Logistica.

---

## 13.5 Navigation → Reading Order

**DECISION REQUIRED**

Determine whether every navigation entry must correspond to a linear spine document.

If Logistica permits navigation to auxiliary/non-linear documents, those documents must not automatically be rejected.

The validator should enforce only what Logistica actually requires.

---

# 14. Archive Integrity

**Classification: HARD REQUIREMENT**

The input/output EPUB must be a readable ZIP archive.

The validator must reject:

* invalid ZIP archives;
* unreadable archive entries;
* required files that cannot be extracted/read;
* structurally inconsistent archive data that prevents required resources from being accessed.

The EPUB should not contain ambiguous duplicate paths that could cause different ZIP readers to select different resources.

---

# 15. Validator Guarantees

The validator itself is part of the Archivum architecture.

A successful validation means:

> Tribune Logistica may consume the EPUB without needing to reconstruct missing structural information or recover from a known contract violation.

The validator must:

* be deterministic;
* produce structured diagnostics;
* distinguish errors from warnings;
* identify the affected EPUB component;
* identify the relevant path/ID/reference where possible;
* never silently ignore a known hard-contract violation.

Example:

```text
ValidationError {
    category: "spine"
    code: "MISSING_MANIFEST_ITEM"
    location: "content.opf"
    reference: "chapter-17"
    message: "Spine itemref references manifest ID 'chapter-17', but no such manifest item exists."
}
```

---

# 16. Validator vs Repair

The validator must **not repair the EPUB**.

Its job is to answer:

```text
"Does this EPUB satisfy the Logistica contract?"
```

It must not answer:

```text
"Can I make this EPUB satisfy the contract?"
```

That second question belongs to Archivum's analysis and reconstruction stages.

The intended processing model is:

```text
Raw EPUB
   │
   ▼
Analysis
   │
   ▼
Reconstruction / Repair
   │
   ▼
Normalization
   │
   ▼
Validation
   │
   ├── PASS ──► Logistica
   │
   └── FAIL ──► Quarantine / Diagnostic Report
```

---

# 17. Archivum Output Requirements

An EPUB successfully produced by Archivum must:

1. satisfy every HARD REQUIREMENT;
2. satisfy applicable APPLICATION POLICY requirements;
3. be deterministic;
4. be safe to process repeatedly;
5. contain no unresolved structural problems known to violate the contract.

Archivum must not declare an EPUB successful merely because it was able to produce an output file.

The validator is the final authority at the Archivum/Logistica boundary.

---

# 18. Idempotency

**Classification: ARCHIVUM DESIGN REQUIREMENT**

Where practical, Archivum should be idempotent.

Given:

```text
Archivum(A) = B
```

processing `B` again should result in an equivalent canonical EPUB:

```text
Archivum(B) = B'
```

where `B'` is semantically equivalent to `B` and does not require additional repairs.

A normalized EPUB should not continuously accumulate changes every time it passes through Archivum.

---

# 19. Determinism

**Classification: ARCHIVUM DESIGN REQUIREMENT**

Given the same:

* input EPUB;
* Archivum version;
* configuration;

Archivum should produce the same semantic result.

Where byte-for-byte determinism is practical, it should be preferred.

Non-deterministic repair behavior must not affect whether a book passes validation.

---

# 20. Explicitly Not Part of the Contract

The following must not become hard invariants merely because they exist in the current Logistica implementation:

* alphabetical CSS ordering;
* presence of external CSS, unless Book Legion requires it;
* preservation of malformed HTML that Logistica happens to tolerate;
* specific API URL formats used for resource rewriting;
* internal cache behavior;
* mutex usage;
* garbage-collection finalizers;
* the current `epubCache` implementation;
* the current method by which navigation is discovered;
* arbitrary filesystem layout;
* arbitrary archive file ordering.

These are implementation concerns unless demonstrated otherwise.

---

# 21. Open Decisions

Before the validator is finalized, resolve:

* [ ] Is a cover mandatory for every Book Legion book? -> yes
* [ ] Is at least one external CSS file mandatory? No, it is not
* [ ] Must every navigation target be part of the linear spine? Yes
* [ ] Are auxiliary/non-linear documents allowed as navigation targets? yes
* [ ] Which manifest media types can Logistica actually consume? 
* [ ] Does Logistica require navigation to exactly match the spine? yes
* [ ] How should duplicate archive paths be handled? No paths should be duplicated
* [ ] How should duplicate manifest IDs be handled?  no manifest id's should be duplicated
* [ ] Are malformed-but-parser-tolerated navigation documents acceptable as Archivum output? yes, as long as logistica can use them
* [ ] Which EPUB 2/3 structures should Archivum normalize rather than preserve? Not a validator concern
* [ ] Are there Logistica behaviors that should be changed instead of encoded as Archivum requirements? No

---

# 22. Test Corpus Requirement

The contract must be tested against real EPUBs.

The test corpus should contain:

* known-good EPUBs;
* the EPUBs currently present in the library;
* the problematic EPUBs collected from real-world use;
* every EPUB that historically required manual intervention.

Each fixture should preserve the original input.

Suggested structure:

```text
fixtures/
├── epub-001/
│   ├── original.epub
│   ├── expected.md
│   └── ...
├── epub-002/
│   ├── original.epub
│   ├── expected.md
│   └── ...
└── ...
```

The original EPUB must never be modified by the test process.

The corpus is the empirical specification for Archivum's real-world behavior.

---

# 23. Success Criterion

Tribune Archivum 2.0 is successful when:

> Arbitrary EPUB input can either be transformed into an EPUB satisfying this contract, or rejected with a precise diagnostic explaining why safe reconstruction was not possible.

The system must never silently produce an EPUB that Logistica cannot reliably consume.
