package manager_test

import (
	"testing"

	"github.com/book_legion-tribune_logistica/internal/buffer"
	"github.com/book_legion-tribune_logistica/internal/manager"
	types "github.com/book_legion-tribune_logistica/internal/types"
)

func assertCursorSlicesEqual(t *testing.T, got, want []types.Cursor) {
	t.Helper()
	if len(got) != len(want) {
		t.Fatalf("length mismatch: got %d, want %d\n got=%v\nwant=%v", len(got), len(want), got, want)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("cursor mismatch at %d: got %v, want %v", i, got[i], want[i])
		}
	}
}

func TestOrganizerGetChunks_AllAvailable(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{
		0: 2,
		1: 1,
	}

	for ch := 0; ch <= 1; ch++ {
		for c := 0; c <= maxChunks[ch]; c++ {
			buf.Add(buffer.Chunk{
				ID:   types.Cursor{Chapter: ch, Chunk: c},
				Data: []byte{byte(ch*10 + c)},
			})
		}
	}

	org := manager.NewOrganizer(buf, 2)

	start := types.Cursor{0, 0}
	chunks, err := org.GetChunks("t", start, 4, maxChunks)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(chunks) != 4 {
		t.Fatalf("expected 4 chunks, got %d", len(chunks))
	}

	if len(org.OrderList) != 0 {
		t.Fatalf("expected empty OrderList, got %v", org.OrderList)
	}
}

func TestIncorrectIdEmptiesBuffer(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{
		0: 2,
		1: 1,
	}

	for ch := 0; ch <= 1; ch++ {
		for c := 0; c <= maxChunks[ch]; c++ {
			buf.Add(buffer.Chunk{
				ID:   types.Cursor{Chapter: ch, Chunk: c},
				Data: []byte{byte(ch*10 + c)},
			})
		}
	}

	org := manager.NewOrganizer(buf, 2)

	start := types.Cursor{0, 0}
	chunks, err := org.GetChunks("a", start, 4, maxChunks)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(chunks) != 0 {
		t.Fatalf("expected 0 chunks, got %d", len(chunks))
	}
}

func TestOrganizerGetChunks_MissingFirstChunk_ReturnsNothing(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{
		0: 2,
	}

	// Buffer has later chunk, but not the first
	buf.Add(buffer.Chunk{
		ID:   types.Cursor{0, 1},
		Data: []byte{1},
	})

	org := manager.NewOrganizer(buf, 2)

	start := types.Cursor{0, 0}
	chunks, _ := org.GetChunks("t", start, 3, maxChunks)

	if len(chunks) != 0 {
		t.Fatalf("expected 0 chunks, got %d", len(chunks))
	}

	expectedOrder := []types.Cursor{
		{0, 0},
		{0, 2},
	}

	assertCursorSlicesEqual(t, org.OrderList, expectedOrder)
}

func TestOrganizerGetChunks_GapInMiddleStopsReturn(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{
		0: 3,
	}

	buf.Add(buffer.Chunk{ID: types.Cursor{0, 0}, Data: []byte{0}})
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 2}, Data: []byte{2}})

	org := manager.NewOrganizer(buf, 2)

	start := types.Cursor{0, 0}
	chunks, _ := org.GetChunks("t", start, 4, maxChunks)

	if len(chunks) != 1 {
		t.Fatalf("expected 1 contiguous chunk, got %d", len(chunks))
	}

	expectedOrder := []types.Cursor{
		{0, 1},
		{0, 3},
	}

	assertCursorSlicesEqual(t, org.OrderList, expectedOrder)
}

func TestOrganizerGetChunks_MultiChapter_ContiguousOnly(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{
		0: 1,
		1: 2,
	}

	buf.Add(buffer.Chunk{ID: types.Cursor{0, 0}, Data: []byte{0}})
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 1}, Data: []byte{1}})
	buf.Add(buffer.Chunk{ID: types.Cursor{1, 0}, Data: []byte{10}})

	org := manager.NewOrganizer(buf, 2)

	start := types.Cursor{0, 0}
	chunks, _ := org.GetChunks("t", start, 4, maxChunks)

	if len(chunks) != 3 {
		t.Fatalf("expected 3 contiguous chunks, got %d", len(chunks))
	}

	expectedOrder := []types.Cursor{
		{1, 1},
		{1, 2},
	}

	assertCursorSlicesEqual(t, org.OrderList, expectedOrder)
}

