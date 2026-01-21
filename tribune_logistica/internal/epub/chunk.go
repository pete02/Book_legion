package epub

import (
	"regexp"
)

type Chunk struct {
	Index int
	Start int
	End   int
}

type ChunkPolicy struct {
	TargetSize     int
	MinSize        int
	MaxSize        int
	MinSnippetSize int
}

func NewPolicy(target int, min int, max int, snippet int) ChunkPolicy {
	return ChunkPolicy{
		TargetSize:     target,
		MinSize:        min,
		MaxSize:        max,
		MinSnippetSize: snippet,
	}
}

func TextChunk(linear LinearText, p ChunkPolicy) []Chunk {
	var chunks []Chunk
	text := linear.FullText

	reSentence := regexp.MustCompile(`.+?[.!?](?:\s+|$)`)
	sentences := reSentence.FindAllStringIndex(text, -1)

	chunkStart := 0
	chunkSize := 0
	index := 0

	for _, sentence := range sentences {
		sentStart, sentEnd := sentence[0], sentence[1]
		size := sentEnd - sentStart
		if chunkSize+size <= p.TargetSize {
			chunkSize += size
			continue
		}

		if chunkSize > 0 {
			start, end := trimOffsets(text, chunkStart, sentStart)
			if start < end {
				chunks = append(chunks, Chunk{
					Index: index,
					Start: start,
					End:   end,
				})
				index++
			}
		}

		if size > p.MaxSize {
			// long sentence: split by words
			c := SplitSentenceByWords(text, sentStart, sentEnd, p.MaxSize)
			for i := range c {
				c[i].Index += index
			}
			index += len(c)
			chunks = append(chunks, c...)
			// next chunk starts after this long sentence
			chunkStart = sentEnd
			chunkSize = 0
			continue
		}

		chunkStart = sentStart
		chunkSize = size
	}

	if chunkStart < len(text) {
		start, end := trimOffsets(text, chunkStart, len(text))
		if start < end {
			chunks = append(chunks, Chunk{
				Index: index,
				Start: start,
				End:   end,
			})
		}
	}

	return chunks
}

func trimOffsets(text string, start, end int) (int, int) {
	for start < end && isWhitespace(rune(text[start])) {
		start++
	}
	for end > start && isWhitespace(rune(text[end-1])) {
		end--
	}
	return start, end
}

func SplitSentenceByWords(text string, start, end, maxSize int) []Chunk {
	var chunks []Chunk
	offset := start
	index := 0

	for offset < end {
		nextEnd := offset + maxSize
		if nextEnd >= end {
			nextEnd = end
		} else {
			// backtrack to last whitespace in the window
			foundSpace := false
			for i := nextEnd; i > offset; i-- {
				if isWhitespace(rune(text[i-1])) {
					nextEnd = i // split after the space
					foundSpace = true
					break
				}
			}
			if !foundSpace {
				nextEnd = offset + maxSize
				if nextEnd > end {
					nextEnd = end
				}
			}
		}

		// trim leading spaces
		for offset < nextEnd && isWhitespace(rune(text[offset])) {
			offset++
		}
		// trim trailing spaces
		for nextEnd > offset && isWhitespace(rune(text[nextEnd-1])) {
			nextEnd--
		}

		if offset < nextEnd {
			chunks = append(chunks, Chunk{
				Index: index,
				Start: offset,
				End:   nextEnd,
			})
			index++
		}

		offset = nextEnd
		// skip leading spaces for next chunk
		for offset < end && isWhitespace(rune(text[offset])) {
			offset++
		}
	}

	return chunks
}

func isWhitespace(r rune) bool {
	return r == ' ' || r == '\n' || r == '\t' || r == '\r'
}

func PrettyChunks(chunks []Chunk, linear LinearText) []string {
	out := make([]string, len(chunks))
	for i, c := range chunks {
		if c.Start >= 0 && c.End <= len(linear.FullText) {
			out[i] = linear.FullText[c.Start:c.End]
		} else {
			out[i] = "" // safety fallback
		}
	}
	return out
}
