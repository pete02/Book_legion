package storage

type Storage interface {
	Query(table string, filter map[string]interface{}) ([]map[string]interface{}, error)
	Insert(table string, primaryKey string, row map[string]interface{}) error
	GetAll(table string) ([]map[string]interface{}, error)
	Delete(table string, filter map[string]interface{}) error
}
