package api

import (
	"net/http"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/login"
	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/storage"
)

type API struct {
	Manager *manager.Organizer
	DB      storage.Storage
	Policy  epub.ChunkPolicy
}

func New(manager *manager.Organizer, db storage.Storage, policy epub.ChunkPolicy) API {
	return API{
		Manager: manager,
		DB:      db,
		Policy:  policy,
	}
}

func (api *API) AuthCheck(w http.ResponseWriter, r *http.Request) (string, bool) {
	authHeader := r.Header.Get("Authorization")
	if authHeader == "" || !strings.HasPrefix(authHeader, "Bearer ") {
		http.Error(w, "Missing or invalid Authorization header", http.StatusBadRequest)
		return "", false
	}

	authToken := strings.TrimPrefix(authHeader, "Bearer ")
	userID, ok := login.VerifyAuthToken(authToken)
	if !ok {
		http.Error(w, "Unauthorized access", http.StatusUnauthorized)
		return "", false
	}

	return userID, true
}
