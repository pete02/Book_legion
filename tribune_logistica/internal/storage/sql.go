package storage

import (
	"database/sql"
	"errors"
	"fmt"
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

func (s *SQLStorage) Insert(table string, row map[string]interface{}) error {
	if len(row) == 0 {
		return errors.New("empty row")
	}

	columns := ""
	placeholders := ""
	values := []interface{}{}
	i := 0
	for k, v := range row {
		if i > 0 {
			columns += ", "
			placeholders += ", "
		}
		columns += k
		placeholders += "?"
		values = append(values, v)
		i++
	}

	query := fmt.Sprintf("INSERT INTO %s (%s) VALUES (%s)", table, columns, placeholders)
	_, err := s.db.Exec(query, values...)
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
