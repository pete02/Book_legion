package storage

import (
	"database/sql"
	"errors"
	"fmt"
	"strings"
)

type SQLStorage struct {
	db *sql.DB
}

func NewSQLStorage(db *sql.DB) *SQLStorage {
	return &SQLStorage{db: db}
}

func (s *SQLStorage) Query(table string, filter map[string]interface{}) ([]map[string]interface{}, error) {
	if len(filter) == 0 {
		return s.GetAll(table)
	}

	query := fmt.Sprintf("SELECT * FROM %s WHERE ", table)
	args := []interface{}{}
	i := 1
	for k, v := range filter {
		if i > 1 {
			query += " AND "
		}
		query += fmt.Sprintf("%s = ?", k)
		args = append(args, v)
		i++
	}

	rows, err := s.db.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return rowsToMap(rows)
}

func sqliteType(val interface{}) string {
	switch val.(type) {
	case int, int32, int64, bool:
		return "INTEGER"
	case float32, float64:
		return "REAL"
	case string:
		return "TEXT"
	default:
		return "TEXT"
	}
}

func (s *SQLStorage) Insert(table string, primaryKey string, row map[string]interface{}) error {
	if len(row) == 0 {
		return errors.New("empty row")
	}

	// Ensure table exists
	pkType := sqliteType(row[primaryKey])
	_, err := s.db.Exec(fmt.Sprintf(
		"CREATE TABLE IF NOT EXISTS %s (%s %s PRIMARY KEY)",
		table,
		primaryKey,
		pkType,
	))
	if err != nil {
		return fmt.Errorf("failed to create table: %w", err)
	}

	// Add other columns dynamically
	for col, val := range row {
		if col == primaryKey {
			continue
		}
		colType := sqliteType(val)
		_, err := s.db.Exec(fmt.Sprintf(
			"ALTER TABLE %s ADD COLUMN %s %s",
			table,
			col,
			colType,
		))
		if err != nil && !strings.Contains(err.Error(), "duplicate column name") {
			return fmt.Errorf("failed to add column %s: %w", col, err)
		}
	}

	columns := make([]string, 0, len(row))
	placeholders := make([]string, 0, len(row))
	assignments := make([]string, 0, len(row))
	values := make([]interface{}, 0, len(row))

	for col, val := range row {
		columns = append(columns, col)
		placeholders = append(placeholders, "?")
		values = append(values, val)
		if col != primaryKey {
			assignments = append(assignments, fmt.Sprintf("%s = excluded.%s", col, col))
		}
	}

	query := fmt.Sprintf(`
		INSERT INTO %s (%s)
		VALUES (%s)
		ON CONFLICT(%s) DO UPDATE SET %s
	`,
		table,
		strings.Join(columns, ", "),
		strings.Join(placeholders, ", "),
		primaryKey,
		strings.Join(assignments, ", "),
	)

	_, err = s.db.Exec(query, values...)
	return err
}
func (s *SQLStorage) GetAll(table string) ([]map[string]interface{}, error) {
	rows, err := s.db.Query(fmt.Sprintf("SELECT * FROM %s", table))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return rowsToMap(rows)
}

func (s *SQLStorage) Delete(table string, filter map[string]interface{}) error {
	if len(filter) == 0 {
		return fmt.Errorf("refusing to delete without filter")
	}

	var (
		conds []string
		args  []interface{}
	)

	for k, v := range filter {
		conds = append(conds, fmt.Sprintf("%s = ?", k))
		args = append(args, v)
	}

	query := fmt.Sprintf(
		"DELETE FROM %s WHERE %s",
		table,
		strings.Join(conds, " AND "),
	)

	_, err := s.db.Exec(query, args...)
	return err
}

// helper: convert sql.Rows to []map[string]interface{}
func rowsToMap(rows *sql.Rows) ([]map[string]interface{}, error) {
	cols, err := rows.Columns()
	if err != nil {
		return nil, err
	}

	var result []map[string]interface{}
	for rows.Next() {
		values := make([]interface{}, len(cols))
		valuePtrs := make([]interface{}, len(cols))
		for i := range values {
			valuePtrs[i] = &values[i]
		}

		if err := rows.Scan(valuePtrs...); err != nil {
			return nil, err
		}

		m := make(map[string]interface{})
		for i, col := range cols {
			m[col] = values[i]
		}
		result = append(result, m)
	}

	return result, nil
}
