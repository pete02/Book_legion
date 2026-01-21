package api_test

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/api"
	"github.com/book_legion-tribune_logistica/internal/buffer"
	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/storage"
	"github.com/book_legion-tribune_logistica/internal/types"
)

func setupAPI(t *testing.T) api.API {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)
	store, err := storage.NewJSONStorage(tmpFile)

	if err != nil {
		t.Fatal("could not create storage")
	}

	buf := buffer.NewBuffer("t")

	manager := manager.NewOrganizer(buf, 3)
	policy := epub.ChunkPolicy{TargetSize: 50, MaxSize: 60}

	api := api.New(manager, store, policy)

	return api
}

func setupRegister(t *testing.T, api api.API, username, password string) *http.Response {
	t.Helper()
	token := "Long Token"
	os.Setenv("ADMIN_TOKEN", token)

	// Prepare request body
	registerBody := map[string]string{"username": username, "password": password}
	buf, _ := json.Marshal(registerBody)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/register", bytes.NewBuffer(buf))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+token)
	w := httptest.NewRecorder()
	api.RegisterUser(w, req)

	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusCreated {
		t.Fatalf("Register failed: %d", resp.StatusCode)
	}

	return resp
}

func setupLogin(t *testing.T, api api.API, username string, password string) *http.Response {
	loginBody := map[string]string{"username": username, "password": password}
	buf, _ := json.Marshal(loginBody)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/login", bytes.NewBuffer(buf))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	api.LoginUser(w, req)

	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("Login success: expected status 200, got %d", resp.StatusCode)
	}

	return resp
}

func setupUser(t *testing.T, api api.API, username string, password string) (string, string) {
	setupRegister(t, api, username, password)
	resp := setupLogin(t, api, username, password)

	var loginResp map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&loginResp); err != nil {
		t.Fatalf("Failed to decode login response: %v", err)
	}

	refreshToken, ok := loginResp["refresh_token"].(string)
	if !ok || refreshToken == "" {
		t.Fatal("Login did not return refresh_token")
	}
	auth_token, ok := loginResp["auth_token"].(string)
	if !ok {
		t.Fatal("Login success: missing auth_token")
	}
	return refreshToken, auth_token
}

func setupAPIWithAuth(t *testing.T) (api.API, string) {
	api := setupAPI(t)
	_, token := setupUser(t, api, "pete", "password123")
	return api, token
}

func setupBook(t *testing.T, api api.API, fp string) {
	book1 := library.Book{
		ID:          "b1",
		Title:       "Book One",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesOrder: 1,
		FilePath:    fp,
	}
	book2 := library.Book{
		ID:          "b1",
		Title:       "Book One",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesOrder: 1,
		FilePath:    fp,
	}
	err := library.SaveBook(api.DB, book1)

	if err != nil {
		t.Fatal("Could not save book")
	}
	err = library.SaveBook(api.DB, book2)
	if err != nil {
		t.Fatal("Could not save book")
	}

}
func setupManifest(t *testing.T, api api.API) {
	manifest := library.Manifest{
		Series: []library.SeriesEntry{
			{
				SeriesID:    "s1",
				SeriesName:  "Series one", // optional
				FirstBookID: "b1",
			},
		},
	}

	err := library.SaveManifest(api.DB, manifest)
	if err != nil {
		t.Fatalf("Could not save manifest: %v", err)
	}
}
func createTestEpub(t *testing.T, api api.API) string {
	coverData := []byte{0x89, 0x50, 0x4E, 0x47}
	files := map[string]string{
		"OEBPS/cover.png":  string(coverData),
		"OEBPS/style1.css": "body { color: red; }",
		"META-INF/container.xml": `
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>`,

		"OEBPS/content.opf": `
<package version="3.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="chap1" href="text/ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="chap2" href="text/ch2.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chap1"/>
    <itemref idref="chap2"/>
  </spine>
</package>`,

		"OEBPS/text/ch1.xhtml": "<html><body>Chapter 1</body></html>",
		"OEBPS/text/ch2.xhtml": "<html><body>Chapter 2</body></html>",

		// nav.toc in NCX format
		"OEBPS/toc.ncx": `
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="uid"/>
    <meta name="dtb:depth" content="1"/>
  </head>
  <docTitle><text>Test Book</text></docTitle>
  <navMap>
    <navPoint id="np1" playOrder="0">
      <navLabel><text>Chapter One</text></navLabel>
      <content src="text/ch1.xhtml"/>
    </navPoint>
    <navPoint id="np2" playOrder="1">
      <navLabel><text>Chapter Two</text></navLabel>
      <content src="text/ch2.xhtml"/>
    </navPoint>
  </navMap>
</ncx>`}

	t.Helper()

	dir := t.TempDir()
	epubPath := filepath.Join(dir, "test.epub")

	f, err := os.Create(epubPath)
	if err != nil {
		t.Fatalf("failed to create epub file: %v", err)
	}
	defer f.Close()

	zw := zip.NewWriter(f)

	for name, content := range files {
		w, err := zw.Create(name)
		if err != nil {
			t.Fatalf("failed to create zip entry %s: %v", name, err)
		}
		_, err = w.Write([]byte(content))
		if err != nil {
			t.Fatalf("failed to write zip entry %s: %v", name, err)
		}
	}

	if err := zw.Close(); err != nil {
		t.Fatalf("failed to close zip writer: %v", err)
	}

	setupBook(t, api, epubPath)
	setupManifest(t, api)

	return epubPath
}

