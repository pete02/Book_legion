package login

import (
	"os"
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

	// Create a test user
	password := "mysecretpassword"
	passwordHash, err := argon2id.CreateHash(password, argon2id.DefaultParams)
	if err != nil {
		t.Fatalf("failed to hash password: %v", err)
	}

	user := User{
		Username:     "pete",
		PasswordHash: passwordHash,
	}

	// Insert user
	if err := InsertUser(store, user); err != nil {
		t.Fatalf("InsertUser failed: %v", err)
	}

	return store, user, password
}

// --------------------
// Test VerifyUser with correct password
func TestVerifyUser_CorrectPassword(t *testing.T) {
	store, _, password := setupTestUser(t)

	refreshToken, err := VerifyUser(store, "pete", password)
	if err != nil {
		t.Fatalf("VerifyUser failed: %v", err)
	}
	if refreshToken == "" {
		t.Fatal("Expected refresh token, got empty string")
	}
}

// Test VerifyUser with incorrect password
func TestVerifyUser_IncorrectPassword(t *testing.T) {
	store, _, _ := setupTestUser(t)

	_, err := VerifyUser(store, "pete", "wrongpassword")
	if err == nil {
		t.Fatal("Expected error for wrong password, got nil")
	}
}

// Test GenerateAuthToken with valid refresh token
func TestGenerateAuthToken_ValidToken(t *testing.T) {
	store, _, password := setupTestUser(t)

	refreshToken, err := VerifyUser(store, "pete", password)
	if err != nil {
		t.Fatalf("VerifyUser failed: %v", err)
	}

	authToken, err := GenerateAuthToken(store, refreshToken)
	if err != nil {
		t.Fatalf("GenerateAuthToken failed: %v", err)
	}
	if authToken == "" {
		t.Fatal("Expected auth token, got empty string")
	}

	username, ok := VerifyAuthToken(authToken)
	if !ok {
		t.Fatal("Expected auth token to be valid")
	}
	if username != "pete" {
		t.Fatalf("Expected username 'pete', got %s", username)
	}
}

// Test GenerateAuthToken with invalid refresh token
func TestGenerateAuthToken_InvalidToken(t *testing.T) {
	store, _, _ := setupTestUser(t)

	_, err := GenerateAuthToken(store, "invalidtoken")
	if err == nil {
		t.Fatal("Expected error for invalid refresh token")
	}
}

// Test VerifyAuthToken expiry
func TestVerifyAuthToken_Expiry(t *testing.T) {
	store, _, password := setupTestUser(t)

	refreshToken, err := VerifyUser(store, "pete", password)
	if err != nil {
		t.Fatalf("VerifyUser failed: %v", err)
	}

	// Temporarily set TTL to 1ms
	SetAuthTokenTTL(1 * time.Millisecond)
	defer SetAuthTokenTTL(15 * time.Minute) // reset after test

	authToken, err := GenerateAuthToken(store, refreshToken)
	if err != nil {
		t.Fatalf("GenerateAuthToken failed: %v", err)
	}

	time.Sleep(5 * time.Millisecond) // wait for token to expire

	_, ok := VerifyAuthToken(authToken)
	if ok {
		t.Fatal("Expected expired token to fail verification")
	}
}

// Test VerifyAuthToken with completely invalid token
func TestVerifyAuthToken_InvalidToken(t *testing.T) {
	_, ok := VerifyAuthToken("notarealtoken")
	if ok {
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
	refreshToken, err := VerifyUser(store1, "alice", password)
	if err != nil {
		t.Fatalf("VerifyUser failed: %v", err)
	}
	if refreshToken == "" {
		t.Fatal("Expected refresh token, got empty string")
	}

	// 4️⃣ Save the store to disk
	if err := store1.Save(tmpFile); err != nil {
		t.Fatalf("Failed to save store: %v", err)
	}

	// 5️⃣ Re-create the store (simulate app restart)
	store2, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create store2: %v", err)
	}

	// 6️⃣ Verify that refresh token still works
	authToken, err := GenerateAuthToken(store2, refreshToken)
	if err != nil {
		t.Fatalf("GenerateAuthToken failed: %v", err)
	}
	if authToken == "" {
		t.Fatal("Expected auth token, got empty string")
	}

	username, ok := VerifyAuthToken(authToken)
	if !ok {
		t.Fatal("Auth token verification failed")
	}
	if username != "alice" {
		t.Fatalf("Expected username 'alice', got %s", username)
	}

	// 7️⃣ Verify password still works after reload
	refreshToken2, err := VerifyUser(store2, "alice", password)
	if err != nil {
		t.Fatalf("VerifyUser failed after reload: %v", err)
	}
	if refreshToken2 == "" {
		t.Fatal("Expected refresh token after reload, got empty string")
	}
}
