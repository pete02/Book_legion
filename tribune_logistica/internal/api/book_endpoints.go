package api

import (
	"bytes"
	"crypto/sha1"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"image/jpeg"
	"log"
	"mime"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/login"
	"github.com/book_legion-tribune_logistica/internal/types"
	"github.com/disintegration/imaging"
)

func (a *API) GetCursor(rr http.ResponseWriter, req *http.Request) {
	user, ok := req.Context().Value(userContextKey).(login.User)
	if !ok {
		http.Error(rr, "Unauthorized", http.StatusUnauthorized)
		return
	}

	cursor, err := types.LoadUserCursor(a.DB, user.Username, req.PathValue("bookID"))
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
	user, ok := req.Context().Value(userContextKey).(login.User)
	if !ok {
		http.Error(rr, "Unauthorized", http.StatusUnauthorized)
		return
	}

	var cursor types.UserCursor
	if err := json.NewDecoder(req.Body).Decode(&cursor); err != nil {
		http.Error(rr, "Failed to decode cursor", http.StatusBadRequest)
		return
	}

	if user.Username != cursor.UserID {
		http.Error(rr, "Unauthorized", http.StatusUnauthorized)
		return
	}

	if ok, err := library.BookExists(a.DB, cursor.BookID); !ok || err != nil {
		http.Error(rr, "Book not found", http.StatusNotFound)
		return
	}

	epub, err := epub.Load(a.DB, cursor.BookID)
	if err != nil || cursor.Cursor.Index == 0 {
		if err := cursor.SaveUserCursor(a.DB); err != nil {
			log.Printf("[Api] SaveCursor: failed to save cursor: %v", err)
			http.Error(rr, "Failed to save cursor", http.StatusInternalServerError)
			return
		}
		log.Println("[API] SaveCursor: Could not load the book, trusting the cursor")
		rr.WriteHeader(http.StatusCreated)
		return
	}

	cursor, err = epub.FindNearestAllowedSplit(cursor)
	if err != nil {
		if err := cursor.SaveUserCursor(a.DB); err != nil {
			log.Printf("[Api] SaveCursor: failed to save cursor: %v", err)
			http.Error(rr, "Failed to save cursor", http.StatusInternalServerError)
			return
		}
		log.Println("[API] SaveCursor: Could not load the book, trusting the cursor")
		rr.WriteHeader(http.StatusCreated)
		return
	}

	log.Printf("[Api] Saving cursor: %v ", cursor.Cursor)

	if err := cursor.SaveUserCursor(a.DB); err != nil {
		log.Printf("[Api] SaveCursor: failed to save cursor: %v", err)
		http.Error(rr, "Failed to save cursor", http.StatusInternalServerError)
		return
	}
	rr.WriteHeader(http.StatusCreated)
}

