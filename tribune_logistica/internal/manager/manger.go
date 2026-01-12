package manager

import (
	"fmt"
	"sync"
	"time"

	"github.com/book_legion-tribune_logistica/internal/buffer"
	types "github.com/book_legion-tribune_logistica/internal/types"
)

// Organizer handles ordered requests and keeps track of future chunks
type Organizer struct {
	Buf        *buffer.Buffer
	BufferSize int
	OrderList  []types.UserCursor
	orderSet   map[types.UserCursor]bool
	mu         sync.Mutex
	cond       *sync.Cond
}

// buffer size determines the minimum amount to keep in buffer. Max amount generated is 2*buffer size
func NewOrganizer(buf *buffer.Buffer, bufferSize int) *Organizer {
	o := &Organizer{
		Buf:        buf,
		BufferSize: bufferSize,
		OrderList:  make([]types.UserCursor, 0),
		orderSet:   make(map[types.UserCursor]bool, bufferSize*2),
	}
	o.cond = sync.NewCond(&o.mu)
	return o
}

func (o *Organizer) Clear() {
	o.Buf.Clear()
	o.mu.Lock()
	defer o.mu.Unlock()
	o.OrderList = o.OrderList[:0]
	o.orderSet = make(map[types.UserCursor]bool)
}

func (o *Organizer) GetUserChunks(start types.UserCursor, count int, maxChunks map[int]int) ([]types.Chunk, error) {
	return o.GetChunks(start.BookID+start.UserID, start, count, maxChunks)
}

func (o *Organizer) GetChunks(id string, start types.UserCursor, count int, maxChunks map[int]int) ([]types.Chunk, error) {
	o.mu.Lock()
	defer o.mu.Unlock()
	fmt.Printf("Asked: %v", start)
	o.idCheck(id)
	fmt.Printf("ID ok")

	if !o.ensureStartExists(start, maxChunks) {
		for {
			o.mu.Unlock()
			time.Sleep(10 * time.Millisecond)
			o.mu.Lock()
			if o.Buf.Has(start) {
				break
			}
		}
	}
	fmt.Printf("start ok ok")
	result, lastReturned := o.collectContiguousChunks(start, count, maxChunks)
	fmt.Printf("Result gotten")
	missing := o.computeMissingCursors(lastReturned, maxChunks)
	o.updateOrderList(missing, maxChunks)
	o.TrimBuffer(lastReturned, maxChunks)

	return result, nil
}

func (o *Organizer) idCheck(id string) {
	if id != o.Buf.Id {
		// instead of Clear(), initialize a new buffer instance
		o.Buf = buffer.NewBuffer(id)
		o.OrderList = nil
		o.orderSet = make(map[types.UserCursor]bool, o.BufferSize*2)
	}
}

func (o *Organizer) ensureStartExists(start types.UserCursor, maxChunks map[int]int) bool {

	if !o.Buf.Has(start) {

		o.addToOrderLocked(start)
		extra := nextCursors(start, o.BufferSize*2, maxChunks)
		for _, c := range extra {
			o.addToOrderLocked(c)
		}
		return false
	} else {
		return true
	}
}

func (o *Organizer) collectContiguousChunks(start types.UserCursor, count int, maxChunks map[int]int) ([]types.Chunk, types.UserCursor) {
	var result []types.Chunk
	cur := start

	for range count {
		data, ok := o.Buf.Get(cur)
		if !ok {
			break
		}
		result = append(result, types.Chunk{ID: cur, Data: data})

		maxChunk := maxChunks[cur.Cursor.Chapter]
		cur.Cursor.Next(maxChunk, len(maxChunks)-1)
	}

	var last types.UserCursor
	if len(result) > 0 {
		last = result[len(result)-1].ID
	} else {
		last = start
	}

	return result, last
}

// computeMissingCursors returns up to BufferSize cursors that are missing after `start`
func (o *Organizer) computeMissingCursors(start types.UserCursor, maxChunks map[int]int) []types.UserCursor {
	var missing []types.UserCursor
	cur := start
	for i := 0; i < o.BufferSize; i++ {
		if !o.Buf.Has(cur) {
			missing = append(missing, cur)
		}
		maxChunk, ok := maxChunks[cur.Cursor.Chapter]
		if !ok {
			maxChunk = 0
		}
		cur.Cursor.Next(maxChunk, len(maxChunks)-1)
	}
	return missing
}

func (o *Organizer) updateOrderList(missing []types.UserCursor, maxChunks map[int]int) {

	if len(missing) > 0 {
		lastMissing := missing[len(missing)-1]
		missing = append(missing, nextCursors(lastMissing, o.BufferSize*2, maxChunks)...)
	}

	for _, c := range missing {
		o.addToOrderLocked(c)
	}
}

func (o *Organizer) TrimBuffer(c types.UserCursor, maxChunks map[int]int) {
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

func nextCursors(start types.UserCursor, n int, maxChunks map[int]int) []types.UserCursor {
	cursors := make([]types.UserCursor, 0, n)
	cur := start
	for range n {
		cursors = append(cursors, cur)
		maxChunk := maxChunks[cur.Cursor.Chapter]
		cur.Cursor.Next(maxChunk, len(maxChunks)-1) // assume last chapter is len(maxChunks)-1
	}
	return cursors
}

func (o *Organizer) addToOrderLocked(c types.UserCursor) {
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
		o.OrderList = append(o.OrderList, types.UserCursor{}) // expand slice
		copy(o.OrderList[idx+1:], o.OrderList[idx:])
		o.OrderList[idx] = c
	}

	o.orderSet[c] = true
}

// expected return from fetchFn is chunk, ok
func (o *Organizer) StartOrderProcessor(fetchFn func(types.UserCursor) (types.Chunk, bool)) chan struct{} {
	stop := make(chan struct{})

	go func() {
		for {
			select {
			case <-stop:
				return
			default:
				var toProcess []types.UserCursor
				o.mu.Lock()
				toProcess = append(toProcess, o.OrderList...)
				o.mu.Unlock()

				for _, c := range toProcess {
					chunk, ok := fetchFn(c)
					if !ok {
						continue
					}

					o.Buf.Add(chunk)
					o.mu.Lock()

					idx := -1
					for i, existing := range o.OrderList {
						if existing == c {
							idx = i
							break
						}
					}
					if idx >= 0 {
						o.OrderList = append(o.OrderList[:idx], o.OrderList[idx+1:]...)
						delete(o.orderSet, c)
					}
					o.cond.Broadcast()
					o.mu.Unlock()
				}
			}
		}
	}()

	return stop
}

//test heplers:

func (o *Organizer) AddToOrderForTest(c types.UserCursor) {
	o.mu.Lock()
	defer o.mu.Unlock()
	if !o.orderSet[c] && !o.Buf.Has(c) {
		o.OrderList = append(o.OrderList, c)
		o.orderSet[c] = true
	}
}

func (o *Organizer) MuLockTest() {
	o.mu.Lock()
}

func (o *Organizer) MuUnlockTest() {
	o.mu.Unlock()
}
