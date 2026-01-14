package epub

import (
	"bytes"
	"strings"

	"golang.org/x/net/html"
)

type TextSpan struct {
	Node        *html.Node
	NodeStart   int
	NodeEnd     int
	GlobalStart int
	GlobalEnd   int
}

type LinearText struct {
	Spans      []TextSpan
	TotalChars int
	FullText   string
}

var blockElements = map[string]bool{
	"p": true, "div": true, "h1": true, "h2": true, "h3": true,
	"h4": true, "h5": true, "h6": true, "li": true, "blockquote": true,
	"section": true, "article": true, "pre": true,
}

// LinearizeChapter converts HTML to linear text with proper spans and periods.
func LinearizeChapter(doc *html.Node) LinearText {
	var spans []TextSpan
	var buf bytes.Buffer
	globalOffset := 0

	isPunctuation := func(r byte) bool {
		return r == '.' || r == '!' || r == '?'
	}

	addText := func(n *html.Node, text string) {
		// Skip text nodes that are completely whitespace
		if strings.TrimSpace(text) == "" {
			return
		}

		// Replace newlines with space
		text = strings.ReplaceAll(text, "\n", " ")

		nodeStart := 0
		nodeEnd := len(text)
		buf.WriteString(text)

		spans = append(spans, TextSpan{
			Node:        n,
			NodeStart:   nodeStart,
			NodeEnd:     nodeEnd,
			GlobalStart: globalOffset,
			GlobalEnd:   globalOffset + nodeEnd,
		})
		globalOffset += nodeEnd
	}

	var walk func(n *html.Node)
	walk = func(n *html.Node) {
		switch n.Type {
		case html.TextNode:
			addText(n, n.Data)

		case html.ElementNode:
			// skip unwanted tags
			switch n.Data {
			case "script", "style", "head", "title", "meta", "link",
				"img", "video", "audio", "iframe", "canvas", "svg":
				return
			}

			for c := n.FirstChild; c != nil; c = c.NextSibling {
				walk(c)
			}

			// Add ". " after block elements if needed
			if blockElements[n.Data] && buf.Len() > 0 {
				lastChar := buf.String()[buf.Len()-1]
				if !isPunctuation(lastChar) && lastChar != ' ' {
					buf.WriteString(". ")
					globalOffset += 2
				}
			}

		default:
			for c := n.FirstChild; c != nil; c = c.NextSibling {
				walk(c)
			}
		}
	}

	walk(doc)

	return LinearText{
		Spans:      spans,
		TotalChars: globalOffset,
		FullText:   buf.String(),
	}
}
