package library_test

import (
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/library"
	"github.com/book_legion-tribune_logistica/internal/storage"
	_ "modernc.org/sqlite"
)

func TestSaveAndLoadBook(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	book := library.Book{
		ID:          "b1",
		Title:       "Book One",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesOrder: 1,
		FilePath:    "/tmp/fakefile1.epub",
	}

	if err := library.SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	loaded, err := library.LoadBook(store, "b1")
	if err != nil {
		t.Fatalf("LoadBook failed: %v", err)
	}

	if loaded.ID != book.ID || loaded.Title != book.Title || loaded.SeriesOrder != book.SeriesOrder {
		t.Errorf("Loaded book %+v; want %+v", loaded, book)
	}
}

func TestSaveAndChangeBook(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	book := library.Book{
		ID:          "b1",
		Title:       "Book One",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesOrder: 1,
		FilePath:    "/tmp/fakefile1.epub",
	}

	if err := library.SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	book.FilePath = "/tmp/changed.epub"
	if err = library.SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	loaded, err := library.LoadBook(store, "b1")
	if err != nil {
		t.Fatalf("LoadBook failed: %v", err)
	}

	if loaded.ID != book.ID || loaded.Title != book.Title || loaded.SeriesOrder != book.SeriesOrder || loaded.FilePath != book.FilePath {
		t.Errorf("Loaded book %+v; want %+v", loaded, book)
	}
}

func TestLoadBookNotFound(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)
	_, err := library.LoadBook(store, "nonexistent")
	if err == nil {
		t.Fatal("expected error loading nonexistent book")
	}
}

func TestValidateBooksDeletesOrphan(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	// create fake book file
	existingPath := tmpFile + ".epub"
	if f, err := os.Create(existingPath); err != nil {
		t.Fatalf("failed to create fake file: %v", err)
	} else {
		f.Close()
	}

	books := []library.Book{
		{ID: "b1", FilePath: existingPath},
		{ID: "b2", FilePath: "/tmp/nonexistent.epub"},
	}

	for _, b := range books {
		if err := library.SaveBook(store, b); err != nil {
			t.Fatalf("SaveBook failed: %v", err)
		}
	}

	if err := library.ValidateBooks(store); err != nil {
		t.Fatalf("ValidateBooks failed: %v", err)
	}

	rows, _ := store.Query("books", nil)
	if len(rows) != 1 {
		t.Fatalf("expected 1 remaining book, got %d", len(rows))
	}
	if rows[0]["id"] != "b1" {
		t.Fatalf("unexpected remaining book: %+v", rows[0])
	}

	// cleanup
	os.Remove(existingPath)
}

func TestSaveAndLoadManifest(t *testing.T) {
	tmpFile := "test_manifest.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	manifest := library.Manifest{
		Series: []library.SeriesEntry{
			{SeriesID: "s1", SeriesName: "Series 1", FirstBookID: "b1"},
			{SeriesID: "s2", SeriesName: "Series 2", FirstBookID: "b2"},
		},
	}

	if err := library.SaveManifest(store, manifest); err != nil {
		t.Fatalf("SaveManifest failed: %v", err)
	}

	loaded, err := library.LoadManifest(store)
	if err != nil {
		t.Fatalf("LoadManifest failed: %v", err)
	}

	if len(loaded.Series) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(loaded.Series))
	}

	for i, entry := range loaded.Series {
		want := manifest.Series[i]
		if entry != want {
			t.Errorf("mismatch at index %v: got %+v, want %+v", i, entry, want)
		}
	}
}

func TestSaveBookAndLoadManifest(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	book := library.Book{
		ID:          "b1",
		Title:       "Book One",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesOrder: 1,
		FilePath:    "/tmp/fakefile1.epub",
	}

	if err := library.SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	loaded, err := library.LoadManifest(store)
	if err != nil {
		t.Fatalf("LoadManifest failed: %v", err)
	}

	if len(loaded.Series) == 0 {
		t.Fatalf("No series inserted")
	}

	// Search for the series entry
	var found bool
	for _, entry := range loaded.Series {
		if entry.SeriesID == book.SeriesID {
			found = true
			if entry.FirstBookID != book.ID {
				t.Fatalf("Wrong book in series: got %v, want %v", entry.FirstBookID, book.ID)
			}
			break
		}
	}
	if !found {
		t.Fatalf("Series %v not found in manifest", book.SeriesID)
	}
}

