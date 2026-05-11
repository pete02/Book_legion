# Lector — Page Turn Renderer: Specification

**Version:** 1.2  
**Status:** Ready for test specification  
**Scope:** Viewport-based page navigation for EPUB chapter HTML content

---

## 1. Overview

Lector renders EPUB chapters — which are ordinary HTML documents with no intrinsic pagination — as a series of "pages", where each page is defined as the content that fits within the current viewport height. The user navigates forward and backward using Next and Prev buttons. There is no scrolling.

The renderer must:
- Divide chapter HTML into viewport-sized pages at clean sentence boundaries
- Never display broken or unclosed HTML
- Persist reading position across sessions
- Handle chapter transitions seamlessly

---

## 2. Definitions

| Term | Definition |
|---|---|
| **Page** | The portion of a chapter's HTML content that fits within the viewport height at a given font size and layout. Not a fixed unit — reflows on resize. |
| **Viewport** | The visible reading area. Height is obtained at runtime from the DOM. |
| **Page boundary** | The character offset in the raw HTML string at which one page ends and the next begins. Always falls at a sentence boundary. |
| **Sentence boundary** | The position immediately after a sentence-ending character (`.`, `!`, `?`) followed by optional closing punctuation or whitespace. See §5.1 for full rules. |
| **Page offset** | A `usize` character index into the raw HTML string marking the start of a page. |
| **Healed HTML** | A slice of the chapter HTML that has been post-processed so all opened tags are properly closed. |
| **Cursor** | A persisted struct of `(chapter_idx, page_offset)` identifying the reader's last position. |

---

## 3. Core Design Principles

| Principle | Requirement |
|---|---|
| **No user scroll** | `#book-container` MUST use `overflow: hidden` and `height: 100vh`. The native browser scrollbar MUST be disabled. |
| **Button-driven navigation** | All navigation MUST be triggered by Next / Prev buttons. Keyboard and swipe are out of scope for v1. |
| **DOM-driven layout** | Viewport height MUST be obtained from the live DOM via `getBoundingClientRect`. It MUST NOT be hardcoded or estimated. |
| **Sentence-boundary cuts** | Page boundaries MUST fall at sentence boundaries (§5.1). Cuts mid-word or mid-sentence are not permitted. |
| **HTML healing** | Every page slice MUST be healed before rendering (§5.2). Broken or unclosed tags MUST NOT appear in the viewport. |
| **Virtualization** | Only the current page's healed HTML MUST be in the live DOM. Prior and future page content MUST NOT be rendered. |
| **Signal-based state** | State MUST be managed with local Dioxus Signals. Global mutable state (`TextHandler` or equivalent) MUST NOT be used. |

---

## 4. Module Structure

```
src/renderer/
├── page_view.rs     # Top-level Dioxus component; navigation logic; state hook
├── layout.rs        # Page boundary calculation (DOM measurement + sentence snapping)
├── html_healer.rs   # HTML tag balancing and closing for arbitrary slices
└── cursor.rs        # Serialize / deserialize reading position to/from storage
```

---

## 5. Functional Requirements

### 5.1 Sentence Boundary Detection

A **sentence boundary** is defined as a position in the text (not inside a tag) that satisfies all of the following:

1. The preceding non-whitespace character is one of: `.` `!` `?`
2. Optionally followed by one or more closing punctuation characters: `)` `]` `"` `'` `»`
3. Followed by whitespace or end-of-string

**Additional rules:**

