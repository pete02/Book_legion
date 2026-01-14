package api_test

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/login"
)

func TestLibraryEndpoints(t *testing.T) {
	api, validToken := setupAPIWithAuth(t) // Returns api and a valid auth token
	if _, ok := login.VerifyAuthToken(validToken); !ok {
		t.Fatal("Bad auth token")
	}

	setupBook(t, api, "test")
	setupManifest(t, api)
	// ----------------- 2.1 Get Single Book -----------------
	bookID := "b1"
	seriesID := "s1"

	t.Run("GetSingleBook_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/books/"+bookID, nil)
		req.Header.Set("Authorization", "Bearer "+validToken)
		w := httptest.NewRecorder()

		api.GetBook(w, req)

		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		var bookResp map[string]interface{}
		if err := json.NewDecoder(resp.Body).Decode(&bookResp); err != nil {
			t.Fatalf("Failed to decode book response: %v", err)
		}

		if id, ok := bookResp["id"].(string); !ok || id != bookID {
			t.Fatalf("Expected book id %s, got %v", bookID, bookResp["id"])
		}
	})
	t.Run("GetSeries_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/series/"+seriesID, nil)
		req.Header.Set("Authorization", "Bearer "+validToken)
		w := httptest.NewRecorder()

		api.GetSeries(w, req)

		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		// Decode as a slice of books
		var booksResp []map[string]interface{}
		if err := json.NewDecoder(resp.Body).Decode(&booksResp); err != nil {
			t.Fatalf("Failed to decode books response: %v", err)
		}

		if len(booksResp) == 0 {
			t.Fatalf("Expected at least one book in series %s, got 0", seriesID)
		}

		// Optionally, check that a specific book is in the list
		found := false
		for _, b := range booksResp {
			if id, ok := b["id"].(string); ok && id == bookID {
				found = true
				break
			}
		}
		if !found {
			t.Fatalf("Expected book id %s not found in series response: %+v", bookID, booksResp)
		}
	})

	t.Run("GetSingleBook_Unauthorized", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/books/"+bookID, nil)
		req.Header.Set("Authorization", "Bearer invalid_token")
		w := httptest.NewRecorder()

		api.GetBook(w, req)

		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("Expected 401 Unauthorized, got %d", resp.StatusCode)
		}
	})

	// ----------------- 2.2 Get Library Manifest -----------------
	t.Run("GetManifest_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/manifest", nil)
		req.Header.Set("Authorization", "Bearer "+validToken)
		w := httptest.NewRecorder()

		api.GetManifest(w, req)

		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		var manifestResp struct {
			Series []library.SeriesEntry `json:"series"`
		}
		if err := json.NewDecoder(resp.Body).Decode(&manifestResp); err != nil {
			t.Fatalf("Failed to decode manifest response: %v", err)
		}

		if len(manifestResp.Series) == 0 {
			t.Fatal("Manifest: missing or empty series list")
		}

		// Find series s1
		var firstBookID string
		found := false
		for _, entry := range manifestResp.Series {
			if entry.SeriesID == "s1" {
				firstBookID = entry.FirstBookID
				found = true
				break
			}
		}
		if !found {
			t.Fatal("Manifest: series s1 not found")
		}

		if firstBookID != "b1" {
			t.Fatalf("Expected first_book_id b1, got %v", firstBookID)
		}
	})

	t.Run("GetManifest_Unauthorized", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/manifest", nil)
		req.Header.Set("Authorization", "Bearer bad_token")
		w := httptest.NewRecorder()

		api.GetManifest(w, req)

		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("Expected 401 Unauthorized, got %d", resp.StatusCode)
		}
	})
}
