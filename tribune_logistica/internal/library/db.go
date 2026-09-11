package library

import (
	"database/sql"
	"errors"
	"fmt"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

const BooksTable = "books"
const SeriesTable = "series"
const UserAccessTable = "user_access"

func loadBookRow(store *storage.SQLStorage, id string, userID *string) (Book, error) {
	var entry Book

	err := store.DB.QueryRow(
		`
		SELECT
			b.id,
			b.title,
			b.author_id,
			b.series_id,
			s.series_name,
			b.series_order,
			b.file_path
		FROM `+BooksTable+` b
		JOIN `+SeriesTable+` s
			ON b.series_id = s.series_id
		WHERE
			b.id = ?
			AND (
				(
					? IS NULL
					AND NOT EXISTS (
						SELECT 1
						FROM `+UserAccessTable+` a
						WHERE a.book_id = b.id
					)
				)
				OR (
					? IS NOT NULL
					AND EXISTS (
						SELECT 1
						FROM `+UserAccessTable+` a
						WHERE a.book_id = b.id
							AND a.user_id = ?
					)
				)
			)
		`,
		id,
		userID,
		userID,
		userID,
	).Scan(
		&entry.ID,
		&entry.Title,
		&entry.AuthorID,
		&entry.SeriesID,
		&entry.SeriesName,
		&entry.SeriesOrder,
		&entry.FilePath,
	)

	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return Book{}, fmt.Errorf("book not found: %s", id)
		}
		return Book{}, err
	}

	return entry, nil
}

func loadBookRowAuth(store *storage.SQLStorage, id string) (Book, error) {
	var entry Book

	err := store.DB.QueryRow(
		`
		SELECT
			b.id,
			b.title,
			b.author_id,
			b.series_id,
			s.series_name,
			b.series_order,
			b.file_path
		FROM `+BooksTable+` b
		JOIN `+SeriesTable+` s
			ON b.series_id = s.series_id
		WHERE
			b.id = ?
		`,
		id,
	).Scan(
		&entry.ID,
		&entry.Title,
		&entry.AuthorID,
		&entry.SeriesID,
		&entry.SeriesName,
		&entry.SeriesOrder,
		&entry.FilePath,
	)

	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return Book{}, fmt.Errorf("book not found: %s", id)
		}
		return Book{}, err
	}

	return entry, nil
}

func saveBookRow(store *storage.SQLStorage, b Book) error {
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

func saveSeriesRow(store *storage.SQLStorage, entry SeriesEntry) error {
	return store.Insert("series", "series_id", map[string]interface{}{
		"series_id":   entry.SeriesID,
		"series_name": entry.SeriesName,
	})
}

func loadSeriesRow(store *storage.SQLStorage, seriesID string, userID *string) ([]Book, error) {
	rows, err := store.DB.Query(
		`
		SELECT
			b.id,
			b.title,
			b.author_id,
			b.series_id,
			s.series_name,
			b.series_order,
			b.file_path
		FROM `+BooksTable+` b
		JOIN `+SeriesTable+` s
			ON b.series_id = s.series_id
		WHERE
			b.series_id = ?
			AND (
				(
					? IS NULL
					AND NOT EXISTS (
						SELECT 1
						FROM `+UserAccessTable+` a
						WHERE a.book_id = b.id
					)
				)
				OR EXISTS (
					SELECT 1
					FROM `+UserAccessTable+` a
					WHERE a.book_id = b.id
						AND a.user_id = ?
				)
			)
		ORDER BY b.series_order ASC
		`,
		seriesID,
		userID,
		userID,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var books []Book

	for rows.Next() {
		var book Book

		if err := rows.Scan(
			&book.ID,
			&book.Title,
			&book.AuthorID,
			&book.SeriesID,
			&book.SeriesName,
			&book.SeriesOrder,
			&book.FilePath,
		); err != nil {
			return nil, err
		}

		books = append(books, book)
	}

	if err := rows.Err(); err != nil {
		return nil, err
	}

	if len(books) == 0 {
		return nil, fmt.Errorf("series not found: %s", seriesID)
	}

	return books, nil
}

func loadManifest(store *storage.SQLStorage, userID *string) (Manifest, error) {
	rows, err := store.DB.Query(
		`
		SELECT
			s.series_id,
			s.series_name,
			b.id AS first_book_id
		FROM `+SeriesTable+` s
		JOIN `+BooksTable+` b
			ON b.id = (
				SELECT b2.id
				FROM `+BooksTable+` b2
				WHERE
					b2.series_id = s.series_id
					AND (
						(
							? IS NULL
							AND NOT EXISTS (
								SELECT 1
								FROM `+UserAccessTable+` a
								WHERE a.book_id = b2.id
							)
						)
						OR EXISTS (
							SELECT 1
							FROM `+UserAccessTable+` a
							WHERE a.book_id = b2.id
								AND a.user_id = ?
						)
					)
				ORDER BY b2.series_order ASC
				LIMIT 1
			)
		ORDER BY s.series_id
		`,
		userID,
		userID,
	)

	if err != nil {
		return Manifest{}, err
	}

	defer rows.Close()

	var manifest Manifest

	for rows.Next() {
		var entry SeriesEntry

		err := rows.Scan(
			&entry.SeriesID,
			&entry.SeriesName,
			&entry.FirstBookID,
		)
		if err != nil {
			return Manifest{}, err
		}

		manifest.Series = append(manifest.Series, entry)
	}

	if err := rows.Err(); err != nil {
		return Manifest{}, err
	}

	return manifest, nil
}

func deleteSeriesRow(store *storage.SQLStorage, id string) error {
	_, err := store.DB.Exec(
		"DELETE FROM "+SeriesTable+" WHERE series_id = ?",
		id,
	)
	return err
}

func deleteBookRow(store *storage.SQLStorage, id string) error {
	_, err := store.DB.Exec(
		"DELETE FROM "+BooksTable+" WHERE id = ?",
		id,
	)
	return err
}

func loadAllBookRows(store *storage.SQLStorage) ([]Book, error) {
	rows, err := store.DB.Query(
		`
		SELECT
			b.id,
			b.title,
			b.author_id,
			b.series_id,
			s.series_name,
			b.series_order,
			b.file_path
		FROM ` + BooksTable + ` b
		JOIN ` + SeriesTable + ` s
			ON b.series_id = s.series_id
		ORDER BY b.series_id, b.series_order ASC
		`,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var books []Book

	for rows.Next() {
		var book Book

		err := rows.Scan(
			&book.ID,
			&book.Title,
			&book.AuthorID,
			&book.SeriesID,
			&book.SeriesName,
			&book.SeriesOrder,
			&book.FilePath,
		)
		if err != nil {
			return nil, err
		}

		books = append(books, book)
	}

	if err := rows.Err(); err != nil {
		return nil, err
	}

	return books, nil
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
