package api

import (
	"encoding/json"
	"log"
	"net/http"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/library"
)

func (api *API) GetBook(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 || pathParts[4] == "" {
		http.Error(w, "Book ID missing", http.StatusBadRequest)
		return
	}
	bookID := pathParts[4]

	book, err := library.LoadBook(api.DB, bookID)

	if err != nil {
		http.Error(w, "BookID incorrect, or book missing", http.StatusNoContent)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(book)
}

func (api *API) GetSeries(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 || pathParts[4] == "" {
		http.Error(w, "Series ID missing", http.StatusBadRequest)
		return
	}
	SeriesID := pathParts[4]

	book, err := library.LoadBooks(api.DB, SeriesID)

	if err != nil {
		http.Error(w, "BookID incorrect, or book missing", http.StatusNoContent)
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(book)
}

func (api *API) DeleteBook(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodDelete {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 || pathParts[4] == "" {
		http.Error(w, "Series ID missing", http.StatusBadRequest)
		return
	}
	BookID := pathParts[4]

	err := library.DeleteBook(api.DB, BookID)

	if err != nil {
		log.Printf("Error in deleting Book: %v", err)
		http.Error(w, "BookID incorrect, or book missing", http.StatusNoContent)
	}
	w.WriteHeader(http.StatusOK)
}

func (api *API) DeleteSeries(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodDelete {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	pathParts := strings.Split(r.URL.Path, "/")
	if len(pathParts) < 5 || pathParts[4] == "" {
		http.Error(w, "Series ID missing", http.StatusBadRequest)
		return
	}
	SeriesID := pathParts[4]

	err := library.DeleteSeries(api.DB, SeriesID)

	if err != nil {
		log.Printf("Error in deleting series: %v", err)
		http.Error(w, "SeriesID incorrect, or book exists", http.StatusNoContent)
	}
	w.WriteHeader(http.StatusOK)
}

func (api *API) GetManifest(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	manifest, err := library.LoadManifest(api.DB)

	if err != nil {
		http.Error(w, "could not fetch manifest", http.StatusInternalServerError)
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(manifest)
}

func (api *API) UpdateSeriesName(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	_, ok := api.AuthCheck(w, r)
	if !ok {
		return
	}

	seriesID := r.PathValue("id")
	if seriesID == "" {
		http.Error(w, "Missing series ID", http.StatusBadRequest)
		return
	}

	var req struct {
		Name string `json:"name"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}
	if req.Name == "" {
		http.Error(w, "Name is required", http.StatusBadRequest)
		return
	}

	if err := library.UpdateSeriesName(api.DB, seriesID, req.Name); err != nil {
		log.Printf("Failed to update series name: %v", err)
		http.Error(w, "Failed to update series name", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusOK)
}
