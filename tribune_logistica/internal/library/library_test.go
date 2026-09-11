package library

import (
	"database/sql"

	"os"
	"path/filepath"
	"testing"

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

func ptr(s string) *string {
	return &s
}

func TestSaveAndLoadBook(t *testing.T) {
	store := setupTestDB(t)

	book := Book{
		ID:          "book-1",
		Title:       "The First Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "The Series",
		SeriesOrder: 1,
		FilePath:    "/books/book-1.epub",
	}

	if err := SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook() error = %v", err)
	}

	got, err := LoadBook(store, book.ID, nil)
	if err != nil {
		t.Fatalf("LoadBook() error = %v", err)
	}

	if got != book {
		t.Errorf("LoadBook() = %+v, want %+v", got, book)
	}
}

func TestLoadBookNotFound(t *testing.T) {
	store := setupTestDB(t)

	_, err := LoadBook(store, "does-not-exist", nil)
	if err == nil {
		t.Fatal("LoadBook() expected error for missing book")
	}
}

func TestSaveAndLoadMultipleBooks(t *testing.T) {
	store := setupTestDB(t)

	books := []Book{
		{
			ID:          "book-1",
			Title:       "Book One",
			AuthorID:    "author-1",
			SeriesID:    "series-1",
			SeriesName:  "Series One",
			SeriesOrder: 1,
			FilePath:    "/books/book-1.epub",
		},
		{
			ID:          "book-2",
			Title:       "Book Two",
			AuthorID:    "author-1",
			SeriesID:    "series-1",
			SeriesName:  "Series One",
			SeriesOrder: 2,
			FilePath:    "/books/book-2.epub",
		},
	}

	for _, book := range books {
		if err := SaveBook(store, book); err != nil {
			t.Fatalf("SaveBook(%q) error = %v", book.ID, err)
		}
	}

	for _, want := range books {
		got, err := LoadBook(store, want.ID, nil)
		if err != nil {
			t.Fatalf("LoadBook(%q) error = %v", want.ID, err)
		}

		if got != want {
			t.Errorf("LoadBook(%q) = %+v, want %+v", want.ID, got, want)
		}
	}
}

func TestLoadBookAccess(t *testing.T) {
	store := setupTestDB(t)

	unrestricted := Book{
		ID:          "book-public",
		Title:       "Public Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    "/books/public.epub",
	}

	restricted := Book{
		ID:          "book-private",
		Title:       "Private Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 2,
		FilePath:    "/books/private.epub",
	}

	if err := SaveBook(store, unrestricted); err != nil {
		t.Fatalf("SaveBook(unrestricted) error = %v", err)
	}

	if err := SaveBook(store, restricted); err != nil {
		t.Fatalf("SaveBook(restricted) error = %v", err)
	}

	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, restricted.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	// Anonymous users can access unrestricted books.
	got, err := LoadBook(store, unrestricted.ID, nil)
	if err != nil {
		t.Fatalf("anonymous LoadBook(public) error = %v", err)
	}

	if got != unrestricted {
		t.Errorf("anonymous LoadBook(public) = %+v, want %+v", got, unrestricted)
	}

	// Anonymous users cannot access explicitly restricted books.
	_, err = LoadBook(store, restricted.ID, nil)
	if err == nil {
		t.Fatal("anonymous LoadBook(private) expected error")
	}

	// An explicitly authorized user can access the restricted book.
	got, err = LoadBook(store, restricted.ID, ptr("user-1"))
	if err != nil {
		t.Fatalf("authorized LoadBook(private) error = %v", err)
	}

	if got != restricted {
		t.Errorf("authorized LoadBook(private) = %+v, want %+v", got, restricted)
	}

	// An authenticated but unauthorized user cannot access it.
	_, err = LoadBook(store, restricted.ID, ptr("user-2"))
	if err == nil {
		t.Fatal("unauthorized LoadBook(private) expected error")
	}

	// Authenticated users do not automatically get unrestricted books.
	_, err = LoadBook(store, unrestricted.ID, ptr("user-1"))
	if err == nil {
		t.Fatal("authenticated LoadBook(public) expected error")
	}
}

