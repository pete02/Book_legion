package storage

import (
	"encoding/json"
	"errors"
	"os"
	"sync"
)

type JSONStorage struct {
	data map[string][]map[string]interface{}
	path string
	mu   sync.RWMutex
}

// NewJSONStorage loads data from a JSON file if it exists
func NewJSONStorage(path string) (*JSONStorage, error) {
	js := &JSONStorage{
		data: make(map[string][]map[string]interface{}),
		path: path,
	}

	file, err := os.Open(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return js, nil // return empty storage if file missing
		}
		return nil, err
	}
	defer file.Close()

	decoder := json.NewDecoder(file)
	if err := decoder.Decode(&js.data); err != nil {
		return nil, err
	}

	return js, nil
}

// Save writes current data back to JSON file
func (js *JSONStorage) Save() error {
	js.mu.RLock()
	defer js.mu.RUnlock()

	file, err := os.Create(js.path)
	if err != nil {
		return err
	}
	defer file.Close()

	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")
	return encoder.Encode(js.data)
}

func (js *JSONStorage) Insert(table string, row map[string]interface{}) error {
	js.mu.Lock()
	defer js.mu.Unlock()

	js.data[table] = append(js.data[table], row)
	return nil
}

func (js *JSONStorage) Query(table string, filter map[string]interface{}) ([]map[string]interface{}, error) {
	js.mu.RLock()
	defer js.mu.RUnlock()

	rows, ok := js.data[table]
	if !ok {
		return nil, errors.New("table not found")
	}

	if len(filter) == 0 {
		return rows, nil
	}

	var result []map[string]interface{}
	for _, row := range rows {
		match := true
		for key, value := range filter {
			if row[key] != value {
				match = false
				break
			}
		}
		if match {
			result = append(result, row)
		}
	}
	return result, nil
}

func (js *JSONStorage) GetAll(table string) ([]map[string]interface{}, error) {
	js.mu.RLock()
	defer js.mu.RUnlock()

	rows, ok := js.data[table]
	if !ok {
		return nil, errors.New("table not found")
	}

	// Return a copy to prevent accidental modification
	result := make([]map[string]interface{}, len(rows))
	for i, row := range rows {
		copyRow := make(map[string]interface{}, len(row))
		for k, v := range row {
			copyRow[k] = v
		}
		result[i] = copyRow
	}

	return result, nil
}
