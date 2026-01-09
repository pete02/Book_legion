package storage

type Storage interface {
	Query(table string, filter map[string]interface{}) ([]map[string]interface{}, error)
	Insert(table string, row map[string]interface{}) error
	GetAll(table string) ([]map[string]interface{}, error)
}
