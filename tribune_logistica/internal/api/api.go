package api

import (
	"net/http"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/login"
	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/storage"
)

type API struct {
	Manager *manager.Organizer
	DB      storage.Storage
}

func New(manager *manager.Organizer, db storage.Storage) API {
	return API{
		Manager: manager,
		DB:      db,
	}
}

func (api *API) AuthCheck(w http.ResponseWriter, r *http.Request) (string, bool) {
	authHeader := r.Header.Get("Authorization")
	if authHeader == "" || !strings.HasPrefix(authHeader, "Bearer ") {
		http.Error(w, "Missing or invalid Authorization header", http.StatusBadRequest)
		return "", false
	}

	authToken := strings.TrimPrefix(authHeader, "Bearer ")
	userID, err := login.VerifyUserSession(authToken)
	if err != nil {
		http.Error(w, "Unauthorized access", http.StatusUnauthorized)
		return "", false
	}

	return userID.Username, true
}

func (a *API) RequestCheck(w http.ResponseWriter, r *http.Request, method string) (string, bool) {
	if r.Method != method {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return "", false
	}
	return a.AuthCheck(w, r)
}
