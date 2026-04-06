package api

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/types"
)

func (api *API) GetCursor(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	userID, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 || pathParts[4] == "" {
		http.Error(w, "Book ID missing", http.StatusBadRequest)
		return
	}
	bookID := pathParts[4]

	cursor, err := types.LoadUserCursor(api.DB, userID, bookID)

	if err != nil {
		log.Printf("error with cursor: %v\n", err)
		http.Error(w, "Could not load cursor", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(cursor)
}

type ChapterResponse struct {
	ChapterIndex int    `json:"chapter_index"`
	NumChunks    int    `json:"num_chunks"`
	Text         string `json:"text"` // XHTML content
}

func (api *API) GetChapter(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// ---- AUTH CHECK ----
	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 7 {
		http.Error(w, "Invalid URL, missing book_id or chapter_index", http.StatusBadRequest)
		return
	}

	bookID := pathParts[4]
	chapterIndexStr := pathParts[6]
	chapterIndex, err := strconv.Atoi(chapterIndexStr)
	if err != nil {
		http.Error(w, "chapter_index must be an integer", http.StatusBadRequest)
		return
	}

	epub, err := epub.Load(api.DB, bookID)
	if err != nil {
		log.Printf("Tried to load epub, failed due to %v", err)
		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	chapterText, err := epub.ExtractChapter(chapterIndex)
	if err != nil {
		log.Printf("Tried to load chapter, failed due to %v", err)
		http.Error(w, "Error in loading chapter", http.StatusInternalServerError)
		return
	}
	log.Printf("chapter %d: extracted %d bytes", chapterIndex, len(chapterText))

	// If the extracted chapter is XHTML/HTML, this is the correct MIME type.
	// Use text/plain if you explicitly want no markup semantics.
	w.Header().Set("Content-Type", "application/xhtml+xml; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(chapterText))
}

type progressResponse struct {
	Progress float32 `json:"progress"`
}

func (api *API) GetChapterProgress(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// ---- AUTH CHECK ----
	userID, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 {
		http.Error(w, "Invalid URL, missing book_id or chapter_index", http.StatusBadRequest)
		return
	}

	bookID := pathParts[4]

	epub, err := epub.Load(api.DB, bookID)

	if err != nil {
		log.Printf("Tried to load epub from %v, failed due to %v", epub.Path, err)
		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	cursor, err := types.LoadUserCursor(api.DB, userID, bookID)

	if err != nil {
		log.Printf("error with cursor: %v\n", err)
		http.Error(w, "Could not load cursor", http.StatusInternalServerError)
		return
	}

	fmt.Println("test ok?")
	progress, err := epub.ChapterProgress(cursor, api.Policy)
	if err != nil {
		log.Printf("Error with Chapter progress: %v\n", err)
		http.Error(w, "Error in loading chapter", http.StatusInternalServerError)
		return
	}
	resp := progressResponse{
		Progress: progress,
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(resp)
}

func (api *API) GetBookProgress(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// ---- AUTH CHECK ----
	userID, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 {
		http.Error(w, "Invalid URL, missing book_id or chapter_index", http.StatusBadRequest)
		return
	}

	bookID := pathParts[4]

	epub, err := epub.Load(api.DB, bookID)

	if err != nil {
		log.Printf("Tried to load epub from %v, failed due to %v", epub.Path, err)
		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	cursor, err := types.LoadUserCursor(api.DB, userID, bookID)

	if err != nil {
		log.Printf("error with cursor: %v\n", err)
		http.Error(w, "Could not load cursor", http.StatusInternalServerError)
		return
	}

	progress, err := epub.BookProgress(cursor, api.Policy)

	if err != nil {
		http.Error(w, "Error in loading chapter", http.StatusInternalServerError)
		return
	}

	resp := progressResponse{
		Progress: progress,
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(resp)
}

type ChunksRequest struct {
	UserCursor  types.UserCursor `json:"user_cursor"`
	RequestSize int              `json:"requestSize"`
}

type ChunkResponse struct {
	Data   string       `json:"data"`   // chunk content
	Cursor types.Cursor `json:"cursor"` // chapter & chunk
}

func (api *API) GetChunks(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	userID, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	var req ChunksRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}

	if req.UserCursor.UserID != userID {
		log.Printf("Wrong userID: %v, expected: %v", req.UserCursor.UserID, userID)
		http.Error(w, "UserID does not match token", http.StatusUnauthorized)
		return
	}

	epub, err := epub.Load(api.DB, req.UserCursor.BookID)

	if err != nil {
		log.Printf("Error in getting chunks: %v", err)

		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	maxchunks := epub.MaxChunkMap(api.Policy)

	chunks, err := api.Manager.GetUserChunks(req.UserCursor, req.RequestSize, maxchunks)

	if err != nil {
		log.Printf("Manager error in Get Chunks: %v", err)
		http.Error(w, "Manager error", http.StatusInternalServerError)
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(chunks)
}

func (api *API) GetCursorText(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	userID, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 || pathParts[4] == "" {
		http.Error(w, "Book ID missing", http.StatusBadRequest)
		return
	}
	bookID := pathParts[4]

	cursor, err := types.LoadUserCursor(api.DB, userID, bookID)

	if err != nil {
		log.Printf("error with cursor: %v\n", err)
		http.Error(w, "Could not load cursor", http.StatusInternalServerError)
		return
	}

	epub, err := epub.Load(api.DB, bookID)

	if err != nil {
		log.Printf("error with Epub: %v\n", err)
		http.Error(w, "Could not load epub", http.StatusInternalServerError)
		return
	}

	text, err := epub.ExtractChunk(cursor.Cursor.Chapter, cursor.Cursor.Chunk, api.Policy)
	if err != nil {
		log.Printf("error with Chunk: %v\n", err)
		http.Error(w, "Could not load Chunk text", http.StatusInternalServerError)
		return
	}

	textCursor := types.TextCursor{Cursor: cursor, Text: text}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(textCursor)
}

func (api *API) SaveCursorText(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	userID, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 8 {
		http.Error(w, "Invalid path", http.StatusBadRequest)
		return
	}

	bookID := pathParts[4]
	chapterStr := pathParts[6]

	if bookID == "" || chapterStr == "" {
		http.Error(w, "Book ID or chapter index missing", http.StatusBadRequest)
		return
	}

	chapterIndex, err := strconv.Atoi(chapterStr)
	if err != nil {
		http.Error(w, "Invalid chapter index", http.StatusBadRequest)
		return
	}

	var req types.CursorLocateRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}

	if strings.TrimSpace(req.SnippetHTML) == "" {
		http.Error(w, "snippet_html is required", http.StatusBadRequest)
		return
	}

	epub, err := epub.Load(api.DB, bookID)
	if err != nil {
		log.Printf("error loading epub: %v\n", err)
		http.Error(w, "Could not load epub", http.StatusInternalServerError)
		return
	}

	cursor, err := epub.CalculateCursorPlace(
		chapterIndex,
		req.SnippetHTML,
		api.Policy,
	)
	if err != nil {
		log.Printf("Error in text to chunk: %v\n", err)
		switch err.Error() {
		case "snippet too short to uniquely locate cursor":
			http.Error(w, err.Error(), http.StatusBadRequest)
		case "example text not found in chapter":
			http.Error(w, err.Error(), http.StatusNotFound)
		default:
			log.Printf("cursor calculation error: %v\n", err)
			http.Error(w, "Could not calculate cursor", http.StatusInternalServerError)
		}
		return
	}

	userCursor := types.UserCursor{
		UserID: userID,
		BookID: bookID,
		Cursor: cursor,
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(userCursor)
}

func (api *API) GetNav(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 6 || pathParts[4] == "" {
		http.Error(w, "Book ID missing", http.StatusBadRequest)
		return
	}
	bookID := pathParts[4]

	epub, err := epub.Load(api.DB, bookID)

	if err != nil {
		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	navItems := epub.Nav

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(navItems)
}

func (api *API) GetCover(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 6 || pathParts[4] == "" {
		http.Error(w, "Book ID missing", http.StatusBadRequest)
		return
	}
	bookID := pathParts[4]

	epub, err := epub.Load(api.DB, bookID)

	if err != nil {
		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	coverData, coverName, err := epub.ExtractCover()
	if err != nil {
		fmt.Printf("Error in getting cover: %v\n", err)
		http.Error(w, "Could not get cover", http.StatusInternalServerError)
		return
	}

	contentType := "application/octet-stream"
	if strings.HasSuffix(strings.ToLower(coverName), ".jpg") || strings.HasSuffix(strings.ToLower(coverName), ".jpeg") {
		contentType = "image/jpeg"
	} else if strings.HasSuffix(strings.ToLower(coverName), ".png") {
		contentType = "image/png"
	}

	w.Header().Set("Content-Type", contentType)
	w.WriteHeader(http.StatusOK)
	w.Write(coverData)

}

func (api *API) TextToCursorGetCSSFiles(w http.ResponseWriter, r *http.Request) {

}

func (api *API) GetCSSFiles(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 6 || pathParts[4] == "" {
		http.Error(w, "Book ID missing", http.StatusBadRequest)
		return
	}
	bookID := pathParts[4]

	epub, err := epub.Load(api.DB, bookID)

	if err != nil {
		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	cssData, err := epub.ExtractCSS()
	if err != nil {
		http.Error(w, "Could not load css", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write(cssData)
}

func (api *API) SaveCursor(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// ---- AUTH CHECK ----
	userID, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	// ---- PARSE REQUEST BODY ----
	var req types.UserCursor
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}

	// Optional: ensure the userID in body matches token
	if req.UserID != userID {
		log.Printf("Wrong userID, expected: %v, Got %v\n", userID, req.UserID)
		http.Error(w, "UserID does not match token", http.StatusUnauthorized)
		return
	}

	log.Printf("saving cursor: %v\n", req)
	err := types.SaveUserCursor(api.DB, req)

	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		return

	}
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(`{"status":"ok"}`))
}

func (api *API) SaveBook(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	var req library.Book
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}

	err := library.SaveBook(api.DB, req)

	if err != nil {
		log.Printf("Failed to save book: %v", err)
		http.Error(w, "Failed to save book", http.StatusInternalServerError)
		return
	} else {
		w.WriteHeader(http.StatusOK)
	}
}
