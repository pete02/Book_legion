package api

import (
	"encoding/json"
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
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(book)
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