func TestLoadManifest(t *testing.T) {
	store := setupTestDB(t)

	books := []Book{
		{
			ID:          "series-1-book-1",
			Title:       "Series One Book One",
			AuthorID:    "author-1",
			SeriesID:    "series-1",
			SeriesName:  "Series One",
			SeriesOrder: 1,
			FilePath:    "/books/series-1-book-1.epub",
		},
		{
			ID:          "series-1-book-2",
			Title:       "Series One Book Two",
			AuthorID:    "author-1",
			SeriesID:    "series-1",
			SeriesName:  "Series One",
			SeriesOrder: 2,
			FilePath:    "/books/series-1-book-2.epub",
		},
		{
			ID:          "series-2-book-1",
			Title:       "Series Two Book One",
			AuthorID:    "author-2",
			SeriesID:    "series-2",
			SeriesName:  "Series Two",
			SeriesOrder: 1,
			FilePath:    "/books/series-2-book-1.epub",
		},
	}

	for _, book := range books {
		if err := SaveBook(store, book); err != nil {
			t.Fatalf("SaveBook(%q) error = %v", book.ID, err)
		}
	}

	manifest, err := LoadManifest(store, nil)
	if err != nil {
		t.Fatalf("LoadManifest() error = %v", err)
	}

	if len(manifest.Series) != 2 {
		t.Fatalf("LoadManifest() returned %d series, want 2", len(manifest.Series))
	}

	if manifest.Series[0].SeriesID != "series-1" {
		t.Errorf("first series ID = %q, want %q",
			manifest.Series[0].SeriesID, "series-1")
	}

	if manifest.Series[0].SeriesName != "Series One" {
		t.Errorf("first series name = %q, want %q",
			manifest.Series[0].SeriesName, "Series One")
	}

	if manifest.Series[0].FirstBookID != "series-1-book-1" {
		t.Errorf("first series first book = %q, want %q",
			manifest.Series[0].FirstBookID, "series-1-book-1")
	}

	if manifest.Series[1].SeriesID != "series-2" {
		t.Errorf("second series ID = %q, want %q",
			manifest.Series[1].SeriesID, "series-2")
	}

	if manifest.Series[1].FirstBookID != "series-2-book-1" {
		t.Errorf("second series first book = %q, want %q",
			manifest.Series[1].FirstBookID, "series-2-book-1")
	}
}

func TestLoadManifestAccess(t *testing.T) {
	store := setupTestDB(t)

	publicBook := Book{
		ID:          "public-book",
		Title:       "Public Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    "/books/public.epub",
	}

	privateBook := Book{
		ID:          "private-book",
		Title:       "Private Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 2,
		FilePath:    "/books/private.epub",
	}

	if err := SaveBook(store, publicBook); err != nil {
		t.Fatalf("SaveBook(public) error = %v", err)
	}

	if err := SaveBook(store, privateBook); err != nil {
		t.Fatalf("SaveBook(private) error = %v", err)
	}

	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, privateBook.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	// Anonymous users see the unrestricted book.
	manifest, err := LoadManifest(store, nil)
	if err != nil {
		t.Fatalf("anonymous LoadManifest() error = %v", err)
	}

	if len(manifest.Series) != 1 {
		t.Fatalf("anonymous manifest has %d series, want 1",
			len(manifest.Series))
	}

	if manifest.Series[0].FirstBookID != publicBook.ID {
		t.Errorf(
			"anonymous first book = %q, want %q",
			manifest.Series[0].FirstBookID,
			publicBook.ID,
		)
	}

	// user-1 sees only the explicitly granted book.
	manifest, err = LoadManifest(store, ptr("user-1"))
	if err != nil {
		t.Fatalf("user LoadManifest() error = %v", err)
	}

	if len(manifest.Series) != 1 {
		t.Fatalf("user manifest has %d series, want 1",
			len(manifest.Series))
	}

	if manifest.Series[0].FirstBookID != privateBook.ID {
		t.Errorf(
			"user first book = %q, want %q",
			manifest.Series[0].FirstBookID,
			privateBook.ID,
		)
	}

	// user-2 has no access, so they see no series.
	manifest, err = LoadManifest(store, ptr("user-2"))
	if err != nil {
		t.Fatalf("unauthorized LoadManifest() error = %v", err)
	}

	if len(manifest.Series) != 0 {
		t.Errorf(
			"unauthorized manifest has %d series, want 0",
			len(manifest.Series),
		)
	}
}

