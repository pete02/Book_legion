package types

// Cursor represents a location in the audiobook
type Cursor struct {
	Chapter int `json:"chapter"`
	Index   int `json:"chunk"`
}

func (c *Cursor) Next(maxIndex int, maxChapter int) {
	if c.Index == maxIndex {
		if c.Chapter < maxChapter {
			c.Chapter += 1
			c.Index = 0
		}
	} else {
		c.Index += 1
	}
}

func (c *Cursor) Prev(maxIndex int, minChapter int) {
	if c.Index == 0 {
		if c.Chapter > minChapter {
			c.Chapter -= 1
			c.Index = maxIndex
		}
	} else {
		c.Index -= 1
	}
}

func (c Cursor) StepBack(steps int, minChapter int, maxIndexs map[int]int) Cursor {
	cur := c
	for range steps {
		maxIndex, ok := maxIndexs[cur.Chapter-1]
		if !ok {
			maxIndex = 0
		}
		cur.Prev(maxIndex, minChapter)
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
		if a.Index < b.Index {
			return -1
		} else if a.Index > b.Index {
			return 1
		}
		return 0
	}
}
