// internal/buffer/types.go
package types

// Cursor represents a location in the audiobook
type Cursor struct {
	Chapter int
	Chunk   int
}

func (c *Cursor) Next(maxChunk int, maxChapter int) {
	if c.Chunk == maxChunk {
		if c.Chapter < maxChapter {
			c.Chapter += 1
			c.Chunk = 0
		}
	} else {
		c.Chunk += 1
	}
}

func (c *Cursor) Prev(maxChunk int, minChapter int) {
	if c.Chunk == 0 {
		if c.Chapter > minChapter {
			c.Chapter -= 1
			c.Chunk = maxChunk
		}
	} else {
		c.Chunk -= 1
	}
}

func (c Cursor) StepBack(steps int, minChapter int, maxChunks map[int]int) Cursor {
	cur := c
	for range steps {
		maxChunk, ok := maxChunks[cur.Chapter-1]
		if !ok {
			maxChunk = 0
		}
		cur.Prev(maxChunk, minChapter)
	}
	return cur
}

// compareCursor returns -1 if a < b, 0 if a == b, 1 if a > b
func (a Cursor) CompareCursor(b Cursor) int {
	if a.Chapter < b.Chapter {
		return -1
	} else if a.Chapter > b.Chapter {
		return 1
	} else { // same chapter
		if a.Chunk < b.Chunk {
			return -1
		} else if a.Chunk > b.Chunk {
			return 1
		}
		return 0
	}
}

// Chunk represents an audio Chunk
type Chunk struct {
	ID   Cursor
	Data []byte
}
