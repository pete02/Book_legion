# Lector — Page Turn Renderer

## Unit Test Specification

**Version:** 1.0
**Derived from:** Lector Page Turn Renderer Specification v1.2
**Scope:** Unit-level and deterministic integration tests for renderer core modules

---

# 1. Purpose

This document defines a deterministic unit test suite for the Lector Page Turn Renderer. The goal is to validate correctness of:

* Sentence boundary detection
* HTML healing
* Page boundary calculation (forward)
* Reverse page calculation (backward navigation)
* Trailing end-tag inclusion
* Cursor persistence
* Chapter loading normalization
* Edge-case resilience

The tests assume a pure or near-pure implementation of:

* `layout.rs`
* `html_healer.rs`
* `cursor.rs`
* `page_view.rs` (logic only; DOM interactions are mocked)

---

# 2. Testing Strategy

## 2.1 Test Levels

| Level                           | Description                                               |
| ------------------------------- | --------------------------------------------------------- |
| Unit tests                      | Pure functions: sentence detection, healing, tokenization |
| Deterministic integration tests | Page boundary calculation using mocked DOM height         |
| Snapshot tests                  | Healed HTML output consistency                            |
| Property-based tests            | Robustness under malformed HTML inputs                    |

## 2.2 Mocking Strategy

The following must be mocked:

* `#book-container.clientHeight`
* DOM insertion/appending
* `infra::chapters::fetch_chapter`
* `infra::cursor` storage

---

# 3. Test Data Fixtures

## 3.1 Canonical Chapter Fixture

```html
<div>
  <p>He ran fast. She followed him.</p>
  <p>Dr. Smith arrived at 3.14 pm. It was late.</p>
</div>
```

## 3.2 Abbreviation Fixture

```text
Mr. Dr. Prof. e.g. i.e. etc. St. Ave.
```

## 3.3 Malformed HTML Fixture

```html
<div><p>Unclosed paragraph
<span>Nested text</div>
```

---

# 4. Sentence Boundary Tests

## 4.1 Basic sentence detection

### Test

Input: `"Hello world. Next sentence."`

### Expect

Boundaries at:

* after `world.`
* after `sentence.`

---

## 4.2 Abbreviation handling

### Test

Input: `"Dr. Smith went to St. Paul."`

### Expect

* No boundary after `Dr.`
* Boundary only after `Paul.`

---

## 4.3 Decimal preservation

### Test

Input: `"Value is 3.14 and stable."`

### Expect

* No boundary at `3.14`
* Boundary at `stable.`

---

## 4.4 Ellipsis handling

### Test

Input: `"He paused... then spoke."`

### Expect

* No boundary inside ellipsis
* Boundary after `spoke.`

---

## 4.5 Quote and punctuation wrapping

### Test

Input: `"She said \"hello!\" Then left."`

### Expect

* Boundary after `hello!"`
* Boundary after `left.`

---

# 5. HTML Healing Tests

## 5.1 Balanced HTML slice

Input:

```html
<p>Hello <em>world</em>.</p>
```

Expect:

* Output unchanged

---

## 5.2 Missing closing tag

Input:

```html
<p>Hello <em>world.</p>
```

Expect:

```html
<p>Hello <em>world.</em></p>
```

---

## 5.3 Orphan closing tag removal

Input slice:

```html
</p><p>Hello</p>
```

Expect:

```html
<p>Hello</p>
```

---

## 5.4 Nested healing order

Input:

```html
<div><p><em>Text</p>
```

Expect:

```html
<div><p><em>Text</em></p></div>
```

---

## 5.5 Void elements untouched

Input:

```html
<p>Image <img src="x"></p>
```

Expect:

* `<img>` unchanged
* no closing tag added

---

# 6. Page Boundary Calculation Tests (Forward)

## 6.1 Single-page fit

### Setup

* container height: 1000px
* content height: 800px

### Expect

* `start_offset = 0`
* `end_offset = html.len()`

---

## 6.2 Overflow snap to sentence

### Setup

* sentences: 3 sentences
* overflow occurs mid second sentence

### Expect

* page ends at end of sentence 1
* not mid-sentence

---

## 6.3 No sentence boundary fallback

### Input

Long paragraph without punctuation

### Expect

* fallback to `</p>` or `</div>` boundary

---

## 6.4 Oversized image node

Input:

```html
<p><img src="large.png"></p>
```

Expect:

* image occupies its own page

---

# 7. Trailing End-Tag Inclusion Tests

## 7.1 Basic inclusion

Input:

```html
<p>Sentence one.</p>
```

Expect:

* `</p>` included in same page

---

## 7.2 Multiple closing tags

Input:

```html
<div><p>Text.</p></div>
```

Expect:

* both `</p>` and `</div>` included

---

## 7.3 Whitespace skipping

Input:

```html
<p>Text.</p>   <!-- comment -->\n</div>
```

Expect:

* comment ignored
* `</div>` included

---

# 8. Reverse Page Calculation Tests

## 8.1 Basic backward navigation

### Setup

* 3 pages of content
* current at page 2

### Expect

* Prev returns page 1 start offset

---

## 8.2 Reverse sentence snap

Input causes overflow mid sentence

Expect:

* snap to previous sentence boundary

---

## 8.3 Round-trip consistency

### Test

```
forward(page_n) -> backward(result) -> forward
```

### Expect

* final page start offset >= original
* difference ≤ 1 sentence

---

## 8.4 Chapter start edge

If at offset 0:

Expect:

* no crash
* stay at first chapter or load previous chapter

---


# 10. Chapter Loading Tests

## 10.1 Entity decoding

Input:

```html
Tom &amp; Jerry
```

Expect:

```text
Tom & Jerry
```

---

## 10.2 No entities path

Input:

* plain HTML

Expect:

* unchanged string

---

# 11. Error Handling Tests

## 11.1 Empty chapter

Expect:

* no crash
* empty render state

---

## 11.2 Malformed HTML

Expect:

* healer produces best-effort valid output
* no panic

---

## 11.3 Overshot cursor

Expect:

* reset to 0
* warning logged

---

# 12. Performance Tests

## 12.1 Large chapter (200KB)

Expect:

* page calculation ≤ 100ms

---

## 12.2 Token traversal complexity

Expect:

* linear time O(n)

---

# 13. Resize Behavior Tests

## 13.1 Resize maintains offset

Expect:

* `start_offset` unchanged
* only `end_offset` recomputed

---

## 13.2 Immediate recalculation

Expect:

* no debounce delay
* deterministic reflow

---

# 14. Navigation Button State Tests

## 14.1 First page

Expect:

* Prev hidden
* Next visible

---

## 14.2 Last page

Expect:

* Next hidden
* Prev visible

---

## 14.3 Middle page

Expect:

* both visible

---

# 15. Property-Based Tests

## 15.1 Random HTML fuzzing

Property:

* healer never produces unbalanced tags

---

## 15.2 Random sentence splitting

Property:

* no page ends mid-sentence

---

# 16. Acceptance Criteria

All tests must:

* Pass deterministically
* Be independent (no shared state)
* Not rely on real DOM
* Not rely on timing or async behavior except fetch retry test

---

# 17. Coverage Target

| Module               | Coverage |
| -------------------- | -------- |
| layout.rs            | ≥ 90%    |
| html_healer.rs       | ≥ 95%    |
| cursor.rs            | ≥ 85%    |
| page_view.rs (logic) | ≥ 80%    |

---

# End of Specification