- Abbreviations MUST NOT be treated as sentence boundaries. The abbreviation list is hardcoded in `layout.rs` and is the single source of truth. It MUST include at minimum:

  | Category | Entries |
  |---|---|
  | Titles | `Mr`, `Mrs`, `Ms`, `Dr`, `Prof`, `Rev`, `Sr`, `Jr`, `Sgt`, `Cpl`, `Pvt`, `Gov`, `Pres` |
  | Academic | `Ph`, `B`, `M`, `D` (as in `Ph.D`, `B.Sc`, `M.A`) |
  | Latin | `e.g`, `i.e`, `etc`, `cf`, `vs`, `al` (as in `et al`) |
  | Geographic | `St`, `Ave`, `Blvd`, `Rd`, `Mt`, `Ft` |
  | Months | `Jan`, `Feb`, `Mar`, `Apr`, `Jun`, `Jul`, `Aug`, `Sep`, `Oct`, `Nov`, `Dec` |
  | Other common | `No`, `Vol`, `Fig`, `Dept`, `approx`, `est`, `max`, `min` |

  The list is checked case-insensitively against the word immediately preceding the `.`. New entries MAY be added without a spec revision.
- Ellipses (`...` or `…`) MUST NOT be treated as sentence boundaries.
- A decimal number (e.g. `3.14`) MUST NOT be treated as a sentence boundary.
- Detection MUST operate on text nodes only; character offsets inside HTML tag markup (between `<` and `>`) MUST be skipped entirely.
- If no sentence boundary is found before the viewport fills, the renderer MUST snap back to the last sentence boundary found so far. If no sentence boundary was found at all in the current search range, the renderer MUST snap to the nearest preceding paragraph end tag (`</p>`, `</div>`, `</li>`, `</blockquote>`).

### 5.2 HTML Healing

When a raw HTML slice `html[start..end]` is extracted, it may contain:
- Tags opened before `end` but not closed within the slice
- End tags whose matching open tag is before `start`

**Healing rules:**

1. Parse the slice with a tag-stack parser (not regex). For every open tag pushed onto the stack that has no matching close tag within the slice, append the corresponding close tag at the end of the slice, in reverse stack order (innermost first).
2. Orphaned close tags (whose open tag is before `start`) MUST be silently dropped; they MUST NOT appear in the healed output.
3. Void elements (`<br>`, `<img>`, `<hr>`, `<input>`, `<link>`, `<meta>`) MUST NOT be pushed onto the tag stack and MUST NOT generate synthetic close tags.
4. Healing MUST be performed on the slice AFTER the sentence boundary has been determined, not before.
5. The healed slice MUST be semantically equivalent to the original content for the range it covers: healing MUST only add closing tags, never modify or remove content.

**Example:**

Raw chapter HTML (abbreviated):
```html
<div class="chapter"><p>He ran. <em>Fast.</em> She followed.</p><p>Next paragraph.</p></div>
```

Slice `[0..55]` (cuts inside `</p>`):
```
<div class="chapter"><p>He ran. <em>Fast.</em> She followed.
```

Healed output:
```html
<div class="chapter"><p>He ran. <em>Fast.</em> She followed.</p></div>
```

*(The slice was snapped to the sentence boundary after "followed." and then healed.)*

### 5.3 Trailing End-Tag Inclusion

After snapping to a sentence boundary, the renderer MUST scan forward in the raw HTML and include any end tags that immediately follow the boundary position in the current page slice, before healing.

**Rationale:** If a sentence is the last in a `<p>` or `<blockquote>`, the closing tag logically belongs with that sentence, not with the next page.

**Whitespace definition for this section:**

For the purposes of trailing end-tag inclusion, the following are all treated as whitespace — i.e. they are skipped over transparently when scanning for end tags:
- ASCII whitespace: space, tab, newline, carriage return
- HTML comments: `<!-- ... -->`
- XML processing instructions: `<? ... ?>`
- Any other content that produces no visible output when rendered by the browser

**Algorithm:**

1. After finding sentence boundary position `end`, scan forward from `end`.
2. Skip any whitespace (as defined above).
3. While the next token (after skipping whitespace) is an HTML end tag (`</tagname>`), advance `end` past it and return to step 2.
4. Stop when the next non-whitespace token is not an end tag.
5. Use the updated `end` as the slice end before healing.

### 5.4 Page Boundary Calculation

**Input:** Raw chapter HTML string, start page offset.  
**Output:** `(start_offset, end_offset)` — both character indices into the raw HTML.

**Height measurement:**

