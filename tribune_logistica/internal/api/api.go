package api

import (
	"log"
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

func (a API) ReadOnly() string {
	return "read_only"
}

func (a API) write() string {
	return "write"
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

	if authToken == "" {
		log.Println("Login failed")
		http.Error(w, "Missing authentication token", http.StatusUnauthorized)
		return "", false
	}

	userID, err := login.VerifyUserSession(authToken)
	if err != nil {
		log.Println("Unauthorized access")
		http.Error(w, "Unauthorized access", http.StatusUnauthorized)
		return "", false
	}

	return userID.Username, true
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

func (a *API) RequestCheck(w http.ResponseWriter, r *http.Request, method string, permission string) (string, bool) {
	if r.Method != method {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return "", false
	}
	if permission == a.ReadOnly() {
		get := a.GetTokenCheck(w, r)
		if !get {
			_, ok := a.AuthCheck(w, r)
			return "", ok
		} else {
			return "", get
		}

	}

	return a.AuthCheck(w, r)
}