func setupTestDB(t *testing.T) *sql.DB {
	t.Helper()

	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatalf("failed to open sqlite db: %v", err)
	}

	return db
}

func TestSaveTwoBooksAndLoadManifest(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)
	db := setupTestDB(t)
	store := storage.NewSQLStorage(db)

	book2 := library.Book{
		ID:          "b2",
		Title:       "Book 2",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesName:  "series",
		SeriesOrder: 2,
		FilePath:    "/tmp/fakefile1.epub",
	}

	if err := library.SaveBook(store, book2); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	book1 := library.Book{
		ID:          "b1",
		Title:       "Book One",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesName:  "series",
		SeriesOrder: 1,
		FilePath:    "/tmp/fakefile1.epub",
	}

	if err := library.SaveBook(store, book1); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	loaded, err := library.LoadManifest(store)
	if err != nil {
		t.Fatalf("LoadManifest failed: %v", err)
	}
	fmt.Printf("series: %v", loaded.Series)
	if len(loaded.Series) == 0 {
		t.Fatalf("No series inserted")
	}

	// Find the series entry
	var entry library.SeriesEntry
	var found bool
	for _, e := range loaded.Series {
		if e.SeriesID == book1.SeriesID {
			entry = e
			found = true
			break
		}
	}
	if !found {
		t.Fatalf("Series %v not found in manifest", book1.SeriesID)
	}

	// FirstBookID should be the one with lowest SeriesOrder
	if entry.FirstBookID != book1.ID {
		t.Fatalf("Wrong first book in series: got %v, want %v", entry.FirstBookID, book1.ID)
	}
}

func TestLoadBooks(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	// Seed some books
	books := []library.Book{
		{ID: "b1", Title: "Book 1", AuthorID: "a1", SeriesID: "s1", SeriesName: "Series One", SeriesOrder: 1, FilePath: "/tmp/b1.epub"},
		{ID: "b2", Title: "Book 2", AuthorID: "a1", SeriesID: "s1", SeriesName: "Series One", SeriesOrder: 2, FilePath: "/tmp/b2.epub"},
		{ID: "b3", Title: "Book 3", AuthorID: "a2", SeriesID: "s2", SeriesName: "Series Two", SeriesOrder: 1, FilePath: "/tmp/b3.epub"},
	}

	for _, b := range books {
		if err := library.SaveBook(store, b); err != nil {
			t.Fatalf("SaveBook failed: %v", err)
		}
	}

	tests := []struct {
		name        string
		seriesID    string
		wantIDs     []string
		expectError bool
	}{
		{"Load series s1", "s1", []string{"b1", "b2"}, false},
		{"Load series s2", "s2", []string{"b3"}, false},
		{"Load nonexistent series", "s3", []string{}, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := library.LoadBooks(store, tt.seriesID)
			if (err != nil) != tt.expectError {
				t.Fatalf("LoadBooks() error = %v, expectError %v", err, tt.expectError)
			}

			if len(got) != len(tt.wantIDs) {
				t.Fatalf("expected %d books, got %d", len(tt.wantIDs), len(got))
			}

			gotIDs := make([]string, len(got))
			for i, b := range got {
				gotIDs[i] = b.ID
			}

			for i, id := range tt.wantIDs {
				if gotIDs[i] != id {
					t.Errorf("book %d: got %v, want %v", i, gotIDs[i], id)
				}
			}
		})
	}
}