func TestValidateBooks(t *testing.T) {
	store := setupTestDB(t)

	tempDir := t.TempDir()

	validPath := filepath.Join(tempDir, "valid.epub")
	missingPath := filepath.Join(tempDir, "missing.epub")

	if err := os.WriteFile(validPath, []byte("test epub"), 0644); err != nil {
		t.Fatalf("failed to create test EPUB: %v", err)
	}

	validBook := Book{
		ID:          "valid-book",
		Title:       "Valid Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    validPath,
	}

	missingBook := Book{
		ID:          "missing-book",
		Title:       "Missing Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 2,
		FilePath:    missingPath,
	}

	if err := SaveBook(store, validBook); err != nil {
		t.Fatalf("SaveBook(valid) error = %v", err)
	}

	if err := SaveBook(store, missingBook); err != nil {
		t.Fatalf("SaveBook(missing) error = %v", err)
	}

	// ValidateBooks should inspect all books, regardless of user access.
	err := ValidateBooks(store)
	if err == nil {
		t.Fatal("ValidateBooks() expected error for missing EPUB")
	}
}

func TestDeleteBook(t *testing.T) {
	store := setupTestDB(t)

	tempDir := t.TempDir()
	epubPath := filepath.Join(tempDir, "book.epub")

	if err := os.WriteFile(epubPath, []byte("test epub"), 0644); err != nil {
		t.Fatalf("failed to create test EPUB: %v", err)
	}

	book := Book{
		ID:          "book-1",
		Title:       "Book One",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    epubPath,
	}

	if err := SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook() error = %v", err)
	}

	if _, err := os.Stat(epubPath); err != nil {
		t.Fatalf("test EPUB was not created: %v", err)
	}

	if err := DeleteBook(store, book.ID); err != nil {
		t.Fatalf("DeleteBook() error = %v", err)
	}

	if _, err := os.Stat(epubPath); !os.IsNotExist(err) {
		t.Errorf("EPUB still exists after DeleteBook()")
	}

	var count int
	err := store.DB.QueryRow(`
		SELECT COUNT(*)
		FROM books
		WHERE id = ?
	`, book.ID).Scan(&count)
	if err != nil {
		t.Fatalf("failed to query deleted book: %v", err)
	}

	if count != 0 {
		t.Errorf("database contains %d copies of deleted book, want 0", count)
	}
}

func TestDeleteBookMissingEPUB(t *testing.T) {
	store := setupTestDB(t)

	book := Book{
		ID:          "book-1",
		Title:       "Book One",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    "/does/not/exist/book.epub",
	}

	if err := SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook() error = %v", err)
	}

	if err := DeleteBook(store, book.ID); err != nil {
		t.Fatalf("DeleteBook() should tolerate missing EPUB: %v", err)
	}

	var count int
	err := store.DB.QueryRow(`
		SELECT COUNT(*)
		FROM books
		WHERE id = ?
	`, book.ID).Scan(&count)
	if err != nil {
		t.Fatalf("failed to query deleted book: %v", err)
	}

	if count != 0 {
		t.Errorf("database contains %d copies of deleted book, want 0", count)
	}
}

func TestDeleteSeries(t *testing.T) {
	store := setupTestDB(t)

	if err := DeleteSeries(store, "does-not-exist"); err == nil {
		t.Fatal("DeleteSeries() expected error for missing series")
	}

	book := Book{
		ID:          "book-1",
		Title:       "Book One",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    "/books/book-1.epub",
	}

	if err := SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook() error = %v", err)
	}

	if err := DeleteSeries(store, "series-1"); err == nil {
		t.Fatal("DeleteSeries() expected error for non-empty series")
	}

	if err := DeleteBook(store, book.ID); err != nil {
		t.Fatalf("DeleteBook() error = %v", err)
	}

	if err := DeleteSeries(store, "series-1"); err != nil {
		t.Fatalf("DeleteSeries() error = %v", err)
	}

	var count int
	err := store.DB.QueryRow(`
		SELECT COUNT(*)
		FROM series
		WHERE series_id = ?
	`, "series-1").Scan(&count)
	if err != nil {
		t.Fatalf("failed to query deleted series: %v", err)
	}

	if count != 0 {
		t.Errorf("database contains %d copies of deleted series, want 0", count)
	}
}

