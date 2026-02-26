package api_test

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/login"
)

// ------------------ TESTS ------------------

func TestFailRegisterUserWithoutCorrectToken(t *testing.T) {
	token := "Long Token"
	os.Setenv("TRIBUNE_LOGISTICA_ADMIN_TOKEN", token)
	api := setupAPI(t)
	registerBody := map[string]string{"username": "username", "password": "password"}
	buf, _ := json.Marshal(registerBody)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/register", bytes.NewBuffer(buf))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+"bla bla")
	w := httptest.NewRecorder()
	api.RegisterUser(w, req)

}

func TestRegisterAndLogin(t *testing.T) {
	// setup API with dummy user store
	api := setupAPI(t)
	username := "pete"
	password := "secret123"
	resp := setupRegister(t, api, username, password)
	defer resp.Body.Close()

	var regResp map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&regResp); err != nil {
		t.Fatalf("Register: failed to decode response: %v", err)
	}

	if success, ok := regResp["success"].(bool); !ok || !success {
		t.Fatalf("Register: expected success=true, got %v", regResp)
	}

	// ----- LOGIN SUCCESS -----
	resp = setupLogin(t, api, username, password)

	var loginResp map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&loginResp); err != nil {
		t.Fatalf("Login success: failed to decode response: %v", err)
	}
	auth_token, ok := loginResp["auth_token"].(string)
	if !ok {
		t.Fatal("Login success: missing auth_token")
	}
	if _, ok := loginResp["refresh_token"].(string); !ok {
		t.Fatal("Login success: missing refresh_token")
	}
	if _, ok := loginResp["expires_in"].(float64); !ok {
		t.Fatal("Login success: missing expires_in")
	}

	if str, ok := login.VerifyAuthToken(auth_token); !ok || str != username {
		t.Fatalf("Bad auth token, %v", str)
	}

	// ----- LOGIN FAILURE -----
	loginBody := `{"username":"pete","password":"wrongpassword"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/login", bytes.NewBufferString(loginBody))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()

	api.LoginUser(w, req)

	resp = w.Result()
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("Login failure: expected status 401, got %d", resp.StatusCode)
	}
}

func TestRefreshAuthToken(t *testing.T) {
	api := setupAPI(t)

	// ----- setup: Create a user and get refresh token -----
	username := "pete"
	password := "secret123"

	// Register user
	refreshToken, _ := setupUser(t, api, username, password)

	// ----- TEST 1: SUCCESSFUL REFRESH -----
	refreshBody := map[string]string{"refresh_token": refreshToken}
	buf, _ := json.Marshal(refreshBody)

	req := httptest.NewRequest(http.MethodPost, "/api/v1/refreshtoken", bytes.NewBuffer(buf))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	api.RefreshTokenHandler(w, req)

	resp := w.Result()
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("Expected 200 OK, got %d", resp.StatusCode)
	}

	var refreshResp map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&refreshResp); err != nil {
		t.Fatalf("Failed to decode refresh response: %v", err)
	}

	if _, ok := refreshResp["auth_token"].(string); !ok {
		t.Fatal("Refresh: missing auth_token")
	}
	if _, ok := refreshResp["expires_in"].(float64); !ok {
		t.Fatal("Refresh: missing expires_in")
	}

	// ----- TEST 2: INVALID REFRESH TOKEN -----
	invalidBody := map[string]string{"refresh_token": "invalid_token"}
	buf, _ = json.Marshal(invalidBody)

	req = httptest.NewRequest(http.MethodPost, "/api/v1/refresh", bytes.NewBuffer(buf))
	req.Header.Set("Content-Type", "application/json")
	w = httptest.NewRecorder()
	api.RefreshTokenHandler(w, req)

	resp = w.Result()
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusUnauthorized {
		t.Fatalf("Invalid token: expected 401, got %d", resp.StatusCode)
	}
}
