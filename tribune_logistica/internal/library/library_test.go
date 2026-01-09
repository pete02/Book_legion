package library

import (
	"os"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

func TestSaveAndLoadBook(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	book := Book{
		ID:          "b1",
		Title:       "Book One",
		AuthorID:    "a1",
		SeriesID:    "s1",
		SeriesOrder: 1,
		FilePath:    "/tmp/fakefile1.epub",
	}

	if err := SaveBook(store, book); err != nil {
		t.Fatalf("SaveBook failed: %v", err)
	}

	loaded, err := LoadBook(store, "b1")
	if err != nil {
		t.Fatalf("LoadBook failed: %v", err)
	}

	if loaded.ID != book.ID || loaded.Title != book.Title || loaded.SeriesOrder != book.SeriesOrder {
		t.Errorf("Loaded book %+v; want %+v", loaded, book)
	}
}

func TestLoadBookNotFound(t *testing.T) {
	tmpFile := "test_books.json"
	defer os.Remove(tmpFile)

	store, _ := storage.NewJSONStorage(tmpFile)
	_, err := LoadBook(store, "nonexistent")
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

	books := []Book{
		{ID: "b1", FilePath: existingPath},
		{ID: "b2", FilePath: "/tmp/nonexistent.epub"},
	}

	for _, b := range books {
		if err := SaveBook(store, b); err != nil {
			t.Fatalf("SaveBook failed: %v", err)
		}
	}

	if err := ValidateBooks(store); err != nil {
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

	manifest := Manifest{
		Series: []ManifestEntry{
			{SeriesID: "s1", FirstBookID: "b1"},
			{SeriesID: "s2", FirstBookID: "b2"},
		},
	}

	if err := SaveManifest(store, manifest); err != nil {
		t.Fatalf("SaveManifest failed: %v", err)
	}

	loaded, err := LoadManifest(store)
	if err != nil {
		t.Fatalf("LoadManifest failed: %v", err)
	}

	if len(loaded.Series) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(loaded.Series))
	}

	for i, entry := range loaded.Series {
		if entry != manifest.Series[i] {
			t.Errorf("entry %d mismatch: got %+v, want %+v", i, entry, manifest.Series[i])
		}
	}
}
