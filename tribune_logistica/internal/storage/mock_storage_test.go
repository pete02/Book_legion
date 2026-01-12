package storage

import (
	"os"
	"reflect"
	"testing"
)

func TestJSONStorage_InsertAndQuery(t *testing.T) {
	// Use empty JSONStorage (no file)
	js := &JSONStorage{
		data: make(map[string][]map[string]interface{}),
	}

	// Insert some rows
	err := js.Insert("books", "id", map[string]interface{}{"id": 1, "title": "1984", "author": "Orwell"})
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}
	err = js.Insert("books", "id", map[string]interface{}{"id": 2, "title": "Brave New World", "author": "Huxley"})
	if err != nil {
		t.Fatalf("Insert failed: %v", err)
	}

	// Query by author
	rows, err := js.Query("books", map[string]interface{}{"author": "Orwell"})
	if err != nil {
		t.Fatalf("Query failed: %v", err)
	}

	if len(rows) != 1 {
		t.Fatalf("Expected 1 row, got %d", len(rows))
	}

	expected := map[string]interface{}{"id": 1, "title": "1984", "author": "Orwell"}
	if !reflect.DeepEqual(rows[0], expected) {
		t.Errorf("Expected %v, got %v", expected, rows[0])
	}

	// Query all rows
	allRows, err := js.GetAll("books")
	if err != nil {
		t.Fatalf("GetAll failed: %v", err)
	}

	if len(allRows) != 2 {
		t.Errorf("Expected 2 rows, got %d", len(allRows))
	}
}

func TestJSONStorage_QueryEmptyTable(t *testing.T) {
	js := &JSONStorage{
		data: make(map[string][]map[string]interface{}),
	}

	_, err := js.Query("nonexistent", nil)
	if err == nil {
		t.Errorf("Expected error querying nonexistent table")
	}
}

func TestJSONStorage_SaveAndLoad(t *testing.T) {
	// Temporary file
	tmpFile := "test_data.json"
	defer os.Remove(tmpFile)

	js := &JSONStorage{
		data: make(map[string][]map[string]interface{}),
		path: tmpFile,
	}

	js.Insert("books", "id", map[string]interface{}{"id": 1, "title": "1984", "author": "Orwell"})
	err := js.Save()
	if err != nil {
		t.Fatalf("Save failed: %v", err)
	}

	// Load from file
	js2, err := NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}

	rows, err := js2.GetAll("books")
	if err != nil {
		t.Fatalf("GetAll failed: %v", err)
	}

	if len(rows) != 1 || rows[0]["title"] != "1984" {
		t.Errorf("Data mismatch after load: %v", rows)
	}
}

func TestJSONStorage_InsertTwice(t *testing.T) {
	// Temporary file
	tmpFile := "test_data.json"
	defer os.Remove(tmpFile)

	js := &JSONStorage{
		data: make(map[string][]map[string]interface{}),
		path: tmpFile,
	}

	js.Insert("books", "id", map[string]interface{}{"id": 1, "title": "1983", "author": "Orwell"})
	err := js.Save()
	if err != nil {
		t.Fatalf("Save failed: %v", err)
	}

	js.Insert("books", "id", map[string]interface{}{"id": 1, "title": "1984", "author": "Orwell"})
	err = js.Save()
	if err != nil {
		t.Fatalf("Save failed: %v", err)
	}

	rows, err := js.GetAll("books")
	if err != nil {
		t.Fatalf("GetAll failed: %v", err)
	}

	if len(rows) != 1 || rows[0]["title"] != "1984" {
		t.Errorf("Data mismatch after load: %v", rows)
	}
}