func TestGetCursor(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	epubPath := createTestEpub(t, api)

	bookID := filepath.Base(epubPath)

	t.Run("GetCursor_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/cursors/"+bookID, nil)
		req.Header.Set("Authorization", "Bearer "+token)
		w := httptest.NewRecorder()

		api.GetCursor(w, req)

		resp := w.Result()
		defer resp.Body.Close()
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		var cursorResp map[string]interface{}
		if err := json.NewDecoder(resp.Body).Decode(&cursorResp); err != nil {
			t.Fatal(err)
		}

		if cursorResp["BookID"] != bookID {
			t.Fatalf("Expected BookID %s, got %v", bookID, cursorResp["BookID"])
		}
	})

	t.Run("GetCursor_Unauthorized", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/cursors/"+bookID, nil)
		req.Header.Set("Authorization", "Bearer badtoken")
		w := httptest.NewRecorder()

		api.GetCursor(w, req)
		resp := w.Result()
		defer resp.Body.Close()
		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("Expected 401 Unauthorized, got %d", resp.StatusCode)
		}
	})
}

func TestGetCursorText(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)

	bookID := "b1"

	t.Run("GetCursor_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/cursors/"+bookID+"/text", nil)
		req.Header.Set("Authorization", "Bearer "+token)
		w := httptest.NewRecorder()

		api.GetCursorText(w, req)

		resp := w.Result()
		defer resp.Body.Close()
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}
		var cursorResp types.TextCursor
		if err := json.NewDecoder(resp.Body).Decode(&cursorResp); err != nil {
			t.Fatal(err)
		}

		if cursorResp.Cursor.BookID != bookID {
			t.Fatalf("Expected BookID %s, got %v: %v", bookID, cursorResp.Cursor.BookID, cursorResp)
		}
	})

	t.Run("GetCursor_Unauthorized", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/cursors/"+bookID+"/text", nil)
		req.Header.Set("Authorization", "Bearer badtoken")
		w := httptest.NewRecorder()

		api.GetCursorText(w, req)
		resp := w.Result()
		defer resp.Body.Close()
		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("Expected 401 Unauthorized, got %d", resp.StatusCode)
		}
	})
}

func TestCalculateCursorFromText(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)

	bookID := "b1"
	chapterIndex := 0

	t.Run("CalculateCursor_Success", func(t *testing.T) {
		body := map[string]string{
			"snippet_html": "<p>Chapter 1</p>",
		}

		b, err := json.Marshal(body)
		if err != nil {
			t.Fatal(err)
		}

		req := httptest.NewRequest(
			http.MethodPost,
			fmt.Sprintf("/api/v1/books/%s/chapters/%d/cursor", bookID, chapterIndex),
			bytes.NewReader(b),
		)
		req.Header.Set("Authorization", "Bearer "+token)
		req.Header.Set("Content-Type", "application/json")

		w := httptest.NewRecorder()

		api.CalculateCursorFromText(w, req)

		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		var cursorResp types.UserCursor
		if err := json.NewDecoder(resp.Body).Decode(&cursorResp); err != nil {
			t.Fatal(err)
		}

		if cursorResp.BookID != bookID {
			t.Fatalf(
				"Expected BookID %s, got %v: %+v",
				bookID,
				cursorResp.BookID,
				cursorResp,
			)
		}
	})

	t.Run("CalculateCursor_Unauthorized", func(t *testing.T) {
		body := map[string]string{
			"snippet_html": "<p>Chapter 1</p>",
		}

		b, _ := json.Marshal(body)

		req := httptest.NewRequest(
			http.MethodPost,
			fmt.Sprintf("/api/v1/books/%s/chapters/%d/cursor", bookID, chapterIndex),
			bytes.NewReader(b),
		)
		req.Header.Set("Authorization", "Bearer badtoken")
		req.Header.Set("Content-Type", "application/json")

		w := httptest.NewRecorder()

		api.CalculateCursorFromText(w, req)

		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("Expected 401 Unauthorized, got %d", resp.StatusCode)
		}
	})
}

