# Lector — New Page Turn Renderer Architecture

---

## 1. Core Design Principles

| Principle | Description |
|---|---|
| **No User Scroll** | `#book-container` uses `overflow: hidden` and `height: 100vh`. Native browser scrollbar is disabled. |
| **Button-Driven Navigation** | Two overlaid buttons (Next / Prev) trigger page turns. Each turn calculates the next visible range from DOM layout. |
| **DOM-Driven Layout** | Uses `getBoundingClientRect` to determine viewport height and find which HTML node fits within it — no manual offset calculation in Rust. |
| **Virtualization** | Only the currently visible "page" of HTML is rendered to the DOM. |
| **Signal-Based State** | Local Dioxus Signals manage chapter, page index, and loading state — no global `TextHandler` pollution. |

---

## 2. Module Structure

```
src/renderer/
├── page_view.rs   # Main component and navigation logic
├── layout.rs      # Page boundary calculation using DOM measurements
└── cursor.rs      # Save/load reading position (chapter index + page offset)
```

---

## 3. Implementation Plan

### Phase 1 — Foundation & State Management

**Goal:** Establish a clean state model for the page renderer.

**Define `PageViewState` signals:**

```rust
#[derive(Clone, PartialEq, Debug)]
pub struct PageViewState {
    pub book_id: String,
    pub chapter_idx: usize,
    pub chapter_content: Option<String>, // Raw HTML
    pub current_page_start_offset: usize, // Character index in HTML
    pub loading: bool,
    pub error: Option<String>,
}
```

**Create `use_page_view` hook:**
- Initialize `PageViewState` with `book_id`
- Load initial chapter and page position from cursor storage
- Expose signals for state updates

---

### Phase 2 — Chapter Loading & Parsing

**Goal:** Robustly fetch and store chapter content.

**Fetch chapter:**
- Use `infra::chapters::fetch_chapter` to get HTML
- Store raw HTML in `PageViewState.chapter_content`
- Decode HTML entities using `html_escape` during fetch

**Cursor synchronisation:**
- On load, read cursor from `infra::cursor::get_cursor`
- If cursor exists → set `current_page_start_offset` to the stored character index
- If no cursor → set `current_page_start_offset` to `0`

---

### Phase 3 — DOM Rendering & Page Calculation

**Goal:** Render only the visible page to the DOM.

**Virtual DOM fragment:**
- Calculate the visible range `[start_char, end_char]` for the current page
- Slice the HTML string to this range
- Render the slice into `<div id="book-container">`

**Visible range calculation:**
- Use `getBoundingClientRect` on the container to get its height
- Traverse HTML node-by-node, accumulating rendered height via a temporary DOM element
- Stop when accumulated height exceeds container height — this defines `end_char`

**Rendering loop:**
- On initial load: render the first page
- On button click: calculate the next page and update the DOM

---

### Phase 4 — Button Navigation Logic

**Goal:** Handle Next and Prev button clicks.

**Next button:**
- If at end of current page → calculate and render the next page
- If at end of chapter → fetch next chapter
- Update `current_page_start_offset` and re-render

**Prev button:**
- If at start of current page → calculate and render the previous page
- If at start of chapter → fetch previous chapter
- Update `current_page_start_offset` and re-render

**Button overlay:**
- Overlay Next and Prev on the reading container (e.g. left/right edges or bottom corners)
- Hide buttons when at chapter start or end

---

### Phase 5 — Cursor Saving & Error Handling

**Goal:** Persist reading progress and handle errors gracefully.

**Cursor saving:**
- Debounce saves (e.g. every 5 seconds or on chapter change)
- Persist `chapter_idx` and `current_page_start_offset`

**Error handling:**
- On fetch failure: set `PageViewState.error` and show a user-friendly message
- Implement retry for transient network errors
- Log errors via `tracing` — do not crash the renderer

---

## 4. Technical Details

### 4.1 Page Boundary Calculation

```rust
fn calculate_page_boundaries(
    html: &str,
    container_height: f64,
    start_offset: usize
) -> (usize, usize) {
    let mut accumulated_height = 0.0;
    let mut end_offset = start_offset;

    // Create a temporary DOM element to measure node heights
    let temp_div = web_sys::window().unwrap().document().unwrap()
        .create_element("div").unwrap();

    let mut idx = start_offset;
    while idx < html.len() {
        if let Some(node_start) = find_next_node(html, idx) {
            if let Some(node_end) = find_node_end(html, node_start) {
                let node_html = &html[node_start..node_end];

                temp_div.set_inner_html(node_html);
                let rect = temp_div.get_bounding_client_rect();
                accumulated_height += rect.height();

                if accumulated_height > container_height {
                    end_offset = node_start; // Cut before this node
                    break;
                }

                idx = node_end;
            } else {
                idx = node_start + 1;
            }
        } else {
            idx += 1;
        }
    }

    (start_offset, end_offset)
}
```

### 4.2 Button Overlay HTML

```html
<div id="book-container">
    <!-- Page content rendered here -->
</div>
<div id="button-overlay">
    <button id="prev-btn">Previous</button>
    <button id="next-btn">Next</button>
</div>
```

### 4.3 Scroll Prevention CSS

```css
#book-container {
    overflow: hidden;
    height: 100vh;
    position: relative;
}
```

---

## 5. Migration Strategy

1. **Parallel development** — Implement the new renderer in a new module without removing `TextHandler`
2. **Feature flag** — Add a config flag to switch between old and new renderers
3. **Testing** — Write comprehensive tests covering edge cases: empty chapters, malformed HTML, rapid button clicks
4. **Gradual rollout** — Enable the new renderer for a subset of users or books before full deployment
5. **Deprecation** — Once stable, remove `TextHandler` and all related code

---

## 6. Risks & Mitigations

| Risk | Mitigation |
|---|---|
| DOM-to-HTML mapping is complex and error-prone | Use a robust HTML parser (e.g. `html5ever`) to build a reliable token map |
| Performance issues with large chapters | Implement virtualization (render only visible nodes) and debounce button clicks |
| Cross-browser compatibility with `getBoundingClientRect` | Test on Chrome, Firefox, and Safari; use polyfills if necessary |
