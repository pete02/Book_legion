package storage

import (
	"database/sql"
	"testing"

	_ "modernc.org/sqlite"
)

func setupTestDB(t *testing.T) *sql.DB {
	t.Helper()

	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatalf("failed to open sqlite db: %v", err)
	}

	return db
}

func TestSQLStorage_InsertAndGetAll(t *testing.T) {
	db := setupTestDB(t)
	st, err := NewSQLStorage(db)
	if err != nil {
		t.Fatalf("failed to create SQLStorage: %v", err)
	}

	err = st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  30,
	})
	if err != nil {
		t.Fatalf("insert failed: %v", err)
	}

	rows, err := st.GetAll("users")
	if err != nil {
		t.Fatalf("getall failed: %v", err)
	}

	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}

	if rows[0]["name"] != "Alice" {
		t.Fatalf("unexpected name: %v", rows[0]["name"])
	}
}

func TestSQLStorage_InsertTwice(t *testing.T) {
	db := setupTestDB(t)
	st, err := NewSQLStorage(db)
	if err != nil {
		t.Fatalf("failed to create SQLStorage: %v", err)
	}

	err = st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  30,
	})
	if err != nil {
		t.Fatalf("insert failed: %v", err)
	}

	err = st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  40,
	})
	if err != nil {
		t.Fatalf("insert failed: %v", err)
	}

	rows, err := st.GetAll("users")
	if err != nil {
		t.Fatalf("getall failed: %v", err)
	}

	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}

	if rows[0]["name"] != "Alice" {
		t.Fatalf("unexpected name: %v", rows[0]["name"])
	}
	if rows[0]["age"] != int64(40) {
		t.Fatalf("unexpected age: %v, %T", rows[0]["age"], rows[0]["age"])
	}
}

func TestSQLStorage_QueryWithFilter(t *testing.T) {
	db := setupTestDB(t)
	st, err := NewSQLStorage(db)
	if err != nil {
		t.Fatalf("failed to create SQLStorage: %v", err)
	}

	st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  30,
	})
	st.Insert("users", "name", map[string]interface{}{
		"name": "Bob",
		"age":  40,
	})

	rows, err := st.Query("users", map[string]interface{}{
		"name": "Bob",
	})
	if err != nil {
		t.Fatalf("query failed: %v", err)
	}

	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}

	if rows[0]["name"] != "Bob" {
		t.Fatalf("unexpected row: %+v", rows[0])
	}
}

func TestSQLStorage_Delete(t *testing.T) {
	db := setupTestDB(t)
	st, err := NewSQLStorage(db)
	if err != nil {
		t.Fatalf("failed to create SQLStorage: %v", err)
	}

	st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
	})

	err = st.Delete("users", map[string]interface{}{
		"name": "Alice",
	})
	if err != nil {
		t.Fatalf("delete failed: %v", err)
	}

	rows, err := st.GetAll("users")
	if err != nil {
		t.Fatalf("getall failed: %v", err)
	}

	if len(rows) != 0 {
		t.Fatalf("expected 0 rows, got %d", len(rows))
	}
}

func TestSQLStorage_DeleteWithoutFilter(t *testing.T) {
	db := setupTestDB(t)
	st, err := NewSQLStorage(db)
	if err != nil {
		t.Fatalf("failed to create SQLStorage: %v", err)
	}

	err = st.Delete("users", map[string]interface{}{})
	if err == nil {
		t.Fatal("expected error when deleting without filter")
	}
}