func TestGetAbsolutePath(t *testing.T) {
	tests := []struct {
		name        string
		libraryRoot string
		stored      string
		expected    string
	}{
		{
			name:        "absolute path unchanged",
			libraryRoot: "/library",
			stored:      "/other/place/book.epub",
			expected:    "/other/place/book.epub",
		},
		{
			name:        "relative path joined",
			libraryRoot: "/library",
			stored:      "Author/Book.epub",
			expected:    filepath.Join("/library", "Author/Book.epub"),
		},
		{
			name:        "nested relative path joined",
			libraryRoot: "/library",
			stored:      "Author/Series/Book.epub",
			expected:    filepath.Join("/library", "Author/Series/Book.epub"),
		},
		{
			name:        "empty library root",
			libraryRoot: "",
			stored:      "Author/Book.epub",
			expected:    filepath.Join("", "Author/Book.epub"),
		},
		{
			name:        "empty stored path",
			libraryRoot: "/library",
			stored:      "",
			expected:    filepath.Join("/library", ""),
		},
		{
			name:        "library root with trailing slash",
			libraryRoot: "/library/",
			stored:      "Author/Book.epub",
			expected:    filepath.Join("/library/", "Author/Book.epub"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// isolate environment per test
			old := os.Getenv("LIBRARY_ROOT")
			defer os.Setenv("LIBRARY_ROOT", old)

			err := os.Setenv("LIBRARY_ROOT", tt.libraryRoot)
			if err != nil {
				t.Fatalf("failed to set env: %v", err)
			}

			result := library.GetAbsolutePath(tt.stored)

			if result != tt.expected {
				t.Errorf("expected %q, got %q", tt.expected, result)
			}
		})
	}
}

func TestDeleteSeries_Success(t *testing.T) {
	tmpFile := "test_manifest_delete.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	manifest := library.Manifest{
		Series: []library.SeriesEntry{
			{SeriesID: "s1", SeriesName: "Series 1", FirstBookID: "b1"},
			{SeriesID: "s2", SeriesName: "Series 2", FirstBookID: "b2"},
		},
	}

	book := library.Book{
		ID:          "b1",
		SeriesID:    "s1",
		SeriesName:  "Series 1",
		SeriesOrder: 1,
		FilePath:    "file.epub",
	}
	if err := library.SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	if err := library.SaveManifest(store, manifest); err != nil {
		t.Fatalf("SaveManifest failed: %v", err)
	}

	// delete series s1 (no books exist referencing it)
	if err := library.DeleteSeries(store, "s2"); err != nil {
		t.Fatalf("DeleteSeries failed: %v", err)
	}

	loaded, err := library.LoadManifest(store)
	t.Logf("manifest: %+v", loaded)
	if err != nil {
		t.Fatalf("LoadManifest failed: %v", err)
	}
	if len(loaded.Series) != 1 {
		t.Fatalf("expected 1 entry after delete, got %d", len(loaded.Series))
	}

	if loaded.Series[0].SeriesID != "s1" {
		t.Fatalf("expected remaining series to be s1, got %s", loaded.Series[0].SeriesID)
	}
}

func TestDeleteSeriesNoBooks(t *testing.T) {
	tmpFile := "test_manifest_delete.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	manifest := library.Manifest{
		Series: []library.SeriesEntry{
			{SeriesID: "s1", SeriesName: "Series 1", FirstBookID: "b1"},
			{SeriesID: "s2", SeriesName: "Series 2", FirstBookID: "b2"},
		},
	}

	if err := library.SaveManifest(store, manifest); err != nil {
		t.Fatalf("SaveManifest failed: %v", err)
	}

	// delete series s1 (no books exist referencing it)
	if err := library.DeleteSeries(store, "s2"); err != nil {
		t.Fatalf("DeleteSeries failed: %v", err)
	}

	loaded, err := library.LoadManifest(store)
	t.Logf("manifest: %+v", loaded)
	if err != nil {
		t.Fatalf("LoadManifest failed: %v", err)
	}
	if len(loaded.Series) != 1 {
		t.Fatalf("expected 1 entry after delete, got %d", len(loaded.Series))
	}

	if loaded.Series[0].SeriesID != "s1" {
		t.Fatalf("expected remaining series to be s1, got %s", loaded.Series[0].SeriesID)
	}
}

func TestDeleteSeries_RefuseWhenBooksExist(t *testing.T) {
	tmpFile := "test_manifest_delete_books.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	// create a book referencing series
	book := library.Book{
		ID:          "b1",
		SeriesID:    "s1",
		SeriesName:  "Series 1",
		SeriesOrder: 1,
		FilePath:    "file.epub",
	}

	if err := library.SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	// attempt delete (should fail)
	if err := library.DeleteSeries(store, "s1"); err == nil {
		t.Fatalf("expected error deleting non-empty series")
	}
}

