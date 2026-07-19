package api

import (
	"encoding/json"
	"fmt"
	"log"
	"mime"
	"net/http"
	"path/filepath"
	"strconv"

	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/types"
)

func (a *API) GetCursor(rr http.ResponseWriter, req *http.Request) {
	user, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}

	cursor, err := types.LoadUserCursor(a.DB, user, req.PathValue("bookID"))
	if err != nil {
		log.Printf("[Api] Failed to load cursor for user %v: %v", user, err)
		http.Error(rr, "Failed to load cursor", http.StatusInternalServerError)
		return
	}

	rr.Header().Set("Content-Type", "application/json")
	rr.WriteHeader(http.StatusOK)
	json.NewEncoder(rr).Encode(cursor)

}

func (a *API) SaveCursor(rr http.ResponseWriter, req *http.Request) {
	user, ok := a.RequestCheck(rr, req, http.MethodPost)
	if !ok {
		return
	}

	var cursor types.UserCursor
	if err := json.NewDecoder(req.Body).Decode(&cursor); err != nil {
		http.Error(rr, "Failed to decode cursor", http.StatusBadRequest)
		return
	}

	if user != cursor.UserID {
		http.Error(rr, "Unauthorized", http.StatusUnauthorized)
		return
	}

	log.Printf("[Api] Saving cursor: %v ", cursor.Cursor)

	if err := cursor.SaveUserCursor(a.DB); err != nil {
		fmt.Printf("failed to save cursor: %v", err)
		http.Error(rr, "Failed to save cursor", http.StatusInternalServerError)
		return
	}

	rr.WriteHeader(http.StatusCreated)
}

func (a *API) GetChapter(rr http.ResponseWriter, req *http.Request) {
	_, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}

	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		log.Printf("[Api] Failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	chapterIndex, err := strconv.Atoi(req.PathValue("chapterIndex"))
	if err != nil {
		log.Printf("[Api] Invalid chapter index: %v", err)
		http.Error(rr, "Invalid chapter index", http.StatusBadRequest)
		return
	}

	chapter, err := epub.GetChapter(chapterIndex)
	if err != nil {
		fmt.Printf("failed to load chapter: %v }\n", err)
		http.Error(rr, "Failed to load chapter", http.StatusInternalServerError)
		return
	}
	rr.Header().Set("Content-Type", "text/html")
	rr.WriteHeader(http.StatusOK)
	rr.Write([]byte(chapter))
}

func (a *API) GetNav(rr http.ResponseWriter, req *http.Request) {
	_, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}
	navId := req.PathValue("bookID")

	epub, err := epub.Load(a.DB, navId)
	if err != nil {
		log.Printf("[Api] Failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	nav, err := epub.GetToc()
	if err != nil {
		log.Printf("[Api] Failed to load Toc from the book: %v", err)
		http.Error(rr, "Failed to load Toc from the book", http.StatusInternalServerError)
		return
	}

	rr.Header().Set("Content-Type", "application/json")
	rr.WriteHeader(http.StatusOK)
	json.NewEncoder(rr).Encode(nav)
}

func (a *API) GetChapterProgress(rr http.ResponseWriter, req *http.Request) {
	user, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}
	cursor, err := types.LoadUserCursor(a.DB, user, req.PathValue("bookID"))
	if err != nil {
		http.Error(rr, "Failed to load cursor", http.StatusInternalServerError)
		return
	}
	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		fmt.Printf("failed to load epub: %v\n", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}
	chapter, err := epub.GetChapter(cursor.Cursor.Chapter)
	if err != nil {
		fmt.Printf("Failed to load %v chapter %d: %v", cursor.BookID, cursor.Cursor.Chapter, err)
		http.Error(rr, "Failed to load chapter", http.StatusInternalServerError)
		return
	}
	progress := cursor.Cursor.Index / len(chapter)

	rr.Header().Set("Content-Type", "application/json")
	rr.WriteHeader(http.StatusOK)
	json.NewEncoder(rr).Encode(map[string]interface{}{"progress": progress})
}

func (a *API) GetBookProgress(rr http.ResponseWriter, req *http.Request) {
	user, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}
	cursor, err := types.LoadUserCursor(a.DB, user, req.PathValue("bookID"))
	if err != nil {
		http.Error(rr, "Failed to load cursor", http.StatusInternalServerError)
		return
	}
	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		fmt.Printf("failed to load epub: %v\n", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}
	chapter, err := epub.GetChapter(cursor.Cursor.Chapter)
	if err != nil {
		fmt.Printf("Failed to load %v chapter %d: %v", cursor.BookID, cursor.Cursor.Chapter, err)
		http.Error(rr, "Failed to load chapter", http.StatusInternalServerError)
		return
	}

	chapters, err := epub.GetToc()
	if err != nil {
		fmt.Printf("Failed to load Toc for book %v: %v", cursor.BookID, err)
		http.Error(rr, "Failed to load Toc", http.StatusInternalServerError)
		return
	}

	chapterWeight := 1.0 / float64(len(chapters))
	chapterProgress := cursor.Cursor.Index / len(chapter)

	bookProgress := float64(cursor.Cursor.Chapter)/chapterWeight + float64(chapterProgress)*chapterWeight

	rr.Header().Set("Content-Type", "application/json")
	rr.WriteHeader(http.StatusOK)
	json.NewEncoder(rr).Encode(map[string]interface{}{"progress": bookProgress})
}

func (a *API) GetCover(rr http.ResponseWriter, req *http.Request) {
	_, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}

	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		fmt.Printf("failed to load epub: %v\n", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	cover, img_ype, err := epub.GetCover()
	if err != nil {
		fmt.Printf("failed to load cover: %v\n", err)
		http.Error(rr, "Failed to load cover", http.StatusInternalServerError)
		return
	}
	rr.Header().Set("Content-Type", img_ype)
	rr.WriteHeader(http.StatusOK)
	rr.Write(cover)
}

func (a *API) GetCSS(rr http.ResponseWriter, req *http.Request) {
	_, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}

	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		fmt.Printf("failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	css, err := epub.GetCSS()
	if err != nil {
		fmt.Printf("failed to load CSS: %v", err)
		http.Error(rr, "Failed to load CSS", http.StatusInternalServerError)
		return
	}
	rr.Header().Set("Content-Type", "text/css")
	rr.WriteHeader(http.StatusOK)
	rr.Write(css)
}

func (a *API) GetFile(rr http.ResponseWriter, req *http.Request) {
	_, ok := a.RequestCheck(rr, req, http.MethodGet)
	if !ok {
		return
	}

	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	file, err := epub.GetFile(req.PathValue("file_path"))
	if err != nil {
		fmt.Printf("failed to load file: %v", err)
		http.Error(rr, "Failed to load file", http.StatusInternalServerError)
		return
	}
	contentType := mime.TypeByExtension(filepath.Ext(req.PathValue("file_path")))
	if contentType == "" {
		contentType = http.DetectContentType(file)
	}

	rr.Header().Set("Content-Type", contentType)
	rr.WriteHeader(http.StatusOK)
	rr.Write(file)
}
