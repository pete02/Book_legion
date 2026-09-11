package api

import (
	"bytes"
	"database/sql"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/book_legion-tribune_logistica/internal/login"
	"github.com/book_legion-tribune_logistica/internal/storage"
	_ "modernc.org/sqlite"
)

// ---------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------

const testAdminToken = "test-admin-token"

func setupTestDB(t *testing.T) *storage.SQLStorage {
	t.Helper()

	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatalf("failed to open test database: %v", err)
	}

	db.SetMaxOpenConns(1)

	store, err := storage.NewSQLStorage(db)
	if err != nil {
		db.Close()
		t.Fatalf("failed to create SQL storage: %v", err)
	}

	t.Cleanup(func() {
		db.Close()
	})

	return store
}

func withAdminToken(t *testing.T) {
	t.Helper()
	os.Setenv("TRIBUNE_LOGISTICA_ADMIN_TOKEN", testAdminToken)
	t.Cleanup(func() { os.Unsetenv("TRIBUNE_LOGISTICA_ADMIN_TOKEN") })
}

// NewTestAPI spins up an API backed by a real, throwaway JSONStorage file
// so handlers exercise their actual DB read/write paths instead of a mock.
func NewTestAPI(t *testing.T) *API {
	return &API{DB: setupTestDB(t)}
}

// seedUser writes a user directly via the login package (bypassing the HTTP
// handler) so Login/Refresh tests don't depend on RegisterUser working.
func seedUser(t *testing.T, a *API, username, password string) {
	t.Helper()
	user, err := login.NewUser(username, password, "0000")
	if err != nil {
		t.Fatalf("failed to build user %q: %v", username, err)
	}
	if err := login.InsertUser(a.DB, user); err != nil {
		t.Fatalf("failed to insert user %q: %v", username, err)
	}
}

// uniqueUsername avoids collisions in login's package-level, in-memory
// sessionStore, which is keyed by username and shared across tests running
// in this binary.
func uniqueUsername(t *testing.T) string {
	t.Helper()
	return fmt.Sprintf("%s_%d", strings.ReplaceAll(t.Name(), "/", "_"), time.Now().UnixNano())
}

func jsonBody(t *testing.T, v interface{}) *bytes.Reader {
	t.Helper()
	b, err := json.Marshal(v)
	if err != nil {
		t.Fatalf("failed to marshal request body: %v", err)
	}
	return bytes.NewReader(b)
}

func decodeJSON(t *testing.T, rr *httptest.ResponseRecorder, v interface{}) {
	t.Helper()
	if err := json.NewDecoder(rr.Body).Decode(v); err != nil {
		t.Fatalf("failed to decode response body %q: %v", rr.Body.String(), err)
	}
}

// ---------------------------------------------------------------------
// RegisterUser
// ---------------------------------------------------------------------

