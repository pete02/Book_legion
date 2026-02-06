# Page Architecture

This document defines the **authoritative paging architecture** for text rendering and navigation.

The goal is **determinism, reload-safety, and zero hidden state**.

---

## Core Invariant

> **The pager must be stateless with respect to history.**

All paging decisions must be derived exclusively from:

* `chapter_html` (authoritative source text)
* `current_offset` (HTML offset into chapter)
* `alignment intent` (top or bottom)

No previous pages, offsets, or visible text fragments may be stored.

---

## System Roles

### Renderer (Authoritative)

The renderer is the *only* component allowed to:

* Measure the DOM
* Decide what text fits on screen
* Trim text to page boundaries

It produces **exact page boundaries**.

### Pager (Pure Logic)

The pager:

* Has **no memory**
* Does **not** know about the DOM
* Only derives next/previous offsets from the current render result

---

## Data Structures

### RenderRequest

```rust
struct RenderRequest {
    start_offset: usize,
    align: Align, // Top | Bottom
}
```

### RenderResult

```rust
struct RenderResult {
    start_offset: usize,
    end_offset: usize,
    at_chapter_start: bool,
    at_chapter_end: bool,
}
```

---

## Renderer Responsibilities (Critical)

The renderer **must guarantee**:

> `start_offset` is the **earliest HTML offset that is visible on screen** after trimming.

This must hold for **both** alignment modes:

* **Align::Top** — page aligned at the top
* **Align::Bottom** — page aligned at the bottom

If this invariant holds, paging is correct and history-free.

---

## Paging Logic

### Next Page

```rust
fn next_page(current: &RenderResult) -> Option<RenderRequest> {
    if current.at_chapter_end {
        None
    } else {
        Some(RenderRequest {
            start_offset: current.end_offset,
            align: Align::Top,
        })
    }
}
```

### Previous Page

```rust
fn previous_page(current: &RenderResult) -> Option<RenderRequest> {
    if current.at_chapter_start {
        None
    } else {
        Some(RenderRequest {
            start_offset: current.start_offset,
            align: Align::Bottom,
        })
    }
}
```

**Key property:**

* No stored history
* No guessed offsets
* Fully reload-safe

---

## Chapter Boundary Detection

The renderer determines:

* `at_chapter_start` when `start_offset == 0`
* `at_chapter_end` when `end_offset == chapter_html.len()`

The pager never inspects text content.

---

## Alignment Semantics

* **Forward navigation** → `Align::Top`
* **Backward navigation** → `Align::Bottom`

Alignment is an *intent*, not a layout hack.

The renderer decides how to trim text accordingly.

---

## Forbidden Patterns

The following are **explicitly disallowed**:

* Storing previous pages
* Storing previous offsets
* Storing visible text fragments
* Inferring paging from scroll position
* Pager inspecting DOM or text content

These introduce reload bugs and state drift.
---
## Summary

* Renderer owns layout and trimming
* Pager derives navigation purely
* Chapter HTML is the single source of truth
* No history, ever

This architecture is intentionally strict to prevent subtle paging b

Phase 1 – Viewport Measurement

Determine which TextRuns fit in the viewport given current font size and line height.

Trim partially visible lines and split elements if necessary.

Phase 2 – Page Offset Calculation

Compute RenderResult { start_offset, end_offset, at_chapter_start, at_chapter_end } from TextRuns.

Support Align::Top (forward paging) and Align::Bottom (backward paging).

Sentence / Comma Splitting

Ensure page breaks only occur at sentence boundaries or , characters.

Renderer Function Interface

Finalize the renderer function signature:

fn render_page(chapter_html: &str, start_offset: usize, align: Align) -> RenderResult


Integration with Pager

Pager remains stateless, deriving next/previous RenderRequest from current RenderResult.

Robust Unit / Integration Tests

Validate:

Multi-paragraph chapters

Nested elements

Alignment correctness

Reload-safe offsets

Performance Considerations

Measure parsing/rendering speed on large chapters.

Optimize for repeated calls without storing history.