func (a *API) GetChapter(rr http.ResponseWriter, req *http.Request) {

	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		log.Printf("[Api] GetChapter: Failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	chapterIndex, err := strconv.Atoi(req.PathValue("chapterIndex"))
	if err != nil {
		log.Printf("[Api] GetChapter: Invalid chapter index: %v", err)
		http.Error(rr, "Invalid chapter index", http.StatusBadRequest)
		return
	}

	chapter, err := epub.GetChapter(chapterIndex)
	if err != nil {
		log.Printf("[Api] GetChapter: failed to load chapter: %v", err)
		http.Error(rr, "Failed to load chapter", http.StatusInternalServerError)
		return
	}
	rr.Header().Set("Content-Type", "text/html")
	rr.WriteHeader(http.StatusOK)
	rr.Write([]byte(chapter))
}

func (a *API) GetNav(rr http.ResponseWriter, req *http.Request) {

	navId := req.PathValue("bookID")

	epub, err := epub.Load(a.DB, navId)
	if err != nil {
		log.Printf("[Api] GetNav: Failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	nav, err := epub.GetToc()
	if err != nil {
		log.Printf("[Api] GetNav: Failed to load Toc from the book: %v", err)
		http.Error(rr, "Failed to load Toc from the book", http.StatusInternalServerError)
		return
	}

	rr.Header().Set("Content-Type", "application/json")
	rr.WriteHeader(http.StatusOK)
	json.NewEncoder(rr).Encode(nav)
}

func (a *API) GetChapterProgress(rr http.ResponseWriter, req *http.Request) {
	user, ok := req.Context().Value(userContextKey).(login.User)
	if !ok {
		log.Println("[Api] GetChapterProgress: Could not get user")
		return
	}

	cursor, err := types.LoadUserCursor(a.DB, user.Username, req.PathValue("bookID"))
	if err != nil {
		http.Error(rr, "Failed to load cursor", http.StatusInternalServerError)
		return
	}
	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		log.Printf("[Api] GetChapterProgress: failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}
	chapter, err := epub.GetChapter(cursor.Cursor.Chapter)
	if err != nil {
		log.Printf("[Api] GetChapterProgress: Failed to load %v chapter %d: %v", cursor.BookID, cursor.Cursor.Chapter, err)
		http.Error(rr, "Failed to load chapter", http.StatusInternalServerError)
		return
	}
	progress := cursor.Cursor.Index / len(chapter)

	rr.Header().Set("Content-Type", "application/json")
	rr.WriteHeader(http.StatusOK)
	json.NewEncoder(rr).Encode(map[string]interface{}{"progress": progress})
}

func (a *API) GetBookProgress(rr http.ResponseWriter, req *http.Request) {
	user, ok := req.Context().Value(userContextKey).(login.User)
	if !ok {
		log.Println("[Api] GetBookProgress: Could not get user")
		return
	}

	cursor, err := types.LoadUserCursor(a.DB, user.Username, req.PathValue("bookID"))
	if err != nil {
		http.Error(rr, "Failed to load cursor", http.StatusInternalServerError)
		return
	}
	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		log.Printf("[Api] GetBookProgress: failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}
	chapter, err := epub.GetChapter(cursor.Cursor.Chapter)
	if err != nil {
		log.Printf("[Api] GetBookProgress: Failed to load %v chapter %d: %v", cursor.BookID, cursor.Cursor.Chapter, err)
		http.Error(rr, "Failed to load chapter", http.StatusInternalServerError)
		return
	}

	chapters, err := epub.GetToc()
	if err != nil {
		log.Printf("[Api] GetBookProgress: Failed to load Toc for book %v: %v", cursor.BookID, err)
		http.Error(rr, "Failed to load Toc", http.StatusInternalServerError)
		return
	}

	chapterWeight := 1.0 / float64(len(chapters))
	log.Printf("chapter weight: %f\n", chapterWeight)
	chapterProgress := float64(cursor.Cursor.Index) / float64(len(chapter))

	bookProgress := (float64(cursor.Cursor.Chapter) + chapterProgress) * chapterWeight
	log.Printf("book progress: (%f+%f)*%f=%f", float64(cursor.Cursor.Chapter), chapterProgress, chapterWeight, bookProgress)
	log.Printf("chapter: %d", cursor.Cursor.Chapter)
	rr.Header().Set("Content-Type", "application/json")
	rr.WriteHeader(http.StatusOK)
	json.NewEncoder(rr).Encode(map[string]interface{}{"progress": bookProgress})
}

const thumbCacheDir = "./data/cover_cache" // adjust to wherever your app stores derived data

func (a *API) GetCover(rr http.ResponseWriter, req *http.Request) {
	user, ok := req.Context().Value(userContextKey).(login.User)
	if !ok {
		return
	}

	bookID := req.PathValue("bookID")

	epub, err := epub.Load(a.DB, bookID)
	if err != nil {
		fmt.Printf("failed to load epub: %v\n", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	cover, imgType, err := epub.GetCover()
	if err != nil {
		log.Printf("[Api] GetCover: failed to load cover: %v", err)
		http.Error(rr, "Failed to load cover", http.StatusInternalServerError)
		return
	}

	if pin := req.Header.Get("Pin"); pin != "" && pin == user.Pin {
		rr.Header().Set("Cache-Control", "no-store")
	} else {
		rr.Header().Set("Cache-Control", "public, max-age=604800, immutable")
	}

	// Optional resize: /cover?width=300
	widthParam := req.URL.Query().Get("width")
	if widthParam == "" {

		// No resize requested: serve original, but still let the browser cache it.
		rr.Header().Set("Content-Type", imgType)
		rr.WriteHeader(http.StatusOK)
		rr.Write(cover)
		return
	}

	width, err := strconv.Atoi(widthParam)
	if err != nil || width <= 0 || width > 2000 {
		http.Error(rr, "invalid width", http.StatusBadRequest)
		return
	}

	thumb, thumbType, err := getOrCreateThumbnail(bookID, cover, width)
	if err != nil {
		log.Printf("[Api] GetCover: failed to create thumbnail: %v", err)
		// Fall back to serving the original rather than failing the request.
		rr.Header().Set("Content-Type", imgType)
		rr.WriteHeader(http.StatusOK)
		rr.Write(cover)
		return
	}

	rr.Header().Set("Content-Type", thumbType)
	rr.WriteHeader(http.StatusOK)
	rr.Write(thumb)
}

// getOrCreateThumbnail returns a resized JPEG for the given cover bytes,
// using an on-disk cache keyed by book ID + width + a hash of the source
// bytes (so a replaced/updated cover naturally busts the cache).
func getOrCreateThumbnail(bookID string, original []byte, width int) ([]byte, string, error) {
	hash := sha1.Sum(original)
	key := fmt.Sprintf("%s_%d_%s.jpg", bookID, width, hex.EncodeToString(hash[:8]))
	cachePath := filepath.Join(thumbCacheDir, key)

	if data, err := os.ReadFile(cachePath); err == nil {
		return data, "image/jpeg", nil // cache hit
	}

	src, err := imaging.Decode(bytes.NewReader(original))
	if err != nil {
		return nil, "", fmt.Errorf("decode cover: %w", err)
	}

	resized := imaging.Resize(src, width, 0, imaging.Lanczos) // 0 = preserve aspect ratio

	var buf bytes.Buffer
	if err := jpeg.Encode(&buf, resized, &jpeg.Options{Quality: 82}); err != nil {
		return nil, "", fmt.Errorf("encode thumbnail: %w", err)
	}

	if err := os.MkdirAll(thumbCacheDir, 0755); err != nil {
		log.Printf("[Api] getOrCreateThumbnail: failed to create cache dir: %v", err)
	} else if err := os.WriteFile(cachePath, buf.Bytes(), 0644); err != nil {
		log.Printf("[Api] getOrCreateThumbnail: failed to write cache file: %v", err)
		// non-fatal: still return the generated thumbnail even if caching failed
	}

	return buf.Bytes(), "image/jpeg", nil
}

func (a *API) GetCSS(rr http.ResponseWriter, req *http.Request) {
	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		log.Printf("[Api] GetCSS: failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	css, err := epub.GetCSS()
	if err != nil {
		log.Printf("[Api] GetCSS: failed to load CSS: %v", err)
		http.Error(rr, "Failed to load CSS", http.StatusInternalServerError)
		return
	}
	rr.Header().Set("Content-Type", "text/css")
	rr.WriteHeader(http.StatusOK)
	rr.Write(css)
}

func (a *API) GetFile(rr http.ResponseWriter, req *http.Request) {
	fileName := req.URL.Query().Get("file")
	log.Printf("[API] Requesting file %s", fileName)
	if fileName == "" {
		http.Error(rr, "File not specified", http.StatusBadRequest)
		return
	}

	epub, err := epub.Load(a.DB, req.PathValue("bookID"))
	if err != nil {
		log.Printf("[API] GetFile: failed to load epub: %v", err)
		http.Error(rr, "Failed to load epub", http.StatusInternalServerError)
		return
	}

	file, err := epub.GetFile(fileName)
	if err != nil {
		log.Printf("[API] GetFile: failed to load file: %v", err)
		http.Error(rr, "Failed to load file", http.StatusInternalServerError)
		return
	}
	contentType := mime.TypeByExtension(filepath.Ext(fileName))
	if contentType == "" {
		contentType = http.DetectContentType(file)
	}
	rr.Header().Set("Content-Type", contentType)
	rr.WriteHeader(http.StatusOK)
	rr.Write(file)
}

func (api *API) SaveBook(w http.ResponseWriter, r *http.Request) {
	user, ok := r.Context().Value(userContextKey).(login.User)
	if !ok {
		return
	}

	var req library.Book
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}

	// The user making the change must already have access to the book.
	// A nonexistent/unrestricted book passes this check.
	hasAccess, err := login.UserHasAccess(api.DB, user.Username, req.ID)
	if err != nil {
		log.Printf("[API] Failed to check book access: %v", err)
		http.Error(w, "Failed to check book access", http.StatusInternalServerError)
		return
	}

	if !hasAccess {
		log.Printf("[API] Attempted to save inaccessible book %q by user %q", req.ID, user)
		http.Error(w, "Failed to save book", http.StatusInternalServerError)
		return
	}
	if err := library.SaveBook(api.DB, req); err != nil {
		log.Printf("[API] Failed to save book: %v", err)
		http.Error(w, "Failed to save book", http.StatusInternalServerError)
		return
	}

	if accessHeader := r.Header.Get("X-Book-Access"); accessHeader != "" {
		for _, username := range strings.Split(accessHeader, ",") {
			username = strings.TrimSpace(username)
			if username == "" {
				continue
			}

			accessUser := login.User{Username: username}
			if err := login.InsertUserAccess(api.DB, accessUser, req.ID); err != nil {
				log.Printf("[API] Failed to grant book access to %q: %v", username, err)
				http.Error(w, "Failed to update book access", http.StatusInternalServerError)
				return
			}
		}
	}

	w.WriteHeader(http.StatusOK)
}
