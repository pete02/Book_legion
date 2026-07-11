package login

import (
	"os"
	"sync"
	"testing"
	"time"

	"github.com/alexedwards/argon2id"
	"github.com/book_legion-tribune_logistica/internal/storage"
)

func setupTestUser(t *testing.T) (*storage.JSONStorage, User, string) {
	t.Helper() // marks this as a helper for nicer test output

	// Create temporary JSONStorage
	tmpFile := "test_data.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create storage: %v", err)
	}
	password := "mysecretpassword"

	user, err := NewUser("pete", password)
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

	ok := VerifyUserSession(sessionUser.Username, sessionUser.authToken)
	if ok != nil {
		t.Fatal("Expected auth token to be valid, {}", ok)
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
	err = VerifyUserSession(newSessionUser.Username, sessionUser.authToken)
	if err == nil {
		t.Fatal("Expected old auth token to be invalid")
	}

	_, err = RefreshAuthToken(newSessionUser.Username, sessionUser.refreshToken)
	if err == nil {
		t.Fatal("expected old refresh token to be invalid")
	}

	// Verify that the new auth token is valid
	err = VerifyUserSession(newSessionUser.Username, newSessionUser.authToken)
	if err != nil {
		t.Fatal("Expected new auth token to be valid")
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

	err = VerifyUserSession(user.Username, user.authToken)
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

	err = VerifyUserSession("notarealtoken", "notarealtoken")
	if err == nil {
		t.Fatal("Expected invalid token to fail verification")
	}
}

func TestRefreshTokenPersistence(t *testing.T) {
	// 1️⃣ Create temporary JSON file
	tmpFile := "test_persistence.json"
	defer os.Remove(tmpFile)

	// 2️⃣ Create first store and insert user
	store1, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create store1: %v", err)
	}

	password := "mypassword"
	passwordHash, err := argon2id.CreateHash(password, argon2id.DefaultParams)
	if err != nil {
		t.Fatalf("failed to hash password: %v", err)
	}

	user := User{
		Username:     "alice",
		PasswordHash: passwordHash,
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

	// 4️⃣ Save the store to disk
	if err := store1.Save(); err != nil {
		t.Fatalf("Failed to save store: %v", err)
	}

	// 5️⃣ Re-create the store (simulate app restart)
	store2, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create store2: %v", err)
	}

	// 6️⃣ Verify that refresh token still works
	user, err = RefreshAuthToken(user.Username, user.refreshToken)
	if err != nil {
		t.Fatalf("GenerateAuthToken failed: %v", err)
	}
	if user.authToken == "" {
		t.Fatal("Expected auth token, got empty string")
	}

	err = VerifyUserSession(user.Username, user.authToken)
	if err != nil {
		t.Fatal("Auth token verification failed")
	}
	if user.Username != "alice" {
		t.Fatalf("Expected username 'alice', got %s", user.Username)
	}

	// 7️⃣ Verify password still works after reload
	user2, err := NewUserSession("alice", password, store2)
	if err != nil {
		t.Fatalf("NewUserSession failed after reload: %v", err)
	}
	if user2.refreshToken == "" {
		t.Fatal("Expected refresh token after reload, got empty string")
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

func setupMultipleUsers(t *testing.T, usernames []string) (*storage.JSONStorage, map[string]string) {
	t.Helper() // marks this as a helper for nicer test output

	// Create temporary JSONStorage
	tmpFile := "test_data.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create storage: %v", err)
	}
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
		ok := VerifyUserSession(u, at)
		if ok != nil {
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
			ok := VerifyUserSession(user.Username, user.authToken)
			if ok != nil {
				t.Errorf("Auth token verification failed for %s", user.Username)
			}
		}(u)
	}

	wg.Wait()

	// Verify all stored auth tokens
	authTokens.Range(func(key, value interface{}) bool {
		at := key.(string)
		u := value.(string)

		err := VerifyUserSession(u, at)
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

		err = VerifyUserSession(u, user.authToken)
		if err == nil {
			t.Fatalf("Expired auth token for %s should not be valid", u)
		}
	}
}