func TestJSONStorage_QueryWithFilters(t *testing.T) {
	// Create in-memory JSONStorage
	js := &JSONStorage{
		data: make(map[string][]map[string]interface{}),
	}

	// Insert multiple books
	books := []map[string]interface{}{
		{"id": 1, "title": "1984", "author": "Orwell", "year": 1949},
		{"id": 2, "title": "Animal Farm", "author": "Orwell", "year": 1945},
		{"id": 3, "title": "Brave New World", "author": "Huxley", "year": 1932},
		{"id": 4, "title": "Fahrenheit 451", "author": "Bradbury", "year": 1953},
	}

	for _, book := range books {
		if err := js.Insert("books", "id", book); err != nil {
			t.Fatalf("Insert failed: %v", err)
		}
	}

	orwellBooks, err := js.Query("books", map[string]interface{}{"author": "Orwell"})
	if err != nil {
		t.Fatalf("Query failed: %v", err)
	}

	if len(orwellBooks) != 2 {
		t.Errorf("Expected 2 books by Orwell, got %d", len(orwellBooks))
	}

	expectedTitles := map[string]bool{"1984": true, "Animal Farm": true}
	for _, b := range orwellBooks {
		if !expectedTitles[b["title"].(string)] {
			t.Errorf("Unexpected book: %v", b)
		}
	}

	yearBooks, err := js.Query("books", map[string]interface{}{"year": 1932})
	if err != nil {
		t.Fatalf("Query failed: %v", err)
	}

	if len(yearBooks) != 1 || yearBooks[0]["title"] != "Brave New World" {
		t.Errorf("Expected 'Brave New World', got %v", yearBooks)
	}

	specificBook, err := js.Query("books", map[string]interface{}{"author": "Orwell", "year": 1945})
	if err != nil {
		t.Fatalf("Query failed: %v", err)
	}

	if len(specificBook) != 1 || specificBook[0]["title"] != "Animal Farm" {
		t.Errorf("Expected 'Animal Farm', got %v", specificBook)
	}

	noBooks, err := js.Query("books", map[string]interface{}{"author": "Unknown"})
	if err != nil {
		t.Fatalf("Query failed: %v", err)
	}
	if len(noBooks) != 0 {
		t.Errorf("Expected 0 books, got %d", len(noBooks))
	}

	allBooks, err := js.Query("books", map[string]interface{}{})
	if err != nil {
		t.Fatalf("Query failed: %v", err)
	}
	if !reflect.DeepEqual(allBooks, js.data["books"]) {
		t.Errorf("Expected all books, got %v", allBooks)
	}
}

func TestJSONStorageDelete_SingleRow(t *testing.T) {
	tmp := "test_delete_single.json"
	defer os.Remove(tmp)

	store, err := NewJSONStorage(tmp)
	if err != nil {
		t.Fatalf("failed to create storage: %v", err)
	}

	store.Insert("books", "id", map[string]interface{}{
		"id":    "b1",
		"title": "Book 1",
	})
	store.Insert("books", "id", map[string]interface{}{
		"id":    "b2",
		"title": "Book 2",
	})

	err = store.Delete("books", map[string]interface{}{"id": "b1"})
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	rows, _ := store.Query("books", nil)
	if len(rows) != 1 {
		t.Fatalf("expected 1 row remaining, got %d", len(rows))
	}

	if rows[0]["id"] != "b2" {
		t.Fatalf("unexpected remaining row: %+v", rows[0])
	}
}

func TestJSONStorageDelete_MultipleRows(t *testing.T) {
	tmp := "test_delete_multi.json"
	defer os.Remove(tmp)

	store, _ := NewJSONStorage(tmp)

	store.Insert("cursors", "user_id", map[string]interface{}{
		"user_id": "u1",
		"book_id": "b1",
	})
	store.Insert("cursors", "user_id", map[string]interface{}{
		"user_id": "u1",
		"book_id": "b2",
	})
	store.Insert("cursors", "user_id", map[string]interface{}{
		"user_id": "u2",
		"book_id": "b1",
	})

	err := store.Delete("cursors", map[string]interface{}{"user_id": "u1"})
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	rows, _ := store.Query("cursors", nil)
	if len(rows) != 1 {
		t.Fatalf("expected 1 row remaining, got %d", len(rows))
	}

	if rows[0]["user_id"] != "u2" {
		t.Fatalf("unexpected remaining row: %+v", rows[0])
	}
}

func TestJSONStorageDelete_NoMatch(t *testing.T) {
	tmp := "test_delete_nomatch.json"
	defer os.Remove(tmp)

	store, _ := NewJSONStorage(tmp)

	store.Insert("books", "user_id", map[string]interface{}{"id": "b1"})

	err := store.Delete("books", map[string]interface{}{"id": "does-not-exist"})
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	rows, _ := store.Query("books", nil)
	if len(rows) != 1 {
		t.Fatalf("expected row to remain untouched")
	}
}

func TestJSONStorageDelete_EmptyFilterRejected(t *testing.T) {
	tmp := "test_delete_empty.json"
	defer os.Remove(tmp)

	store, _ := NewJSONStorage(tmp)

	err := store.Delete("books", map[string]interface{}{})
	if err == nil {
		t.Fatal("expected error when deleting with empty filter")
	}
}

func TestJSONStorageDelete_Persistence(t *testing.T) {
	tmp := "test_delete_persist.json"
	defer os.Remove(tmp)

	store, _ := NewJSONStorage(tmp)
	store.Insert("books", "user_id", map[string]interface{}{"id": "b1"})
	store.Insert("books", "user_id", map[string]interface{}{"id": "b2"})
	store.Save()

	store.Delete("books", map[string]interface{}{"id": "b1"})

	store.Save()
	reloaded, _ := NewJSONStorage(tmp)
	rows, _ := reloaded.Query("books", nil)

	if len(rows) != 1 || rows[0]["id"] != "b2" {
		t.Fatalf("delete was not persisted: %+v", rows)
	}
}
