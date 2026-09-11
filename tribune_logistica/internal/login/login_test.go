package login

import (
	"database/sql"
	"sync"
	"testing"
	"time"

	"github.com/alexedwards/argon2id"
	"github.com/book_legion-tribune_logistica/internal/storage"
	_ "modernc.org/sqlite"
)

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

func setupTestUser(t *testing.T) (*storage.SQLStorage, User, string) {
	t.Helper() // marks this as a helper for nicer test output

	store := setupTestDB(t)
	password := "mysecretpassword"

	user, _ := NewUser("pete", password, "0000")
	// Insert user
	if err := InsertUser(store, user); err != nil {
		t.Fatalf("InsertUser failed: %v", err)
	}

	return store, user, password
}

// --------------------
// Test VerifyUserLogin with correct password
func TestVerifyUserLogin_CorrectPassword(t *testing.T) {
	store, _, password := setupTestUser(t)

	user, err := VerifyUserLogin("pete", password, store)
	if err != nil {
		t.Fatalf("VerifyUserLogin failed: %v", err)
	}
	if user.refreshToken == "" {
		t.Fatal("Expected refresh token, got empty string")
	}
	if user.authToken == "" {
		t.Fatal("Expected auth token, got empty string")
	}
}

// Test VerifyUserLogin with incorrect password
func TestVerifyUserLogin_IncorrectPassword(t *testing.T) {
	store, _, _ := setupTestUser(t)

	_, err := VerifyUserLogin("pete", "wrongpassword", store)
	if err == nil {
		t.Fatal("Expected error for wrong password, got nil")
	}
}

func TestLoginWithNonexistentUser(t *testing.T) {
	store, _, _ := setupTestUser(t)

	_, err := VerifyUserLogin("nonexistent", "password", store)
	if err == nil {
		t.Fatal("Expected error for nonexistent user, got nil")
	}
}

func TestNewUserSession(t *testing.T) {
	store, user, password := setupTestUser(t)
	session, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}
	_, exists := sessionStore.byAuthToken[session.authToken]
	if !exists {
		t.Fatalf("session not registered")
	}

	_, exists = sessionStore.byRefreshToken[session.refreshToken]
	if !exists {
		t.Fatalf("session not registered with refresh token")
	}
	_, ok := getSessionByrefreshToken(session.refreshToken)
	if ok != nil {
		t.Fatalf("could not get session by refresh token")

	}

	_, exists = sessionStore.byUser[session.Username]
	if !exists {
		t.Fatalf("session not registered with username")
	}
	_, ok = getSessionByUser(session.Username)
	if ok != nil {
		t.Fatalf("could not get session by username")

	}
}

// Test generating auth token with valid refresh token
func TestGenerateAuthTokenWithValidRefreshToken(t *testing.T) {
	store, user, password := setupTestUser(t)

	sessionUser, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}

	sessionUser, err = RefreshAuthToken(sessionUser.Username, sessionUser.refreshToken)
	if err != nil {
		t.Fatalf("GenerateAuthToken failed: %v", err)
	}
	if sessionUser.authToken == "" {
		t.Fatal("Expected auth token, got empty string")
	}

	checkUser, err := VerifyUserSession(sessionUser.authToken)
	if err != nil {
		t.Fatal("Expected auth token to be valid, {}", err)
	}

	if checkUser.Username != sessionUser.Username {
		t.Fatal("expected same username")
	}
}

// Test GenerateAuthToken with invalid refresh token
func TestGenerateAuthTokenWithInvalidRefreshToken(t *testing.T) {
	store, user, password := setupTestUser(t)

	_, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}
	_, err = RefreshAuthToken(user.Username, "invalidtoken")
	if err == nil {
		t.Fatal("Expected error for invalid refresh token")
	}
}

