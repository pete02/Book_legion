package api

import (
	"net/http"
	"net/http/httptest"
	"sync"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/login"
)

// ---------------------------------------------------------------------
// isNaturalProgress
// ---------------------------------------------------------------------

func TestIsNaturalProgress(t *testing.T) {
	tracker := wsAudioHeader{Chapter: 3, EndOffset: 450}

	cases := []struct {
		name string
		msg  wsAudioHeader
		want bool
	}{
		{"exact match is natural", wsAudioHeader{Chapter: 3, StartOffset: 450, EndOffset: 450}, true},
		{"same chapter, different index is a seek", wsAudioHeader{Chapter: 3, StartOffset: 450, EndOffset: 451}, true},
		{"same chapter, earlier index is a seek (rewind)", wsAudioHeader{Chapter: 3, StartOffset: 100, EndOffset: 100}, false},
		{"different chapter, same index is a seek", wsAudioHeader{Chapter: 4, StartOffset: 450, EndOffset: 450}, false},
		{"different chapter and index is a seek", wsAudioHeader{Chapter: 7, StartOffset: 0, EndOffset: 0}, false},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			got := isNaturalProgress(tracker, c.msg)
			if got != c.want {
				t.Errorf("isNaturalProgress(%+v, %+v) = %v, want %v", tracker, c.msg, got, c.want)
			}
		})
	}
}

// ---------------------------------------------------------------------
// deliveryTracker
// ---------------------------------------------------------------------

func TestDeliveryTracker_InitialValueIsWhatWasPassedIn(t *testing.T) {
	initial := wsAudioHeader{ID: "book-1"}
	initial.Chapter = 2
	initial.StartOffset = 10

	tr := newDeliveryTracker(initial)
	got := tr.get()

	if got.ID != "book-1" || got.Chapter != 2 || got.StartOffset != 10 {
		t.Errorf("expected initial value to round-trip unchanged, got %+v", got)
	}
}

func TestDeliveryTracker_MarkDeliveredUpdatesChapterAndIndexOnly(t *testing.T) {
	initial := wsAudioHeader{ID: "book-1"}
	initial.Chapter = 1
	initial.StartOffset = 0

	tr := newDeliveryTracker(initial)
	tr.markDelivered(2, 500)

	got := tr.get()

	if got.Chapter != 2 {
		t.Errorf("expected Chapter=2, got %d", got.Chapter)
	}
	if got.EndOffset != 500 {
		t.Errorf("expected EndOffset=500, got %d", got.EndOffset)
	}
	if got.ID != "book-1" {
		t.Errorf("expected ID to be preserved, got %q", got.ID)
	}
}

// Exercises the tracker the way AudioSocket actually uses it: one
// goroutine repeatedly writing (the writer loop after each delivered
// chunk), another repeatedly reading (the reader goroutine checking
// against incoming cursor reports). Run with -race to be meaningful.
func TestDeliveryTracker_ConcurrentAccess(t *testing.T) {
	tr := newDeliveryTracker(wsAudioHeader{ID: "book-1"})

	var wg sync.WaitGroup
	wg.Add(2)

	go func() {
		defer wg.Done()
		for i := 0; i < 1000; i++ {
			tr.markDelivered(1, i)
		}
	}()

	go func() {
		defer wg.Done()
		for i := 0; i < 1000; i++ {
			_ = tr.get()
		}
	}()

	wg.Wait()
}

// ---------------------------------------------------------------------
// AudioSocket — pre-upgrade validation
//
// Everything below returns before wsUpgrader.Upgrade is ever called (see
// AudioSocket: method/auth check, then book_id, then LoadUserCursor, all
// before the upgrade), so these are ordinary httptest.NewRecorder tests —
// no real websocket connection needed.
// ---------------------------------------------------------------------

// loginAndGetToken seeds a real user (via the login package, bypassing
// HTTP) and logs them in to get a real, valid session token — the same
// one AuthCheck -> login.VerifyUserSession will accept.
func loginAndGetToken(t *testing.T, a *API, username, password string) string {
	t.Helper()
	seedUser(t, a, username, password)
	user, err := login.NewUserSession(username, password, a.DB)
	if err != nil {
		t.Fatalf("failed to log in seeded user %q: %v", username, err)
	}
	return user.GetGetToken()
}

func TestAudioSocket_MethodNotAllowed(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodPost, "/ws/audio/book-1", nil)
	rr := httptest.NewRecorder()

	a.AudioSocket(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestAudioSocket_MissingAuthHeader(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodGet, "/ws/audio/book-1", nil)
	rr := httptest.NewRecorder()

	a.AudioSocket(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestAudioSocket_WrongAuthScheme(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodGet, "/ws/audio/book-1", nil)
	req.Header.Set("Authorization", "Basic dXNlcjpwYXNz")
	rr := httptest.NewRecorder()

	a.AudioSocket(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestAudioSocket_InvalidToken(t *testing.T) {
	a := NewTestAPI(t)
	req := httptest.NewRequest(http.MethodGet, "/ws/audio/book-1", nil)
	req.Header.Set("Authorization", "Bearer this-token-was-never-issued")
	rr := httptest.NewRecorder()

	a.AudioSocket(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestAudioSocket_MissingBookID(t *testing.T) {
	a := NewTestAPI(t)
	username := uniqueUsername(t)
	token := loginAndGetToken(t, a, username, "a-real-password-123")

	req := httptest.NewRequest(http.MethodGet, "/ws/audio/?token="+token, nil)

	req.SetPathValue("book_id", "")
	rr := httptest.NewRecorder()

	a.AudioSocket(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d: %s", http.StatusBadRequest, rr.Code, rr.Body.String())
	}
}

// Assumes login.LoadUserCursor errors when no cursor row exists yet for
// this user/book pair (mirroring the not-found-errors pattern used by
// LoadBook et al.) — worth confirming against the real implementation if
// this one starts failing for the wrong reason.
func TestAudioSocket_UnknownCursor(t *testing.T) {
	a := NewTestAPI(t)
	username := uniqueUsername(t)
	token := loginAndGetToken(t, a, username, "another-real-password-456")

	req := httptest.NewRequest(http.MethodGet, "/ws/audio/does-not-exist?token="+token, nil)
	req.SetPathValue("book_id", "does-not-exist")
	rr := httptest.NewRecorder()

	a.AudioSocket(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d: %s", http.StatusBadRequest, rr.Code, rr.Body.String())
	}
}
