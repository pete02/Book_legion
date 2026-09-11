package library

import (
	"testing"

	_ "modernc.org/sqlite"
)

func stringPtr(s string) *string {
	return &s
}

func testBook() Book {
	return Book{
		ID:          "book-1",
		Title:       "The First Book",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "The Series",
		SeriesOrder: 1,
		FilePath:    "/books/book-1.epub",
	}
}

func testSeries() SeriesEntry {
	return SeriesEntry{
		SeriesID:   "series-1",
		SeriesName: "The Series",
	}
}

// -----------------------------------------------------------------------------
// saveBookRow
// -----------------------------------------------------------------------------

func TestSaveBookRow(t *testing.T) {
	store := setupTestDB(t)
	book := testBook()

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	var (
		id          string
		title       string
		authorID    string
		seriesID    string
		seriesOrder int
		filePath    string
	)

	err := store.DB.QueryRow(`
		SELECT id, title, author_id, series_id, series_order, file_path
		FROM books
		WHERE id = ?
	`, book.ID).Scan(
		&id,
		&title,
		&authorID,
		&seriesID,
		&seriesOrder,
		&filePath,
	)
	if err != nil {
		t.Fatalf("failed to read saved book: %v", err)
	}

	if id != book.ID {
		t.Errorf("id = %q, want %q", id, book.ID)
	}
	if title != book.Title {
		t.Errorf("title = %q, want %q", title, book.Title)
	}
	if authorID != book.AuthorID {
		t.Errorf("authorID = %q, want %q", authorID, book.AuthorID)
	}
	if seriesID != book.SeriesID {
		t.Errorf("seriesID = %q, want %q", seriesID, book.SeriesID)
	}
	if seriesOrder != book.SeriesOrder {
		t.Errorf("seriesOrder = %d, want %d", seriesOrder, book.SeriesOrder)
	}
	if filePath != book.FilePath {
		t.Errorf("filePath = %q, want %q", filePath, book.FilePath)
	}
}

// -----------------------------------------------------------------------------
// loadBookRowAuth
// -----------------------------------------------------------------------------

func TestLoadBookRowAuth(t *testing.T) {
	store := setupTestDB(t)
	book := testBook()

	if err := saveSeriesRow(store, testSeries()); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	got, err := loadBookRowAuth(store, book.ID)
	if err != nil {
		t.Fatalf("loadBookRowAuth() error = %v", err)
	}

	if got != book {
		t.Errorf("loadBookRowAuth() = %+v, want %+v", got, book)
	}
}

func TestLoadBookRowAuthNotFound(t *testing.T) {
	store := setupTestDB(t)

	_, err := loadBookRowAuth(store, "does-not-exist")
	if err == nil {
		t.Fatal("loadBookRowAuth() expected error")
	}

	want := "book not found: does-not-exist"
	if err.Error() != want {
		t.Errorf("error = %q, want %q", err.Error(), want)
	}
}

// -----------------------------------------------------------------------------
// loadBookRow access control
// -----------------------------------------------------------------------------

func TestLoadBookRowAnonymous(t *testing.T) {
	store := setupTestDB(t)
	book := testBook()

	if err := saveSeriesRow(store, testSeries()); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	got, err := loadBookRow(store, book.ID, nil)
	if err != nil {
		t.Fatalf("loadBookRow() error = %v", err)
	}

	if got != book {
		t.Errorf("loadBookRow() = %+v, want %+v", got, book)
	}
}

func TestLoadBookRowAnonymousCannotAccessRestrictedBook(t *testing.T) {
	store := setupTestDB(t)
	book := testBook()

	if err := saveSeriesRow(store, testSeries()); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, book.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	_, err = loadBookRow(store, book.ID, nil)
	if err == nil {
		t.Fatal("loadBookRow() expected anonymous access to be denied")
	}
}

