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
	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/storage"
	"github.com/book_legion-tribune_logistica/internal/tts"
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
		log.Fatalf("Failed to load config: %v", err)
	}

	storage, err := createStorage(*config)
	if err != nil {
		log.Fatalf("Failed to create storage: %v", err)
	}

	textBuilder := func(id types.ChunkIdentifier) (types.TextChunk, error) {
		epub, err := epub.Load(storage, id.ID)
		if err != nil {
			return types.TextChunk{}, err
		}
		html, err := epub.GetChapter(id.Chapter)
		if err != nil {
			return types.TextChunk{}, err
		}
		return types.BuildTextChunk(string(html), id, types.DefaultChunkConfig())
	}

	var audioBuilder manager.AudioFetcher
	if config.TTSBackend == TTSMock {
		audioBuilder = tts.MockAudioFetcher()
	} else {
		audioBuilder = tts.RealAudioFetcher(config.TTSAPIURL)
	}
	manager := manager.NewOrganizer(textBuilder, audioBuilder, 10)

	api := api.New(manager, storage)

	r := chi.NewRouter()
	r.Get("/api/v1/cursors/{bookID}", api.GetCursor)
	r.Post("/api/v1/cursors/save", api.SaveCursor)
	r.Get("/api/v1/audio/{bookID}", api.AudioSocket)
	r.Post("/api/v1/register", api.RegisterUser)
	r.Post("/api/v1/login", api.LoginUser)
	r.Post("/api/v1/changepin", api.ChangePin)
	r.Post("/api/v1/refreshtoken", api.RefreshTokenHandler)

	r.Get("/api/v1/books/{bookID}/chapters/{chapterIndex}", api.GetChapter)
	r.Get("/api/v1/books/{bookID}/nav", api.GetNav)
	r.Get("/api/v1/books/{bookID}/cover", api.GetCover)
	r.Get("/api/v1/books/{bookID}/css", api.GetCSS)
	r.Get("/api/v1/books/{bookID}/file", api.GetFile)

	r.Get("/api/v1/book/{bookID}/chapterprogress", api.GetChapterProgress)
	r.Get("/api/v1/book/{bookID}/bookprogress", api.GetBookProgress)
	r.Get("/api/v1/manifest", api.GetManifest)
	r.Get("/api/v1/series/{seriesID}", api.GetSeries)
	r.Get("/api/v1/books/{bookID}", api.GetBook)
	r.Post("/api/v1/savebook", api.SaveBook)
	r.Post("/api/v1/updateseries/{id}", api.UpdateSeriesName)
	r.Delete("/api/v1/deleteseries/{SeriesID}", api.DeleteSeries)
	r.Delete("/api/v1/deletebook/{SeriesID}", api.DeleteBook)

	fmt.Println("Server listening on http://localhost:8000")
	if err := http.ListenAndServe(":8000", r); err != nil {
		fmt.Printf("Server failed: %v\n", err)
	}
}

func createStorage(cfg Config) (*storage.SQLStorage, error) {
	switch cfg.DBType {
	case DBJSON:
		return nil, fmt.Errorf("JSON storage is not supported")
	case DBSQLite:
		{
			db, err := sql.Open("sqlite", cfg.DBPath)
			if err != nil {
				log.Fatalf("failed to open sqlite db: %v", err)
				return nil, err
			}
			if _, err := db.Exec("PRAGMA journal_mode=WAL;"); err != nil {
				log.Fatalf("failed to enable WAL mode: %v", err)
				return nil, err
			}

			return storage.NewSQLStorage(db)
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
			return storage.NewSQLStorage(db)
		}
	}
}

func FromEnv() (*Config, error) {
	cfg := &Config{}

	dbType := os.Getenv("DB_TYPE")
	if dbType == "" {
		return nil, fmt.Errorf("DB_TYPE is required")
	}

	library := os.Getenv("LIBRARY_ROOT")
	if library == "" {
		return nil, fmt.Errorf("LIBRARY_ROOT is required")
	}

	adminToken := os.Getenv("TRIBUNE_LOGISTICA_ADMIN_TOKEN")
	if adminToken == "" {
		return nil, fmt.Errorf("TRIBUNE_LOGISTICA_ADMIN_TOKEN must be set")
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
