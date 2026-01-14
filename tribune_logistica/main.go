package main

import (
	"database/sql"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/go-chi/chi/v5"
	_ "modernc.org/sqlite"

	"github.com/book_legion-tribune_logistica/internal/api"
	"github.com/book_legion-tribune_logistica/internal/buffer"
	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/storage"
	"github.com/book_legion-tribune_logistica/internal/types"
	// replace with actual module path
)

type DBType string
type TTSBackend string

const (
	DBJSON   DBType = "json"
	DBSQLite DBType = "sqlite"
	DBAPI    DBType = "api"

	TTSMock TTSBackend = "mock"
	TTSHTTP TTSBackend = "http"
)

type Config struct {
	DBType DBType
	DBPath string

	TTSBackend TTSBackend
	TTSAPIURL  string
}

func main() {
	config, err := FromEnv()
	if err != nil {
		fmt.Printf("Error in fetching Env values: %v", err)
	}

	buf := buffer.NewBuffer("id")
	manager := manager.NewOrganizer(buf, 10)

	pol := epub.NewPolicy(500, 400, 700)

	storage, err := createStorage(*config)

	api := api.New(manager, storage, pol)

	if config.TTSBackend == TTSMock {
		fetchFn := func(c types.UserCursor) (types.Chunk, bool) {
			return TTS_fetch_mock(c, api)
		}
		go manager.StartOrderProcessor(fetchFn)
	}

	r := chi.NewRouter()
	r.Post("/api/v1/register", api.RegisterUser)
	r.Post("/api/v1/login", api.LoginUser)
	r.Post("/api/v1/refreshtoken", api.RefreshTokenHandler)

	r.Get("/api/v1/manifest", api.GetManifest)
	r.Get("/api/v1/cursors/{bookID}", api.GetCursor)
	r.Get("/api/v1/books/{bookID}", api.GetBook)
	r.Get("/api/v1/series/{seriesID}", api.GetSeries)
	r.Get("/api/v1/books/{bookID}/chapters/{chapterIndex}", api.GetChapter)
	r.Get("/api/v1/books/{bookID}/chunks", api.GetChunks)
	r.Get("/api/v1/books/{bookID}/nav", api.GetNav)
	r.Get("/api/v1/books/{bookID}/cover", api.GetCover)
	r.Get("/api/v1/books/{bookID}/css", api.GetCSSFiles)

	r.Post("/api/v1/cursors/save", api.SaveCursor)
	r.Post("/api/v1/savebook", api.SaveBook)

	fmt.Println("Server listening on http://localhost:8000")
	if err := http.ListenAndServe(":8000", r); err != nil {
		fmt.Printf("Server failed: %v\n", err)
	}
}

func TTS_fetch_mock(c types.UserCursor, api api.API) (types.Chunk, bool) {

	chunk, err := extractChunk(c, api)
	if err != nil {
		fmt.Printf("Error in fetching chunk: %v", err)
		return types.Chunk{}, false
	}
	ch := types.Chunk{
		ID:   c,
		Data: []byte(chunk),
	}

	return ch, true
}

func extractChunk(c types.UserCursor, api api.API) (string, error) {
	epub, err := epub.Load(api.DB, c.BookID)
	if err != nil {
		return "", err
	}
	fmt.Printf("Extract: %d, %d\n", c.Cursor.Chapter, c.Cursor.Chunk)
	return epub.ExtractChunk(c.Cursor.Chapter, c.Cursor.Chunk, api.Policy)
}

func createStorage(cfg Config) (storage.Storage, error) {
	switch cfg.DBType {
	case DBJSON:
		return storage.NewJSONStorage(cfg.DBPath)
	case DBSQLite:
		{
			db, err := sql.Open("sqlite", cfg.DBPath)
			if err != nil {
				log.Fatalf("failed to open sqlite db: %v", err)
			}
			return storage.NewSQLStorage(db), err
		}
	case DBAPI:
		// Skip for now if you don't support API DB
		return nil, fmt.Errorf("API DB not implemented")
	default:
		{
			db, err := sql.Open("sqlite3", ":memory:")
			if err != nil {
				log.Fatal(err)
			}
			return storage.NewSQLStorage(db), err
		}
	}
}

func FromEnv() (*Config, error) {
	cfg := &Config{}

	dbType := os.Getenv("DB_TYPE")
	if dbType == "" {
		return nil, fmt.Errorf("DB_TYPE is required")
	}

	adminToken := os.Getenv("ADMIN_TOKEN")
	if adminToken == "" {
		return nil, fmt.Errorf("Admin token must be set")
	}

	switch DBType(dbType) {
	case DBJSON, DBSQLite, DBAPI:
		cfg.DBType = DBType(dbType)
	default:
		return nil, fmt.Errorf("invalid DB_TYPE: %s", dbType)
	}

	cfg.DBPath = os.Getenv("DB_PATH")
	if cfg.DBPath == "" {
		return nil, fmt.Errorf("DB_PATH is required")
	}

	ttsBackend := os.Getenv("TTS_BACKEND")
	if ttsBackend == "" {
		cfg.TTSBackend = TTSMock
	} else {
		switch TTSBackend(ttsBackend) {
		case TTSMock, TTSHTTP:
			cfg.TTSBackend = TTSBackend(ttsBackend)
		default:
			return nil, fmt.Errorf("invalid TTS_BACKEND: %s", ttsBackend)
		}
	}

	cfg.TTSAPIURL = os.Getenv("TTS_API_URL")

	if cfg.TTSBackend == TTSHTTP && cfg.TTSAPIURL == "" {
		return nil, fmt.Errorf("TTS_API_URL is required when TTS_BACKEND=http")
	}

	return cfg, nil
}