func TestLoadBookRowAuthorizedUser(t *testing.T) {
	store := setupTestDB(t)
	book := testBook()

	if err := saveSeriesRow(store, testSeries()); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, book.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	got, err := loadBookRow(store, book.ID, stringPtr("user-1"))
	if err != nil {
		t.Fatalf("loadBookRow() error = %v", err)
	}

	if got != book {
		t.Errorf("loadBookRow() = %+v, want %+v", got, book)
	}
}

func TestLoadBookRowUnauthorizedUser(t *testing.T) {
	store := setupTestDB(t)
	book := testBook()

	if err := saveSeriesRow(store, testSeries()); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, book.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	_, err = loadBookRow(store, book.ID, stringPtr("user-2"))
	if err == nil {
		t.Fatal("loadBookRow() expected unauthorized access to be denied")
	}
}

func TestLoadBookRowAuthenticatedUserCannotAccessUnrestrictedBook(t *testing.T) {
	store := setupTestDB(t)
	book := testBook()

	if err := saveSeriesRow(store, testSeries()); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	_, err := loadBookRow(store, book.ID, stringPtr("user-1"))
	if err == nil {
		t.Fatal("loadBookRow() expected authenticated user without access to be denied")
	}
}

func TestLoadBookRowNotFound(t *testing.T) {
	store := setupTestDB(t)

	_, err := loadBookRow(store, "does-not-exist", nil)
	if err == nil {
		t.Fatal("loadBookRow() expected error")
	}

	want := "book not found: does-not-exist"
	if err.Error() != want {
		t.Errorf("error = %q, want %q", err.Error(), want)
	}
}

// -----------------------------------------------------------------------------
// saveSeriesRow
// -----------------------------------------------------------------------------

func TestSaveSeriesRow(t *testing.T) {
	store := setupTestDB(t)
	series := testSeries()

	if err := saveSeriesRow(store, series); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	var (
		id   string
		name string
	)

	err := store.DB.QueryRow(`
		SELECT series_id, series_name
		FROM series
		WHERE series_id = ?
	`, series.SeriesID).Scan(&id, &name)
	if err != nil {
		t.Fatalf("failed to read saved series: %v", err)
	}

	if id != series.SeriesID {
		t.Errorf("series ID = %q, want %q", id, series.SeriesID)
	}

	if name != series.SeriesName {
		t.Errorf("series name = %q, want %q", name, series.SeriesName)
	}
}

// -----------------------------------------------------------------------------
// loadSeriesRow
// -----------------------------------------------------------------------------

func TestLoadSeriesRow(t *testing.T) {
	store := setupTestDB(t)

	series := testSeries()

	if err := saveSeriesRow(store, series); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	book1 := testBook()

	if err := saveBookRow(store, book1); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	book2 := book1
	book2.ID = "book-2"
	book2.Title = "The Second Book"
	book2.SeriesOrder = 2

	if err := saveBookRow(store, book2); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	got, err := loadSeriesRow(store, series.SeriesID, nil)
	if err != nil {
		t.Fatalf("loadSeriesRow() error = %v", err)
	}

	want := SeriesEntry{
		SeriesID:    series.SeriesID,
		SeriesName:  series.SeriesName,
		FirstBookID: book1.ID,
	}

	if got != want {
		t.Errorf("loadSeriesRow() = %+v, want %+v", got, want)
	}
}

