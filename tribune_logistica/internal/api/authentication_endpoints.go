package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/login"
)

type LoginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type LoginResponse struct {
	AuthToken    string `json:"auth_token"`
	RefreshToken string `json:"refresh_token"`
	ExpiresIn    int    `json:"expires_in"`
}

type RegisterResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
}

type RefreshRequest struct {
	RefreshToken string `json:"refresh_token"`
}

type RefreshResponse struct {
	AuthToken string `json:"auth_token"`
	ExpiresIn int    `json:"expires_in"` // seconds
}

func (api *API) RegisterUser(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	authHeader := r.Header.Get("Authorization")
	expectedToken := os.Getenv("TRIBUNE_LOGISTICA_ADMIN_TOKEN")

	if !strings.HasPrefix(authHeader, "Bearer ") || len(authHeader) <= 7 {
		http.Error(w, "Missing or invalid Authorization header", http.StatusUnauthorized)
		return
	}

	token := authHeader[7:] // skip "Bearer "
	if token != expectedToken {
		http.Error(w, "Unauthorized: invalid token", http.StatusUnauthorized)
		return
	}

	// Decode request body
	var req LoginRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}

	if req.Username == "" || req.Password == "" {
		http.Error(w, "Username and password required", http.StatusBadRequest)
		return
	}

	// Create user object
	user, err := login.NewUser(req.Username, req.Password)
	if err != nil {
		http.Error(w, "Could not create a user", http.StatusInternalServerError)
		return
	}

	// Save user in DB
	err = login.InsertUser(api.DB, user)
	if err != nil {
		http.Error(w, "Could not save user", http.StatusInternalServerError)
		return
	}

	// Respond
	resp := RegisterResponse{
		Success: true,
		Message: "User created successfully",
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(resp)
}

func (api *API) LoginUser(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req LoginRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}
	fmt.Printf("Logging in: %v\n", req.Username)
	refresh_token, err := login.VerifyUser(api.DB, req.Username, req.Password)
	if err != nil {
		http.Error(w, "Wrong credentials", http.StatusUnauthorized)
		return
	}

	auth_token, err := login.GenerateAuthToken(api.DB, refresh_token)
	if err != nil {
		http.Error(w, "Could not generate auth token", http.StatusInternalServerError)
	}

	resp := LoginResponse{
		AuthToken:    auth_token,
		RefreshToken: refresh_token,
		ExpiresIn:    int(login.GetAuthTokenTTL().Seconds()),
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(resp)
}

func (api *API) RefreshTokenHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req RefreshRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "Invalid JSON body", http.StatusBadRequest)
		return
	}

	if req.RefreshToken == "" {
		http.Error(w, "refresh_token required", http.StatusBadRequest)
		return
	}

	newAuthToken, err := login.GenerateAuthToken(api.DB, req.RefreshToken)
	if err != nil {
		http.Error(w, "Could not generate auth token", http.StatusUnauthorized)

	}

	expiresIn := login.GetAuthTokenTTL().Seconds()

	resp := RefreshResponse{
		AuthToken: newAuthToken,
		ExpiresIn: int(expiresIn),
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(resp)
}
