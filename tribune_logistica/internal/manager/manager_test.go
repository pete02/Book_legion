package manager

import (
	"sync"
	"testing"
	"time"

	"github.com/book_legion-tribune_logistica/internal/buffer"
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

	org := NewOrganizer(buf, 2)

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

func TestOrganizerGetuserChunks_AllAvailable(t *testing.T) {
	curs := types.Cursor{0, 0}
	start := types.UserCursor{"u1", "b1", curs}

	buf := buffer.NewBuffer(start.BookID + start.UserID)
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

	org := NewOrganizer(buf, 2)

	chunks, err := org.GetUserChunks(start, 4, maxChunks)
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

func TestOrganizerGetChunks_GapInMiddleStopsReturn(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{
		0: 3,
	}

	buf.Add(buffer.Chunk{ID: types.Cursor{0, 0}, Data: []byte{0}})
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 2}, Data: []byte{2}})

	org := NewOrganizer(buf, 2)

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

	org := NewOrganizer(buf, 2)

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

func TestManagerDoesNotTrimWhenBelowHalfBuffer(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{0: 5}

	for i := 0; i <= 2; i++ {
		buf.Add(buffer.Chunk{
			ID:   types.Cursor{0, i},
			Data: []byte{byte(i)},
		})
	}

	org := NewOrganizer(buf, 6) // half = 3

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

	org := NewOrganizer(buf, 6) // half = 3

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

	org := NewOrganizer(buf, 4) // half = 2

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

func makeChunk(id types.Cursor, data string) types.Chunk {
	return types.Chunk{ID: id, Data: []byte(data)}
}

func TestStartOrderProcessor_HappyPath(t *testing.T) {
	buf := buffer.NewBuffer("buf-happy")
	org := NewOrganizer(buf, 5)

	// Add several cursors to order list
	cursors := []types.Cursor{
		{Chapter: 0, Chunk: 0},
		{Chapter: 0, Chunk: 1},
		{Chapter: 0, Chunk: 2},
	}

	for _, c := range cursors {
		org.AddToOrderForTest(c)
	}

	// fetchFn simulates 2-second fetch per cursor
	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		time.Sleep(2 * time.Second)
		return makeChunk(c, "data"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	// Wait enough for all fetches to complete
	time.Sleep(time.Duration(len(cursors)*2+1) * time.Second)

	// Verify all chunks are now in the buffer
	for _, c := range cursors {
		data, ok := buf.Get(c)
		if !ok {
			t.Errorf("expected chunk %v in buffer, but not found", c)
		}
		if string(data) != "data" {
			t.Errorf("unexpected data for chunk %v: %s", c, string(data))
		}
	}

	// Verify OrderList is empty
	org.MuLockTest()
	defer org.MuUnlockTest()
	if len(org.OrderList) != 0 {
		t.Errorf("expected OrderList to be empty, but has %d items", len(org.OrderList))
	}
}

func TestProcessor_PartialSuccess(t *testing.T) {
	buf := buffer.NewBuffer("buf-partial")
	org := NewOrganizer(buf, 5)

	cursors := []types.Cursor{
		{Chapter: 0, Chunk: 0},
		{Chapter: 0, Chunk: 1},
		{Chapter: 0, Chunk: 2},
	}

	for _, c := range cursors {
		org.AddToOrderForTest(c)
	}

	// Only even chunks succeed
	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		if c.Chunk%2 == 0 {
			return makeChunk(c, "ok"), true
		}
		return types.Chunk{}, false
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	// Wait a bit to let processor run
	// No time.Sleep needed in real unit, since fetchFn is instant
	time.Sleep(10 * time.Millisecond)
	// Verify buffer contains only successful chunks
	for _, c := range cursors {
		data, ok := buf.Get(c)
		if c.Chunk%2 == 0 {
			if !ok {
				t.Errorf("expected chunk %v in buffer", c)
			}
			if string(data) != "ok" {
				t.Errorf("unexpected data for chunk %v: %s", c, string(data))
			}
		} else {
			if ok {
				t.Errorf("unexpected chunk %v in buffer", c)
			}
		}
	}

	// OrderList contains only failed cursors
	org.MuLockTest()
	defer org.MuUnlockTest()
	for _, c := range org.OrderList {
		if c.Chunk%2 == 0 {
			t.Errorf("chunk %v should have been removed from OrderList", c)
		}
	}
}

func TestProcessor_ConcurrentClear(t *testing.T) {
	buf := buffer.NewBuffer("buf-clear")
	org := NewOrganizer(buf, 5)

	cursors := []types.Cursor{
		{Chapter: 0, Chunk: 0},
		{Chapter: 0, Chunk: 1},
	}

	for _, c := range cursors {
		org.AddToOrderForTest(c)
	}

	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		return makeChunk(c, "ok"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	// Concurrent Clear
	org.Clear()
	time.Sleep(10 * time.Millisecond)
	// Verify buffer may contain nothing and OrderList is empty
	org.MuLockTest()
	defer org.MuUnlockTest()
	if len(org.OrderList) != 0 {
		t.Errorf("expected OrderList empty after Clear, got %v", org.OrderList)
	}
}

func TestProcessor_EmptyOrderListThenAdd(t *testing.T) {
	buf := buffer.NewBuffer("buf-empty")
	org := NewOrganizer(buf, 5)

	// Start processor with empty list
	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		return makeChunk(c, "ok"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	// Add orders dynamically
	c := types.Cursor{Chapter: 0, Chunk: 0}
	org.AddToOrderForTest(c)
	time.Sleep(10 * time.Millisecond)
	// Verify chunk added to buffer
	data, ok := buf.Get(c)
	if !ok || string(data) != "ok" {
		t.Errorf("expected dynamic chunk in buffer")
	}

	time.Sleep(10 * time.Millisecond)
	// OrderList should be empty
	org.MuLockTest()
	defer org.MuUnlockTest()
	if len(org.OrderList) != 0 {
		t.Errorf("expected OrderList empty after dynamic addition")
	}
}

func TestProcessor_DuplicateCursors(t *testing.T) {
	buf := buffer.NewBuffer("buf-dup")
	org := NewOrganizer(buf, 5)

	c := types.Cursor{Chapter: 0, Chunk: 0}

	// Add same cursor multiple times
	org.AddToOrderForTest(c)
	org.AddToOrderForTest(c)
	org.AddToOrderForTest(c)

	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		return makeChunk(c, "ok"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)
	time.Sleep(10 * time.Millisecond)
	// Verify only one chunk in buffer
	data, ok := buf.Get(c)
	if !ok || string(data) != "ok" {
		t.Errorf("expected chunk in buffer")
	}

	// Verify OrderList is empty
	org.MuLockTest()
	defer org.MuUnlockTest()
	if len(org.OrderList) != 0 {
		t.Errorf("expected OrderList empty after processing duplicates")
	}
}

func TestProcessor_StopChannel(t *testing.T) {
	buf := buffer.NewBuffer("buf-stop")
	org := NewOrganizer(buf, 5)

	c := types.Cursor{Chapter: 0, Chunk: 0}
	org.AddToOrderForTest(c)

	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		return makeChunk(c, "ok"), true
	}

	stop := org.StartOrderProcessor(fetchFn)

	// Immediately stop
	close(stop)

	time.Sleep(10 * time.Millisecond)
	// Buffer may or may not have the chunk depending on timing
	// Verify no panic and OrderList is either empty or contains cursor
	org.MuLockTest()
	defer org.MuUnlockTest()
	if len(org.OrderList) > 1 {
		t.Errorf("OrderList should have <= 1 item, got %d", len(org.OrderList))
	}
}

func TestProcessor_HighConcurrencyStress(t *testing.T) {
	buf := buffer.NewBuffer("buf-stress")
	org := NewOrganizer(buf, 10)

	// Add 50 cursors
	for i := range 50 {
		c := types.Cursor{Chapter: 0, Chunk: i}
		org.AddToOrderForTest(c)
	}

	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		return makeChunk(c, "ok"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	// Concurrently add/remove orders
	var wg sync.WaitGroup
	for i := 50; i < 60; i++ {
		wg.Add(1)
		go func(chunk int) {
			defer wg.Done()
			c := types.Cursor{Chapter: 0, Chunk: chunk}
			org.AddToOrderForTest(c)
		}(i)
	}
	wg.Wait()

	// Verify all expected chunks in buffer
	for i := range 60 {
		c := types.Cursor{Chapter: 0, Chunk: i}
		data, ok := buf.Get(c)
		if !ok || string(data) != "ok" {
			t.Errorf("expected chunk %v in buffer", c)
		}
	}

	time.Sleep(10 * time.Millisecond)
	// OrderList should be empty
	org.MuLockTest()
	defer org.MuUnlockTest()
	if len(org.OrderList) != 0 {
		t.Errorf("expected OrderList empty after stress test")
	}
}

func TestOrganizerOrderList_Invariants(t *testing.T) {
	buf := buffer.NewBuffer("t")
	maxChunks := map[int]int{0: 3}

	// Add starting chunk so GetChunks doesn't block
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 0}, Data: []byte{0}})
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 1}, Data: []byte{1}})

	org := NewOrganizer(buf, 10)
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