func TestRegisterUser_MethodNotAllowed(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodGet, "/register", nil)
	rr := httptest.NewRecorder()

	a.RegisterUser(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestRegisterUser_MissingAuthHeader(t *testing.T) {
	withAdminToken(t)
	a := &API{}
	req := httptest.NewRequest(http.MethodPost, "/register", jsonBody(t, LoginRequest{Username: "alice", Password: "secret"}))
	rr := httptest.NewRecorder()

	a.RegisterUser(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestRegisterUser_MalformedBearerHeader(t *testing.T) {
	withAdminToken(t)
	a := &API{}

	req := httptest.NewRequest(http.MethodPost, "/register", jsonBody(t, LoginRequest{Username: "alice", Password: "secret"}))
	req.Header.Set("Authorization", "Bearer") // len <= 7, no token at all
	rr := httptest.NewRecorder()

	a.RegisterUser(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestRegisterUser_WrongScheme(t *testing.T) {
	withAdminToken(t)
	a := &API{}

	req := httptest.NewRequest(http.MethodPost, "/register", jsonBody(t, LoginRequest{Username: "alice", Password: "secret"}))
	req.Header.Set("Authorization", "Basic "+testAdminToken)
	rr := httptest.NewRecorder()

	a.RegisterUser(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestRegisterUser_InvalidToken(t *testing.T) {
	withAdminToken(t)
	a := &API{}

	req := httptest.NewRequest(http.MethodPost, "/register", jsonBody(t, LoginRequest{Username: "alice", Password: "secret"}))
	req.Header.Set("Authorization", "Bearer wrong-token")
	rr := httptest.NewRecorder()

	a.RegisterUser(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestRegisterUser_InvalidJSON(t *testing.T) {
	withAdminToken(t)
	a := &API{}

	req := httptest.NewRequest(http.MethodPost, "/register", bytes.NewReader([]byte("{not-valid-json")))
	req.Header.Set("Authorization", "Bearer "+testAdminToken)
	rr := httptest.NewRecorder()

	a.RegisterUser(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestRegisterUser_MissingCredentials(t *testing.T) {
	withAdminToken(t)

	cases := []struct {
		name string
		body LoginRequest
	}{
		{"empty username", LoginRequest{Username: "", Password: "secret"}},
		{"empty password", LoginRequest{Username: "alice", Password: ""}},
		{"both empty", LoginRequest{Username: "", Password: ""}},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			a := &API{}
			req := httptest.NewRequest(http.MethodPost, "/register", jsonBody(t, c.body))
			req.Header.Set("Authorization", "Bearer "+testAdminToken)
			rr := httptest.NewRecorder()

			a.RegisterUser(rr, req)

			if rr.Code != http.StatusBadRequest {
				t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
			}
		})
	}
}

func TestRegisterUser_Success(t *testing.T) {
	withAdminToken(t)
	a := NewTestAPI(t)
	username := uniqueUsername(t)
	password := "correct horse battery staple"

	req := httptest.NewRequest(http.MethodPost, "/register", jsonBody(t, RegisterRequest{
		Username: username,
		Password: password,
		Pin:      "0000",
	}))
	req.Header.Set("Authorization", "Bearer "+testAdminToken)
	rr := httptest.NewRecorder()

	a.RegisterUser(rr, req)

	if rr.Code != http.StatusCreated {
		t.Fatalf("expected %d, got %d: %s", http.StatusCreated, rr.Code, rr.Body.String())
	}
	if ct := rr.Header().Get("Content-Type"); ct != "application/json" {
		t.Errorf("expected Content-Type application/json, got %q", ct)
	}

	var resp RegisterResponse
	decodeJSON(t, rr, &resp)
	if !resp.Success {
		t.Errorf("expected Success=true, got false (message: %q)", resp.Message)
	}

	// The registered user should now actually be able to log in against
	// the same DB the handler wrote to — proves the write really landed,
	// not just that the handler said "success".
	if _, err := login.NewUserSession(username, password, a.DB); err != nil {
		t.Errorf("expected newly registered user to be able to log in, got error: %v", err)
	}
}

// ---------------------------------------------------------------------
// LoginUser
// ---------------------------------------------------------------------

func TestLoginUser_MethodNotAllowed(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodGet, "/login", nil)
	rr := httptest.NewRecorder()

	a.LoginUser(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestLoginUser_InvalidJSON(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodPost, "/login", bytes.NewReader([]byte("not json")))
	rr := httptest.NewRecorder()

	a.LoginUser(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestLoginUser_UnknownUser(t *testing.T) {
	a := NewTestAPI(t)
	req := httptest.NewRequest(http.MethodPost, "/login", jsonBody(t, LoginRequest{
		Username: uniqueUsername(t), // never seeded
		Password: "whatever",
	}))
	rr := httptest.NewRecorder()

	a.LoginUser(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestLoginUser_WrongPassword(t *testing.T) {
	a := NewTestAPI(t)
	username := uniqueUsername(t)
	seedUser(t, a, username, "correct-password")

	req := httptest.NewRequest(http.MethodPost, "/login", jsonBody(t, LoginRequest{
		Username: username,
		Password: "wrong-password",
	}))
	rr := httptest.NewRecorder()

	a.LoginUser(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestLoginUser_Success(t *testing.T) {
	a := NewTestAPI(t)
	username := uniqueUsername(t)
	password := "hunter2-but-longer-and-not-real"
	seedUser(t, a, username, password)

	req := httptest.NewRequest(http.MethodPost, "/login", jsonBody(t, LoginRequest{
		Username: username,
		Password: password,
	}))
	rr := httptest.NewRecorder()

	a.LoginUser(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, rr.Code, rr.Body.String())
	}
	if ct := rr.Header().Get("Content-Type"); ct != "application/json" {
		t.Errorf("expected Content-Type application/json, got %q", ct)
	}

	var resp LoginResponse
	decodeJSON(t, rr, &resp)
	if resp.AuthToken == "" {
		t.Error("expected non-empty auth_token")
	}
	if resp.RefreshToken == "" {
		t.Error("expected non-empty refresh_token")
	}
	if want := int(login.GetAuthTokenTTL().Seconds()); resp.ExpiresIn != want {
		t.Errorf("expected expires_in=%d, got %d", want, resp.ExpiresIn)
	}

	// The token issued should actually be valid against the session store.
	if user, err := login.VerifyUserSession(resp.AuthToken); err != nil && user.Username != username {
		t.Errorf("expected returned auth token to be a valid session for correct user, got error: %v", err)
	}
}

// ---------------------------------------------------------------------
// RefreshTokenHandler
// ---------------------------------------------------------------------

func TestRefreshTokenHandler_MethodNotAllowed(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodGet, "/refresh", nil)
	rr := httptest.NewRecorder()

	a.RefreshTokenHandler(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Errorf("expected %d, got %d", http.StatusMethodNotAllowed, rr.Code)
	}
}

func TestRefreshTokenHandler_InvalidJSON(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodPost, "/refresh", bytes.NewReader([]byte("{bad")))
	rr := httptest.NewRecorder()

	a.RefreshTokenHandler(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestRefreshTokenHandler_MissingRefreshToken(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodPost, "/refresh", jsonBody(t, RefreshRequest{
		Username:     "alice",
		RefreshToken: "",
	}))
	rr := httptest.NewRecorder()

	a.RefreshTokenHandler(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Errorf("expected %d, got %d", http.StatusBadRequest, rr.Code)
	}
}

func TestRefreshTokenHandler_UnknownRefreshToken(t *testing.T) {
	a := &API{}
	req := httptest.NewRequest(http.MethodPost, "/refresh", jsonBody(t, RefreshRequest{
		Username:     uniqueUsername(t),
		RefreshToken: "this-refresh-token-was-never-issued",
	}))
	rr := httptest.NewRecorder()

	a.RefreshTokenHandler(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}

func TestRefreshTokenHandler_Success(t *testing.T) {
	a := NewTestAPI(t)
	username := uniqueUsername(t)
	password := "another-not-real-password"
	seedUser(t, a, username, password)

	// Log in first to establish a session and get a real refresh token,
	// same as a real client would.
	loginReq := httptest.NewRequest(http.MethodPost, "/login", jsonBody(t, LoginRequest{
		Username: username,
		Password: password,
	}))
	loginRR := httptest.NewRecorder()
	a.LoginUser(loginRR, loginReq)
	if loginRR.Code != http.StatusOK {
		t.Fatalf("setup login failed: %d: %s", loginRR.Code, loginRR.Body.String())
	}
	var loginResp LoginResponse
	decodeJSON(t, loginRR, &loginResp)

	refreshReq := httptest.NewRequest(http.MethodPost, "/refresh", jsonBody(t, RefreshRequest{
		Username:     username,
		RefreshToken: loginResp.RefreshToken,
	}))
	refreshRR := httptest.NewRecorder()
	a.RefreshTokenHandler(refreshRR, refreshReq)

	if refreshRR.Code != http.StatusOK {
		t.Fatalf("expected %d, got %d: %s", http.StatusOK, refreshRR.Code, refreshRR.Body.String())
	}

	var refreshResp RefreshResponse
	decodeJSON(t, refreshRR, &refreshResp)
	if refreshResp.AuthToken == "" {
		t.Error("expected non-empty auth_token")
	}
	if refreshResp.AuthToken == loginResp.AuthToken {
		t.Error("expected refresh to issue a new auth token, got the same one back")
	}
	if want := int(login.GetAuthTokenTTL().Seconds()); refreshResp.ExpiresIn != want {
		t.Errorf("expected expires_in=%d, got %d", want, refreshResp.ExpiresIn)
	}

	// Old auth token should be dead, new one should be live.
	if _, err := login.VerifyUserSession(loginResp.AuthToken); err == nil {
		t.Error("expected old auth token to be invalidated after refresh")
	}
	if _, err := login.VerifyUserSession(refreshResp.AuthToken); err != nil {
		t.Errorf("expected new auth token to be valid, got error: %v", err)
	}
}

func TestRefreshTokenHandler_UsernameMismatchRejected(t *testing.T) {
	// A refresh_token is tied to the username it was issued for
	// (login.verifyUserrefreshToken returns "Hijacked refresh token" on
	// mismatch) — this makes sure that protection is actually wired up
	// through the HTTP handler and not just tested in isolation.
	a := NewTestAPI(t)
	username := uniqueUsername(t)
	password := "yet-another-not-real-password"
	seedUser(t, a, username, password)

	loginReq := httptest.NewRequest(http.MethodPost, "/login", jsonBody(t, LoginRequest{
		Username: username,
		Password: password,
	}))
	loginRR := httptest.NewRecorder()
	a.LoginUser(loginRR, loginReq)
	if loginRR.Code != http.StatusOK {
		t.Fatalf("setup login failed: %d: %s", loginRR.Code, loginRR.Body.String())
	}
	var loginResp LoginResponse
	decodeJSON(t, loginRR, &loginResp)

	refreshReq := httptest.NewRequest(http.MethodPost, "/refresh", jsonBody(t, RefreshRequest{
		Username:     "someone-else-" + username,
		RefreshToken: loginResp.RefreshToken,
	}))
	rr := httptest.NewRecorder()
	a.RefreshTokenHandler(rr, refreshReq)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected %d, got %d", http.StatusUnauthorized, rr.Code)
	}
}
