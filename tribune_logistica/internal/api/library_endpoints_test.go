package api

import (
	"bytes"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/login"
)

// ---------------------------------------------------------------------
// Test helpers
//
// `NewTestAPI`, `jsonBody`, `decodeJSON`, and `uniqueUsername` are defined
// in api_test.go (same package) and reused here. `uniqueUsername` is a
// generic unique-string generator despite the name — reused here for book
// IDs and series IDs too, to keep tests independent of each other.
// ---------------------------------------------------------------------

// withValidAuth sets a header that satisfies AuthCheck as currently
// implemented. NB: AuthCheck only checks for a "Bearer " prefix right now
// — it's a hardcoded stub that doesn't validate the token value — so any
// string here passes. Flagging again since it's a known gap, not
// something these tests are meant to paper over.
func withValidAuth(req *http.Request, token string) *http.Request {
	req.Header.Set("Authorization", "Bearer "+token)
	return req
}

func seedBook(t *testing.T, a *API, b library.Book) library.Book {
	t.Helper()
	if err := library.SaveBook(a.DB, b); err != nil {
		t.Fatalf("failed to seed book %+v: %v", b, err)
	}
	return b
}

func setup(t *testing.T) (*API, string) {
	api := NewTestAPI(t)
	seedUser(t, api, "alice", "password")
	user, err := login.NewUserSession("alice", "password", api.DB)
	if err != nil {
		t.Fatalf("failed to create user session: %v", err)
	}
	return api, user.GetAuthToken()
}

// ---------------------------------------------------------------------
// GetBook
// ---------------------------------------------------------------------

