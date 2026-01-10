package api

import (
	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/storage"
)

type API struct {
	manager manager.Organizer
	dp      storage.Storage
}
