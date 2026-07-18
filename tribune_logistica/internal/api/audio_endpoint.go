package api

import (
	"context"
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"

	types "github.com/book_legion-tribune_logistica/internal/types"
)

const writeWait = 5 * time.Second

var wsUpgrader = websocket.Upgrader{
	ReadBufferSize:  4096,
	WriteBufferSize: 4096,
	// TODO: restrict to your actual frontend origin(s) before shipping —
	// wide open for now so this is easy to test locally.
	CheckOrigin: func(r *http.Request) bool { return true },
}

// wsCursorMessage is what the client sends to report/seek its position.
// Wire-format only — deliberately not types.UserCursor itself, since the
// client only ever needs to tell us where it is within the current book,
// not the book ID (that's fixed for the life of the connection).
type wsCursorMessage struct {
	Chapter int `json:"chapter"`
	Index   int `json:"index"`
}

// wsAudioHeader precedes each binary audio frame so the client knows what
// it's about to receive.
type wsAudioHeader struct {
	ID          string `json:"id"`
	Chapter     int    `json:"chapter"`
	StartOffset int    `json:"start_offset"`
	EndOffset   int    `json:"end_offset"`
}

// deliveryTracker records the position corresponding to the end of the
// most recently delivered chunk on this connection. This — not
// Organizer.next — is the correct reference point for deciding whether an
// incoming client cursor report is natural forward progress or an actual
// seek: `next` reflects how far the Organizer has *produced*, which can
// run ahead of the client by up to bufferSize chunks, so comparing against
// it would misfire constantly. Comparing against what this connection has
// actually sent is exact.
//
// Written by the writer loop (after each successful delivery), read by the
// reader goroutine (on each incoming cursor message) — hence the mutex.
type deliveryTracker struct {
	mu   sync.Mutex
	last types.UserCursor
}

func newDeliveryTracker(initial types.UserCursor) *deliveryTracker {
	return &deliveryTracker{last: initial}
}

func (t *deliveryTracker) markDelivered(chapter, endOffset int) {
	t.mu.Lock()
	t.last.Cursor.Chapter = chapter
	t.last.Cursor.Index = endOffset
	t.mu.Unlock()
}

func (t *deliveryTracker) get() types.UserCursor {
	t.mu.Lock()
	defer t.mu.Unlock()
	return t.last
}

// isNaturalProgress reports whether a client-reported cursor position
// matches what's actually been delivered on this connection so far —
// i.e. whether it's normal playback progress rather than a seek. Pulled
// out as its own function so this decision (the whole point of
// deliveryTracker existing) can be unit tested without a real websocket
// connection.
func isNaturalProgress(delivered types.UserCursor, msg wsCursorMessage) bool {
	return msg.Chapter == delivered.Cursor.Chapter && msg.Index == delivered.Cursor.Index
}

