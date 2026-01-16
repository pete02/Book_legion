package storage

import (
	"database/sql"
	"testing"

	_ "modernc.org/sqlite"
)

func setupTestDB(t *testing.T) *sql.DB {
	t.Helper()

	db, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatalf("failed to open sqlite db: %v", err)
	}

	return db
}

func TestSQLStorage_InsertAndGetAll(t *testing.T) {
	db := setupTestDB(t)
	st := NewSQLStorage(db)

	err := st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  30,
	})
	if err != nil {
		t.Fatalf("insert failed: %v", err)
	}

	rows, err := st.GetAll("users")
	if err != nil {
		t.Fatalf("getall failed: %v", err)
	}

	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}

	if rows[0]["name"] != "Alice" {
		t.Fatalf("unexpected name: %v", rows[0]["name"])
	}
}

func TestSQLStorage_InsertTwice(t *testing.T) {
	db := setupTestDB(t)
	st := NewSQLStorage(db)

	err := st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  30,
	})
	if err != nil {
		t.Fatalf("insert failed: %v", err)
	}

	err = st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  40,
	})
	if err != nil {
		t.Fatalf("insert failed: %v", err)
	}

	rows, err := st.GetAll("users")
	if err != nil {
		t.Fatalf("getall failed: %v", err)
	}

	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}

	if rows[0]["name"] != "Alice" {
		t.Fatalf("unexpected name: %v", rows[0]["name"])
	}
	if rows[0]["age"] != int64(40) {
		t.Fatalf("unexpected age: %v, %T", rows[0]["age"], rows[0]["age"])
	}
}

func TestSQLStorage_QueryWithFilter(t *testing.T) {
	db := setupTestDB(t)
	st := NewSQLStorage(db)

	st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
		"age":  30,
	})
	st.Insert("users", "name", map[string]interface{}{
		"name": "Bob",
		"age":  40,
	})

	rows, err := st.Query("users", map[string]interface{}{
		"name": "Bob",
	})
	if err != nil {
		t.Fatalf("query failed: %v", err)
	}

	if len(rows) != 1 {
		t.Fatalf("expected 1 row, got %d", len(rows))
	}

	if rows[0]["name"] != "Bob" {
		t.Fatalf("unexpected row: %+v", rows[0])
	}
}

func TestSQLStorage_Delete(t *testing.T) {
	db := setupTestDB(t)
	st := NewSQLStorage(db)

	st.Insert("users", "name", map[string]interface{}{
		"name": "Alice",
	})

	err := st.Delete("users", map[string]interface{}{
		"name": "Alice",
	})
	if err != nil {
		t.Fatalf("delete failed: %v", err)
	}

	rows, err := st.GetAll("users")
	if err != nil {
		t.Fatalf("getall failed: %v", err)
	}

	if len(rows) != 0 {
		t.Fatalf("expected 0 rows, got %d", len(rows))
	}
}

func TestSQLStorage_DeleteWithoutFilter(t *testing.T) {
	db := setupTestDB(t)
	st := NewSQLStorage(db)

	err := st.Delete("users", map[string]interface{}{})
	if err == nil {
		t.Fatal("expected error when deleting without filter")
	}
}
