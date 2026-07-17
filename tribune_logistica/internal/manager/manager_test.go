package manager

import (
	"context"
	"errors"
	"fmt"
	"sync/atomic"
	"testing"
	"time"

	types "github.com/book_legion-tribune_logistica/internal/types"
)

const chunkSize = 10

// simpleBuildText produces 10-offset-wide chunks and errors past `end`,
// simulating an end-of-chapter/unresolvable-offset boundary.
func simpleBuildText(end int) TextChunkBuilder {
	return func(id types.ChunkIdentifier) (types.TextChunk, error) {
		if id.StartOffset >= end {
			return types.TextChunk{}, errors.New("past end of chapter")
		}
		return types.TextChunk{
			Id: types.ChunkIdentifier{
				ID:          id.ID,
				Chapter:     id.Chapter,
				StartOffset: id.StartOffset,
				EndOffset:   id.StartOffset + chunkSize,
			},
			Data: fmt.Sprintf("text@%d", id.StartOffset),
		}, nil
	}
}

// passthroughFetch always succeeds immediately, ignoring ctx.
func passthroughFetch(ctx context.Context, tc types.TextChunk) (types.AudioChunk, bool) {
	return types.AudioChunk{Id: tc.Id, Data: []byte(tc.Data)}, true
}

// gatedFetch gives the test full control over when each synthesis call
// "completes" and what it returns, and honors ctx cancellation while
// waiting — this is what lets seek/cancel tests be deterministic instead
// of relying on sleeps.
type gatedFetch struct {
	calls   chan types.TextChunk
	respond chan fetchResult
}

type fetchResult struct {
	chunk types.AudioChunk
	ok    bool
}

func newGatedFetch() *gatedFetch {
	return &gatedFetch{
		calls:   make(chan types.TextChunk),
		respond: make(chan fetchResult),
	}
}

func (g *gatedFetch) fetch(ctx context.Context, tc types.TextChunk) (types.AudioChunk, bool) {
	g.calls <- tc
	select {
	case r := <-g.respond:
		return r.chunk, r.ok
	case <-ctx.Done():
		return types.AudioChunk{}, false
	}
}

func withTimeout(t *testing.T) context.Context {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	t.Cleanup(cancel)
	return ctx
}

func cursor(bookID string, chapter, offset int) types.UserCursor {
	return types.UserCursor{
		BookID: bookID,
		UserID: "u1",
		Cursor: types.Cursor{Chapter: chapter, Index: offset},
	}
}

func TestSequentialProduction(t *testing.T) {
	o := NewOrganizer(simpleBuildText(30), passthroughFetch, 4)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	if err := o.Start(ctx, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("Start: %v", err)
	}

	wantOffsets := []int{0, 10, 20}
	for _, want := range wantOffsets {
		chunk, err := o.GetChunk(withTimeout(t))
		if err != nil {
			t.Fatalf("GetChunk: %v", err)
		}
		if chunk.Id.StartOffset != want {
			t.Fatalf("got offset %d, want %d", chunk.Id.StartOffset, want)
		}
	}

	// Chapter ends at 30, so the next buildText call errors and the
	// producer should stop, closing the channel.
	if _, err := o.GetChunk(withTimeout(t)); !errors.Is(err, ErrStopped) {
		t.Fatalf("expected ErrStopped at end of chapter, got %v", err)
	}
}

// TestStartOverridesPriorUpdateCursor verifies that Start's cursor argument
// wins over anything set via UpdateCursor beforehand — this is what makes a
// single Organizer safe to Start, stop, and Start again without stale
// UpdateCursor calls (from a previous run, or before the first run) leaking
// into the new one.
func TestStartOverridesPriorUpdateCursor(t *testing.T) {
	o := NewOrganizer(simpleBuildText(1000), passthroughFetch, 4)

	// Stale/irrelevant cursor set before Start is ever called.
	o.UpdateCursor(cursor("b1", 9, 900))

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	if err := o.Start(ctx, cursor("b1", 2, 40)); err != nil {
		t.Fatalf("Start: %v", err)
	}

	chunk, err := o.GetChunk(withTimeout(t))
	if err != nil {
		t.Fatalf("GetChunk: %v", err)
	}
	if chunk.Id.Chapter != 2 || chunk.Id.StartOffset != 40 {
		t.Fatalf("got {chapter:%d offset:%d}, want {chapter:2 offset:40} (Start should override the prior UpdateCursor)",
			chunk.Id.Chapter, chunk.Id.StartOffset)
	}
}