// AudioSocket streams synthesized audio for a book over a WebSocket.
//
// Wire protocol:
//   - Client -> server: a JSON text message shaped like wsCursorMessage,
//     sent any time the client's playback position changes (seek or
//     periodic progress report).
//   - Server -> client: for each chunk, one JSON text message shaped like
//     wsAudioHeader, immediately followed by one binary message containing
//     that chunk's raw audio bytes.
//
// Route registration (Go 1.22+ ServeMux, matching the {id} pattern already
// used by UpdateSeriesName):
//
//	mux.HandleFunc("GET /ws/audio/{book_id}", api.AudioSocket)
//
// NOTE: api.Manager is a single, shared *manager.Organizer — Start()
// returns ErrAlreadyStarted if it's already running, so this endpoint
// supports one active playback session at a time across the whole API
// instance, not one per user. Flagging again in case that's not the
// intent long-term.
func (api *API) AudioSocket(w http.ResponseWriter, r *http.Request) {
	userID, ok := api.RequestCheck(w, r, http.MethodGet)
	if !ok {
		return
	}

	bookID := r.PathValue("book_id")
	if bookID == "" {
		http.Error(w, "book_id is required", http.StatusBadRequest)
		return
	}

	cursor, err := types.LoadUserCursor(api.DB, userID, bookID)
	if err != nil {
		http.Error(w, "could not load reading position", http.StatusInternalServerError)
		return
	}

	conn, err := wsUpgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("[audio] upgrade failed for user %s: %v", userID, err)
		return
	}
	defer conn.Close()

	ctx, cancel := context.WithCancel(r.Context())
	defer cancel()

	if err := api.Manager.Start(ctx, cursor); err != nil {
		log.Printf("[audio] failed to start playback for user %s: %v", userID, err)
		_ = conn.WriteControl(websocket.CloseMessage,
			websocket.FormatCloseMessage(websocket.ClosePolicyViolation, "playback already in progress"),
			time.Now().Add(writeWait))
		return
	}

	// Nothing has been delivered yet, so the client's own starting
	// position (what we just loaded and started the Organizer from) is
	// the correct initial "last delivered" reference.
	tracker := newDeliveryTracker(cursor)

	// Reader: the connection's only reader. For each reported position,
	// decides natural-progress vs seek by comparing against what's
	// actually been delivered (tracker), not against the Organizer's own
	// lookahead — and cancels ctx the moment the connection goes away,
	// which is what stops both the writer loop below and the Organizer's
	// internal producer goroutine.
	readerDone := make(chan struct{})
	go func() {
		defer close(readerDone)
		defer cancel()

		for {
			var msg wsCursorMessage
			if err := conn.ReadJSON(&msg); err != nil {
				if websocket.IsUnexpectedCloseError(err, websocket.CloseNormalClosure, websocket.CloseGoingAway) {
					log.Printf("[audio] connection error for user %s: %v", userID, err)
				}
				return
			}

			reported := cursor // copy, to carry over BookID
			reported.Cursor.Chapter = msg.Chapter
			reported.Cursor.Index = msg.Index

			delivered := tracker.get()
			natural := isNaturalProgress(delivered, msg)

			if !natural {
				api.Manager.UpdateCursor(reported)
			}

			// Persist either way, so a reload picks up wherever they
			// actually left off.
			//
			// TODO: confirm the real name/signature — assuming a
			// SaveUserCursor counterpart to LoadUserCursor. Also worth
			// considering throttling this (e.g. only every N seconds or
			// M chunks) if clients report position frequently, since
			// this fires on every message including natural-progress
			// pings.
			if err := reported.SaveUserCursor(api.DB); err != nil {
				log.Printf("[audio] failed to persist cursor for user %s: %v", userID, err)
			}
		}
	}()

	// Writer: this loop is the connection's only writer — gorilla/websocket
	// connections don't support concurrent writes from multiple goroutines,
	// so all outbound traffic (audio here, the close frame above, which
	// happens before this starts) funnels through here.
	for {
		chunk, err := api.Manager.GetChunk(ctx)
		if err != nil {
			// Either manager.ErrStopped (Organizer reached the end of the
			// book, or was stopped) or ctx cancellation (client
			// disconnected). Either way, we're done.
			break
		}

		header := wsAudioHeader{
			ID:          chunk.Id.ID,
			Chapter:     chunk.Id.Chapter,
			StartOffset: chunk.Id.StartOffset,
			EndOffset:   chunk.Id.EndOffset,
		}

		conn.SetWriteDeadline(time.Now().Add(writeWait))
		if err := conn.WriteJSON(header); err != nil {
			log.Printf("[audio] failed to write header for user %s: %v", userID, err)
			break
		}
		conn.SetWriteDeadline(time.Now().Add(writeWait))
		if err := conn.WriteMessage(websocket.BinaryMessage, chunk.Data); err != nil {
			log.Printf("[audio] failed to write audio for user %s: %v", userID, err)
			break
		}

		tracker.markDelivered(chunk.Id.Chapter, chunk.Id.EndOffset)
	}

	// If we broke out of the writer loop for our own reasons (a write
	// failure, not a disconnect the reader already caught), make sure the
	// Organizer stops and the reader goroutine winds down too.
	cancel()
	<-readerDone
}
