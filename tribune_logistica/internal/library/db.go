package library

import (
	"fmt"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

func loadBookRow(store storage.Storage, id string) (Book, error) {
	rows, err := store.Query("books", map[string]interface{}{"id": id})
	if err != nil {
		return Book{}, err
	}
	if len(rows) == 0 {
		return Book{}, fmt.Errorf("book not found: %s", id)
	}
	return rowToBook(rows[0])
}

func saveBookRow(store storage.Storage, b Book) error {
	return store.Insert("books", "id", map[string]interface{}{
		"id":           b.ID,
		"title":        b.Title,
		"author_id":    b.AuthorID,
		"series_id":    b.SeriesID,
		"series_order": b.SeriesOrder,
		"file_path":    b.FilePath,
		// series_name removed
	})
}

func rowToBook(row map[string]interface{}) (Book, error) {
	seriesOrder, err := asInt(row["series_order"])
	if err != nil {
		return Book{}, fmt.Errorf("invalid series_order for book %v: %w", row["id"], err)
	}
	return Book{
		ID:          row["id"].(string),
		Title:       row["title"].(string),
		AuthorID:    row["author_id"].(string),
		SeriesID:    row["series_id"].(string),
		SeriesOrder: seriesOrder,
		FilePath:    GetAbsolutePath(row["file_path"].(string)),
		// SeriesName left empty, filled by enrichBookWithSeriesName
	}, nil
}

func enrichBookWithSeriesName(store storage.Storage, b Book) (Book, error) {
	entries, err := loadAllSeriesRows(store)
	if err != nil {
		return b, nil // non-fatal, just return book without name
	}
	for _, entry := range entries {
		if entry.SeriesID == b.SeriesID {
			b.SeriesName = entry.SeriesName
			return b, nil
		}
	}
	return b, nil // series not found, still non-fatal
}

func enrichBooksWithSeriesNames(store storage.Storage, books []Book) ([]Book, error) {
	entries, err := loadAllSeriesRows(store)
	if err != nil {
		return books, nil
	}
	nameByID := make(map[string]string, len(entries))
	for _, entry := range entries {
		nameByID[entry.SeriesID] = entry.SeriesName
	}
	for i := range books {
		books[i].SeriesName = nameByID[books[i].SeriesID]
	}
	return books, nil
}

func saveSeriesRow(store storage.Storage, entry SeriesEntry) error {
	return store.Insert("manifest", "series_id", map[string]interface{}{
		"series_id":     entry.SeriesID,
		"series_name":   entry.SeriesName,
		"first_book_id": entry.FirstBookID,
	})
}

func loadAllSeriesRows(store storage.Storage) ([]SeriesEntry, error) {
	rows, err := store.Query("manifest", nil)
	if err != nil {
		return []SeriesEntry{}, nil
	}
	entries := make([]SeriesEntry, 0, len(rows))
	for _, row := range rows {
		entries = append(entries, SeriesEntry{
			SeriesID:    row["series_id"].(string),
			SeriesName:  row["series_name"].(string),
			FirstBookID: row["first_book_id"].(string),
		})
	}
	return entries, nil
}

func deleteSeriesRow(store storage.Storage, id string) error {
	return store.Delete("manifest", map[string]interface{}{"series_id": id})
}

func loadAllBookRows(store storage.Storage, filter map[string]interface{}) ([]Book, error) {
	rows, err := store.Query("books", filter)
	if err != nil {
		return nil, fmt.Errorf("failed to query books: %w", err)
	}
	books := make([]Book, 0, len(rows))
	for _, row := range rows {
		b, err := rowToBook(row)
		if err != nil {
			return nil, err
		}
		books = append(books, b)
	}
	return books, nil
}

func deleteBookRow(store storage.Storage, id string) error {
	return store.Delete("books", map[string]interface{}{"id": id})
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
