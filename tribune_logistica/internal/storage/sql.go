package storage

import (
	"database/sql"
	"errors"
	"fmt"
	"log"
	"strings"
)

type SQLStorage struct {
	DB *sql.DB
}

const currentSchemaVersion = 2

func NewSQLStorage(DB *sql.DB) (*SQLStorage, error) {
	s := &SQLStorage{DB: DB}
	err := s.initialize()
	if err != nil {
		return nil, err
	}
	return s, nil
}

func NewSQLStorageWithoutInit(DB *sql.DB) *SQLStorage {
	return &SQLStorage{DB: DB}
}

func (s *SQLStorage) initialize() error {
	log.Println("Initializing")
	s.ensureSchemaVersionTable()
	version, exists, err := s.schemaVersion()
	if err != nil {
		return err
	}

	if !exists {
		if err := s.initSchema(); err != nil {
			return err
		}
		return s.setSchemaVersion(currentSchemaVersion)
	}

	return s.migrate(version)
}

func (s *SQLStorage) migrate(version int) error {
	for version < currentSchemaVersion {
		switch version {
		case 1:
			if err := s.migrateV1ToV2(); err != nil {
				return err
			}
			version = 2

		default:
			return fmt.Errorf("unsupported database schema version: %d", version)
		}
	}

	return nil
}

func (s *SQLStorage) ensureSchemaVersionTable() error {
	_, err := s.DB.Exec(`
		CREATE TABLE IF NOT EXISTS schema_version (
			version INTEGER NOT NULL
		)
	`)
	return err
}

func (s *SQLStorage) schemaVersion() (int, bool, error) {
	var version int

	err := s.DB.QueryRow(`
		SELECT version
		FROM schema_version
		LIMIT 1
	`).Scan(&version)

	if err == sql.ErrNoRows {
		return 0, false, nil
	}
	if err != nil {
		return 0, false, err
	}

	return version, true, nil
}

func (s *SQLStorage) setSchemaVersion(version int) error {
	_, err := s.DB.Exec(`
		DELETE FROM schema_version
	`)
	if err != nil {
		return err
	}

	_, err = s.DB.Exec(`
		INSERT INTO schema_version (version)
		VALUES (?)
	`, version)

	return err
}

