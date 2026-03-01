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

func SaveBook(store storage.Storage, b Book) error {
	row := map[string]interface{}{
		"id":           b.ID,
		"title":        b.Title,
		"author_id":    b.AuthorID,
		"series_id":    b.SeriesID,
		"series_order": b.SeriesOrder,
		"series_name":  b.SeriesName,
		"file_path":    b.FilePath,
	}
	err := AddBookToManifest(store, b)
	if err != nil {
		return err
	}

	return store.Insert("books", "id", row)
}

func LoadBook(store storage.Storage, id string) (Book, error) {
	rows, err := store.Query("books", map[string]interface{}{
		"id": id,
	})
	if err != nil {
		return Book{}, err
	}
	if len(rows) == 0 {
		return Book{}, fmt.Errorf("book not found: %s", id)
	}

	row := rows[0]

	seriesOrder, err := asInt(row["series_order"])
	if err != nil {
		return Book{}, fmt.Errorf("invalid series_order for book %s: %w", id, err)
	}

	resolved := GetAbsolutePath(row["file_path"].(string))

	return Book{
		ID:          row["id"].(string),
		Title:       row["title"].(string),
		AuthorID:    row["author_id"].(string),
		SeriesID:    row["series_id"].(string),
		SeriesName:  row["series_name"].(string),
		SeriesOrder: seriesOrder,
		FilePath:    resolved,
	}, nil
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

func LoadBooks(store storage.Storage, seriesID string) ([]Book, error) {
	rows, err := store.Query("books", map[string]interface{}{
		"series_id": seriesID,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to query books: %w", err)
	}

	books := make([]Book, 0, len(rows))
	for _, row := range rows {
		seriesOrder, err := asInt(row["series_order"])
		if err != nil {
			return nil, fmt.Errorf("invalid series_order for book %v: %w", row["id"], err)
		}

		resolved := GetAbsolutePath(row["file_path"].(string))

		book := Book{
			ID:          row["id"].(string),
			Title:       row["title"].(string),
			AuthorID:    row["author_id"].(string),
			SeriesID:    row["series_id"].(string),
			SeriesName:  row["series_name"].(string),
			SeriesOrder: seriesOrder,
			FilePath:    resolved,
		}
		books = append(books, book)
	}

	return books, nil
}

func ValidateBooks(store storage.Storage) error {
	rows, err := store.Query("books", nil)
	if err != nil {
		return err
	}

	for _, row := range rows {
		bookID, ok := row["id"].(string)
		if !ok {
			return fmt.Errorf("invalid book id type: %T", row["id"])
		}

		filePath, ok := row["file_path"].(string)
		if !ok {
			return fmt.Errorf("invalid file_path type for book %s", bookID)
		}

		if _, err := os.Stat(filePath); err != nil {
			if os.IsNotExist(err) {
				// orphaned DB entry → delete
				if err := store.Delete("books", map[string]interface{}{
					"id": bookID,
				}); err != nil {
					return fmt.Errorf("failed to delete orphaned book %s: %w", bookID, err)
				}
				continue
			}
			// real filesystem error
			return fmt.Errorf("failed to stat file for book %s: %w", bookID, err)
		}
	}

	return nil
}

type SeriesEntry struct {
	SeriesID    string `json:"series_id"`
	SeriesName  string `json:"series_name,omitempty"`
	FirstBookID string `json:"first_book_id"`
}

type Manifest struct {
	Series []SeriesEntry `json:"series"`
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
			return fmt.Errorf("failed to load other book: %w", err)
		}

		if book.SeriesOrder < firstBook.SeriesOrder {
			manifest.Series[index].FirstBookID = book.ID
		}
	}

	return SaveManifest(store, manifest)
}

func DeleteBook(store storage.Storage, bookID string) error {
	if bookID == "" {
		return fmt.Errorf("bookID is required")
	}

	if err := store.Delete("books", map[string]interface{}{
		"id": bookID,
	}); err != nil {
		return fmt.Errorf("failed to delete book %s: %w", bookID, err)
	}

	return nil
}

func SaveManifest(store storage.Storage, m Manifest) error {
	// delete existing manifest rows
	rows, err := store.Query("manifest", nil)
	if err == nil {
		for _, row := range rows {
			if id, ok := row["series_id"].(string); ok {
				_ = store.Delete("manifest", map[string]interface{}{
					"series_id": id,
				})
			}
		}
	}

	seen := make(map[string]bool)
	for _, entry := range m.Series {
		if seen[entry.SeriesID] {
			continue
		}
		seen[entry.SeriesID] = true

		row := map[string]interface{}{
			"series_id":     entry.SeriesID,
			"series_name":   entry.SeriesName,
			"first_book_id": entry.FirstBookID,
		}
		if err := store.Insert("manifest", "series_id", row); err != nil {
			return err
		}
	}
	return nil
}

func DeleteSeries(store storage.Storage, seriesID string) error {
	// First, check if any books still reference this series
	rows, err := store.Query("books", map[string]interface{}{
		"series_id": seriesID,
	})
	if err != nil {
		// if storage backend doesn’t have the table yet, treat as empty
		if strings.Contains(err.Error(), "table not found") {
			rows = nil
		} else {
			return fmt.Errorf("failed to query books for series %s: %w", seriesID, err)
		}
	}
	if len(rows) > 0 {
		return fmt.Errorf("cannot delete series %s: series is not empty", seriesID)
	}

	// Load manifest
	manifest, err := LoadManifest(store)
	if err != nil {
		return fmt.Errorf("failed to load manifest: %w", err)
	}

	// Filter out the series entry
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

	// Save manifest (this will reinsert remaining entries)
	return SaveManifest(store, manifest)
}

func LoadManifest(store storage.Storage) (Manifest, error) {
	rows, err := store.Query("manifest", nil)
	if err != nil {
		return Manifest{Series: []SeriesEntry{}}, nil
	}

	seriesEntries := make([]SeriesEntry, 0, len(rows))
	for _, row := range rows {
		seriesID, _ := row["series_id"].(string)
		seriesName, _ := row["series_name"].(string)
		firstBookID, _ := row["first_book_id"].(string)

		seriesEntries = append(seriesEntries, SeriesEntry{
			SeriesID:    seriesID,
			SeriesName:  seriesName,
			FirstBookID: firstBookID,
		})
	}

	return Manifest{Series: seriesEntries}, nil
}

func asInt(v interface{}) (int, error) {
	switch t := v.(type) {
	case int:
		return t, nil
	case int32:
		return int(t), nil
	case int64:
		return int(t), nil
	case float64:
		return int(t), nil
	default:
		return 0, fmt.Errorf("cannot convert %T to int", v)
	}
}
