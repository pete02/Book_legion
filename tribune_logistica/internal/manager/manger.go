package manager

import (
	"context"
	"errors"
	"log"
	"sync"

	types "github.com/book_legion-tribune_logistica/internal/types"
)

// TextChunkBuilder resolves a ChunkIdentifier (StartOffset trusted, EndOffset
// ignored) into a TextChunk, using nearestAllowedSplit-style logic to find
// the next valid boundary and slicing chapter HTML between them.
type TextChunkBuilder func(id types.ChunkIdentifier) (types.TextChunk, error)

// AudioFetcher synthesizes audio for a TextChunk. Must respect ctx
// cancellation for in-progress synthesis.
type AudioFetcher func(ctx context.Context, chunk types.TextChunk) (types.AudioChunk, bool)

var ErrStopped = errors.New("organizer stopped")
var ErrAlreadyStarted = errors.New("organizer already started")

type Organizer struct {
	mu sync.Mutex

	buildText  TextChunkBuilder
	fetchAudio AudioFetcher
	bufferSize int

	cursor  types.UserCursor      // last position reported by the client
	next    types.ChunkIdentifier // identifier the producer is currently targeting
	current types.ChunkIdentifier // identifier of the chunk most recently returned by GetChunk

	seek chan types.UserCursor // latest pending seek; capacity 1, newest wins
	out  chan types.AudioChunk // produced audio, ready for GetChunk

	cancel  context.CancelFunc
	started bool
}

func NewOrganizer(buildText TextChunkBuilder, fetchAudio AudioFetcher, bufferSize int) *Organizer {
	return &Organizer{
		buildText:  buildText,
		fetchAudio: fetchAudio,
		bufferSize: bufferSize,
		seek:       make(chan types.UserCursor, 1),
		out:        make(chan types.AudioChunk, bufferSize),
	}
}

func cursorToIdentifier(uc types.UserCursor) types.ChunkIdentifier {
	return types.ChunkIdentifier{
		ID:          uc.BookID,
		Chapter:     uc.Cursor.Chapter,
		StartOffset: uc.Cursor.Index,
	}
}

// --- the three public functions ---

// Start begins production from uc, discarding any position previously set
// via UpdateCursor. This makes a single Organizer safe to Start, cancel (via
// the ctx passed in), and Start again later without stale state from a prior
// run — or from UpdateCursor calls made before this Start — leaking in: the
// cursor, the seek signal, and the output buffer are all reset here.
func (o *Organizer) Start(ctx context.Context, uc types.UserCursor) error {
	o.mu.Lock()
	if o.started {
		o.mu.Unlock()
		return ErrAlreadyStarted
	}
	runCtx, cancel := context.WithCancel(ctx)
	o.cancel = cancel
	o.started = true
	o.cursor = uc
	initial := cursorToIdentifier(uc)
	o.next = initial

	seekCh := make(chan types.UserCursor, 1)
	outCh := make(chan types.AudioChunk, o.bufferSize)
	o.seek = seekCh
	o.out = outCh
	o.mu.Unlock()

	go o.run(runCtx, initial, seekCh, outCh)
	return nil
}

// UpdateCursor reports the client's current position. If this position is
// different from where the producer is already headed, it's treated as a
// seek: production jumps there, discarding any buffered-but-undelivered
// chunk from the old position. If it matches what the producer is already
// targeting, this is just the client confirming normal forward progress —
// not a jump — so nothing is signalled and no in-flight work is discarded.
func (o *Organizer) UpdateCursor(uc types.UserCursor) {
	o.mu.Lock()
	o.cursor = uc
	target := cursorToIdentifier(uc)
	natural := target.StartOffset == o.next.StartOffset
	seekCh := o.seek
	o.mu.Unlock()

	if natural {
		return
	}
	setLatest(seekCh, uc)
}

// GetChunk blocks until the next produced AudioChunk is ready, the organizer
// is stopped (ErrStopped), or ctx is cancelled.
func (o *Organizer) GetChunk(ctx context.Context) (types.AudioChunk, error) {
	o.mu.Lock()
	out := o.out
	o.mu.Unlock()

	select {
	case chunk, ok := <-out:
		if !ok {
			log.Println("Organizer: output channel closed")
			return types.AudioChunk{}, ErrStopped
		}
		o.mu.Lock()
		o.current.StartOffset = chunk.Id.EndOffset
		o.mu.Unlock()
		return chunk, nil
	case <-ctx.Done():
		log.Println("Organizer: context cancelled")
		return types.AudioChunk{}, ctx.Err()
	}
}

// --- internal producer ---

func (o *Organizer) run(ctx context.Context, initial types.ChunkIdentifier, seek chan types.UserCursor, out chan types.AudioChunk) {
	defer func() {
		close(out)
		o.mu.Lock()
		o.started = false
		o.mu.Unlock()
	}()

	next := initial

	for {
		// Priority check #1: don't even start building/synthesizing for
		// `next` if a seek is already waiting — that work would just be
		// thrown away.
		select {
		case <-ctx.Done():
			return
		case seekTo := <-seek:
			next = o.applySeek(seekTo)
			continue
		default:
		}

		textChunk, err := o.buildText(next)
		if err != nil {
			// End of chapter/book, or an unresolvable offset.
			// TODO: chapter-transition handling per design doc section 7.
			log.Printf("Organizer: Got error while building text chunk: %s", err)
			return
		}

		audioChunk, ok := o.fetchAudio(ctx, textChunk)
		if ctx.Err() != nil {
			return
		}
		if !ok {
			log.Printf("Organizer: Got error while fetching audio: %s", err)
			continue // TODO: retry/backoff policy on synthesis failure
		}

		// Priority check #2: synthesis just finished, but a seek may have
		// arrived while it was running. Re-check before attempting to
		// deliver — a chunk synthesized for a position we've since jumped
		// away from must not be handed to the client.
		select {
		case seekTo := <-seek:
			next = o.applySeek(seekTo)
			continue
		default:
		}

		select {
		case out <- audioChunk:
		case <-ctx.Done():
			return
		case seekTo := <-seek:
			next = o.applySeek(seekTo)
			continue
		}

		next = types.ChunkIdentifier{
			ID:          next.ID,
			Chapter:     textChunk.Id.Chapter,
			StartOffset: textChunk.Id.EndOffset,
		}
		o.setNext(next)
	}
}

func (o *Organizer) applySeek(uc types.UserCursor) types.ChunkIdentifier {
	id := cursorToIdentifier(uc)
	o.setNext(id)
	return id
}

func (o *Organizer) setNext(id types.ChunkIdentifier) {
	o.mu.Lock()
	o.next = id
	o.mu.Unlock()
}

func setLatest(ch chan types.UserCursor, uc types.UserCursor) {
	for {
		select {
		case ch <- uc:
			return
		default:
			select {
			case <-ch:
			default:
			}
		}
	}
}
