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

// --- nearestAllowedSplit ---
//
// ASSUMPTION FLAGGED: this implements split-point detection (sentence-end
// punctuation + block tag boundaries) from scratch, since I don't have
// visibility into whatever nearestAllowedSplit implementation already
// exists elsewhere in the codebase per the design doc's issue list. If
// that logic already exists, this should probably be replaced with a call
// into it instead of maintaining a second, possibly-inconsistent notion of
// "allowed split."
//
// ASSUMPTION FLAGGED: offsets are treated as rune indices into the HTML
// string (i.e. "character" offsets, matching htmlCharOffset in the design
// doc), not byte indices. If the frontend computes offsets as JS
// string.length (UTF-16 code units), this will disagree with it for any
// character outside the Basic Multilingual Plane.

// SplitResult is a single scanned segment: starting from wherever the scan
// began, the nearest position at or after that point where it's valid to
// stop.
type SplitResult struct {
	Offset       int    // rune offset of the split point (first rune NOT included in Text)
	Words        int    // word count contained in Text
	Text         string // HTML-stripped text of the segment, words joined by single spaces
	EndOfContent bool   // true if this stopped because the content ran out, not because a real split point was found
}

// blockBoundaryTags are tags that count as a valid place to stop even
// without terminal sentence punctuation — e.g. a list item or heading with
// no trailing period — when encountered closing or self-closing. Adjust
// this set once real chapter HTML structure is available; it's a
// reasonable starting guess, not verified against actual book markup.
var blockBoundaryTags = map[string]bool{
	"p": true, "div": true, "li": true, "blockquote": true, "tr": true,
	"h1": true, "h2": true, "h3": true, "h4": true, "h5": true, "h6": true,
}

// NearestAllowedSplit scans html starting at the rune offset `start` and
// returns the nearest position at or after start where it's valid to stop:
// either a sentence-ending word or a closing/self-closing block-level tag,
// with at least minWords words accumulated since start. Candidates with
// fewer than minWords are skipped over, not returned. If the content ends
// before such a point is found, everything scanned is returned with
// EndOfContent set — running out of content is always an implicit valid
// place to stop, regardless of minWords.
//
// This is the single primitive behind both chunk building (BuildTextChunk
// calls it repeatedly to accumulate a target word count) and, later,
// snapping an arbitrary seek position to a valid chunk boundary — both are
// fundamentally "find the next place it's OK to cut, from here."
func NearestAllowedSplit(html string, start int, minWords int) SplitResult {
	return nearestAllowedSplit([]rune(html), start, minWords)
}

// nearestAllowedSplit is the rune-slice-based implementation. Exposed
// separately from NearestAllowedSplit so repeated calls against the same
// chapter (as BuildTextChunk makes) don't each pay to re-convert the whole
// string to []rune.
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
				// Only block-level tags (paragraph/list/heading/etc.) break
				// a word. Inline tags like <em> or <strong> are stripped
				// silently, letting the word accumulate across them — HTML
				// markup like "<strong>world</strong>." has no whitespace
				// between the tag and the period, and treating every tag
				// as a word boundary would incorrectly split that into
				// "world" and "." as two separate words.
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

// endsSentence reports whether word ends a sentence, after trimming common
// trailing closing punctuation (quotes, parens) that can follow a
// terminator. This is a simple heuristic — it does not attempt to handle
// abbreviations (e.g. "Mr.", "e.g.") and will over-split on those.
func endsSentence(word string) bool {
	word = strings.TrimRight(word, "\"')]”’")
	if word == "" {
		return false
	}
	last := word[len(word)-1]
	return last == '.' || last == '!' || last == '?'
}

// blockTagKind reports whether tag (the full "<...>" text) is a block-level
// tag at all, and if so, whether it's a valid split point on its own
// (closing or self-closing — e.g. </p>, <br>) as opposed to just an opening
// tag (<p>, <li>), which still breaks a word but isn't itself a place to
// stop, since it opens new content rather than concluding any.
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
	// TargetWords is the word count a chunk aims for. Scanning continues
	// past this only if no qualifying split point has been found yet.
	TargetWords int
	// MinSplitWords is the minimum word count a candidate split point must
	// have accumulated to be usable. A sentence boundary reached after only
	// a couple of words is rejected as a split point (though those words
	// still count toward the eventual chunk once a later split qualifies).
	MinSplitWords int
}

// DefaultChunkConfig targets ~15s of speech at ~140 WPM (a round number
// between the typical 130-160 WPM speech range), with a minimum split size
// chosen to avoid producing near-empty leftover chunks.
func DefaultChunkConfig() ChunkConfig {
	return ChunkConfig{
		TargetWords:   35,
		MinSplitWords: 15,
	}
}

// ErrEndOfChapter is returned when id.StartOffset is already at or past the
// end of the chapter's content, i.e. there is nothing left to build a chunk
// from. A chunk that runs up against the end of the chapter mid-build is
// still returned successfully (as a final, possibly-short chunk); this
// error only fires on the *next* call after that, when StartOffset points
// past the end. Callers doing chapter-transition handling should treat this
// as their end-of-chapter signal.
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