func TestOrganizerOrderList_Invariants(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{0: 3}

	buf.Add(buffer.Chunk{ID: types.Cursor{0, 1}, Data: []byte{1}})

	org := manager.NewOrganizer(buf, 3)
	org.GetChunks("t", types.Cursor{0, 0}, 4, maxChunks)

	seen := make(map[types.Cursor]bool)
	for _, c := range org.OrderList {
		if seen[c] {
			t.Fatalf("duplicate cursor %v in OrderList", c)
		}
		seen[c] = true

		if _, ok := buf.Get(c); ok {
			t.Fatalf("cursor %v in OrderList already exists in buffer", c)
		}
	}

	// Ensure sorted
	for i := 1; i < len(org.OrderList); i++ {
		if org.OrderList[i-1].CompareCursor(org.OrderList[i]) > 0 {
			t.Fatalf("OrderList not sorted: %v", org.OrderList)
		}
	}
}

func TestManagerDoesNotTrimWhenBelowHalfBuffer(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{0: 5}

	for i := 0; i <= 2; i++ {
		buf.Add(buffer.Chunk{
			ID:   types.Cursor{0, i},
			Data: []byte{byte(i)},
		})
	}

	org := manager.NewOrganizer(buf, 6) // half = 3

	start := types.Cursor{0, 0}
	_, _ = org.GetChunks("t", start, 3, maxChunks)

	for i := 0; i <= 2; i++ {
		if _, ok := buf.Get(types.Cursor{0, i}); !ok {
			t.Fatalf("unexpected trim of cursor {0,%d}", i)
		}
	}
}

func TestManagerTrimsBackwardBeyondHalfBuffer(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{0: 10}

	for i := 0; i <= 6; i++ {
		buf.Add(buffer.Chunk{
			ID:   types.Cursor{0, i},
			Data: []byte{byte(i)},
		})
	}

	org := manager.NewOrganizer(buf, 6) // half = 3

	start := types.Cursor{0, 3}
	chunks, _ := org.GetChunks("t", start, 3, maxChunks)

	if len(chunks) != 3 {
		t.Fatalf("expected 3 chunks, got %d", len(chunks))
	}

	// anchor = last returned = {0,5}
	// keep last 3 backward + anchor
	expectedKept := []types.Cursor{
		{0, 2},
		{0, 3},
		{0, 4},
		{0, 5},
	}

	for _, c := range expectedKept {
		if _, ok := buf.Get(c); !ok {
			t.Fatalf("expected cursor %v to be kept", c)
		}
	}

	// older than half-buffer
	trimmed := []types.Cursor{
		{0, 0},
		{0, 1},
	}

	for _, c := range trimmed {
		if _, ok := buf.Get(c); ok {
			t.Fatalf("expected cursor %v to be trimmed", c)
		}
	}
}

func TestManagerTrimRespectsChapterBoundaries(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{
		0: 2,
		1: 2,
	}

	// Fill across chapters
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 0}, Data: []byte{0}})
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 1}, Data: []byte{1}})
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 2}, Data: []byte{2}})
	buf.Add(buffer.Chunk{ID: types.Cursor{1, 0}, Data: []byte{10}})
	buf.Add(buffer.Chunk{ID: types.Cursor{1, 1}, Data: []byte{11}})

	org := manager.NewOrganizer(buf, 4) // half = 2

	start := types.Cursor{0, 1}
	org.GetChunks("t", start, 3, maxChunks)

	// anchor = {1,0}
	// keep last 2 backward + anchor
	expectedKept := []types.Cursor{
		{0, 1},
		{0, 2},
		{1, 0},
	}

	for _, c := range expectedKept {
		if _, ok := buf.Get(c); !ok {
			t.Fatalf("expected %v to be kept", c)
		}
	}

	trimmed := []types.Cursor{
		{0, 0},
	}

	for _, c := range trimmed {
		if _, ok := buf.Get(c); ok {
			t.Fatalf("expected %v to be trimmed", c)
		}
	}
}