func TestNewLoginInvalidatesOldAuths(t *testing.T) {
	store, user, password := setupTestUser(t)

	sessionUser, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}

	// Simulate a new login for the same user
	newSessionUser, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}

	// Verify that the old auth token is no longer valid
	_, err = VerifyUserSession(sessionUser.authToken)
	if err == nil {
		t.Fatal("Expected old auth token to be invalid")
	}

	_, err = RefreshAuthToken(newSessionUser.Username, sessionUser.refreshToken)
	if err == nil {
		t.Fatal("expected old refresh token to be invalid")
	}

	// Verify that the new auth token is valid
	user, err = VerifyUserSession(newSessionUser.authToken)
	if err != nil {
		t.Fatal("Expected new auth token to be valid")
	}
	if user.Username != newSessionUser.Username {
		t.Fatal("recieved wrong user")
	}

	_, err = RefreshAuthToken(newSessionUser.Username, newSessionUser.refreshToken)
	if err != nil {
		t.Fatal("expected new refresh token to be valid")
	}

}

// Test VerifyAuthToken expiry
func TestVerifyAuthToken_Expiry(t *testing.T) {
	store, user, password := setupTestUser(t)

	user, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}

	// Temporarily set TTL to 1ms
	SetAuthTokenTTL(1 * time.Millisecond)
	defer SetAuthTokenTTL(15 * time.Minute) // reset after test

	user, err = RefreshAuthToken(user.Username, user.refreshToken)
	if err != nil {
		t.Fatalf("GenerateAuthToken failed: %v", err)
	}

	time.Sleep(5 * time.Millisecond) // wait for token to expire

	_, err = VerifyUserSession(user.authToken)
	if err == nil {
		t.Fatal("Expected expired token to fail verification")
	}

	_, err = RefreshAuthToken(user.Username, user.refreshToken)
	if err != nil {
		t.Fatal("expected refresh token to give new auth token")
	}
}

// Test VerifyAuthToken with completely invalid token
func TestVerifyAuthToken_InvalidToken(t *testing.T) {
	store, user, password := setupTestUser(t)
	_, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}

	_, err = VerifyUserSession("notarealtoken")
	if err == nil {
		t.Fatal("Expected invalid token to fail verification")
	}
}

func TestVerifyGetToken_InvalidToken(t *testing.T) {
	store, user, password := setupTestUser(t)
	_, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}

	ok, err := VerifyGetSession("notarealtoken")
	if err == nil || ok {
		t.Fatal("Expected invalid token to fail verification")
	}
}

func TestRefreshTokenPersistence(t *testing.T) {
	// 2️⃣ Create first store and insert user
	store1 := setupTestDB(t)

	password := "mypassword"
	passwordHash, err := argon2id.CreateHash(password, argon2id.DefaultParams)
	if err != nil {
		t.Fatalf("failed to hash password: %v", err)
	}

	user := User{
		Username:     "alice",
		PasswordHash: passwordHash,
		Pin:          "0000",
	}

	if err := InsertUser(store1, user); err != nil {
		t.Fatalf("InsertUser failed: %v", err)
	}

	// 3️⃣ Log in and get refresh token
	user, err = NewUserSession("alice", password, store1)
	if err != nil {
		t.Fatalf("NewUserSession failed: %v", err)
	}
	if user.refreshToken == "" {
		t.Fatal("Expected refresh token, got empty string")
	}

	// 6️⃣ Verify that refresh token still works
	user, err = RefreshAuthToken(user.Username, user.refreshToken)
	if err != nil {
		t.Fatalf("GenerateAuthToken failed: %v", err)
	}
	if user.authToken == "" {
		t.Fatal("Expected auth token, got empty string")
	}

	_, err = VerifyUserSession(user.authToken)
	if err != nil {
		t.Fatal("Auth token verification failed")
	}
	if user.Username != "alice" {
		t.Fatalf("Expected username 'alice', got %s", user.Username)
	}
}

func TestRefreshAuthTokenWithIncorrectUsername(t *testing.T) {
	store, user, password := setupTestUser(t)

	session, err := NewUserSession(user.Username, password, store)
	if err != nil {
		t.Fatal("expected user session to be setup")
	}

	_, err = RefreshAuthToken("nonexistent", session.refreshToken)
	if err == nil {
		t.Fatal("Expected error for nonexistent user, got nil")
	}
}