The usable page height is obtained from `#book-container` using `clientHeight`, which returns the inner height of the element excluding border and scrollbar but including padding. To avoid content being clipped at the bottom edge, a tunable constant `PAGE_HEIGHT_FUDGE_FACTOR: f64` (default `0.0`, negative values reduce the usable height) is subtracted:

```
usable_height = book_container.client_height() as f64 + PAGE_HEIGHT_FUDGE_FACTOR
```

`PAGE_HEIGHT_FUDGE_FACTOR` MUST be defined as a named constant in `layout.rs` and adjusted manually if clipping is observed. The primary approach is to rely on `clientHeight` alone (`PAGE_HEIGHT_FUDGE_FACTOR = 0.0`); the constant is a fallback for cases where padding or border causes clipping.

**Measurement approach:**

Page boundary calculation uses `#book-container` itself as the measurement surface. Tokens are appended to the container, `clientHeight` is sampled after each token, and the container is restored to its pre-calculation state before rendering the final page. This avoids the need for a separate off-screen measurement element and ensures the layout context is identical to the live reading view.

**Forward page calculation algorithm:**

1. Record the current innerHTML of `#book-container` so it can be restored.
2. Clear `#book-container`.
3. Obtain `usable_height` as above.
4. Starting from `start_offset`, iterate over HTML tokens (open tag, close tag, void tag, text node) using the forward token grammar.
5. For each token, append it to `#book-container` and measure `clientHeight`.
6. When `clientHeight` exceeds `usable_height`, the boundary has been crossed. Record the character offset immediately before the token that caused the overflow.
7. Snap the recorded offset **backward** to the last sentence boundary seen before the overflow point (§5.1).
8. Apply trailing end-tag inclusion (§5.3).
9. Restore `#book-container` to its recorded innerHTML.
10. Return `(start_offset, snapped_end_offset)`.

**Forward token grammar:**

Scanning forward from a position, the next token is identified as:
- If the current character is `<` and the next is `/`: scan forward to `>` — this is a **close tag**.
- If the current character is `<`: scan forward to `>` — this is an **open tag** or **void tag** (void if the tag name matches the void element list in §5.2).
- Otherwise: scan forward to the next `<` or end of string — this is a **text node**.

**Edge cases:**

| Case | Behaviour |
|---|---|
| Chapter is shorter than one viewport | `end_offset` = `html.len()`. Page is the entire chapter. |
| No sentence boundary found in entire page | Snap to the last paragraph-end tag found (`</p>`, `</div>`, `</li>`, `</blockquote>`); if none, use `html.len()`. Log a warning. |
| Single node taller than viewport (e.g. a large `<img>`) | Include the node on its own page regardless of height overflow. |
| Empty chapter HTML | Render an empty container. Do not panic. |
| Malformed HTML (unclosed tags in source) | The healer handles this; the boundary calculator MUST NOT crash on malformed input. |

### 5.5 Navigation

#### 5.5.1 Next Page

1. Compute `new_start = current_end_offset` (the end of the current page).
2. If `new_start >= html.len()`:
   - If there is a next chapter: load it, set `page_offset = 0`.
   - If there is no next chapter: do nothing; hide the Next button.
3. Otherwise: calculate the new page boundaries from `new_start` and re-render.

#### 5.5.2 Prev Page

Backward navigation MUST be calculated in reverse from the current position — it MUST NOT recompute pages from offset 0 forward.

**Reverse token grammar:**

Scanning backwards, a token is identified as follows:

1. Scan backward from the current position until a `>` character is found.
2. If `>` is found, continue scanning backward until either `</` or `<` is found.
   - If `</` is found: the token is a **close tag**. The tag name is the text between `</` and `>`.
   - If `<` is found (without a preceding `/`): the token is an **open tag** or **void tag**. The tag name is the text between `<` and the first whitespace or `>`.
3. If no `<` or `</` is found before the scan start: the token is a **text node** running from the scan start back to the previous `>` (or to the beginning of the string).