func TestUpdateSeriesName(t *testing.T) {
	store := setupTestDB(t)

	book := Book{
		ID:          "book-1",
		Title:       "Book One",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Old Name",
		SeriesOrder: 1,
		FilePath:    "/books/book-1.epub",
	}

	if err := SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook() error = %v", err)
	}

	if err := UpdateSeriesName(
		store,
		"series-1",
		"New Name",
	); err != nil {
		t.Fatalf("UpdateSeriesName() error = %v", err)
	}

	got, err := LoadBook(store, book.ID, nil)
	if err != nil {
		t.Fatalf("LoadBook() error = %v", err)
	}

	if got.SeriesName != "New Name" {
		t.Errorf(
			"SeriesName = %q, want %q",
			got.SeriesName,
			"New Name",
		)
	}
}

func TestUpdateSeriesNameMissingSeries(t *testing.T) {
	store := setupTestDB(t)

	err := UpdateSeriesName(
		store,
		"does-not-exist",
		"New Name",
	)
	if err == nil {
		t.Fatal("UpdateSeriesName() expected error for missing series")
	}
}

func TestLoadBooks(t *testing.T) {
	store := setupTestDB(t)

	books := []Book{
		{
			ID:          "book-1",
			Title:       "Book One",
			AuthorID:    "author-1",
			SeriesID:    "series-1",
			SeriesName:  "Series One",
			SeriesOrder: 1,
			FilePath:    "/books/book-1.epub",
		},
		{
			ID:          "book-2",
			Title:       "Book Two",
			AuthorID:    "author-1",
			SeriesID:    "series-1",
			SeriesName:  "Series One",
			SeriesOrder: 2,
			FilePath:    "/books/book-2.epub",
		},
		{
			ID:          "book-3",
			Title:       "Book Three",
			AuthorID:    "author-2",
			SeriesID:    "series-2",
			SeriesName:  "Series Two",
			SeriesOrder: 1,
			FilePath:    "/books/book-3.epub",
		},
	}

	for _, book := range books {
		if err := SaveBook(store, book); err != nil {
			t.Fatalf("SaveBook(%q) error = %v", book.ID, err)
		}
	}

	got, err := loadAllBookRows(store)
	if err != nil {
		t.Fatalf("LoadBooks() error = %v", err)
	}

	if len(got) != len(books) {
		t.Fatalf(
			"LoadBooks() returned %d books, want %d",
			len(got),
			len(books),
		)
	}

	for i, want := range books {
		if got[i] != want {
			t.Errorf(
				"LoadBooks()[%d] = %+v, want %+v",
				i,
				got[i],
				want,
			)
		}
	}
}

func TestCleanupBooksRemovesMissingBooks(t *testing.T) {
	store := setupTestDB(t)

	tempDir := t.TempDir()

	existingPath := filepath.Join(tempDir, "existing.epub")
	missingPath := filepath.Join(tempDir, "missing.epub")

	if err := os.WriteFile(existingPath, []byte("test epub"), 0644); err != nil {
		t.Fatalf("failed to create existing EPUB: %v", err)
	}

	existingBook := Book{
		ID:          "existing-book",
		Title:       "Existing Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    existingPath,
	}

	missingBook := Book{
		ID:          "missing-book",
		Title:       "Missing Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 2,
		FilePath:    missingPath,
	}

	if err := SaveBook(store, existingBook); err != nil {
		t.Fatalf("SaveBook(existing) error = %v", err)
	}

	if err := SaveBook(store, missingBook); err != nil {
		t.Fatalf("SaveBook(missing) error = %v", err)
	}

	if err := CleanupBooks(store); err != nil {
		t.Fatalf("CleanupBooks() error = %v", err)
	}

	if _, err := loadBookRowAuth(store, existingBook.ID); err != nil {
		t.Errorf("existing book was removed: %v", err)
	}

	if _, err := loadBookRowAuth(store, missingBook.ID); err == nil {
		t.Error("missing book was not removed")
	}
}

func TestCleanupBooksDoesNotFailForNoBooks(t *testing.T) {
	store := setupTestDB(t)

	if err := CleanupBooks(store); err != nil {
		t.Fatalf("CleanupBooks() error = %v", err)
	}
}