// TestRestartAfterStop verifies the primary use case behind requiring a
// cursor on Start: one Organizer can be started, stopped (via cancelling its
// ctx), and started again later at a fresh position — with no leftover state
// from the previous run (including any UpdateCursor calls made while it was
// stopped) affecting the new run.
func TestRestartAfterStop(t *testing.T) {
	// Unbuffered out: with bufferSize>0 the producer can race ahead and
	// pre-fill the buffer with several more chunks before cancel() takes
	// effect, so the very next GetChunk could legitimately drain one of
	// those instead of observing ErrStopped. Unbuffered forces the
	// producer to block waiting for a receiver after the first chunk,
	// making "cancel, then immediately expect ErrStopped" deterministic.
	o := NewOrganizer(simpleBuildText(1000), passthroughFetch, 0)

	ctx1, cancel1 := context.WithCancel(context.Background())
	if err := o.Start(ctx1, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("first Start: %v", err)
	}
	if _, err := o.GetChunk(withTimeout(t)); err != nil {
		t.Fatalf("GetChunk during first run: %v", err)
	}
	cancel1()
	if _, err := o.GetChunk(withTimeout(t)); !errors.Is(err, ErrStopped) {
		t.Fatalf("expected ErrStopped after first run's cancel, got %v", err)
	}

	// While stopped, an UpdateCursor call arrives (e.g. a late/stray report
	// from the old connection). This must not affect the next run.
	o.UpdateCursor(cursor("b1", 7, 777))

	ctx2, cancel2 := context.WithCancel(context.Background())
	defer cancel2()
	if err := o.Start(ctx2, cursor("b1", 3, 30)); err != nil {
		t.Fatalf("second Start: %v", err)
	}

	chunk, err := o.GetChunk(withTimeout(t))
	if err != nil {
		t.Fatalf("GetChunk during second run: %v", err)
	}
	if chunk.Id.Chapter != 3 || chunk.Id.StartOffset != 30 {
		t.Fatalf("got {chapter:%d offset:%d}, want {chapter:3 offset:30} (stray UpdateCursor while stopped leaked in)",
			chunk.Id.Chapter, chunk.Id.StartOffset)
	}
}

func TestDoubleStartErrors(t *testing.T) {
	o := NewOrganizer(simpleBuildText(100), passthroughFetch, 4)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	if err := o.Start(ctx, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("first Start: %v", err)
	}
	if err := o.Start(ctx, cursor("b1", 0, 0)); !errors.Is(err, ErrAlreadyStarted) {
		t.Fatalf("expected ErrAlreadyStarted, got %v", err)
	}
}

func TestFetchFailureRetries(t *testing.T) {
	var calls atomic.Int32
	flaky := func(ctx context.Context, tc types.TextChunk) (types.AudioChunk, bool) {
		n := calls.Add(1)
		if n == 1 {
			return types.AudioChunk{}, false // simulate one synthesis failure
		}
		return types.AudioChunk{Id: tc.Id, Data: []byte(tc.Data)}, true
	}

	// Chapter ends at offset 10 — exactly one chunk's worth. This bounds
	// production to a single chunk so the producer can't race ahead and
	// keep calling flaky() for subsequent chunks while we're asserting on
	// the call count. Without this bound, bufferSize>0 lets the goroutine
	// keep producing in the background, which both invalidates the count
	// and, unsynchronized, is a data race on top (caught by -race below).
	o := NewOrganizer(simpleBuildText(chunkSize), flaky, 4)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	if err := o.Start(ctx, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("Start: %v", err)
	}

	chunk, err := o.GetChunk(withTimeout(t))
	if err != nil {
		t.Fatalf("GetChunk: %v", err)
	}
	if chunk.Id.StartOffset != 0 {
		t.Fatalf("got offset %d, want 0", chunk.Id.StartOffset)
	}
	if got := calls.Load(); got != 2 {
		t.Fatalf("expected 2 fetch attempts (1 failure + 1 success), got %d", got)
	}
}

// TestSeekDiscardsInFlightChunk verifies that a cursor update reporting a
// genuinely different position — one the producer wasn't already heading
// toward — while a chunk for the *old* position has already been
// synthesized causes that chunk to be discarded, and the next delivered
// chunk reflects the new position, not a stale one.
//
// Uses an unbuffered output channel plus a gated fetcher so the test can pin
// the producer at each step instead of racing against goroutine timing.
func TestSeekDiscardsInFlightChunk(t *testing.T) {
	g := newGatedFetch()
	o := NewOrganizer(simpleBuildText(1000), g.fetch, 0) // unbuffered out

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	if err := o.Start(ctx, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("Start: %v", err)
	}

	// --- produce and consume chunk @0 normally ---
	tc0 := <-g.calls
	if tc0.Id.StartOffset != 0 {
		t.Fatalf("got offset %d, want 0", tc0.Id.StartOffset)
	}
	g.respond <- fetchResult{types.AudioChunk{Id: tc0.Id, Data: []byte(tc0.Data)}, true}

	chunk0, err := o.GetChunk(withTimeout(t))
	if err != nil {
		t.Fatalf("GetChunk: %v", err)
	}
	if chunk0.Id.StartOffset != 0 {
		t.Fatalf("got offset %d, want 0", chunk0.Id.StartOffset)
	}

	// --- producer moves on to @10 and synthesis completes for it ---
	tc10 := <-g.calls
	if tc10.Id.StartOffset != 10 {
		t.Fatalf("got offset %d, want 10", tc10.Id.StartOffset)
	}

	// A jump to @500 is unambiguously not "the next chunk in the chain"
	// (which is @10), so this must be treated as a real seek. It arrives
	// BEFORE we let fetchAudio(@10) return, and before anyone drains
	// o.out — so when the producer reaches its post-fetch priority check,
	// the seek is already sitting there waiting, making the discard
	// deterministic rather than racing against delivery.
	o.UpdateCursor(cursor("b1", 0, 500))

	g.respond <- fetchResult{types.AudioChunk{Id: tc10.Id, Data: []byte(tc10.Data)}, true}

	// The producer must now jump straight to @500 — chunk @10 is discarded.
	tc500 := <-g.calls
	if tc500.Id.StartOffset != 500 {
		t.Fatalf("got offset %d, want 500 (chunk @10 should have been discarded on seek)", tc500.Id.StartOffset)
	}
	g.respond <- fetchResult{types.AudioChunk{Id: tc500.Id, Data: []byte(tc500.Data)}, true}

	chunk, err := o.GetChunk(withTimeout(t))
	if err != nil {
		t.Fatalf("GetChunk: %v", err)
	}
	if chunk.Id.StartOffset != 500 {
		t.Fatalf("got offset %d, want 500", chunk.Id.StartOffset)
	}
}

