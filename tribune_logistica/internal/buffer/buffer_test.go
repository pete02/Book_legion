package buffer

import (
	"reflect"
	"sync"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/types"
)

func TestBufferAddGet(t *testing.T) {
	buf := NewBuffer("")

	c1 := Cursor{Chapter: 0, Chunk: 0}
	data1 := []byte("chunk1")
	buf.Add(Chunk{ID: c1, Data: data1})

	got, ok := buf.Get(c1)
	if !ok {
		t.Fatalf("expected chunk at %v to exist", c1)
	}
	if !reflect.DeepEqual(got, data1) {
		t.Errorf("got %v, want %v", got, data1)
	}
}

func TestBufferHas(t *testing.T) {
	buf := NewBuffer("")

	c1 := Cursor{Chapter: 0, Chunk: 0}
	c2 := Cursor{Chapter: 0, Chunk: 1}
	data1 := []byte("chunk1")
	buf.Add(Chunk{ID: c1, Data: data1})

	if !buf.Has(c1) {
		t.Fatalf("Buf does not contain cursor %v", c1)
	}

	if buf.Has(c2) {
		t.Fatalf("Buf contains incorrectly cursor %v", c2)
	}
}

func TestBufferClear(t *testing.T) {
	buf := NewBuffer("")

	buf.Add(Chunk{ID: Cursor{Chapter: 0, Chunk: 0}, Data: []byte("a")})
	buf.Clear()

	if len(buf.Store) != 0 {
		t.Errorf("expected Store to be empty after Clear, got %d items", len(buf.Store))
	}
}

func TestBufferTrimSingleChapter(t *testing.T) {
	buf := NewBuffer("")

	maxChunks := map[int]int{0: 5} // chapter 0 has 5 chunks: 0..4

	// add 5 chunks
	for i := 0; i <= 5; i++ {
		c := Cursor{Chapter: 0, Chunk: i}
		buf.Add(Chunk{ID: c, Data: []byte{byte(i)}})
	}

	// trim last 3 steps from chunk 4
	buf.Trim(Cursor{Chapter: 0, Chunk: 4}, 2, 0, maxChunks)

	expected := []Cursor{
		{Chapter: 0, Chunk: 2},
		{Chapter: 0, Chunk: 3},
		{Chapter: 0, Chunk: 4},
		{Chapter: 0, Chunk: 5},
	}

	for _, c := range expected {
		if _, ok := buf.Get(c); !ok {
			t.Errorf("expected cursor %v to exist after trim", c)
		}
	}

	if _, ok := buf.Get(Cursor{Chapter: 0, Chunk: 0}); ok {
		t.Errorf("cursor {0,1} should have been trimmed")
	}
}

func TestBufferTrimMultipleChapters(t *testing.T) {
	buf := NewBuffer("")

	// chapter -> max chunk
	maxChunks := map[int]int{
		0: 2, // 0..2
		1: 3, // 0..3
		2: 1, // 0..1
	}

	// add chunks across chapters
	addChunk := func(chapter, chunk int) {
		buf.Add(Chunk{
			ID:   Cursor{Chapter: chapter, Chunk: chunk},
			Data: []byte{byte(chapter*10 + chunk)},
		})
	}

	addChunk(0, 0)
	addChunk(0, 1)
	addChunk(0, 2)
	addChunk(1, 0)
	addChunk(1, 1)
	addChunk(1, 2)
	addChunk(1, 3)
	addChunk(2, 0)
	addChunk(2, 1)

	// trim last 4 steps from cursor {2,1}
	buf.Trim(Cursor{Chapter: 2, Chunk: 1}, 4, 0, maxChunks)

	// expected to keep last 5 chunks:
	expected := []Cursor{
		{Chapter: 1, Chunk: 1},
		{Chapter: 1, Chunk: 2},
		{Chapter: 1, Chunk: 3},
		{Chapter: 2, Chunk: 0},
		{Chapter: 2, Chunk: 1},
	}

	// check that expected exist
	for _, c := range expected {
		if _, ok := buf.Get(c); !ok {
			t.Errorf("expected cursor %v to exist after trim", c)
		}
	}

	// check that older chunks are gone
	removed := []Cursor{
		{Chapter: 0, Chunk: 0},
		{Chapter: 0, Chunk: 1},
		{Chapter: 0, Chunk: 2},
		{Chapter: 1, Chunk: 0},
	}

	for _, c := range removed {
		if _, ok := buf.Get(c); ok {
			t.Errorf("expected cursor %v to be trimmed", c)
		}
	}
}

func TestTrimEmptyBuffer(t *testing.T) {
	buf := NewBuffer("")

	// Should not panic or fail
	maxChunks := map[int]int{0: 2}
	buf.Trim(Cursor{Chapter: 0, Chunk: 0}, 3, 0, maxChunks)
}

