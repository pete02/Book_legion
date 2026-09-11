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
	r.Route("/api/v1", func(r chi.Router) {
		r.Use(noCache)
		r.With(api.ReadAccess).Get("/manifest", api.GetManifest)

		// Book endpoints
		r.Route("/books/{bookID}", func(r chi.Router) {
			r.Use(api.ReadAccess)
			r.Use(api.BookExists)
			r.Use(api.AccessCheck)

			r.Get("/", api.GetBook)
			r.Get("/chapters/{chapterIndex}", api.GetChapter)
			r.Get("/nav", api.GetNav)
			r.Get("/cover", api.GetCover)
			r.Get("/css", api.GetCSS)
			r.Get("/file", api.GetFile)

			r.Get("/chapterprogress", api.GetChapterProgress)
			r.Get("/bookprogress", api.GetBookProgress)
		})

		r.With(api.ReadAccess).Get("/series/{seriesID}", api.GetSeries)

		r.With(api.AccessCheck).Get("/cursors/{bookID}", api.GetCursor)
		r.Post("/cursors/save", api.SaveCursor)

		r.Get("/audio/{bookID}", api.AudioSocket)

		r.Post("/register", api.RegisterUser)
		r.Post("/login", api.LoginUser)
		r.Post("/changepin", api.ChangePin)
		r.Post("/refreshtoken", api.RefreshTokenHandler)

		r.With(api.WriteAccess).Post("/savebook", api.SaveBook)

		r.With(api.WriteAccess).Post(
			"/updateseries/{id}",
			api.UpdateSeriesName,
		)

		r.With(api.WriteAccess).Delete(
			"/books/{bookID}",
			api.DeleteBook,
		)

		r.With(api.WriteAccess).Delete(
			"/series/{seriesID}",
			api.DeleteSeries,
		)
	})

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

func noCache(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Cache-Control", "no-store")
		next.ServeHTTP(w, r)
	})
}
