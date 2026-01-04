package manager

import (
	"sync"

	"github.com/book_legion-tribune_logistica/internal/buffer"
	types "github.com/book_legion-tribune_logistica/internal/types"
)

// Organizer handles ordered requests and keeps track of future chunks
type Organizer struct {
	Buf        *buffer.Buffer
	BufferSize int
	OrderList  []types.Cursor
	orderSet   map[types.Cursor]bool
	mu         sync.Mutex
}

func NewOrganizer(buf *buffer.Buffer, bufferSize int) *Organizer {
	return &Organizer{
		Buf:        buf,
		BufferSize: bufferSize,
		OrderList:  make([]types.Cursor, 0),
		orderSet:   make(map[types.Cursor]bool, bufferSize*2),
	}
}

func (o *Organizer) Clear() {
	o.Buf.Clear()
	o.mu.Lock()
	defer o.mu.Unlock()
	o.OrderList = o.OrderList[:0]
	o.orderSet = make(map[types.Cursor]bool)
}

func (o *Organizer) GetChunks(id string, start types.Cursor, count int, maxChunks map[int]int) ([]types.Chunk, error) {
	o.handleNewId(id)
	if !o.ensureStartExists(start, maxChunks) {
		return nil, nil
	}

	result, lastReturned := o.collectContiguousChunks(start, count, maxChunks)
	missing := o.computeMissingCursors(lastReturned, maxChunks)
	o.updateOrderList(missing, maxChunks)
	o.TrimBuffer(lastReturned, maxChunks)

	return result, nil
}

func (o *Organizer) handleNewId(id string) {
	if id != o.Buf.Id {
		o.Clear()
	}
}

func (o *Organizer) ensureStartExists(start types.Cursor, maxChunks map[int]int) bool {

	if !o.Buf.Has(start) {

		o.addToOrder(start)
		extra := nextCursors(start, o.BufferSize*2, maxChunks)
		for _, c := range extra {
			o.addToOrder(c)
		}
		return false
	} else {
		return true
	}
}

func (o *Organizer) collectContiguousChunks(start types.Cursor, count int, maxChunks map[int]int) ([]types.Chunk, types.Cursor) {
	var result []types.Chunk
	cur := start

	for range count {
		data, ok := o.Buf.Get(cur)
		if !ok {
			break
		}
		result = append(result, types.Chunk{ID: cur, Data: data})

		maxChunk := maxChunks[cur.Chapter]
		cur.Next(maxChunk, len(maxChunks)-1)
	}

	var last types.Cursor
	if len(result) > 0 {
		last = result[len(result)-1].ID
	} else {
		last = start
	}

	return result, last
}

// computeMissingCursors returns up to BufferSize cursors that are missing after `start`
func (o *Organizer) computeMissingCursors(start types.Cursor, maxChunks map[int]int) []types.Cursor {
	var missing []types.Cursor
	cur := start
	for i := 0; i < o.BufferSize; i++ {
		if !o.Buf.Has(cur) {
			missing = append(missing, cur)
		}
		maxChunk, ok := maxChunks[cur.Chapter]
		if !ok {
			maxChunk = 0
		}
		cur.Next(maxChunk, len(maxChunks)-1)
	}
	return missing
}

func (o *Organizer) updateOrderList(missing []types.Cursor, maxChunks map[int]int) {

	if len(missing) > 0 {
		lastMissing := missing[len(missing)-1]
		missing = append(missing, nextCursors(lastMissing, o.BufferSize*2, maxChunks)...)
	}

	for _, c := range missing {
		o.addToOrder(c)
	}
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

func nextCursors(start types.Cursor, n int, maxChunks map[int]int) []types.Cursor {
	cursors := make([]types.Cursor, 0, n)
	cur := start
	for range n {
		cursors = append(cursors, cur)
		maxChunk := maxChunks[cur.Chapter]
		cur.Next(maxChunk, len(maxChunks)-1) // assume last chapter is len(maxChunks)-1
	}
	return cursors
}

// addToOrder adds a cursor to the order list only if not already present
func (o *Organizer) addToOrder(c types.Cursor) {
	o.mu.Lock()
	defer o.mu.Unlock()

	if o.orderSet[c] || o.Buf.Has(c) {
		return
	}

	// Insert in order to keep OrderList sorted
	idx := len(o.OrderList)
	for i, existing := range o.OrderList {
		if c.CompareCursor(existing) < 0 {
			idx = i
			break
		}
	}

	if idx == len(o.OrderList) {
		o.OrderList = append(o.OrderList, c)
	} else {
		o.OrderList = append(o.OrderList, types.Cursor{}) // expand slice
		copy(o.OrderList[idx+1:], o.OrderList[idx:])
		o.OrderList[idx] = c
	}

	o.orderSet[c] = true
}
