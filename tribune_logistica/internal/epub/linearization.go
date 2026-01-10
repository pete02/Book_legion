package epub

import (
	"bytes"
	"strings"

	"golang.org/x/net/html"
)

type TextSpan struct {
	Node        *html.Node // the source HTML node
	NodeStart   int        // offset in the node's text
	NodeEnd     int
	GlobalStart int // offset in the chapter's linearized text
	GlobalEnd   int
}

type LinearText struct {
	Spans      []TextSpan
	TotalChars int
	FullText   string // concatenation of all spans
}

func LinearizeChapter(doc *html.Node) LinearText {
	var spans []TextSpan
	var buf bytes.Buffer
	globalOffset := 0

	var walk func(n *html.Node)
	walk = func(n *html.Node) {
		switch n.Type {
		case html.TextNode:
			// Trim text nodes that are all whitespace
			text := n.Data
			if strings.TrimSpace(text) != "" {
				nodeStart := 0
				nodeEnd := len(text)

				buf.WriteString(text)

				span := TextSpan{
					Node:        n,
					NodeStart:   nodeStart,
					NodeEnd:     nodeEnd,
					GlobalStart: globalOffset,
					GlobalEnd:   globalOffset + nodeEnd,
				}
				spans = append(spans, span)

				globalOffset += nodeEnd
			}
		case html.ElementNode:
			switch n.Data {
			case "script", "style", "head", "title", "meta", "link",
				"img", "video", "audio", "iframe", "canvas", "svg":
				return
			}
			// Recurse children
			for c := n.FirstChild; c != nil; c = c.NextSibling {
				walk(c)
			}
		default:
			// recurse children for other node types
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
