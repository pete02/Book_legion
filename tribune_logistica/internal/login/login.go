package login

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"sync"
	"time"

	"github.com/alexedwards/argon2id"
	"github.com/book_legion-tribune_logistica/internal/storage" // import your Storage interface
)

type User struct {
	Username     string
	PasswordHash string
	refreshToken string
	authToken    string
}

func (u User) GetAuthToken() string {
	return u.authToken
}
func (u User) GetRefreshToken() string {
	return u.refreshToken
}

func NewUser(username string, password string) (User, error) {
	passwordHash, err := argon2id.CreateHash(password, argon2id.DefaultParams)
	if err != nil {
		return User{}, err
	}

	user := User{
		Username:     username,
		PasswordHash: passwordHash,
		refreshToken: "",
		authToken:    "",
	}

	return user, nil
}

func InsertUser(store storage.Storage, user User) error {
	row := map[string]interface{}{
		"username":      user.Username,
		"password_hash": user.PasswordHash,
		"refresh_token": user.refreshToken,
	}
	return store.Insert("users", "username", row)
}

func getUser(store storage.Storage, username string) (User, error) {
	rows, err := store.Query("users", map[string]interface{}{"username": username})
	if err != nil {
		return User{}, err
	}
	if len(rows) == 0 {
		return User{}, fmt.Errorf("user not found")
	}
	if len(rows) > 1 {
		return User{}, fmt.Errorf("multiple users found")
	}

	return User{
		Username:     username,
		PasswordHash: rows[0]["password_hash"].(string),
		refreshToken: rows[0]["refresh_token"].(string),
		authToken:    "",
	}, nil
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

func VerifyUserLogin(username, password string, storage storage.Storage) (User, error) {
	user, err := getUser(storage, username)
	if err != nil {
		return User{}, err
	}

	if !verifyPassword(password, user.PasswordHash) {
		return User{}, fmt.Errorf("invalid password")
	}

	authToken := GenerateRandomToken(32)
	refreshToken := GenerateRandomToken(32)

	user.refreshToken = refreshToken
	user.authToken = authToken

	err = InsertUser(storage, user)
	if err != nil {
		return User{}, err
	}
	return user, nil
}

type Session struct {
	user             User
	authExpiresAt    time.Time
	refreshExpiresAt time.Time
}

type SessionStore struct {
	mu             sync.Mutex
	authTtl        time.Duration
	refreshTtl     time.Duration
	byAuthToken    map[string]*Session
	byUser         map[string]string
	byRefreshToken map[string]string
}

func newSessionStore(authTtl time.Duration, refreshTtl time.Duration) *SessionStore {
	return &SessionStore{
		authTtl:        authTtl,
		refreshTtl:     refreshTtl,
		byAuthToken:    make(map[string]*Session),
		byUser:         make(map[string]string),
		byRefreshToken: make(map[string]string),
	}
}

func getSessionByUser(username string) (*Session, error) {
	authToken, exists := sessionStore.byUser[username]
	if !exists {
		return nil, fmt.Errorf("No session")
	}

	session, exists := sessionStore.byAuthToken[authToken]
	if !exists {
		return nil, fmt.Errorf("invalid session")
	}

	return session, nil
}

func getSessionByrefreshToken(refreshToken string) (*Session, error) {
	authToken, exists := sessionStore.byRefreshToken[refreshToken]
	if !exists {
		return nil, fmt.Errorf("invalid refresh token")
	}

	session, exists := sessionStore.byAuthToken[authToken]
	if !exists {
		return nil, fmt.Errorf("invalid session")
	}

	return session, nil
}

var sessionStore = newSessionStore(15*time.Minute, 24*time.Hour)

func GetAuthTokenTTL() time.Duration {
	sessionStore.mu.Lock()
	defer sessionStore.mu.Unlock()
	return sessionStore.authTtl
}

func newSession(user User, sessionStore *SessionStore) User {
	session := Session{
		user:             user,
		authExpiresAt:    time.Now().Add(sessionStore.authTtl),
		refreshExpiresAt: time.Now().Add(sessionStore.refreshTtl),
	}

	sessionStore.mu.Lock()
	defer sessionStore.mu.Unlock()

	existing, err := getSessionByUser(user.Username)
	if err == nil {
		delete(sessionStore.byAuthToken, existing.user.authToken)
		delete(sessionStore.byRefreshToken, existing.user.refreshToken)
		delete(sessionStore.byUser, existing.user.Username)
	}

	sessionStore.byUser[user.Username] = user.authToken
	sessionStore.byAuthToken[user.authToken] = &session
	sessionStore.byRefreshToken[user.refreshToken] = user.authToken

	return user
}

func SetAuthTokenTTL(ttl time.Duration) {
	sessionStore.mu.Lock()
	defer sessionStore.mu.Unlock()
	sessionStore.authTtl = ttl
}

func NewUserSession(username string, password string, storage storage.Storage) (User, error) {
	user, err := VerifyUserLogin(username, password, storage)
	if err != nil {
		return User{}, err
	}
	return newSession(user, sessionStore), nil
}

func VerifyUserSession(username string, verifyAuthToken string) error {
	sessionStore.mu.Lock()
	defer sessionStore.mu.Unlock()
	session, err := getSessionByUser(username)
	if err != nil {
		return err
	}

	if time.Now().After(session.authExpiresAt) {
		return fmt.Errorf("auth token expired")
	}

	if session.user.authToken != verifyAuthToken {
		return fmt.Errorf("invalid session")
	}

	return nil
}

func verifyUserrefreshToken(username string, verifyrefreshToken string) (User, error) {
	sessionStore.mu.Lock()
	defer sessionStore.mu.Unlock()
	session, err := getSessionByrefreshToken(verifyrefreshToken)
	if err != nil {
		return User{}, err
	}

	if session.user.Username != username {
		return User{}, fmt.Errorf("Hijacked refresh token")
	}
	if session.user.refreshToken != verifyrefreshToken {
		return User{}, fmt.Errorf("invalid refresh token")
	}
	if time.Now().After(session.refreshExpiresAt) {
		delete(sessionStore.byAuthToken, session.user.authToken)
		delete(sessionStore.byRefreshToken, session.user.refreshToken)
		delete(sessionStore.byUser, session.user.Username)

		return User{}, fmt.Errorf("refresh token expired")

	}

	return session.user, nil
}

func RefreshAuthToken(username string, refreshToken string) (User, error) {
	user, err := verifyUserrefreshToken(username, refreshToken)
	if err != nil {
		return User{}, err
	}
	oldAuthToken := user.authToken

	user.authToken = GenerateRandomToken(32)

	sessionStore.mu.Lock()
	defer sessionStore.mu.Unlock()

	session, err := getSessionByUser(user.Username)
	if err != nil {
		return User{}, err
	}

	session.user = user
	session.authExpiresAt = time.Now().Add(sessionStore.authTtl)

	sessionStore.byAuthToken[user.authToken] = session
	sessionStore.byRefreshToken[user.refreshToken] = user.authToken
	sessionStore.byUser[user.Username] = user.authToken

	delete(sessionStore.byAuthToken, oldAuthToken)

	return user, nil
}

func GenerateRandomToken(length int) string {
	b := make([]byte, length)
	_, _ = rand.Read(b)
	return base64.URLEncoding.EncodeToString(b)
}
