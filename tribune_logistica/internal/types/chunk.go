package types

import (
	"errors"
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

type ChunkIdentifier struct {
	ID          string `json:"BookId"`
	Chapter     int    `json:"Chapter"`
	StartOffset int    `json:"StartOffset"`
	EndOffset   int    `json:"EndOffset"`
}

type TextChunk struct {
	Id   ChunkIdentifier
	Data string
}

type AudioChunk struct {
	Id   ChunkIdentifier
	Data []byte
}

type SplitResult struct {
	Offset       int    // rune offset of the split point (first rune NOT included in Text)
	Words        int    // word count contained in Text
	Text         string // HTML-stripped text of the segment, words joined by single spaces
	EndOfContent bool   // true if this stopped because the content ran out, not because a real split point was found
}

var blockBoundaryTags = map[string]bool{
	"p": true, "div": true, "li": true, "blockquote": true, "tr": true,
	"h1": true, "h2": true, "h3": true, "h4": true, "h5": true, "h6": true,
}

func NearestAllowedSplit(html string, start int, minWords int) SplitResult {
	return nearestAllowedSplit([]rune(html), start, minWords)
}

// maxTagResyncLookahead bounds how far the mid-tag resync (below) will scan
// forward looking for a stray '>'. It should comfortably cover a mangled
// tag/attribute (which is what this exists to recover from) without risking
// a pathological scan across an entire chapter of ordinary prose that
// happens to contain no '<' at all.
const maxTagResyncLookahead = 2000

func NearestFollowingSplit(html string, start int, minWords int) SplitResult {
	runes := []rune(html)
	if start < 0 {
		start = 0
	}
	if start > len(runes) {
		return SplitResult{Offset: len(runes), Words: 0, EndOfContent: true}
	}

	start = resyncIfMidTag(runes, start)

	var currentWord strings.Builder
	words := 0
	i := start

	for i < len(runes) {
		r := runes[i]

		if r == '<' {
			tagStart := i
			for i < len(runes) && runes[i] != '>' {
				i++
			}
			if i >= len(runes) {
				break // unterminated tag; nothing more we can parse
			}
			i++ // consume '>'
			tag := string(runes[tagStart:i])

			isBlock, isBreakPoint := blockTagKind(tag)
			if isBlock {
				if currentWord.Len() > 0 {
					word := currentWord.String()
					currentWord.Reset()
					words++
					if endsSentence(word) && words >= minWords {
						return SplitResult{Offset: tagStart, Words: words}
					}
				}
				// A true block boundary (closing </p>, <br>, etc.) is always a safe
				// place to end a chunk, regardless of minWords — paragraph/line breaks
				// are natural stopping points on their own. We only require words > 0
				// so we don't return a zero-length, no-progress split.
				if isBreakPoint && words > 0 {
					return SplitResult{Offset: i, Words: words}
				}
			}
		} else if unicode.IsSpace(r) {
			if currentWord.Len() > 0 {
				word := currentWord.String()
				currentWord.Reset()
				words++
				offset := i // capture before advancing past the space
				if endsSentence(word) && words >= minWords {
					return SplitResult{Offset: offset, Words: words}
				}
			}
			i++
		} else {
			currentWord.WriteRune(r)
			i++
		}
	}

	if currentWord.Len() > 0 {
		words++
	}
	return SplitResult{Offset: len(runes), Words: words, EndOfContent: true}
}

func nearestAllowedSplit(runes []rune, start int, minWords int) SplitResult {
	if start >= len(runes) {
		return SplitResult{Offset: start, Words: 0, EndOfContent: true}
	}

	start = resyncIfMidTag(runes, start)

	var (
		buf         strings.Builder
		currentWord strings.Builder
		words       int
		i           = start
	)

	// flushWord completes the word currently being accumulated (if any) and
	// reports whether, having just completed it, it ended a sentence.
	flushWord := func() (endedSentence bool) {
		if currentWord.Len() == 0 {
			return false
		}
		word := currentWord.String()
		currentWord.Reset()
		if buf.Len() > 0 {
			buf.WriteByte(' ')
		}
		buf.WriteString(word)
		words++
		return endsSentence(word)
	}

	for i < len(runes) {
		r := runes[i]

		if r == '<' {
			tagStart := i
			for i < len(runes) && runes[i] != '>' {
				i++
			} // Implementation for extracting a chunk of the epub
			if i < len(runes) {
				i++ // consume '>'
			}
			tag := string(runes[tagStart:i])

			isBlock, isBreakPoint := blockTagKind(tag)
			if isBlock {

				flushWord()
				if isBreakPoint && words >= minWords {
					return SplitResult{Offset: i, Words: words, Text: buf.String()}
				}
			}
		} else if unicode.IsSpace(r) {
			offset := i // capture before advancing past the space itself
			endedSentence := flushWord()
			i++
			if endedSentence && words >= minWords {
				return SplitResult{Offset: offset, Words: words, Text: buf.String()}
			}
		} else {
			currentWord.WriteRune(r)
			i++
		}
	}

	// Ran off the end of content without finding a qualifying split.
	flushWord()
	return SplitResult{Offset: len(runes), Words: words, Text: buf.String(), EndOfContent: true}
}

func NearestPrecedingSplit(html string, offset int) int {
	runes := []rune(html)
	if offset < 0 {
		offset = 0
	}
	if offset > len(runes) {
		offset = len(runes)
	}

	lastSplit := 0 // chapter start is always a trivially valid boundary
	var currentWord strings.Builder
	i := 0

	for i < offset {
		r := runes[i]

		if r == '<' {
			tagStart := i
			for i < len(runes) && runes[i] != '>' {
				i++
			}
			if i >= len(runes) {
				break // unterminated tag; nothing more we can parse
			}
			i++ // consume '>'
			tag := string(runes[tagStart:i])

			isBlock, isBreakPoint := blockTagKind(tag)
			if isBlock {
				currentWord.Reset() // tag breaks the word, same as forward scan
				if isBreakPoint {
					lastSplit = i
				}
			}
		} else if unicode.IsSpace(r) {
			if currentWord.Len() > 0 {
				word := currentWord.String()
				currentWord.Reset()
				if endsSentence(word) {
					lastSplit = i
				}
			}
			i++
		} else {
			currentWord.WriteRune(r)
			i++
		}
	}

	return skipNonContent(runes, lastSplit)
}

func skipNonContent(runes []rune, i int) int {
	for i < len(runes) {
		if unicode.IsSpace(runes[i]) {
			i++
		} else if runes[i] == '<' {
			for i < len(runes) && runes[i] != '>' {
				i++
			}
			if i < len(runes) {
				i++
			}
		} else {
			break
		}
	}
	return i
}

func endsSentence(word string) bool {
	word = strings.TrimRight(word, "\"')]”’")
	if word == "" {
		return false
	}
	last := word[len(word)-1]
	return last == '.' || last == '!' || last == '?'
}

// resyncIfMidTag guards against a start offset that lands inside a tag
// (e.g. because it was computed against a different/mangled encoding of the
// same html, or against markup whose escaped '>' didn't line up with a real
// '>' rune at the position a caller expected). In well-formed text, the
// next special character scanning forward from a real boundary is always
// '<' (the next tag) or end-of-content. If instead the next special
// character is a bare '>', that means we're already inside a tag's tail -
// so we discard everything through that '>' and resume scanning right
// after it, rather than letting the tag's remainder leak into the output
// as ordinary words.
func resyncIfMidTag(runes []rune, start int) int {
	limit := start + maxTagResyncLookahead
	if limit > len(runes) {
		limit = len(runes)
	}
	for j := start; j < limit; j++ {
		switch runes[j] {
		case '<':
			return start // real boundary; nothing to resync
		case '>':
			return j + 1 // discard the stray tag tail, resume after it
		}
	}
	return start
}

func blockTagKind(tag string) (isBlock, isBreakPoint bool) {
	inner := strings.Trim(tag, "<>")
	inner = strings.TrimSpace(inner)
	if inner == "" {
		return false, false
	}
	closing := strings.HasPrefix(inner, "/")
	inner = strings.TrimPrefix(inner, "/")
	selfClosing := strings.HasSuffix(inner, "/")
	inner = strings.TrimSuffix(inner, "/")
	if idx := strings.IndexAny(inner, " \t\n"); idx >= 0 {
		inner = inner[:idx]
	}
	name := strings.ToLower(inner)

	if name == "br" {
		return true, true // <br>, <br/>, and (nonstandard) </br> are all breaks
	}
	if blockBoundaryTags[name] {
		return true, closing || selfClosing
	}
	return false, false
}

// --- TextChunk building ---

// ChunkConfig tunes chunk sizing.
type ChunkConfig struct {
	TargetWords   int
	MinSplitWords int
}

func DefaultChunkConfig() ChunkConfig {
	return ChunkConfig{
		TargetWords:   35,
		MinSplitWords: 15,
	}
}

var ErrEndOfChapter = errors.New("start offset is at or past end of chapter content")

// jsonEscapeRe matches \uXXXX unicode escapes and single-char backslash
// escapes (\" \\ \n \t \r \/ ...). This is intentionally not full JSON
// parsing: the surrounding html still has plenty of real, unescaped quotes
// (e.g. class="...") that would make json.Unmarshal reject the string.
var jsonEscapeRe = regexp.MustCompile(`\\u([0-9a-fA-F]{4})|\\(.)`)

// normalizeEscapedMarkup reverses stray JSON-style escaping that sometimes
// leaks into stored html (e.g. a literal `\"` or `\u003e` in place of a
// real `"` or `>`). Tag-boundary detection throughout this file works on
// literal '<' and '>' runes, so any escaping left in place silently
// desyncs computed offsets from the real tag structure - this call is what
// makes BuildTextChunk correct "no matter how the html happens to be
// encoded" going in. It's idempotent: running it on already-clean html is
// a no-op, which is what keeps StartOffset/EndOffset self-consistent
// across repeated calls against the same source string.
func normalizeEscapedMarkup(s string) string {
	return jsonEscapeRe.ReplaceAllStringFunc(s, func(m string) string {
		if strings.HasPrefix(m, `\u`) {
			var r rune
			fmt.Sscanf(m[2:], "%04x", &r)
			return string(r)
		}
		switch m[1] {
		case 'n':
			return "\n"
		case 't':
			return "\t"
		case 'r':
			return "\r"
		case '"':
			return `"`
		case '\\':
			return `\`
		case '/':
			return `/`
		default:
			return m[1:] // unknown escape: drop the backslash, keep the char
		}
	})
}

// BuildTextChunk implements types.TextChunkBuilder's contract: id.EndOffset
// is ignored, id.StartOffset is trusted as the starting rune offset into
// html. It works by repeatedly calling NearestAllowedSplit to accumulate
// segments — each independently satisfying MinSplitWords — until
// TargetWords is reached or the chapter's content runs out.
func BuildTextChunk(html string, id ChunkIdentifier, cfg ChunkConfig) (TextChunk, error) {
	html = normalizeEscapedMarkup(html)
	runes := []rune(html)

	if id.StartOffset < 0 || id.StartOffset >= len(runes) {
		return TextChunk{}, ErrEndOfChapter
	}

	var text strings.Builder
	totalWords := 0
	pos := id.StartOffset

	for {
		seg := nearestAllowedSplit(runes, pos, cfg.MinSplitWords)

		if seg.Words > 0 {
			if text.Len() > 0 {
				text.WriteByte(' ')
			}
			text.WriteString(seg.Text)
			totalWords += seg.Words
		}
		pos = seg.Offset

		if seg.EndOfContent || totalWords >= cfg.TargetWords {
			break
		}
	}

	if totalWords == 0 {
		return TextChunk{}, ErrEndOfChapter
	}

	return TextChunk{
		Id: ChunkIdentifier{
			ID:          id.ID,
			Chapter:     id.Chapter,
			StartOffset: id.StartOffset,
			EndOffset:   pos,
		},
		Data: text.String(),
	}, nil
}
