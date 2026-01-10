package epub

import (
	"strings"
	"testing"

	"golang.org/x/net/html"
)

func TestLinearizeChapter(t *testing.T) {
	tests := []struct {
		name           string
		html           string
		wantFullText   string
		wantSpanCounts int
	}{
		{
			name:           "simple paragraph",
			html:           `<p>Hello World</p>`,
			wantFullText:   "Hello World",
			wantSpanCounts: 1,
		},
		{
			name:           "multiple paragraphs",
			html:           `<div><p>Hello</p><p>World</p></div>`,
			wantFullText:   "HelloWorld",
			wantSpanCounts: 2,
		},
		{
			name:           "nested elements",
			html:           `<div><p>Hello <em>bold</em> World</p></div>`,
			wantFullText:   "Hello bold World",
			wantSpanCounts: 3, // "Hello ", "bold", " World"§
		},
		{
			name:           "whitespace-only nodes skipped",
			html:           `<div>  <p>Hi</p>   </div>`,
			wantFullText:   "Hi",
			wantSpanCounts: 1,
		},
		{
			name:           "skipped script/style tags",
			html:           `<div><p>Hello</p><script>console.log('x');</script><style>.x{}</style><p>World</p></div>`,
			wantFullText:   "HelloWorld",
			wantSpanCounts: 2,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			doc, err := html.Parse(strings.NewReader(tt.html))
			if err != nil {
				t.Fatalf("failed to parse HTML: %v", err)
			}

			linear := LinearizeChapter(doc)

			// Check full text
			if linear.FullText != tt.wantFullText {
				t.Errorf("FullText mismatch:\nwant: %q\ngot:  %q", tt.wantFullText, linear.FullText)
			}

			// Check total chars
			if linear.TotalChars != len(tt.wantFullText) {
				t.Errorf("TotalChars mismatch: want %d, got %d", len(tt.wantFullText), linear.TotalChars)
			}

			// Check span count
			if len(linear.Spans) != tt.wantSpanCounts {
				t.Errorf("Span count mismatch: want %d, got %d", tt.wantSpanCounts, len(linear.Spans))
			}

			// Check span offsets
			offset := 0
			for i, span := range linear.Spans {
				if span.GlobalStart != offset {
					t.Errorf("Span %d GlobalStart mismatch: want %d, got %d", i, offset, span.GlobalStart)
				}
				expectedLen := span.GlobalEnd - span.GlobalStart
				actualLen := len(span.Node.Data)
				if expectedLen != actualLen {
					// Allow spaces trimmed in node? For simplicity we check raw len
					if strings.TrimSpace(span.Node.Data) != "" && expectedLen != actualLen {
						t.Errorf("Span %d length mismatch: want %d, got %d", i, actualLen, expectedLen)
					}
				}
				offset += expectedLen
			}

			// Final offset matches total chars
			if offset != linear.TotalChars {
				t.Errorf("Final offset %d does not match TotalChars %d", offset, linear.TotalChars)
			}
		})
	}
}
