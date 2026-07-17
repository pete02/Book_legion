package types

import (
	"errors"
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

func nearestAllowedSplit(runes []rune, start int, minWords int) SplitResult {
	if start >= len(runes) {
		return SplitResult{Offset: start, Words: 0, EndOfContent: true}
	}

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
			}
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

func endsSentence(word string) bool {
	word = strings.TrimRight(word, "\"')]”’")
	if word == "" {
		return false
	}
	last := word[len(word)-1]
	return last == '.' || last == '!' || last == '?'
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

// BuildTextChunk implements types.TextChunkBuilder's contract: id.EndOffset
// is ignored, id.StartOffset is trusted as the starting rune offset into
// html. It works by repeatedly calling NearestAllowedSplit to accumulate
// segments — each independently satisfying MinSplitWords — until
// TargetWords is reached or the chapter's content runs out.
func BuildTextChunk(html string, id ChunkIdentifier, cfg ChunkConfig) (TextChunk, error) {
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
