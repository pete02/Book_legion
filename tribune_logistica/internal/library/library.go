package library

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

type Book struct {
	ID          string `json:"id"`
	Title       string `json:"title"`
	AuthorID    string `json:"author_id"`
	SeriesID    string `json:"series_id"`
	SeriesName  string `json:"series_name"`
	SeriesOrder int    `json:"series_order"`
	FilePath    string `json:"file_path"`
}

type SeriesEntry struct {
	SeriesID    string `json:"series_id"`
	SeriesName  string `json:"series_name,omitempty"`
	FirstBookID string `json:"first_book_id"`
}

type Manifest struct {
	Series []SeriesEntry `json:"series"`
}

func SaveBook(store storage.Storage, b Book) error {
	if err := AddBookToManifest(store, b); err != nil {
		return err
	}
	return saveBookRow(store, b)
}

func LoadBook(store storage.Storage, id string) (Book, error) {
	book, err := loadBookRow(store, id)
	if err != nil {
		return Book{}, err
	}
	return enrichBookWithSeriesName(store, book)
}

func LoadBooks(store storage.Storage, seriesID string) ([]Book, error) {
	books, err := loadAllBookRows(store, map[string]interface{}{"series_id": seriesID})
	if err != nil {
		return nil, err
	}
	return enrichBooksWithSeriesNames(store, books)
}
func DeleteBook(store storage.Storage, bookID string) error {
	if bookID == "" {
		return fmt.Errorf("bookID is required")
	}
	book, err := loadBookRow(store, bookID)
	if err != nil {
		return fmt.Errorf("failed to load book %s before deletion: %w", bookID, err)
	}
	if book.FilePath != "" {
		if err := os.Remove(book.FilePath); err != nil && !os.IsNotExist(err) {
			return fmt.Errorf("failed to delete epub for book %s: %w", bookID, err)
		}
	}
	return deleteBookRow(store, bookID)
}

func LoadManifest(store storage.Storage) (Manifest, error) {
	entries, err := loadAllSeriesRows(store)
	if err != nil {
		return Manifest{Series: []SeriesEntry{}}, nil
	}
	return Manifest{Series: entries}, nil
}

func SaveManifest(store storage.Storage, m Manifest) error {
	existing, _ := loadAllSeriesRows(store)
	for _, entry := range existing {
		_ = deleteSeriesRow(store, entry.SeriesID)
	}
	seen := make(map[string]bool)
	for _, entry := range m.Series {
		if seen[entry.SeriesID] {
			continue
		}
		seen[entry.SeriesID] = true
		if err := saveSeriesRow(store, entry); err != nil {
			return err
		}
	}
	return nil
}

func ValidateBooks(store storage.Storage) error {
	books, err := loadAllBookRows(store, nil)
	if err != nil {
		return err
	}
	for _, book := range books {
		if _, err := os.Stat(book.FilePath); err != nil {
			if os.IsNotExist(err) {
				_ = deleteBookRow(store, book.ID)
				continue
			}
			return fmt.Errorf("failed to stat file for book %s: %w", book.ID, err)
		}
	}
	return nil
}

func DeleteSeries(store storage.Storage, seriesID string) error {
	books, err := loadAllBookRows(store, map[string]interface{}{"series_id": seriesID})
	if err != nil && !strings.Contains(err.Error(), "table not found") {
		return fmt.Errorf("failed to query books for series %s: %w", seriesID, err)
	}
	if len(books) > 0 {
		return fmt.Errorf("cannot delete series %s: series is not empty", seriesID)
	}
	manifest, err := LoadManifest(store)
	if err != nil {
		return fmt.Errorf("failed to load manifest: %w", err)
	}
	filtered := manifest.Series[:0]
	removed := false
	for _, entry := range manifest.Series {
		if entry.SeriesID == seriesID {
			removed = true
			continue
		}
		filtered = append(filtered, entry)
	}
	if !removed {
		return fmt.Errorf("series %s not found in manifest", seriesID)
	}
	manifest.Series = filtered
	return SaveManifest(store, manifest)
}

func UpdateSeriesName(store storage.Storage, seriesID string, newName string) error {
	if seriesID == "" {
		return fmt.Errorf("seriesID is required")
	}

	manifest, err := LoadManifest(store)
	if err != nil {
		return fmt.Errorf("failed to load manifest: %w", err)
	}

	for i, entry := range manifest.Series {
		if entry.SeriesID == seriesID {
			manifest.Series[i].SeriesName = newName
			return SaveManifest(store, manifest)
		}
	}

	return fmt.Errorf("series %s not found in manifest", seriesID)
}

func AddBookToManifest(store storage.Storage, book Book) error {
	fmt.Printf("saving: %v\n", book)
	manifest, err := LoadManifest(store)
	if err != nil {
		return fmt.Errorf("failed to load manifest: %w", err)
	}

	// Look for existing series entry
	found := false
	index := 0

	for i, entry := range manifest.Series {
		fmt.Printf("got: %v\n", entry.SeriesID)
		if entry.SeriesID == book.SeriesID {
			fmt.Printf("pk\n")
			index = i
			found = true
			break
		}
	}

	// If series not found, add a new entry
	if !found {
		newEntry := SeriesEntry{
			SeriesID:    book.SeriesID,
			SeriesName:  book.SeriesName, // optional, you can fill if available
			FirstBookID: book.ID,
		}
		manifest.Series = append(manifest.Series, newEntry)
	} else {
		bookID := manifest.Series[index].FirstBookID
		firstBook, err := LoadBook(store, bookID)
		if err != nil {
			// First book no longer exists or hasn't been saved yet — claim the spot
			manifest.Series[index].FirstBookID = book.ID
		} else if book.SeriesOrder < firstBook.SeriesOrder {
			manifest.Series[index].FirstBookID = book.ID
		}
	}

	return SaveManifest(store, manifest)
}

func GetAbsolutePath(stored string) string {
	libraryRoot := os.Getenv("LIBRARY_ROOT")

	var resolved string
	if filepath.IsAbs(stored) {
		resolved = stored
	} else {
		resolved = filepath.Join(libraryRoot, stored)
	}

	return resolved
}