func (s *SQLStorage) initSchema() error {
	_, err := s.DB.Exec(`
		CREATE TABLE IF NOT EXISTS series (
			series_id TEXT PRIMARY KEY,
			series_name TEXT NOT NULL CHECK (series_name <> '')
		);

		CREATE TABLE IF NOT EXISTS users (
			username TEXT NOT NULL PRIMARY KEY CHECK (username <> ''),
			password_hash TEXT NOT NULL CHECK (password_hash <> ''),
			pin TEXT NOT NULL CHECK (pin <> ''),
			refresh_token TEXT
		);

		CREATE TABLE IF NOT EXISTS books (
			id TEXT PRIMARY KEY,
			title TEXT NOT NULL CHECK (title <> ''),
			author_id TEXT,
			series_id TEXT NOT NULL,
			series_order INTEGER NOT NULL,
			file_path TEXT NOT NULL CHECK (file_path <> ''),
			FOREIGN KEY (series_id) REFERENCES series(series_id)
		);

		CREATE TABLE IF NOT EXISTS user_access (
			book_id TEXT NOT NULL,
			user_id TEXT NOT NULL,
			PRIMARY KEY (book_id, user_id),
			FOREIGN KEY (book_id) REFERENCES books(id),
			FOREIGN KEY (user_id) REFERENCES users(username)
		);

		CREATE TABLE IF NOT EXISTS UserCursors (
			id TEXT PRIMARY KEY,
			chapter INTEGER NOT NULL,
			chunk INTEGER NOT NULL,
			user_id TEXT NOT NULL,
			book_id TEXT NOT NULL,
			FOREIGN KEY (user_id) REFERENCES users(username),
			FOREIGN KEY (book_id) REFERENCES books(id)
		);

		CREATE TABLE IF NOT EXISTS schema_version (
    		version INTEGER NOT NULL
		);
		INSERT INTO schema_version (version) VALUES (2);
	`)

	return err
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

	rows, err := s.DB.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return RowsToMap(rows)
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
	_, err := s.DB.Exec(fmt.Sprintf(
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
		_, err := s.DB.Exec(fmt.Sprintf(
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

	_, err = s.DB.Exec(query, values...)
	return err
}
func (s *SQLStorage) GetAll(table string) ([]map[string]interface{}, error) {
	rows, err := s.DB.Query(fmt.Sprintf("SELECT * FROM %s", table))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	return RowsToMap(rows)
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

	_, err := s.DB.Exec(query, args...)
	return err
}

// helper: convert sql.Rows to []map[string]interface{}
func RowsToMap(rows *sql.Rows) ([]map[string]interface{}, error) {
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

func (s *SQLStorage) migrateV1ToV2() error {
	log.Println("Migrating database from v1 to v2...")
	tx, err := s.DB.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Users: add pin and remove the obsolete get_token.
	_, err = tx.Exec(`
		CREATE TABLE users_new (
			username TEXT NOT NULL PRIMARY KEY CHECK (username <> ''),
			password_hash TEXT NOT NULL CHECK (password_hash <> ''),
			pin TEXT NOT NULL,
			refresh_token TEXT
		);

		INSERT INTO users_new (
			username,
			password_hash,
			pin,
			refresh_token
		)
		SELECT
			username,
			password_hash,
			'0000',
			refresh_token
		FROM users;

		DROP TABLE users;
		ALTER TABLE users_new RENAME TO users;
	`)
	if err != nil {
		return fmt.Errorf("failed to migrate users: %w", err)
	}

	// Create the new series table and populate it from books.
	_, err = tx.Exec(`
		CREATE TABLE series (
			series_id TEXT PRIMARY KEY,
			series_name TEXT NOT NULL CHECK (series_name <> '')
		);

		INSERT INTO series (series_id, series_name)
		SELECT series_id, COALESCE(MAX(series_name), 'change_me')
		FROM books
		GROUP BY series_id;
	`)
	if err != nil {
		return fmt.Errorf("failed to migrate series: %w", err)
	}

	// Rebuild books without the denormalized series_name column.
	_, err = tx.Exec(`
		CREATE TABLE books_new (
			id TEXT PRIMARY KEY,
			title TEXT NOT NULL CHECK (title <> ''),
			author_id TEXT,
			series_id TEXT NOT NULL,
			series_order INTEGER NOT NULL,
			file_path TEXT NOT NULL CHECK (file_path <> ''),
			FOREIGN KEY (series_id) REFERENCES series(series_id)
		);

		INSERT INTO books_new (
			id,
			title,
			author_id,
			series_id,
			series_order,
			file_path
		)
		SELECT
			id,
			title,
			author_id,
			series_id,
			series_order,
			file_path
		FROM books;

		DROP TABLE books;
		ALTER TABLE books_new RENAME TO books;
	`)
	if err != nil {
		return fmt.Errorf("failed to migrate books: %w", err)
	}

	// Manifest is now derived data and is no longer stored.
	_, err = tx.Exec(`
		DROP TABLE manifest;
	`)
	if err != nil {
		return fmt.Errorf("failed to remove manifest: %w", err)
	}

	// Access control is new; there is nothing to migrate into it.
	_, err = tx.Exec(`
		CREATE TABLE user_access (
			book_id TEXT NOT NULL,
			user_id TEXT NOT NULL,
			PRIMARY KEY (book_id, user_id),
			FOREIGN KEY (book_id) REFERENCES books(id),
			FOREIGN KEY (user_id) REFERENCES users(username)
		);
	`)
	if err != nil {
		return fmt.Errorf("failed to create user_access: %w", err)
	}

	if err := setSchemaVersionTx(tx, 2); err != nil {
		return fmt.Errorf("failed to update schema version: %w", err)
	}

	if err := tx.Commit(); err != nil {
		return fmt.Errorf("failed to commit migration: %w", err)
	}

	return nil
}

func setSchemaVersionTx(tx *sql.Tx, version int) error {
	_, err := tx.Exec(`
		UPDATE schema_version
		SET version = ?
	`, version)
	return err
}