This grammar produces the same token types as forward scanning (open tag, close tag, void tag, text node) and MUST be used consistently throughout the reverse walk.

**Snap direction invariant:**

The snap direction MUST always match the scan direction:
- **Forward scan** (§5.4): when overflow is detected, snap **backward** to the last sentence boundary seen before the overflow point.
- **Backward scan** (§5.5.2): when overflow is detected, snap **forward** to the first sentence boundary encountered while walking backward — i.e. the nearest sentence boundary behind the overflow point in the original text.

This ensures that both scans converge on the same sentence boundary from opposite sides. A small variance of one sentence between the forward and backward result is acceptable and MUST NOT be treated as a bug, because the incremental DOM insertion order may cause the overflow threshold to trigger at a slightly different point.

**Round-trip guard:**

After the backward snap produces `new_start`, a forward page calculation from `new_start` MUST produce a `new_end` that is ≥ `current_start_offset`. If `new_end` < `current_start_offset`, the snap has overshot; in this case, advance `new_start` to the next sentence boundary and retry the forward check once. Log a warning if the guard fires.

**Reverse page calculation algorithm:**

The key insight is that a page boundary is a sentence boundary at which the accumulated rendered height fits within the viewport. To find the page that ends at `current_start_offset`, we work backwards through the HTML:

1. If `current_start_offset == 0`:
   - If there is a previous chapter: load it and seek to its last page (§5.5.3).
   - If there is no previous chapter: do nothing; hide the Prev button.
2. Otherwise:
   a. Set `search_end = current_start_offset`. This is the exclusive upper bound of the page we want to find.
   b. Walk backwards through the raw HTML from `search_end` using the reverse token grammar above.
   c. For each token, prepend it to the front of `#book-container` (inserting before existing content) and measure `clientHeight`.
   d. When `clientHeight` exceeds the usable container height (see §5.4 for height measurement), the boundary has been crossed. Record the character offset immediately after the token that caused the overflow — this is the candidate `new_start`.
   e. Snap `new_start` forward to the next sentence boundary (§5.1) — the first boundary encountered while the scan was walking backward.
   f. Apply the round-trip guard above.
   g. Apply trailing end-tag inclusion in reverse: if `new_start` is preceded immediately by end tags (scanning backward past optional whitespace), advance `new_start` forward past those end tags so they remain on the previous page.
   h. Restore `#book-container` to the current page content (undo the prepended tokens).
   i. Set `current_page_start_offset = new_start`, recalculate `current_page_end_offset` forward from `new_start` (§5.4), and re-render.

**Edge cases specific to backward calculation:**

| Case | Behaviour |
|---|---|
| Reverse search reaches offset 0 before filling the viewport | `new_start = 0`. This is the first page of the chapter. |
| No sentence boundary found while walking backward | Snap to the nearest paragraph-end tag encountered during the walk; if none, use offset 0. |
| Single oversized node encountered while walking backward | Include the node on its own page regardless of height overflow. |
| Round-trip guard fires | Advance `new_start` one sentence boundary forward, log a warning, proceed. |

#### 5.5.3 Seeking to Last Page of a Chapter

When loading a previous chapter, the renderer must display its last page. Use the reverse page calculation (§5.5.2) starting from `html.len()` — i.e. treat the end of the chapter as `search_end` and calculate one page backward. This produces the `start_offset` of the last page directly, without iterating forward through the entire chapter.

#### 5.5.4 Button Visibility

| Condition | Prev button | Next button |
|---|---|---|
| First page of first chapter | Hidden | Visible |
| Last page of last chapter | Visible | Hidden |
| Any other position | Visible | Visible |
| Loading in progress | Both disabled (not hidden) |

### 5.6 State Model

```rust
#[derive(Clone, PartialEq, Debug)]
pub struct PageViewState {
    pub book_id:                    String,
    pub chapter_idx:                usize,
    pub chapter_html:               Option<String>,  // Raw, unparsed HTML for the chapter
    pub current_page_start_offset:  usize,           // Char index: start of current page
    pub current_page_end_offset:    usize,           // Char index: end of current page
    pub total_chapters:             usize,
    pub loading:                    bool,
    pub error:                      Option<String>,
}
```

