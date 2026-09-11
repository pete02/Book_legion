package api

import (
	"fmt"
	"log"
	"net/http"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/login"
	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/storage"
)

type API struct {
	Manager *manager.Organizer
	DB      *storage.SQLStorage
}

func (a API) ReadOnly() string {
	return "read_only"
}

func (a API) write() string {
	return "write"
}
func New(manager *manager.Organizer, db *storage.SQLStorage) API {
	return API{
		Manager: manager,
		DB:      db,
	}
}

func (api *API) AuthCheck(w http.ResponseWriter, r *http.Request) (login.User, bool) {
	var authToken string

	// Standard HTTP Bearer authentication.
	authHeader := r.Header.Get("Authorization")
	if strings.HasPrefix(authHeader, "Bearer ") {
		authToken = strings.TrimPrefix(authHeader, "Bearer ")
	}

	if authToken == "" {
		log.Println("Login failed")
		http.Error(w, "Missing authentication token", http.StatusUnauthorized)
		return login.User{}, false
	}

	userID, err := login.VerifyUserSession(authToken)
	if err != nil {
		log.Println("Unauthorized access")
		http.Error(w, "Unauthorized access", http.StatusUnauthorized)
		return login.User{}, false
	}

	return userID, true
}

func (api *API) GetTokenCheck(w http.ResponseWriter, r *http.Request) bool {
	getToken := r.URL.Query().Get("token")
	if getToken == "" {
		log.Println("Missing get token")
		return false
	}
	ok, err := login.VerifyGetSession(getToken)
	if err != nil || !ok {
		log.Println("Invalid get token")
		return false
	}

	return true
}

// Checks whether or not we have access, and returns user.Username
func (a *API) RequestCheck(w http.ResponseWriter, r *http.Request, method string, permission string) (login.User, bool) {
	if r.Method != method {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return login.User{}, false
	}
	return a.AuthCheck(w, r)
}

func (a *API) AccessCheck(w http.ResponseWriter, r *http.Request, user login.User, bookID string) bool {
	var owner string
	if pin := r.Header.Get("Pin"); pin != "" && pin == user.Pin {
		owner = user.Username
	}

	access, err := login.UserHasAccess(a.DB, owner, bookID)
	if !access || err != nil {
		fmt.Printf("No access for user %v and book %v: %v\n", user.Username, bookID, err)
		http.Error(w, "Failed to load epub", http.StatusInternalServerError)
		return false
	}
	return true
}

func (a *API) Health(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
}
