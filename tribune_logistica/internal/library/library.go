package library

import (
	"fmt"
	"os"
	"path/filepath"

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

func SaveBook(store *storage.SQLStorage, b Book) error {
	series := SeriesEntry{
		SeriesID:    b.SeriesID,
		SeriesName:  b.SeriesName,
		FirstBookID: b.ID,
	}
	err := saveSeriesRow(store, series)
	if err != nil {
		return fmt.Errorf("failed to save series row: %w", err)
	}
	return saveBookRow(store, b)
}

func LoadBook(store *storage.SQLStorage, id string, userID *string) (Book, error) {
	return loadBookRow(store, id, userID)
}

func DeleteBook(store *storage.SQLStorage, bookID string) error {
	if bookID == "" {
		return fmt.Errorf("bookID is required")
	}
	book, err := loadBookRowAuth(store, bookID)
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

func LoadManifest(store *storage.SQLStorage, userID *string) (Manifest, error) {
	return loadManifest(store, userID)
}

func ValidateBooks(store *storage.SQLStorage) error {
	books, err := loadAllBookRows(store)
	if err != nil {
		return err
	}
	for _, book := range books {
		if _, err := os.Stat(book.FilePath); err != nil {
			if os.IsNotExist(err) {
				return fmt.Errorf("EPUB file missing for book %s: %s", book.ID, book.FilePath)
			}
			return fmt.Errorf("failed to stat file for book %s: %w", book.ID, err)
		}
	}
	return nil
}

func CleanupBooks(store *storage.SQLStorage) error {
	books, err := loadAllBookRows(store)
	if err != nil {
		return err
	}

	for _, book := range books {
		if _, err := os.Stat(book.FilePath); err != nil {
			if os.IsNotExist(err) {
				if err := deleteBookRow(store, book.ID); err != nil {
					return fmt.Errorf("failed to delete missing book %s: %w", book.ID, err)
				}
				continue
			}

			return fmt.Errorf("failed to stat file for book %s: %w", book.ID, err)
		}
	}

	return nil
}

func DeleteSeries(store *storage.SQLStorage, seriesID string) error {
	if seriesID == "" {
		return fmt.Errorf("seriesID is required")
	}

	var exists bool
	err := store.DB.QueryRow(
		`
		SELECT EXISTS (
			SELECT 1
			FROM `+SeriesTable+`
			WHERE series_id = ?
		)
		`,
		seriesID,
	).Scan(&exists)
	if err != nil {
		return err
	}

	if !exists {
		return fmt.Errorf("series %s not found", seriesID)
	}

	var bookCount int
	err = store.DB.QueryRow(
		`
		SELECT COUNT(*)
		FROM `+BooksTable+`
		WHERE series_id = ?
		`,
		seriesID,
	).Scan(&bookCount)
	if err != nil {
		return fmt.Errorf("failed to check books for series %s: %w", seriesID, err)
	}

	if bookCount > 0 {
		return fmt.Errorf("cannot delete series %s: series is not empty", seriesID)
	}

	_, err = store.DB.Exec(
		`
		DELETE FROM `+SeriesTable+`
		WHERE series_id = ?
		`,
		seriesID,
	)
	return err
}

func UpdateSeriesName(store *storage.SQLStorage, seriesID string, newName string) error {
	if seriesID == "" {
		return fmt.Errorf("seriesID is required")
	}

	result, err := store.DB.Exec(
		`
		UPDATE `+SeriesTable+`
		SET series_name = ?
		WHERE series_id = ?
		`,
		newName,
		seriesID,
	)
	if err != nil {
		return err
	}

	rowsAffected, err := result.RowsAffected()
	if err != nil {
		return err
	}

	if rowsAffected == 0 {
		return fmt.Errorf("series %s not found", seriesID)
	}

	return nil
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