func TestLoadSeriesRowUsesFirstAccessibleBook(t *testing.T) {
	store := setupTestDB(t)

	series := testSeries()

	if err := saveSeriesRow(store, series); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	book1 := testBook()

	book2 := book1
	book2.ID = "book-2"
	book2.Title = "The Second Book"
	book2.SeriesOrder = 2

	if err := saveBookRow(store, book1); err != nil {
		t.Fatalf("saveBookRow(book1) error = %v", err)
	}

	if err := saveBookRow(store, book2); err != nil {
		t.Fatalf("saveBookRow(book2) error = %v", err)
	}

	// The first book is restricted to user-1.
	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, book1.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	// Anonymous access should therefore select book 2.
	got, err := loadSeriesRow(store, series.SeriesID, nil)
	if err != nil {
		t.Fatalf("anonymous loadSeriesRow() error = %v", err)
	}

	if got.FirstBookID != book2.ID {
		t.Errorf(
			"anonymous first book = %q, want %q",
			got.FirstBookID,
			book2.ID,
		)
	}

	// user-1 should get book 1.
	got, err = loadSeriesRow(store, series.SeriesID, stringPtr("user-1"))
	if err != nil {
		t.Fatalf("user loadSeriesRow() error = %v", err)
	}

	if got.FirstBookID != book1.ID {
		t.Errorf(
			"user first book = %q, want %q",
			got.FirstBookID,
			book1.ID,
		)
	}
}

func TestLoadSeriesRowNotFound(t *testing.T) {
	store := setupTestDB(t)

	_, err := loadSeriesRow(store, "does-not-exist", nil)
	if err == nil {
		t.Fatal("loadSeriesRow() expected error")
	}

	want := "series not found: does-not-exist"
	if err.Error() != want {
		t.Errorf("error = %q, want %q", err.Error(), want)
	}
}

// -----------------------------------------------------------------------------
// loadManifest
// -----------------------------------------------------------------------------

func TestLoadManifestEmpty(t *testing.T) {
	store := setupTestDB(t)

	manifest, err := loadManifest(store, nil)
	if err != nil {
		t.Fatalf("loadManifest() error = %v", err)
	}

	if len(manifest.Series) != 0 {
		t.Fatalf(
			"loadManifest() returned %d series, want 0",
			len(manifest.Series),
		)
	}
}

func TestDBLoadManifest(t *testing.T) {
	store := setupTestDB(t)

	series1 := SeriesEntry{
		SeriesID:   "series-1",
		SeriesName: "Series One",
	}

	series2 := SeriesEntry{
		SeriesID:   "series-2",
		SeriesName: "Series Two",
	}

	if err := saveSeriesRow(store, series1); err != nil {
		t.Fatalf("saveSeriesRow(series1) error = %v", err)
	}

	if err := saveSeriesRow(store, series2); err != nil {
		t.Fatalf("saveSeriesRow(series2) error = %v", err)
	}

	book1 := Book{
		ID:          "book-1",
		Title:       "Book One",
		AuthorID:    "author-1",
		SeriesID:    "series-1",
		SeriesName:  "Series One",
		SeriesOrder: 1,
		FilePath:    "/books/book-1.epub",
	}

	book2 := book1
	book2.ID = "book-2"
	book2.Title = "Book Two"
	book2.SeriesOrder = 2

	book3 := Book{
		ID:          "book-3",
		Title:       "Other Book",
		AuthorID:    "author-2",
		SeriesID:    "series-2",
		SeriesName:  "Series Two",
		SeriesOrder: 1,
		FilePath:    "/books/book-3.epub",
	}

	for _, book := range []Book{book1, book2, book3} {
		if err := saveBookRow(store, book); err != nil {
			t.Fatalf("saveBookRow(%s) error = %v", book.ID, err)
		}
	}

	manifest, err := loadManifest(store, nil)
	if err != nil {
		t.Fatalf("loadManifest() error = %v", err)
	}

	want := Manifest{
		Series: []SeriesEntry{
			{
				SeriesID:    "series-1",
				SeriesName:  "Series One",
				FirstBookID: "book-1",
			},
			{
				SeriesID:    "series-2",
				SeriesName:  "Series Two",
				FirstBookID: "book-3",
			},
		},
	}

	if len(manifest.Series) != len(want.Series) {
		t.Fatalf(
			"loadManifest() returned %d series, want %d",
			len(manifest.Series),
			len(want.Series),
		)
	}

	for i := range want.Series {
		if manifest.Series[i] != want.Series[i] {
			t.Errorf(
				"manifest.Series[%d] = %+v, want %+v",
				i,
				manifest.Series[i],
				want.Series[i],
			)
		}
	}
}

