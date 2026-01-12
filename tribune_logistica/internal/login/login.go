package login

import (
	"crypto/rand"
	"encoding/base64"
	"errors"
	"time"

	"github.com/alexedwards/argon2id"
	"github.com/book_legion-tribune_logistica/internal/storage" // import your Storage interface
)

type User struct {
	Username     string
	PasswordHash string
	RefreshToken string
}

func NewUser(username string, password string) (User, error) {
	passwordHash, err := argon2id.CreateHash(password, argon2id.DefaultParams)
	if err != nil {
		return User{}, err
	}

	user := User{
		Username:     username,
		PasswordHash: passwordHash,
		RefreshToken: "",
	}

	return user, nil
}

var authTokens = make(map[string]string) // authToken → username

// Session expiry
var authTokenTTL = 15 * time.Minute
var authTokenExpiry = make(map[string]time.Time)

func GetAuthTokenTTL() time.Duration {
	return authTokenTTL
}

func SetAuthTokenTTL(ttl time.Duration) {
	authTokenTTL = ttl
}

func InsertUser(store storage.Storage, user User) error {
	row := map[string]interface{}{
		"username":      user.Username,
		"password_hash": user.PasswordHash,
		"refresh_token": user.RefreshToken,
	}
	return store.Insert("users", "username", row)
}

// Verifies user and returns Refresh Token
func VerifyUser(store storage.Storage, username, password string) (string, error) {
	rows, err := store.Query("users", map[string]interface{}{"username": username})
	if err != nil {
		return "", err
	}
	if len(rows) == 0 {
		return "", errors.New("user not found")
	}

	userRow := rows[0]
	storedHash := userRow["password_hash"].(string)
	if !verifyPassword(password, storedHash) {
		return "", errors.New("invalid password")
	}

	refreshToken, ok := userRow["refresh_token"].(string)
	if !ok || refreshToken == "" {
		refreshToken = generateRandomToken(32)
		userRow["refresh_token"] = refreshToken
		store.Insert("users", "username", userRow)
	}

	return refreshToken, nil
}

// --------------------
func GenerateAuthToken(store storage.Storage, refreshToken string) (string, error) {
	rows, err := store.Query("users", map[string]interface{}{"refresh_token": refreshToken})
	if err != nil {
		return "", err
	}
	if len(rows) == 0 {
		return "", errors.New("invalid refresh token")
	}

	username := rows[0]["username"].(string)
	authToken := generateRandomToken(32)

	// Store in memory with TTL
	authTokens[authToken] = username
	authTokenExpiry[authToken] = time.Now().Add(authTokenTTL)

	return authToken, nil
}

// returns true for correct token
func VerifyAuthToken(authToken string) (string, bool) {
	username, ok := authTokens[authToken]
	if !ok {
		return "", false
	}
	if time.Now().After(authTokenExpiry[authToken]) {
		delete(authTokens, authToken)
		delete(authTokenExpiry, authToken)
		return "", false
	}
	return username, true
}

// --------------------
// Helpers
func generateRandomToken(length int) string {
	b := make([]byte, length)
	_, _ = rand.Read(b)
	return base64.URLEncoding.EncodeToString(b)
}

// verifyPassword checks password against argon2id hash
func verifyPassword(password, hash string) bool {
	match, err := argon2id.ComparePasswordAndHash(password, hash)
	if err != nil {
		// Treat any error as verification failure
		return false
	}
	return match
}