The `use_page_view` hook MUST:
- Accept `book_id: String` as its sole parameter.
- Initialize state from the cursor (§5.7) on first render.
- Expose signals for all state fields.
- Expose `go_next()` and `go_prev()` callbacks.

### 5.7 Cursor (Reading Position Persistence)

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cursor {
    pub book_id:     String,
    pub chapter_idx: usize,
    pub page_offset: usize,  // == current_page_start_offset
}
```

**Persistence rules:**

- Save: debounced, at most once every **5 seconds**, and immediately on chapter change.
- Load: on component mount, before first render.
- Storage backend: `infra::cursor` (localStorage or platform equivalent). The renderer MUST NOT depend on storage implementation details.
- If no cursor exists for a book: start at `chapter_idx = 0`, `page_offset = 0`.
- If stored `page_offset` falls inside an HTML tag (due to a content update): snap forward to the next sentence boundary before rendering.

### 5.8 Chapter Loading

- Use `infra::chapters::fetch_chapter(book_id, chapter_idx)` to retrieve HTML.
- On receipt, check whether the HTML string contains any encoded entities: `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&nbsp;`, or any numeric entity matching `&#...;` or `&x...;`.
  - If encoded entities are found: decode the entire string using `html_escape::decode` (or equivalent) to produce a plain Unicode string.
  - If no encoded entities are found: use the string as-is.
- Store the decoded string in `PageViewState.chapter_html` and persist it to the chapter cache so that decoding only occurs once per chapter fetch.
- All downstream modules (boundary calculator, HTML healer, sentence detector) operate exclusively on the decoded Unicode string. No entity handling is required in those modules.
- On success: store the decoded HTML in `PageViewState.chapter_html`.
- On failure: set `PageViewState.error` with a user-readable message; do not crash.
- Retry policy: one automatic retry after 2 seconds for network errors; surface error to user after second failure.
- Set `loading = true` during fetch; set `loading = false` on completion (success or failure).

---

## 6. Non-Functional Requirements

| Category | Requirement |
|---|---|
| **Performance** | Page boundary calculation MUST complete within 100 ms on a mid-range device for chapters up to 200 KB of HTML. |
| **Correctness** | Healed HTML MUST pass validation (no unclosed tags, no orphaned close tags) on 100% of well-formed EPUB chapter inputs. |
| **Resilience** | The renderer MUST NOT panic or crash on malformed HTML, empty chapters, or storage failures. |
| **Accessibility** | Next / Prev buttons MUST have accessible labels (`aria-label="Next page"` / `aria-label="Previous page"`). |
| **Resize handling** | On viewport resize, the current page MUST be immediately re-calculated from `current_page_start_offset` — no debounce. Recalculation is also triggered on every Next / Prev button press, so the viewport height is always sampled fresh. The user MUST remain at the same `start_offset` after a resize; only `end_offset` changes. |

---

## 7. CSS Requirements

```css
#book-container {
    overflow: hidden;
    height: 100vh;
    position: relative;
    /* Font, line-height, and padding defined by theme.
       clientHeight is used for measurement — padding is included in clientHeight,
       so no separate measurement element is needed. */
}

#button-overlay {
    position: fixed;
    bottom: 1rem;
    width: 100%;
    display: flex;
    justify-content: space-between;
    pointer-events: none; /* children re-enable this */
}

#prev-btn, #next-btn {
    pointer-events: all;
}
```

There is no off-screen measurement element. `#book-container` is used directly for layout measurement (see §5.4). If content clipping is observed, adjust `PAGE_HEIGHT_FUDGE_FACTOR` in `layout.rs` before changing CSS.

---

## 8. Error States

