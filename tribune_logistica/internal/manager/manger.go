package manager

import (
	"github.com/book_legion-tribune_logistica/internal/buffer"
	types "github.com/book_legion-tribune_logistica/internal/types"
)

// Organizer handles ordered requests and keeps track of future chunks
type Organizer struct {
	Buf        *buffer.Buffer
	BufferSize int
	OrderList  []types.Cursor
	orderSet   map[types.Cursor]bool
}

func (o *Organizer) GetChunks(start types.Cursor, count int, maxChunks map[int]int) ([]types.Chunk, error) {
	var result []types.Chunk
	cur := start

	// Step 1: collect available chunks
	for range count {
		data, ok := o.Buf.Get(cur)
		if !ok {
			break
		}
		result = append(result, types.Chunk{ID: cur, Data: data})

		maxChunk := maxChunks[cur.Chapter]
		cur.Next(maxChunk, len(maxChunks)-1)
	}

	lastReturned := cur

	// Step 2: check if we have enough elements after lastReturned
	missing := []types.Cursor{}
	curAhead := lastReturned
	availableAhead := 0
	for availableAhead < o.BufferSize {
		_, ok := o.Buf.Get(curAhead)
		if !ok {
			missing = append(missing, curAhead)
		}
		maxChunk := maxChunks[curAhead.Chapter]
		curAhead.Next(maxChunk, len(maxChunks)-1)
		availableAhead++
	}

	// Step 3: extend last cursor to double buffer size
	if len(missing) > 0 {
		lastMissing := missing[len(missing)-1]
		extraCursors := NextCursors(lastMissing, o.BufferSize*2, maxChunks)
		missing = append(missing, extraCursors...)
	}

	for _, m := range missing {
		o.addToOrder(m)
	}
	if len(result) > 0 {
		o.TrimBuffer(result[len(result)-1].ID, maxChunks)
	}

	return result, nil
}

func (o *Organizer) TrimBuffer(c types.Cursor, maxChunks map[int]int) {
	backwards := o.BufferSize / 2
	min := minChapter(maxChunks)
	o.Buf.Trim(c, backwards, min, maxChunks)

}

func minChapter(maxChunks map[int]int) int {
	first := true
	min := 0
	for ch := range maxChunks {
		if first || ch < min {
			min = ch
			first = false
		}
	}
	return min
}

func NextCursors(start types.Cursor, n int, maxChunks map[int]int) []types.Cursor {
	cursors := make([]types.Cursor, 0, n)
	cur := start
	for range n {
		cursors = append(cursors, cur)
		maxChunk := maxChunks[cur.Chapter]
		cur.Next(maxChunk, len(maxChunks)-1) // assume last chapter is len(maxChunks)-1
	}
	return cursors
}

func NewOrganizer(buf *buffer.Buffer, bufferSize int) *Organizer {
	return &Organizer{
		Buf:        buf,
		BufferSize: bufferSize,
		OrderList:  make([]types.Cursor, 0),
		orderSet:   make(map[types.Cursor]bool),
	}
}

// addToOrder adds a cursor to the order list only if not already present
func (o *Organizer) addToOrder(c types.Cursor) {
	if !o.orderSet[c] && !o.Buf.Has(c) {
		o.OrderList = append(o.OrderList, c)
		o.orderSet[c] = true
	}
}