func TestDeleteSeries_NotInManifest(t *testing.T) {
	tmpFile := "test_manifest_delete_missing.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	manifest := library.Manifest{Series: []library.SeriesEntry{}}
	if err := library.SaveManifest(store, manifest); err != nil {
		t.Fatalf("SaveManifest failed: %v", err)
	}

	// delete non-existing series
	if err := library.DeleteSeries(store, "nope"); err == nil {
		t.Fatalf("expected error when series not in manifest")
	}
}

func TestDeleteBook(t *testing.T) {
	tmpFile := "test_delete_book.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	book := library.Book{
		ID:          "b1",
		SeriesID:    "s1",
		SeriesName:  "Series 1",
		SeriesOrder: 1,
		FilePath:    "file.epub",
	}

	if err := library.SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	// delete it
	if err := library.DeleteBook(store, "b1"); err != nil {
		t.Fatalf("DeleteBook failed: %v", err)
	}

	// ensure it’s gone
	rows, err := store.Query("books", map[string]interface{}{
		"id": "b1",
	})
	if err != nil {
		t.Fatalf("query failed: %v", err)
	}
	if len(rows) != 0 {
		t.Fatalf("expected book to be deleted")
	}
}

func TestUpdateSeriesName(t *testing.T) {
	tmpFile := "test_manifest_update_series_name.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)

	// Seed manifest with two series
	manifest := library.Manifest{
		Series: []library.SeriesEntry{
			{SeriesID: "s1", SeriesName: "Old Name", FirstBookID: "b1"},
			{SeriesID: "s2", SeriesName: "Other Series", FirstBookID: "b2"},
		},
	}
	if err := library.SaveManifest(store, manifest); err != nil {
		t.Fatalf("SaveManifest failed: %v", err)
	}

	// Add two books belonging to s1
	b1 := library.Book{ID: "b1", Title: "Book One", AuthorID: "a1", SeriesID: "s1", SeriesOrder: 1, FilePath: "/tmp/b1.epub"}
	b2 := library.Book{ID: "b2", Title: "Book Two", AuthorID: "a1", SeriesID: "s1", SeriesOrder: 2, FilePath: "/tmp/b2.epub"}
	if err := library.SaveBook(store, b1); err != nil {
		t.Fatalf("SaveBook b1 failed: %v", err)
	}
	if err := library.SaveBook(store, b2); err != nil {
		t.Fatalf("SaveBook b2 failed: %v", err)
	}

	// Rename s1
	if err := library.UpdateSeriesName(store, "s1", "New Name"); err != nil {
		t.Fatalf("UpdateSeriesName failed: %v", err)
	}

	// Verify manifest entry was updated
	loaded, err := library.LoadManifest(store)
	if err != nil {
		t.Fatalf("LoadManifest failed: %v", err)
	}
	var found *library.SeriesEntry
	for i, e := range loaded.Series {
		if e.SeriesID == "s1" {
			found = &loaded.Series[i]
			break
		}
	}
	if found == nil {
		t.Fatalf("series s1 not found in manifest after update")
	}
	if found.SeriesName != "New Name" {
		t.Fatalf("expected series name 'New Name', got %q", found.SeriesName)
	}

	// Verify s2 was untouched
	for _, e := range loaded.Series {
		if e.SeriesID == "s2" && e.SeriesName != "Other Series" {
			t.Fatalf("s2 series name was unexpectedly changed to %q", e.SeriesName)
		}
	}

	// Verify both books reflect the new series name when loaded
	for _, id := range []string{"b1", "b2"} {
		book, err := library.LoadBook(store, id)
		if err != nil {
			t.Fatalf("LoadBook %s failed: %v", id, err)
		}
		if book.SeriesName != "New Name" {
			t.Fatalf("book %s: expected SeriesName 'New Name', got %q", id, book.SeriesName)
		}
		t.Logf("book %s SeriesName = %q ✓", id, book.SeriesName)
	}
}