func TestDBLoadManifestAccess(t *testing.T) {
	store := setupTestDB(t)

	series := testSeries()

	if err := saveSeriesRow(store, series); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	book1 := testBook()

	book2 := book1
	book2.ID = "book-2"
	book2.Title = "The Second Book"
	book2.SeriesOrder = 2

	if err := saveBookRow(store, book1); err != nil {
		t.Fatalf("saveBookRow(book1) error = %v", err)
	}

	if err := saveBookRow(store, book2); err != nil {
		t.Fatalf("saveBookRow(book2) error = %v", err)
	}

	// book 1 is available only to user-1.
	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, book1.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	// Anonymous user should see the series through book 2.
	manifest, err := loadManifest(store, nil)
	if err != nil {
		t.Fatalf("anonymous loadManifest() error = %v", err)
	}

	if len(manifest.Series) != 1 {
		t.Fatalf(
			"anonymous manifest contains %d series, want 1",
			len(manifest.Series),
		)
	}

	if manifest.Series[0].FirstBookID != book2.ID {
		t.Errorf(
			"anonymous first book = %q, want %q",
			manifest.Series[0].FirstBookID,
			book2.ID,
		)
	}

	// user-1 should see book 1 as their first accessible book.
	manifest, err = loadManifest(store, stringPtr("user-1"))
	if err != nil {
		t.Fatalf("user loadManifest() error = %v", err)
	}

	if len(manifest.Series) != 1 {
		t.Fatalf(
			"user manifest contains %d series, want 1",
			len(manifest.Series),
		)
	}

	if manifest.Series[0].FirstBookID != book1.ID {
		t.Errorf(
			"user first book = %q, want %q",
			manifest.Series[0].FirstBookID,
			book1.ID,
		)
	}

	// user-2 has no access to either book.
	manifest, err = loadManifest(store, stringPtr("user-2"))
	if err != nil {
		t.Fatalf("unauthorized loadManifest() error = %v", err)
	}

	if len(manifest.Series) != 0 {
		t.Errorf(
			"unauthorized manifest contains %d series, want 0",
			len(manifest.Series),
		)
	}
}

// -----------------------------------------------------------------------------
// deleteSeriesRow
// -----------------------------------------------------------------------------

func TestDeleteSeriesRow(t *testing.T) {
	store := setupTestDB(t)
	series := testSeries()

	if err := saveSeriesRow(store, series); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := deleteSeriesRow(store, series.SeriesID); err != nil {
		t.Fatalf("deleteSeriesRow() error = %v", err)
	}

	var count int

	err := store.DB.QueryRow(`
		SELECT COUNT(*)
		FROM series
		WHERE series_id = ?
	`, series.SeriesID).Scan(&count)
	if err != nil {
		t.Fatalf("failed to check deleted series: %v", err)
	}

	if count != 0 {
		t.Errorf("series count = %d, want 0", count)
	}
}

func TestDeleteSeriesRowMissing(t *testing.T) {
	store := setupTestDB(t)

	if err := deleteSeriesRow(store, "does-not-exist"); err != nil {
		t.Fatalf(
			"deleteSeriesRow() should not error for missing row: %v",
			err,
		)
	}
}

// -----------------------------------------------------------------------------
// deleteBookRow
// -----------------------------------------------------------------------------

func TestDeleteBookRow(t *testing.T) {
	store := setupTestDB(t)

	book := testBook()

	if err := saveSeriesRow(store, testSeries()); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	if err := deleteBookRow(store, book.ID); err != nil {
		t.Fatalf("deleteBookRow() error = %v", err)
	}

	var count int

	err := store.DB.QueryRow(`
		SELECT COUNT(*)
		FROM books
		WHERE id = ?
	`, book.ID).Scan(&count)
	if err != nil {
		t.Fatalf("failed to check deleted book: %v", err)
	}

	if count != 0 {
		t.Errorf("book count = %d, want 0", count)
	}
}

