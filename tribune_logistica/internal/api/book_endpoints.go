package api

import (
	"encoding/json"
	"fmt"
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
		fmt.Printf("error with cursor: %v\n", err)
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
		fmt.Printf("Tried to load epub from %v, failed due to %v", epub.Path, err)
		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	chapterText, err := epub.ExtractChapter(chapterIndex)

	if err != nil {
		http.Error(w, "Error in loading chapter", http.StatusInternalServerError)
		return
	}

	numChunks, err := epub.MaxChunkIndex(chapterIndex, api.Policy)
	if err != nil {
		http.Error(w, "Error in extracting max chunk", http.StatusInternalServerError)
		return
	}

	resp := ChapterResponse{
		ChapterIndex: chapterIndex,
		NumChunks:    numChunks,
		Text:         string(chapterText),
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(resp)
}

type ChunksRequest struct {
	UserCursor  types.UserCursor `json:"UserCursor"`
	RequestSize int              `json:"requestSize"`
}

type ChunkResponse struct {
	Data   string       `json:"data"`   // chunk content
	Cursor types.Cursor `json:"Cursor"` // chapter & chunk
}

func (api *API) GetChunks(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
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
		fmt.Printf("Wrong userID: %v, expected: %v", req.UserCursor.UserID, userID)
		http.Error(w, "UserID does not match token", http.StatusUnauthorized)
		return
	}

	epub, err := epub.Load(api.DB, req.UserCursor.BookID)

	if err != nil {
		fmt.Printf("Error in getting chunks: %v", err)

		http.Error(w, "Could not load epub file", http.StatusInternalServerError)
		return
	}

	maxchunks := epub.MaxChunkMap(api.Policy)

	chunks, err := api.Manager.GetUserChunks(req.UserCursor, req.RequestSize, maxchunks)

	if err != nil {
		fmt.Printf("Manager error in Get Chunks: %v", err)
		http.Error(w, "Manager error", http.StatusInternalServerError)
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(chunks)
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

	// ---- AUTH CHECK ----
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

	coverData, coverName, err := epub.ExtractCover()
	if err != nil {
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
		fmt.Printf("Wrong userID, expected: %v", userID)
		http.Error(w, "UserID does not match token", http.StatusUnauthorized)
		return
	}

	fmt.Printf("saving cursor: %v\n", req)
	err := types.SaveUserCursor(api.DB, req)

	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		return

	}
	w.WriteHeader(http.StatusOK)
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
		fmt.Printf("Failed to save book: %v", err)
		http.Error(w, "Failed to save book", http.StatusInternalServerError)
		return
	} else {
		w.WriteHeader(http.StatusOK)
	}
}