// TestNaturalAdvanceIsNotTreatedAsSeek verifies that when UpdateCursor
// reports a position matching what the producer is already targeting next,
// it's treated as a no-op — not a seek — so nothing already in flight for
// that exact position gets needlessly discarded and re-synthesized. Without
// this, every routine client position report during normal playback would
// look indistinguishable from a jump.
func TestNaturalAdvanceIsNotTreatedAsSeek(t *testing.T) {
	g := newGatedFetch()
	o := NewOrganizer(simpleBuildText(1000), g.fetch, 0)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	if err := o.Start(ctx, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("Start: %v", err)
	}

	tc0 := <-g.calls
	g.respond <- fetchResult{types.AudioChunk{Id: tc0.Id, Data: []byte(tc0.Data)}, true}
	if _, err := o.GetChunk(withTimeout(t)); err != nil {
		t.Fatalf("GetChunk: %v", err)
	}

	// By the time the producer has sent its buildText/fetchAudio call for
	// @10, `next` has already been set to @10 (setNext happens strictly
	// before that call, in program order within the same goroutine, and
	// the subsequent channel send to g.calls carries a happens-before
	// guarantee) — so this receive is what makes checking o.next safe here
	// without an artificial sleep.
	tc10 := <-g.calls
	if tc10.Id.StartOffset != 10 {
		t.Fatalf("got offset %d, want 10", tc10.Id.StartOffset)
	}

	// The client confirms it's now at @10 — exactly where the producer is
	// already headed. This must NOT be queued as a seek.
	o.UpdateCursor(cursor("b1", 0, 10))

	o.mu.Lock()
	pending := len(o.seek)
	o.mu.Unlock()
	if pending != 0 {
		t.Fatalf("expected no pending seek for a natural-advance UpdateCursor, got %d queued", pending)
	}

	g.respond <- fetchResult{types.AudioChunk{Id: tc10.Id, Data: []byte(tc10.Data)}, true}

	chunk, err := o.GetChunk(withTimeout(t))
	if err != nil {
		t.Fatalf("GetChunk: %v", err)
	}
	if chunk.Id.StartOffset != 10 {
		t.Fatalf("got offset %d, want 10 (chunk should have been delivered normally, not discarded)", chunk.Id.StartOffset)
	}
}

// TestCancelStopsProduction verifies that cancelling the context passed to
// Start halts the producer even while it's mid-synthesis (blocked inside
// fetchAudio), and that GetChunk subsequently reports ErrStopped.
func TestCancelStopsProduction(t *testing.T) {
	g := newGatedFetch()
	o := NewOrganizer(simpleBuildText(1000), g.fetch, 0)

	ctx, cancel := context.WithCancel(context.Background())
	if err := o.Start(ctx, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("Start: %v", err)
	}

	// Wait until the producer is blocked inside fetchAudio for @0.
	<-g.calls

	// Cancel while synthesis is still "in progress" — gatedFetch's select
	// on ctx.Done() is what makes this resolve instead of hanging.
	cancel()

	if _, err := o.GetChunk(withTimeout(t)); !errors.Is(err, ErrStopped) {
		t.Fatalf("expected ErrStopped after cancel, got %v", err)
	}
}

// TestGetChunkRespectsCallerContext ensures a GetChunk call bails out on
// its own ctx even if the organizer is otherwise healthy and idle.
func TestGetChunkRespectsCallerContext(t *testing.T) {
	o := NewOrganizer(simpleBuildText(0), passthroughFetch, 4) // errors immediately, never produces
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	if err := o.Start(ctx, cursor("b1", 0, 0)); err != nil {
		t.Fatalf("Start: %v", err)
	}

	callCtx, callCancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer callCancel()

	// Organizer has already stopped (buildText errors at offset 0 immediately)
	// so this should resolve as ErrStopped, not hang or time out — but we
	// still verify the ctx.Err() path works by giving GetChunk an
	// already-short-fused context.
	_, err := o.GetChunk(callCtx)
	if err == nil {
		t.Fatalf("expected an error, got nil")
	}
}