func TestDeleteBookRowMissing(t *testing.T) {
	store := setupTestDB(t)

	if err := deleteBookRow(store, "does-not-exist"); err != nil {
		t.Fatalf(
			"deleteBookRow() should not error for missing row: %v",
			err,
		)
	}
}

// -----------------------------------------------------------------------------
// loadAllBookRows
// -----------------------------------------------------------------------------

func TestLoadAllBookRows(t *testing.T) {
	store := setupTestDB(t)

	series1 := SeriesEntry{
		SeriesID:   "series-1",
		SeriesName: "Series One",
	}

	series2 := SeriesEntry{
		SeriesID:   "series-2",
		SeriesName: "Series Two",
	}

	for _, series := range []SeriesEntry{series1, series2} {
		if err := saveSeriesRow(store, series); err != nil {
			t.Fatalf("saveSeriesRow() error = %v", err)
		}
	}

	books := []Book{
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
			ID:          "book-1",
			Title:       "Book One",
			AuthorID:    "author-1",
			SeriesID:    "series-1",
			SeriesName:  "Series One",
			SeriesOrder: 1,
			FilePath:    "/books/book-1.epub",
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
		if err := saveBookRow(store, book); err != nil {
			t.Fatalf("saveBookRow() error = %v", err)
		}
	}

	got, err := loadAllBookRows(store)
	if err != nil {
		t.Fatalf("loadAllBookRows() error = %v", err)
	}

	want := []Book{
		books[1], // series-1 order 1
		books[0], // series-1 order 2
		books[2], // series-2 order 1
	}

	if len(got) != len(want) {
		t.Fatalf(
			"loadAllBookRows() returned %d books, want %d",
			len(got),
			len(want),
		)
	}

	for i := range want {
		if got[i] != want[i] {
			t.Errorf(
				"loadAllBookRows()[%d] = %+v, want %+v",
				i,
				got[i],
				want[i],
			)
		}
	}
}

func TestLoadAllBookRowsIgnoresAccess(t *testing.T) {
	store := setupTestDB(t)

	series := testSeries()

	if err := saveSeriesRow(store, series); err != nil {
		t.Fatalf("saveSeriesRow() error = %v", err)
	}

	book := testBook()

	if err := saveBookRow(store, book); err != nil {
		t.Fatalf("saveBookRow() error = %v", err)
	}

	_, err := store.DB.Exec(`
		INSERT INTO user_access (book_id, user_id)
		VALUES (?, ?)
	`, book.ID, "user-1")
	if err != nil {
		t.Fatalf("failed to create access row: %v", err)
	}

	books, err := loadAllBookRows(store)
	if err != nil {
		t.Fatalf("loadAllBookRows() error = %v", err)
	}

	if len(books) != 1 {
		t.Fatalf(
			"loadAllBookRows() returned %d books, want 1",
			len(books),
		)
	}

	if books[0] != book {
		t.Errorf("loadAllBookRows() = %+v, want %+v", books[0], book)
	}
}

// -----------------------------------------------------------------------------
// asInt
// -----------------------------------------------------------------------------

func TestAsInt(t *testing.T) {
	tests := []struct {
		name string
		in   interface{}
		want int
	}{
		{
			name: "int",
			in:   int(42),
			want: 42,
		},
		{
			name: "int32",
			in:   int32(42),
			want: 42,
		},
		{
			name: "int64",
			in:   int64(42),
			want: 42,
		},
		{
			name: "float64",
			in:   float64(42),
			want: 42,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := asInt(tt.in)
			if err != nil {
				t.Fatalf("asInt() error = %v", err)
			}

			if got != tt.want {
				t.Errorf("asInt() = %d, want %d", got, tt.want)
			}
		})
	}
}

func TestAsIntUnsupportedType(t *testing.T) {
	_, err := asInt("42")
	if err == nil {
		t.Fatal("asInt() expected error for string")
	}
}