// ----------------- 3.1 Get Chapter -----------------
func TestGetChapter(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)
	bookID := "b1"

	t.Run("GetChapter_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/books/"+bookID+"/chapters/0", nil)
		req.Header.Set("Authorization", "Bearer "+token)
		w := httptest.NewRecorder()

		api.GetChapter(w, req)
		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		var chResp map[string]interface{}
		if err := json.NewDecoder(resp.Body).Decode(&chResp); err != nil {
			t.Fatal(err)
		}

		if chResp["chapter_index"].(float64) != 0 {
			t.Fatalf("Expected chapter_index 0, got %v", chResp["chapter_index"])
		}
	})
}

// ----------------- 3.2 Get Chunks -----------------
func TestGetChunks(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)
	bookID := "b1"

	fetchFn := func(c types.UserCursor) (types.Chunk, bool) {
		// Simulate chunk content based on chapter/chunk
		data := []byte(
			fmt.Sprintf("Chapter %d, Chunk %d", c.Cursor.Chapter, c.Cursor.Chunk),
		)
		chunk := types.Chunk{
			ID:   c,
			Data: data,
		}
		return chunk, true
	}

	// Start the order processor
	stopChan := api.Manager.StartOrderProcessor(fetchFn)
	defer close(stopChan)

	reqBody := map[string]interface{}{
		"UserCursor": map[string]interface{}{
			"UserID": "pete",
			"BookID": bookID,
			"Cursor": map[string]int{"Chapter": 0, "Chunk": 0},
		},
		"requestSize": 2,
	}
	buf, _ := json.Marshal(reqBody)

	t.Run("GetChunks_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/books/"+bookID+"/chunks", bytes.NewBuffer(buf))
		req.Header.Set("Authorization", "Bearer "+token)
		w := httptest.NewRecorder()

		api.GetChunks(w, req)
		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		var chunks []map[string]interface{}
		if err := json.NewDecoder(resp.Body).Decode(&chunks); err != nil {
			t.Fatal(err)
		}

		if len(chunks) != 2 {
			t.Fatalf("Expected 2 chunks, got %d", len(chunks))
		}
	})

	t.Run("GetChunks_Unauthorized", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/books/"+bookID+"/chunks", bytes.NewBuffer(buf))
		req.Header.Set("Authorization", "Bearer badtoken")
		w := httptest.NewRecorder()

		api.GetChunks(w, req)
		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("Expected 401 Unauthorized, got %d", resp.StatusCode)
		}
	})
}

// ----------------- 3.3 Get Nav -----------------
func TestGetNav(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)
	bookID := "b1"

	t.Run("GetNav_Success", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/books/"+bookID+"/nav", nil)
		req.Header.Set("Authorization", "Bearer "+token)
		w := httptest.NewRecorder()

		api.GetNav(w, req)
		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
		}

		var navResp []map[string]interface{}
		if err := json.NewDecoder(resp.Body).Decode(&navResp); err != nil {
			t.Fatal(err)
		}

		if len(navResp) == 0 {
			t.Fatal("Expected at least one nav item")
		}
	})

	t.Run("GetNav_Unauthorized", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/books/"+bookID+"/nav", nil)
		req.Header.Set("Authorization", "Bearer badtoken")
		w := httptest.NewRecorder()

		api.GetNav(w, req)
		resp := w.Result()
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("Expected 401 Unauthorized, got %d", resp.StatusCode)
		}
	})
}
func TestGetCoverImage(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)

	bookID := "b1"

	req := httptest.NewRequest(
		http.MethodGet,
		"/api/v1/books/"+bookID+"/cover",
		nil,
	)
	req.Header.Set("Authorization", "Bearer "+token)

	w := httptest.NewRecorder()
	api.GetCover(w, req)

	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", resp.StatusCode)
	}

	ct := resp.Header.Get("Content-Type")
	if ct != "image/png" && ct != "image/jpeg" {
		t.Fatalf("unexpected Content-Type: %s", ct)
	}

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("failed to read response body: %v", err)
	}

	if len(data) == 0 {
		t.Fatal("expected non-empty image data")
	}
}

func TestGetCSS(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)

	bookID := "b1"

	req := httptest.NewRequest(
		http.MethodGet,
		"/api/v1/books/"+bookID+"/csss",
		nil,
	)
	req.Header.Set("Authorization", "Bearer "+token)

	w := httptest.NewRecorder()
	api.GetCSSFiles(w, req)

	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", resp.StatusCode)
	}

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("failed to read response body: %v", err)
	}

	if len(data) == 0 {
		t.Fatal("expected non-empty image data")
	}

	ss := "body { color: red; }"

	if !strings.Contains(string(data), ss) {
		t.Fatalf("Wrong string: %v", string(data))
	}

}

