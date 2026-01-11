package library

import (
	"fmt"
	"os"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

type Book struct {
	ID          string `json:"id"`
	Title       string `json:"title"`
	AuthorID    string `json:"author_id"`
	SeriesID    string `json:"series_id"`
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
		"file_path":    b.FilePath,
	}
	return store.Insert("books", row)
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

	return Book{
		ID:          row["id"].(string),
		Title:       row["title"].(string),
		AuthorID:    row["author_id"].(string),
		SeriesID:    row["series_id"].(string),
		SeriesOrder: seriesOrder,
		FilePath:    row["file_path"].(string),
	}, nil
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

type ManifestEntry struct {
	SeriesID    string `json:"series_id"`
	FirstBookID string `json:"first_book_id"`
}
type Manifest struct {
	Series []ManifestEntry `json:"series"`
}

func SaveManifest(store storage.Storage, m Manifest) error {
	for _, entry := range m.Series {
		row := map[string]interface{}{
			"series_id":     entry.SeriesID,
			"first_book_id": entry.FirstBookID,
		}
		if err := store.Insert("manifest", row); err != nil {
			return err
		}
	}
	return nil
}

func LoadManifest(store storage.Storage) (Manifest, error) {
	rows, err := store.Query("manifest", nil)
	if err != nil {
		return Manifest{}, err
	}

	entries := make([]ManifestEntry, 0, len(rows))
	for _, row := range rows {
		entries = append(entries, ManifestEntry{
			SeriesID:    row["series_id"].(string),
			FirstBookID: row["first_book_id"].(string),
		})
	}

	return Manifest{Series: entries}, nil
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
