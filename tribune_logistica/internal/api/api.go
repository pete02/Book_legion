package api

import (
	"fmt"
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
	var authToken string

	// Standard HTTP Bearer authentication.
	authHeader := r.Header.Get("Authorization")
	if strings.HasPrefix(authHeader, "Bearer ") {
		authToken = strings.TrimPrefix(authHeader, "Bearer ")
	}

	// Browser WebSockets cannot set Authorization headers, so also allow
	// the JWT to be supplied as ?token=...
	if authToken == "" {
		authToken = r.URL.Query().Get("token")
	}

	if authToken == "" {
		fmt.Println("Login failed")
		http.Error(w, "Missing authentication token", http.StatusUnauthorized)
		return "", false
	}

	userID, err := login.VerifyUserSession(authToken)
	if err != nil {
		fmt.Println("Unauthorized access")
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