func TestSaveCursors(t *testing.T) {
	api, token := setupAPIWithAuth(t)

	payload := map[string]interface{}{
		"UserID": "pete",
		"BookID": "b1",
		"Cursor": map[string]int{"Chapter": 10, "Chunk": 1},
	}

	data, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/cursors/save", bytes.NewBuffer(data))
	req.Header.Set("Authorization", "Bearer "+token)
	req.Header.Set("Content-Type", "application/json")

	w := httptest.NewRecorder()
	api.SaveCursor(w, req)
	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", resp.StatusCode)
	}

	cur, err := types.LoadUserCursor(api.DB, "pete", "b1")
	if err != nil {
		t.Fatal("Failed to load Cursor")
	}

	curs := types.Cursor{Chapter: 10, Chunk: 1}

	if cur.Cursor.CompareCursor(curs) != 0 {
		fmt.Printf("Saved: %v", cur)

		t.Fatal("Wrong cursor")
	}
}

func TestSaveBook(t *testing.T) {
	api, token := setupAPIWithAuth(t)

	payload := map[string]interface{}{
		"id":           "b10",
		"title":        "Book One",
		"author_id":    "a1",
		"series_id":    "s1",
		"series_order": 1,
		"file_path":    "/path/to/book.epub",
	}

	data, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/savebook", bytes.NewBuffer(data))
	req.Header.Set("Authorization", "Bearer "+token)
	req.Header.Set("Content-Type", "application/json")

	w := httptest.NewRecorder()
	api.SaveBook(w, req)
	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		t.Fatalf("expected 200 OK, got %d, body: %s", resp.StatusCode, string(body))
	}

	book, err := library.LoadBook(api.DB, "b10")
	if err != nil {
		t.Fatal("Could not load book")
	}
	if book.FilePath != "/path/to/book.epub" {
		t.Fatalf("Got wrong book: %v", book)
	}
}

func TestSaveCursors_Unauthorized(t *testing.T) {
	api, _ := setupAPIWithAuth(t)

	payload := map[string]interface{}{
		"UserID": "u1",
		"BookID": "b1",
		"Cursor": map[string]int{"Chapter": 1, "Chunk": 1},
	}

	data, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/cursors/save", bytes.NewBuffer(data))
	req.Header.Set("Authorization", "Bearer "+"")

	w := httptest.NewRecorder()
	api.SaveCursor(w, req)
	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("expected 401 Unauthorized, got %d", resp.StatusCode)
	}
}

func TestSaveBook_Unauthorized(t *testing.T) {
	api, _ := setupAPIWithAuth(t)

	payload := map[string]interface{}{
		"id":           "b1",
		"title":        "Book One",
		"author_id":    "a1",
		"series_id":    "s1",
		"series_order": 1,
		"file_path":    "/path/to/book.epub",
	}

	data, _ := json.Marshal(payload)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/savebook", bytes.NewBuffer(data))
	req.Header.Set("Authorization", "Bearer "+"")
	w := httptest.NewRecorder()
	api.SaveBook(w, req)
	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("expected 401 Unauthorized, got %d", resp.StatusCode)
	}
}

func TestChapterProgress(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/books/b1/chapterprogress", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	w := httptest.NewRecorder()
	api.GetChapterProgress(w, req)
	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", resp.StatusCode)
	}

	type progressResponse struct {
		Progress float32 `json:"progress"`
	}

	var body progressResponse
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response body: %v", err)
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/books/b1/chapterprogress", nil)
	req.Header.Set("Authorization", "Bearer "+"")
	w = httptest.NewRecorder()
	api.GetChapterProgress(w, req)
	resp = w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("expected 401 Unauthorized, got %d", resp.StatusCode)
	}
}

func TestBookProgress(t *testing.T) {
	api, token := setupAPIWithAuth(t)
	createTestEpub(t, api)

	req := httptest.NewRequest(http.MethodGet, "/api/v1/books/b1/chapterprogress", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	w := httptest.NewRecorder()
	api.GetBookProgress(w, req)
	resp := w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200 OK, got %d", resp.StatusCode)
	}

	type progressResponse struct {
		Progress float32 `json:"progress"`
	}

	var body progressResponse
	if err := json.NewDecoder(resp.Body).Decode(&body); err != nil {
		t.Fatalf("failed to decode response body: %v", err)
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/books/b1/chapterprogress", nil)
	req.Header.Set("Authorization", "Bearer "+"")
	w = httptest.NewRecorder()
	api.GetBookProgress(w, req)
	resp = w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("expected 401 Unauthorized, got %d", resp.StatusCode)
	}
}