func TestTrimStepsZero(t *testing.T) {
	buf := NewBuffer("")
	c := Cursor{Chapter: 0, Chunk: 0}
	buf.Add(Chunk{ID: c, Data: []byte("data")})

	maxChunks := map[int]int{0: 0}
	buf.Trim(c, 0, 0, maxChunks)

	if _, ok := buf.Get(c); !ok {
		t.Errorf("expected chunk %v to exist after trimming 0 steps", c)
	}
}

func TestTrimStepsNegative(t *testing.T) {
	buf := NewBuffer("")
	c := Cursor{Chapter: 0, Chunk: 0}
	buf.Add(Chunk{ID: c, Data: []byte("data")})

	maxChunks := map[int]int{0: 0}
	// Negative steps should ideally behave like 0 (no deletion)
	buf.Trim(c, -5, 0, maxChunks)

	if _, ok := buf.Get(c); !ok {
		t.Errorf("expected chunk %v to exist after trimming negative steps", c)
	}
}

func TestTrimPastMinChapter(t *testing.T) {
	buf := NewBuffer("")

	// Add first chapter
	buf.Add(Chunk{ID: Cursor{Chapter: 0, Chunk: 0}, Data: []byte("a")})

	maxChunks := map[int]int{
		0: 0,
	}

	// Step back past minChapter should keep it at minChapter
	buf.Trim(Cursor{Chapter: 0, Chunk: 0}, 5, 0, maxChunks)

	if _, ok := buf.Get(Cursor{Chapter: 0, Chunk: 0}); !ok {
		t.Errorf("expected chunk at minChapter to survive trim")
	}
}

func TestTrimUnknownChapterInMaxChunks(t *testing.T) {
	buf := NewBuffer("")
	buf.Add(Chunk{ID: Cursor{Chapter: 1, Chunk: 0}, Data: []byte("b")})

	// maxChunks does not contain chapter 0 or 1
	maxChunks := map[int]int{}

	// Should default unknown chapters to 0 and not panic
	buf.Trim(Cursor{Chapter: 1, Chunk: 0}, 1, 0, maxChunks)

	// Chunk may be removed depending on logic, but must not panic
}

func TestAddOverwriteChunk(t *testing.T) {
	buf := NewBuffer("")
	c := Cursor{Chapter: 0, Chunk: 0}

	buf.Add(Chunk{ID: c, Data: []byte("first")})
	buf.Add(Chunk{ID: c, Data: []byte("second")})

	got, ok := buf.Get(c)
	if !ok {
		t.Fatalf("expected chunk at %v to exist", c)
	}
	if string(got) != "second" {
		t.Errorf("expected chunk to be overwritten with 'second', got %s", got)
	}
}

func makeChunk(id types.Cursor, data string) types.Chunk {
	return types.Chunk{ID: id, Data: []byte(data)}
}

func TestBufferConcurrentAddGet(t *testing.T) {
	buf := NewBuffer("test")

	var wg sync.WaitGroup
	numGoroutines := 10
	numChunks := 10

	// Writer goroutines
	for i := 0; i < numGoroutines; i++ {
		wg.Add(1)
		go func(base int) {
			defer wg.Done()
			for j := range numChunks {
				id := types.Cursor{Chapter: base, Chunk: j}
				buf.Add(makeChunk(id, "data"))
			}
		}(i)
	}

	// Reader goroutines
	for i := range numGoroutines {
		wg.Add(1)
		go func(base int) {
			defer wg.Done()
			for j := range numChunks {
				id := types.Cursor{Chapter: base, Chunk: j}
				buf.Get(id)
				buf.Has(id)
			}
		}(i)
	}

	wg.Wait()
}

func TestBufferConcurrentTrim(t *testing.T) {
	buf := NewBuffer("trim-test")
	numChunks := 10

	// Populate buffer
	for i := range numChunks {
		id := types.Cursor{Chapter: 0, Chunk: i}
		buf.Add(makeChunk(id, "data"))
	}

	var wg sync.WaitGroup
	numGoroutines := 10

	for i := range numGoroutines {
		wg.Add(1)
		go func(offset int) {
			defer wg.Done()
			anchor := types.Cursor{Chapter: 0, Chunk: numChunks - 1 - offset}
			maxChunks := map[int]int{0: numChunks}
			buf.Trim(anchor, 10, 0, maxChunks)
		}(i)
	}

	wg.Wait()
}

func TestBufferConcurrentClear(t *testing.T) {
	buf := NewBuffer("clear-test")
	numChunks := 10

	for i := range numChunks {
		buf.Add(makeChunk(types.Cursor{Chapter: 0, Chunk: i}, "data"))
	}

	var wg sync.WaitGroup
	numGoroutines := 10

	for range numGoroutines {
		wg.Add(1)
		go func() {
			defer wg.Done()
			buf.Clear()
		}()
	}

	wg.Wait()
}