func setupMultipleUsers(t *testing.T, usernames []string) (*storage.SQLStorage, map[string]string) {
	t.Helper() // marks this as a helper for nicer test output

	store := setupTestDB(t)
	passwords := make(map[string]string)

	for _, u := range usernames {
		password := "password_" + u
		hash, err := argon2id.CreateHash(password, argon2id.DefaultParams)
		if err != nil {
			t.Fatalf("failed to hash password for %s: %v", u, err)
		}

		user := User{
			Username:     u,
			PasswordHash: hash,
			Pin:          "0000",
		}
		if err := InsertUser(store, user); err != nil {
			t.Fatalf("InsertUser failed for %s: %v", u, err)
		}

		passwords[u] = password
	}

	return store, passwords
}

// Test multi-user login and token isolation
func TestMultiUserLogin(t *testing.T) {
	usernames := []string{"alice", "bob", "carol"}
	store, passwords := setupMultipleUsers(t, usernames)

	refreshTokens := make(map[string]string)
	authTokens := make(map[string]string)

	for _, u := range usernames {
		// VerifyUserLogin
		user, err := NewUserSession(u, passwords[u], store)
		if err != nil {
			t.Fatalf("NewUserSession failed for %s: %v", u, err)
		}
		refreshTokens[u] = user.refreshToken

		//
		authTokens[u] = user.authToken
	}

	// Check that auth tokens map correctly
	for u, at := range authTokens {
		_, err := VerifyUserSession(at)
		if err != nil {
			t.Fatalf("Auth token for %s is invalid", u)
		}
	}
}

// Test concurrent logins for thread safety
func TestConcurrentLogins(t *testing.T) {
	usernames := []string{"alice", "bob", "carol", "dave", "eve"}
	store, passwords := setupMultipleUsers(t, usernames)

	var wg sync.WaitGroup
	authTokens := sync.Map{} // thread-safe map

	for _, u := range usernames {
		wg.Add(1)
		go func(un string) {
			defer wg.Done()

			user, err := NewUserSession(un, passwords[un], store)
			if err != nil {
				t.Errorf("VerifyUserLogin failed for %s: %v", un, err)
				return
			}

			authTokens.Store(user.authToken, user.Username)

			// Verify immediately
			_, err = VerifyUserSession(user.authToken)
			if err != nil {
				t.Errorf("Auth token verification failed for %s", user.Username)
			}
		}(u)
	}

	wg.Wait()

	// Verify all stored auth tokens
	authTokens.Range(func(key, value interface{}) bool {
		at := key.(string)
		u := value.(string)

		_, err := VerifyUserSession(at)
		if err != nil {
			t.Errorf("Final auth token verification failed for %s", u)
		}
		return true
	})
}

// Test token expiry under multi-user load
func TestMultiUserTokenExpiry(t *testing.T) {
	usernames := []string{"alice", "bob"}
	store, passwords := setupMultipleUsers(t, usernames)

	SetAuthTokenTTL(5 * time.Millisecond) // very short TTL
	defer SetAuthTokenTTL(15 * time.Minute)

	for _, u := range usernames {
		rt, err := NewUserSession(u, passwords[u], store)
		if err != nil {
			t.Fatalf("VerifyUserLogin failed for %s: %v", u, err)
		}

		user, err := RefreshAuthToken(rt.Username, rt.refreshToken)
		if err != nil {
			t.Fatalf("GenerateAuthToken failed for %s: %v", u, err)
		}

		time.Sleep(10 * time.Millisecond) // wait for expiry

		_, err = VerifyUserSession(user.authToken)
		if err == nil {
			t.Fatalf("Expired auth token for %s should not be valid", u)
		}
	}
}

func TestChangeUserPIN(t *testing.T) {
	store := setupTestDB(t)

	user := User{
		Username:     "alice",
		PasswordHash: "hash",
		Pin:          "1234",
	}

	if err := InsertUser(store, user); err != nil {
		t.Fatalf("failed to insert user: %v", err)
	}

	if err := ChangeUserPIN(store, user.Username, "5678"); err != nil {
		t.Fatalf("failed to change PIN: %v", err)
	}

	var pin string
	err := store.DB.QueryRow(`
		SELECT pin
		FROM users
		WHERE username = ?
	`, user.Username).Scan(&pin)
	if err != nil {
		t.Fatalf("failed to read PIN: %v", err)
	}

	if pin != "5678" {
		t.Errorf("expected PIN %q, got %q", "5678", pin)
	}
}