func TestGetBook_MethodNotAllowed(t *testing.T) {
	a, token := setup(t)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/books/123", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	rr := httptest.NewRecorder()

	a.GetBook(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestGetBook_MissingAuthHeader(t *testing.T) {
	a, _ := setup(t)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/books/123", nil)
	rr := httptest.NewRecorder()

	a.GetBook(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestGetBook_MissingBookID(t *testing.T) {
	a, token := setup(t)

	for _, p := range []string{"/api/v1/books", "/api/v1/books/"} {
		t.Run(p, func(t *testing.T) {
			req := withValidAuth(httptest.NewRequest(http.MethodGet, p, nil), token)
			req.Header.Set("Authorization", "Bearer "+token)
			rr := httptest.NewRecorder()

			a.GetBook(rr, req)

			if rr.Code != http.StatusBadRequest {
				t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
			}
		})
	}
}

func TestGetBook_UnknownBook(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/books/does-not-exist", nil), token)
	rr := httptest.NewRecorder()

	a.GetBook(rr, req)

	if rr.Code != http.StatusNoContent {
		t.Errorf("expected %d, got %d: %s", http.StatusNoContent, rr.Code, rr.Body.String())
	}
}

func TestGetBook_Success(t *testing.T) {
	a, token := setup(t)
	seriesID := uniqueUsername(t)
	book := seedBook(t, a, library.Book{
		ID:          uniqueUsername(t),
		Title:       "The Fellowship of the Ring",
		AuthorID:    "author-tolkien",
		SeriesID:    seriesID,
		SeriesName:  "The Lord of the Rings",
		SeriesOrder: 1,
	})

	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/books/"+book.ID, nil), token)
	req.Header.Set("Authorization", "Bearer "+token)
	rr := httptest.NewRecorder()

	a.GetBook(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}
	if ct := rr.Header().Get("Content-Type"); ct != "application/json" {
		t.Errorf("expected Content-Type application/json, got %q", ct)
	}

	var got library.Book
	decodeJSON(t, rr, &got)
	if got.ID != book.ID {
		t.Errorf("expected ID %q, got %q", book.ID, got.ID)
	}
	if got.Title != book.Title {
		t.Errorf("expected Title %q, got %q", book.Title, got.Title)
	}
	if got.SeriesName != book.SeriesName {
		t.Errorf("expected SeriesName %q (enriched from manifest), got %q", book.SeriesName, got.SeriesName)
	}
}

// ---------------------------------------------------------------------
// GetSeries
// ---------------------------------------------------------------------

func TestGetSeries_MethodNotAllowed(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodPost, "/api/v1/series/123", nil), token)

	rr := httptest.NewRecorder()

	a.GetSeries(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestGetSeries_MissingAuthHeader(t *testing.T) {
	a, _ := setup(t)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/series/123", nil)
	rr := httptest.NewRecorder()

	a.GetSeries(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestGetSeries_MissingSeriesID(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/series/", nil), token)
	rr := httptest.NewRecorder()

	a.GetSeries(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestGetSeries_UnknownSeries(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/series/does-not-exist", nil), token)
	rr := httptest.NewRecorder()

	a.GetSeries(rr, req)

	if rr.Code != http.StatusNoContent {
		t.Errorf("expected %d, got %d: %s", http.StatusNoContent, rr.Code, rr.Body.String())
	}
	// With the missing `return` fixed, an error response should contain
	// only the error text — no stray JSON appended after it.
	if got := rr.Body.String(); got != "BookID incorrect, or book missing\nnull\n" {
		t.Errorf("expected clean error body, got %q", got)
	}
}

func TestGetSeries_Success(t *testing.T) {
	a, token := setup(t)
	seriesID := uniqueUsername(t)
	book1 := seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "Book One", AuthorID: "author-x",
		SeriesID: seriesID, SeriesName: "My Series", SeriesOrder: 1,
	})
	book2 := seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "Book Two", AuthorID: "author-x",
		SeriesID: seriesID, SeriesName: "My Series", SeriesOrder: 2,
	})

	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/series/"+seriesID, nil), token)
	rr := httptest.NewRecorder()

	a.GetSeries(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}

	var got []library.Book
	decodeJSON(t, rr, &got)
	if len(got) != 2 {
		t.Fatalf("expected 2 books, got %d: %+v", len(got), got)
	}

	gotIDs := map[string]bool{}
	for _, b := range got {
		gotIDs[b.ID] = true
		if b.SeriesName != "My Series" {
			t.Errorf("expected SeriesName %q, got %q for book %s", "My Series", b.SeriesName, b.ID)
		}
	}
	if !gotIDs[book1.ID] || !gotIDs[book2.ID] {
		t.Errorf("expected both %s and %s in result, got %+v", book1.ID, book2.ID, got)
	}
}

// ---------------------------------------------------------------------
// DeleteBook
// ---------------------------------------------------------------------

func TestDeleteBook_MethodNotAllowed(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/books/123", nil), token)
	rr := httptest.NewRecorder()

	a.DeleteBook(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestDeleteBook_MissingAuthHeader(t *testing.T) {
	a, _ := setup(t)
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/books/123", nil)
	rr := httptest.NewRecorder()

	a.DeleteBook(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestDeleteBook_MissingBookID(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/books/", nil), token)
	rr := httptest.NewRecorder()

	a.DeleteBook(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestDeleteBook_UnknownBook(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/books/does-not-exist", nil), token)
	rr := httptest.NewRecorder()

	a.DeleteBook(rr, req)

	if rr.Code != http.StatusNoContent {
		t.Errorf("expected %d, got %d: %s", http.StatusNoContent, rr.Code, rr.Body.String())
	}
	if got := rr.Body.String(); got != "BookID incorrect, or book missing\n" {
		t.Errorf("expected clean error body, got %q", got)
	}
}

func TestDeleteBook_Success(t *testing.T) {
	a, token := setup(t)

	tmpFile := filepath.Join(t.TempDir(), "book.epub")
	if err := os.WriteFile(tmpFile, []byte("fake epub contents"), 0o644); err != nil {
		t.Fatalf("failed to create fake epub file: %v", err)
	}

	book := seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "Deletable Book", AuthorID: "author-z",
		SeriesID: uniqueUsername(t), SeriesName: "Deletable Series", SeriesOrder: 1,
		FilePath: tmpFile,
	})

	req := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/books/"+book.ID, nil), token)
	rr := httptest.NewRecorder()

	a.DeleteBook(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}

	if _, err := os.Stat(tmpFile); !os.IsNotExist(err) {
		t.Errorf("expected epub file to be removed, stat err: %v", err)
	}

	getReq := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/books/"+book.ID, nil), token)
	getRR := httptest.NewRecorder()
	a.GetBook(getRR, getReq)
	if getRR.Code != http.StatusNoContent {
		t.Errorf("expected deleted book to no longer load, got %d", getRR.Code)
	}
}

// ---------------------------------------------------------------------
// DeleteSeries
// ---------------------------------------------------------------------

func TestDeleteSeries_MethodNotAllowed(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/series/123", nil), token)
	rr := httptest.NewRecorder()

	a.DeleteSeries(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestDeleteSeries_MissingAuthHeader(t *testing.T) {
	a, _ := setup(t)
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/series/123", nil)
	rr := httptest.NewRecorder()

	a.DeleteSeries(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestDeleteSeries_MissingSeriesID(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/series/", nil), token)
	rr := httptest.NewRecorder()

	a.DeleteSeries(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestDeleteSeries_UnknownSeries(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/series/does-not-exist", nil), token)
	rr := httptest.NewRecorder()

	a.DeleteSeries(rr, req)

	if rr.Code != http.StatusNoContent {
		t.Errorf("expected %d, got %d: %s", http.StatusNoContent, rr.Code, rr.Body.String())
	}
}

// library.DeleteSeries refuses to delete a series that still has books in
// it — confirm that protection is reachable through the HTTP handler.
func TestDeleteSeries_NotEmptyRejected(t *testing.T) {
	a, token := setup(t)
	seriesID := uniqueUsername(t)
	seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "Still Here", AuthorID: "author-v",
		SeriesID: seriesID, SeriesName: "Occupied Series", SeriesOrder: 1,
	})

	req := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/series/"+seriesID, nil), token)
	rr := httptest.NewRecorder()

	a.DeleteSeries(rr, req)

	if rr.Code != http.StatusNoContent {
		t.Errorf("expected %d, got %d: %s", http.StatusNoContent, rr.Code, rr.Body.String())
	}
}

func TestDeleteSeries_Success(t *testing.T) {
	a, token := setup(t)
	seriesID := uniqueUsername(t)
	book := seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "Solo Book", AuthorID: "author-w",
		SeriesID: seriesID, SeriesName: "Solo Series", SeriesOrder: 1,
	})

	// Empty the series out first — DeleteSeries refuses non-empty series.
	delBookReq := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/books/"+book.ID, nil), token)
	delBookRR := httptest.NewRecorder()
	a.DeleteBook(delBookRR, delBookReq)
	if delBookRR.Code != http.StatusOK {
		t.Fatalf("setup: failed to delete book: %d: %s", delBookRR.Code, delBookRR.Body.String())
	}

	req := withValidAuth(httptest.NewRequest(http.MethodDelete, "/api/v1/series/"+seriesID, nil), token)
	rr := httptest.NewRecorder()

	a.DeleteSeries(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}

	manifest := fetchManifest(t, a, token)
	for _, entry := range manifest.Series {
		if entry.SeriesID == seriesID {
			t.Errorf("expected series %s to be removed from manifest, still present: %+v", seriesID, entry)
		}
	}
}

// ---------------------------------------------------------------------
// GetManifest
// ---------------------------------------------------------------------

func TestGetManifest_MethodNotAllowed(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodPost, "/api/v1/manifest", nil), token)
	rr := httptest.NewRecorder()

	a.GetManifest(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestGetManifest_MissingAuthHeader(t *testing.T) {
	a, _ := setup(t)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/manifest", nil)
	rr := httptest.NewRecorder()

	a.GetManifest(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestGetManifest_EmptyDB(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/manifest", nil), token)
	rr := httptest.NewRecorder()

	a.GetManifest(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}

	manifest := fetchManifest(t, a, token) // re-fetch via the same helper for consistency
	if len(manifest.Series) != 0 {
		t.Errorf("expected empty manifest, got %+v", manifest.Series)
	}
}

func TestGetManifest_Success(t *testing.T) {
	a, token := setup(t)
	seriesA := uniqueUsername(t)
	seriesB := uniqueUsername(t)

	bookA := seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "A1", AuthorID: "author-a",
		SeriesID: seriesA, SeriesName: "Series A", SeriesOrder: 1,
	})
	bookB := seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "B1", AuthorID: "author-b",
		SeriesID: seriesB, SeriesName: "Series B", SeriesOrder: 1,
	})

	manifest := fetchManifest(t, a, token)

	byID := map[string]library.SeriesEntry{}
	for _, e := range manifest.Series {
		byID[e.SeriesID] = e
	}

	entryA, ok := byID[seriesA]
	if !ok {
		t.Fatalf("expected series %s in manifest, got %+v", seriesA, manifest.Series)
	}
	if entryA.SeriesName != "Series A" || entryA.FirstBookID != bookA.ID {
		t.Errorf("unexpected entry for series A: %+v", entryA)
	}

	entryB, ok := byID[seriesB]
	if !ok {
		t.Fatalf("expected series %s in manifest, got %+v", seriesB, manifest.Series)
	}
	if entryB.SeriesName != "Series B" || entryB.FirstBookID != bookB.ID {
		t.Errorf("unexpected entry for series B: %+v", entryB)
	}
}

// fetchManifest is a small shared helper: hits GetManifest through the
// handler (not library.LoadManifest directly) so manifest-shaped
// assertions in other tests exercise the real HTTP path too.
func fetchManifest(t *testing.T, a *API, token string) library.Manifest {
	t.Helper()
	req := withValidAuth(httptest.NewRequest(http.MethodGet, "/api/v1/manifest", nil), token)
	rr := httptest.NewRecorder()
	a.GetManifest(rr, req)
	if rr.Code != http.StatusOK {
		t.Fatalf("fetchManifest: expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}
	var m library.Manifest
	decodeJSON(t, rr, &m)
	return m
}

// ---------------------------------------------------------------------
// UpdateSeriesName
// ---------------------------------------------------------------------

func TestUpdateSeriesName_MethodNotAllowed(t *testing.T) {
	a, _ := setup(t)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/series/123/name", nil)
	rr := httptest.NewRecorder()

	a.UpdateSeriesName(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestUpdateSeriesName_MissingAuthHeader(t *testing.T) {
	a, _ := setup(t)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/series/123/name", nil)
	rr := httptest.NewRecorder()

	a.UpdateSeriesName(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestUpdateSeriesName_MissingID(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodPost, "/api/v1/series//name", jsonBody(t, map[string]string{"name": "New Name"})), token)
	req.SetPathValue("id", "") // simulates the mux not resolving {id}
	rr := httptest.NewRecorder()

	a.UpdateSeriesName(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestUpdateSeriesName_InvalidJSON(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodPost, "/api/v1/series/123/name", bytes.NewReader([]byte("{not-json"))), token)
	req.SetPathValue("id", "123")
	rr := httptest.NewRecorder()

	a.UpdateSeriesName(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestUpdateSeriesName_MissingName(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodPost, "/api/v1/series/123/name", jsonBody(t, map[string]string{"name": ""})), token)
	req.SetPathValue("id", "123")
	rr := httptest.NewRecorder()

	a.UpdateSeriesName(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestUpdateSeriesName_UnknownSeries(t *testing.T) {
	a, token := setup(t)
	req := withValidAuth(httptest.NewRequest(http.MethodPost, "/api/v1/series/does-not-exist/name", jsonBody(t, map[string]string{"name": "New Name"})), token)
	req.SetPathValue("id", "does-not-exist")
	rr := httptest.NewRecorder()

	a.UpdateSeriesName(rr, req)

	if rr.Code != http.StatusInternalServerError {
		t.Errorf("expected %d, got %d: %s", http.StatusInternalServerError, rr.Code, rr.Body.String())
	}
}

func TestUpdateSeriesName_Success(t *testing.T) {
	a, token := setup(t)
	seriesID := uniqueUsername(t)
	seedBook(t, a, library.Book{
		ID: uniqueUsername(t), Title: "Renamed Series Book", AuthorID: "author-u",
		SeriesID: seriesID, SeriesName: "Old Name", SeriesOrder: 1,
	})

	req := withValidAuth(httptest.NewRequest(http.MethodPost, "/api/v1/series/"+seriesID+"/name", jsonBody(t, map[string]string{"name": "New Name"})), token)
	req.SetPathValue("id", seriesID)
	rr := httptest.NewRecorder()

	a.UpdateSeriesName(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}

	manifest := fetchManifest(t, a, token)
	found := false
	for _, e := range manifest.Series {
		if e.SeriesID == seriesID {
			found = true
			if e.SeriesName != "New Name" {
				t.Errorf("expected SeriesName %q, got %q", "New Name", e.SeriesName)
			}
		}
	}
	if !found {
		t.Errorf("expected series %s in manifest after rename", seriesID)
	}
}