| Condition | User-visible behaviour |
|---|---|
| Chapter fetch fails | Display inline error message with a Retry button. Do not clear the previous page content. |
| Chapter HTML is empty | Display "This section appears to be empty." and show navigation buttons to move away. |
| Cursor points past end of chapter | Log a warning, reset to `page_offset = 0` for that chapter. |
| HTML healer encounters unexpected input | Log the input, return the slice as-is with best-effort closing tags. Do not panic. |

---

## 9. Out of Scope (v1)

- Keyboard navigation (arrow keys, Page Up/Down)
- Touch swipe gestures
- Reflowable font-size changes during a session (requires page recalculation; v2)
- Multiple columns
- Footnote / endnote rendering
- Text search / find-in-book
- Annotations and highlights
- RTL (right-to-left) text — not required for Book App V4; to be considered in a future version
- Cross-browser testing — the renderer is used by a single operator who will test on their own devices

---

## 10. Resolved Design Decisions

| # | Question | Decision |
|---|---|---|
| 1 | Backward navigation strategy | Reverse token walk from `current_start_offset` using the reverse token grammar (§5.5.2). No forward recompute from offset 0. |
| 2 | Snap direction and round-trip invariant | Snap always follows scan direction: backward snap on forward scan, forward snap on backward scan. Round-trip guard defined in §5.5.2. Small one-sentence variance is acceptable. |
| 3 | Measurement surface | `#book-container` used directly. No off-screen div. `clientHeight` is the measurement API (§5.4). |
| 4 | Height box model and clipping | `clientHeight` is the primary measure. `PAGE_HEIGHT_FUDGE_FACTOR` constant in `layout.rs` provides manual tuning if clipping occurs (§5.4). |
| 5 | HTML entity decoding | Check-first on fetch: decode with `html_escape` only if entities are detected. Store decoded Unicode; all downstream modules operate on decoded text (§5.8). |
| 6 | Non-visible content in trailing end-tag scan | HTML comments, processing instructions, and ASCII whitespace are all treated as transparent whitespace during the trailing end-tag scan (§5.3). |
| 7 | Abbreviation list maintenance | Hardcoded in `layout.rs`. List defined in §5.1. Extensible without spec revision. |
| 8 | Cross-browser / device testing | Manual testing by the operator on their own devices. No automated cross-browser requirement. |
| 9 | Resize recalculation timing | Immediate on resize event and on every button press. No debounce. |
| 10 | RTL text support | Out of scope for Book App V4. |

---

## 11. Migration from TextHandler

1. Implement the new renderer in `src/renderer/` without modifying existing code.
2. Gate the new renderer behind a `USE_NEW_RENDERER` compile-time feature flag.
3. Run both renderers in parallel on the same test corpus; compare output.
4. Once parity is confirmed, enable by default and deprecate `TextHandler`.
5. Remove `TextHandler` after one release cycle.

---

## 12. Risks

| Risk | Likelihood | Mitigation |
|---|---|---|
| Sentence boundary detection produces incorrect cuts (e.g. abbreviations not in the hardcoded list) | Medium | The abbreviation list is easily extended; add a regression test for each newly discovered false positive |
| Reverse token walk produces a different page boundary than a forward walk would, beyond the accepted one-sentence variance | Medium | Round-trip guard in §5.5.2 catches overshoots; add a round-trip test to the test suite |
| Mutating `#book-container` during measurement causes a visible flash | Medium | Measurement and restore happen in the same synchronous call before the next browser paint; if flashing is observed, wrap in `requestAnimationFrame` |
| `PAGE_HEIGHT_FUDGE_FACTOR` needs to differ per device or font size | Low | Acceptable for v1 single-operator use; expose as a runtime setting in v2 if needed |
| HTML healer fails silently on pathological EPUB markup | Low | Fuzz the healer with real-world EPUB samples as part of CI |
| Entity check-and-decode produces incorrect results for partially-encoded chapters | Low | Add a test with mixed encoded/decoded content; fall back to always-decode if the check proves unreliable |
| Cursor offset becomes stale after a content update | Low | Cursor-validation step on load snaps to nearest valid sentence boundary |