func TestGetChunks_BlocksUntilOneChunkThenProcessorFillsRest(t *testing.T) {
	buf := buffer.NewBuffer("buf-block")
	org := NewOrganizer(buf, 5)
	maxChunks := map[int]int{0: 4} // 5 chunks total: 0..4

	// Simulate fetchFn with delay per chunk
	var mu sync.Mutex
	fetched := make(map[types.Cursor]bool)

	fetchFn := func(c types.Cursor) (buffer.Chunk, bool) {
		mu.Lock()
		defer mu.Unlock()

		if fetched[c] {
			return buffer.Chunk{}, false
		}

		// mark as fetched
		fetched[c] = true

		// simulate fetching time
		time.Sleep(50 * time.Millisecond)
		return makeChunk(c, "data"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	start := types.Cursor{Chapter: 0, Chunk: 0}

	// Request chunks — should block until at least chunk 0 is available
	result, err := org.GetChunks("buf-block", start, 5, maxChunks)
	if err != nil {
		t.Fatalf("GetChunks returned error: %v", err)
	}

	// Immediately after return, only the first chunk should be guaranteed
	if len(result) != 1 {
		t.Fatalf("expected 1 chunk returned immediately, got %d", len(result))
	}
	if string(result[0].Data) != "data" || result[0].ID != start {
		t.Fatalf("unexpected first chunk: %+v", result[0])
	}

	// Wait a bit longer than processor fetch delay to allow remaining chunks to be added
	time.Sleep(300 * time.Millisecond)

	// Verify that the buffer now contains all chunks 0..4
	for i := 0; i <= 4; i++ {
		c := types.Cursor{Chapter: 0, Chunk: i}
		data, ok := buf.Get(c)
		if !ok {
			t.Errorf("expected chunk %v in buffer", c)
		} else if string(data) != "data" {
			t.Errorf("unexpected data for chunk %v: %s", c, string(data))
		}
	}
}

func TestFetchReordering_LaterChunkArrivesFirst(t *testing.T) {
	buf := buffer.NewBuffer("buf-reorder-1")
	org := NewOrganizer(buf, 5)
	maxChunks := map[int]int{0: 2} // chunks 0,1,2

	start := types.Cursor{Chapter: 0, Chunk: 0}

	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		switch c.Chunk {
		case 0:
			time.Sleep(100 * time.Millisecond) // slow
		case 1:
			time.Sleep(10 * time.Millisecond) // fast
		}
		return makeChunk(c, "data"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	result, err := org.GetChunks("buf-reorder-1", start, 3, maxChunks)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Must return only chunk 0
	if len(result) != 1 {
		t.Fatalf("expected exactly 1 chunk, got %d", len(result))
	}

	if result[0].ID != start {
		t.Fatalf("expected first chunk %v, got %v", start, result[0].ID)
	}

	// Wait for all fetches to complete
	time.Sleep(200 * time.Millisecond)

	// Buffer should now contain all chunks
	for i := 0; i <= 2; i++ {
		c := types.Cursor{Chapter: 0, Chunk: i}
		if _, ok := buf.Get(c); !ok {
			t.Fatalf("expected chunk %v in buffer", c)
		}
	}
}

func TestFetchReordering_GapPreservedDespiteLaterArrival(t *testing.T) {
	buf := buffer.NewBuffer("buf-reorder-2")
	org := NewOrganizer(buf, 5)
	maxChunks := map[int]int{0: 3}

	// Seed chunk 0 so GetChunks can start
	buf.Add(buffer.Chunk{ID: types.Cursor{0, 0}, Data: []byte("seed")})

	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		switch c.Chunk {
		case 1:
			time.Sleep(100 * time.Millisecond) // slow
		case 2:
			time.Sleep(10 * time.Millisecond) // fast
		}
		return makeChunk(c, "data"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	result, err := org.GetChunks("buf-reorder-2", types.Cursor{0, 0}, 4, maxChunks)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Only chunk 0 must be returned
	if len(result) != 1 {
		t.Fatalf("expected 1 chunk, got %d", len(result))
	}
	if result[0].ID != (types.Cursor{0, 0}) {
		t.Fatalf("unexpected chunk returned: %v", result[0].ID)
	}

	time.Sleep(200 * time.Millisecond)

	// Buffer should have chunks 1 and 2 eventually
	for _, i := range []int{1, 2} {
		c := types.Cursor{Chapter: 0, Chunk: i}
		if _, ok := buf.Get(c); !ok {
			t.Fatalf("expected chunk %v in buffer", c)
		}
	}

	// But contiguity must still be respected
	org.MuLockTest()
	defer org.MuUnlockTest()

	for i := 1; i < len(org.OrderList); i++ {
		if org.OrderList[i-1].CompareCursor(org.OrderList[i]) > 0 {
			t.Fatalf("OrderList not sorted: %v", org.OrderList)
		}
	}
}

func TestFetchReordering_AllChunksArriveInReverseOrder(t *testing.T) {
	buf := buffer.NewBuffer("buf-reorder-3")
	org := NewOrganizer(buf, 5)
	maxChunks := map[int]int{0: 4}

	start := types.Cursor{0, 0}

	fetchFn := func(c types.Cursor) (types.Chunk, bool) {
		// Higher chunk number = faster fetch
		time.Sleep(time.Duration(100-c.Chunk*20) * time.Millisecond)
		return makeChunk(c, "data"), true
	}

	stop := org.StartOrderProcessor(fetchFn)
	defer close(stop)

	result, err := org.GetChunks("buf-reorder-3", start, 5, maxChunks)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Must still return only chunk 0
	if len(result) != 1 {
		t.Fatalf("expected 1 chunk, got %d", len(result))
	}
	if result[0].ID != start {
		t.Fatalf("expected chunk %v, got %v", start, result[0].ID)
	}

	time.Sleep(300 * time.Millisecond)

	// Buffer should contain all chunks in the end
	for i := 0; i <= 4; i++ {
		c := types.Cursor{0, i}
		if _, ok := buf.Get(c); !ok {
			t.Fatalf("expected chunk %v in buffer", c)
		}
	}
}
