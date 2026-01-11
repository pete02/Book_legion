// internal/buffer/buffer.go
package buffer

import (
	"sync"

	"github.com/book_legion-tribune_logistica/internal/types"
)

type Cursor = types.UserCursor
type Chunk = types.Chunk

type Buffer struct {
	Id    string
	Store map[Cursor][]byte
	mu    sync.RWMutex
}

func NewBuffer(id string) *Buffer {
	return &Buffer{
		Id:    id,
		Store: make(map[Cursor][]byte),
	}
}

// Adds the cunk data to the map with key of chunk ID (Cursor)
func (b *Buffer) Add(chunk Chunk) {
	b.mu.Lock()
	b.Store[chunk.ID] = chunk.Data
	b.mu.Unlock()
}

func (b *Buffer) Get(c Cursor) ([]byte, bool) {
	b.mu.RLock()
	chunk, ok := b.Store[c]
	b.mu.RUnlock()
	return chunk, ok
}

func (b *Buffer) Has(c Cursor) bool {
	b.mu.RLock()
	_, ok := b.Store[c]
	b.mu.RUnlock()
	return ok
}

func (b *Buffer) Clear() {
	b.mu.Lock()
	b.Store = make(map[Cursor][]byte)
	b.mu.Unlock()
}

// Keeps steps-1 elements from the current element backwards and trims the rest
// if Trim is called for the following buffer, with cursor of {0, 4} and 2 steps, we keep the kept buffer
/*
 	buffer :={
		{Chapter: 0, Chunk: 0},
		{Chapter: 0, Chunk: 1},
		{Chapter: 0, Chunk: 2},
		{Chapter: 0, Chunk: 3},
		{Chapter: 0, Chunk: 4},
		{Chapter: 0, Chunk: 5},
	}

	kept := {
		{Chapter: 0, Chunk: 2},
		{Chapter: 0, Chunk: 3},
		{Chapter: 0, Chunk: 4},
		{Chapter: 0, Chunk: 5},
	}
*/
func (b *Buffer) Trim(anchor Cursor, steps int, minChapter int, maxChunks map[int]int) {
	if steps <= 0 {
		return
	}

	// Compute cutoff outside lock to minimize blocking
	cutoff := anchor.Cursor.StepBack(steps, minChapter, maxChunks)
	if cutoff == anchor.Cursor {
		return
	}

	// Collect keys to delete without holding lock
	b.mu.RLock()
	var toDelete []Cursor
	for k := range b.Store {
		if k.Cursor.CompareCursor(cutoff) < 0 {
			toDelete = append(toDelete, k)
		}
	}
	b.mu.RUnlock()

	// Delete keys with write lock
	if len(toDelete) > 0 {
		b.mu.Lock()
		for _, k := range toDelete {
			delete(b.Store, k)
		}
		b.mu.Unlock()
	}
}