func TestMigrateV1ToV2(t *testing.T) {
	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatal(err)
	}
	db.SetMaxOpenConns(1)

	_, err = db.Exec(`
		CREATE TABLE users (
			username TEXT PRIMARY KEY,
			password_hash TEXT,
			refresh_token TEXT,
			get_token TEXT
		);

		CREATE TABLE manifest (
			series_id TEXT PRIMARY KEY,
			series_name TEXT,
			first_book_id TEXT
		);

		CREATE TABLE books (
			id TEXT PRIMARY KEY,
			title TEXT,
			author_id TEXT,
			series_id TEXT,
			series_order INTEGER,
			series_name TEXT,
			file_path TEXT
		);

		CREATE TABLE UserCursors (
			id TEXT PRIMARY KEY,
			chapter INTEGER,
			chunk INTEGER,
			user_id TEXT,
			book_id TEXT
		);

		CREATE TABLE schema_version (
			version INTEGER NOT NULL
		);

		INSERT INTO schema_version VALUES (1);

		INSERT INTO users (
			username, password_hash, refresh_token, get_token
		) VALUES (
			'user1', 'hash1', 'refresh1', 'old-token'
		);

		INSERT INTO books (
			id, title, author_id, series_id, series_order,
			series_name, file_path
		) VALUES (
			'book1', 'Book One', 'author1', 'series1', 1,
			'Series One', '/books/book1.epub'
		);

		INSERT INTO manifest (
			series_id, series_name, first_book_id
		) VALUES (
			'series1', 'Series One', 'book1'
		);
	`)
	if err != nil {
		t.Fatal(err)
	}

	s := &SQLStorage{DB: db}

	if err := s.migrateV1ToV2(); err != nil {
		t.Fatalf("migration failed: %v", err)
	}

	// users
	var passwordHash, pin, refreshToken string
	err = db.QueryRow(`
		SELECT password_hash, pin, refresh_token
		FROM users
		WHERE username = ?
	`, "user1").Scan(&passwordHash, &pin, &refreshToken)
	if err != nil {
		t.Fatalf("failed to load migrated user: %v", err)
	}

	if passwordHash != "hash1" {
		t.Errorf("password_hash = %q, want %q", passwordHash, "hash1")
	}
	if pin != "" {
		t.Errorf("pin = %q, want empty migration value", pin)
	}
	if refreshToken != "refresh1" {
		t.Errorf("refresh_token = %q, want %q", refreshToken, "refresh1")
	}

	// get_token must no longer exist.
	var getTokenCount int
	err = db.QueryRow(`
		SELECT COUNT(*)
		FROM pragma_table_info('users')
		WHERE name = 'get_token'
	`).Scan(&getTokenCount)
	if err != nil {
		t.Fatal(err)
	}
	if getTokenCount != 0 {
		t.Error("get_token still exists")
	}

	// series
	var seriesName string
	err = db.QueryRow(`
		SELECT series_name
		FROM series
		WHERE series_id = ?
	`, "series1").Scan(&seriesName)
	if err != nil {
		t.Fatalf("failed to load migrated series: %v", err)
	}
	if seriesName != "Series One" {
		t.Errorf("series_name = %q, want %q", seriesName, "Series One")
	}

	// books
	var title, seriesID, filePath string
	var seriesOrder int

	err = db.QueryRow(`
		SELECT title, series_id, series_order, file_path
		FROM books
		WHERE id = ?
	`, "book1").Scan(&title, &seriesID, &seriesOrder, &filePath)
	if err != nil {
		t.Fatalf("failed to load migrated book: %v", err)
	}

	if title != "Book One" {
		t.Errorf("title = %q, want %q", title, "Book One")
	}
	if seriesID != "series1" {
		t.Errorf("series_id = %q, want %q", seriesID, "series1")
	}
	if seriesOrder != 1 {
		t.Errorf("series_order = %d, want 1", seriesOrder)
	}
	if filePath != "/books/book1.epub" {
		t.Errorf("file_path = %q, want %q", filePath, "/books/book1.epub")
	}

	// manifest must be gone.
	var manifestCount int
	err = db.QueryRow(`
		SELECT COUNT(*)
		FROM sqlite_master
		WHERE type = 'table' AND name = 'manifest'
	`).Scan(&manifestCount)
	if err != nil {
		t.Fatal(err)
	}
	if manifestCount != 0 {
		t.Error("manifest table still exists")
	}

	// user_access must exist.
	var accessCount int
	err = db.QueryRow(`
		SELECT COUNT(*)
		FROM sqlite_master
		WHERE type = 'table' AND name = 'user_access'
	`).Scan(&accessCount)
	if err != nil {
		t.Fatal(err)
	}
	if accessCount != 1 {
		t.Error("user_access table was not created")
	}

	// schema version
	var version int
	err = db.QueryRow(`SELECT version FROM schema_version`).Scan(&version)
	if err != nil {
		t.Fatal(err)
	}
	if version != 2 {
		t.Errorf("schema version = %d, want 2", version)
	}
}
