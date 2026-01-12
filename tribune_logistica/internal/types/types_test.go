package types

import (
	"os"
	"testing"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

func TestCursorNext(t *testing.T) {
	cases := []struct {
		name       string
		start      Cursor
		maxChunk   int
		maxChapter int
		want       Cursor
	}{
		{"increment chunk", Cursor{Chapter: 1, Chunk: 0}, 3, 5, Cursor{Chapter: 1, Chunk: 1}},
		{"rollover chapter", Cursor{Chapter: 1, Chunk: 3}, 3, 5, Cursor{Chapter: 2, Chunk: 0}},
		{"max chapter limit", Cursor{Chapter: 5, Chunk: 3}, 3, 5, Cursor{Chapter: 5, Chunk: 3}},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			c := tc.start
			c.Next(tc.maxChunk, tc.maxChapter)
			if c != tc.want {
				t.Errorf("Next() = %+v; want %+v", c, tc.want)
			}
		})
	}
}

func TestCursorPrev(t *testing.T) {
	cases := []struct {
		name       string
		start      Cursor
		maxChunk   int
		minChapter int
		want       Cursor
	}{
		{"decrement chunk", Cursor{Chapter: 1, Chunk: 2}, 3, 0, Cursor{Chapter: 1, Chunk: 1}},
		{"rollover chapter", Cursor{Chapter: 2, Chunk: 0}, 3, 0, Cursor{Chapter: 1, Chunk: 3}},
		{"min chapter limit", Cursor{Chapter: 0, Chunk: 0}, 3, 0, Cursor{Chapter: 0, Chunk: 0}},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			c := tc.start
			c.Prev(tc.maxChunk, tc.minChapter)
			if c != tc.want {
				t.Errorf("Prev() = %+v; want %+v", c, tc.want)
			}
		})
	}
}

func TestCursorStepBack(t *testing.T) {
	maxChunks := map[int]int{
		0: 2, // 0..2
		1: 3, // 0..3
		2: 1, // 0..1
	}

	cases := []struct {
		name       string
		start      Cursor
		steps      int
		minChapter int
		want       Cursor
	}{
		{"single chapter", Cursor{0, 2}, 2, 0, Cursor{0, 0}},
		{"multi chapter", Cursor{2, 1}, 4, 0, Cursor{1, 1}},
		{"past min chapter", Cursor{0, 1}, 5, 0, Cursor{0, 0}},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := tc.start.StepBack(tc.steps, tc.minChapter, maxChunks)
			if got != tc.want {
				t.Errorf(" %v, StepBack() = %+v; want %+v", tc.name, got, tc.want)
			}
		})
	}
}

func TestUserUserCursorSaveAndLoad(t *testing.T) {
	tmpFile := "test_UserCursors.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	UserCursor1 := UserCursor{UserID: "u1", BookID: "b1", Cursor: Cursor{Chapter: 2, Chunk: 1}}
	UserCursor2 := UserCursor{UserID: "u2", BookID: "b1", Cursor: Cursor{Chapter: 2, Chunk: 1}}
	UserCursor3 := UserCursor{UserID: "u1", BookID: "b2", Cursor: Cursor{Chapter: 2, Chunk: 1}}

	UserCursors := []UserCursor{UserCursor1, UserCursor2, UserCursor3}

	// Save all UserCursors
	for _, c := range UserCursors {
		if err := SaveUserCursor(store, c); err != nil {
			t.Fatalf("SaveUserUserCursor failed: %v", err)
		}
	}

	for _, c := range UserCursors {
		loaded, err := LoadUserCursor(store, c.UserID, c.BookID)
		if err != nil {
			t.Fatalf("LoadUserUserCursor failed: %v", err)
		}
		if loaded.Cursor.Chapter != c.Cursor.Chapter || loaded.Cursor.Chunk != c.Cursor.Chunk || loaded.UserID != c.UserID || loaded.BookID != c.BookID {
			t.Errorf("Loaded UserCursor %+v; want %+v", loaded, c)
		}
	}
}

func TestLoadNonExistingCursor(t *testing.T) {
	tmpFile := "test_UserCursors.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	UserCursor1 := UserCursor{UserID: "u1", BookID: "b1", Cursor: Cursor{Chapter: 2, Chunk: 1}}

	if err := SaveUserCursor(store, UserCursor1); err != nil {
		t.Fatalf("SaveUserUserCursor failed: %v", err)
	}

	loaded, err := LoadUserCursor(store, "u1", "b2")

	if err != nil {
		t.Fatalf("LoadUserUserCursor failed: %v", err)
	}

	if loaded.BookID != "b2" {
		t.Error("Loaded wrong book")
	}

	if loaded.UserID != "u1" {
		t.Error("Loaded wrong user")
	}

	if loaded.Cursor.Chapter != 0 || loaded.Cursor.Chunk != 0 {
		t.Error("problems in Cursor")
	}

}

func TestLoadNonExistingCursorAsNew(t *testing.T) {
	tmpFile := "test_UserCursors.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	loaded, err := LoadUserCursor(store, "u1", "b2")

	if err != nil {
		t.Fatalf("LoadUserUserCursor failed: %v", err)
	}

	if loaded.BookID != "b2" {
		t.Error("Loaded wrong book")
	}

	if loaded.UserID != "u1" {
		t.Error("Loaded wrong user")
	}

	if loaded.Cursor.Chapter != 0 || loaded.Cursor.Chunk != 0 {
		t.Error("problems in Cursor")
	}

}

func TestUserUserCursorPersistence(t *testing.T) {
	tmpFile := "test_UserCursors.json"
	defer os.Remove(tmpFile)

	store, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to create JSONStorage: %v", err)
	}

	UserCursor1 := UserCursor{UserID: "u1", BookID: "b1", Cursor: Cursor{Chapter: 2, Chunk: 1}}
	UserCursor2 := UserCursor{UserID: "u2", BookID: "b1", Cursor: Cursor{Chapter: 2, Chunk: 1}}
	UserCursor3 := UserCursor{UserID: "u1", BookID: "b2", Cursor: Cursor{Chapter: 2, Chunk: 1}}

	UserCursors := []UserCursor{UserCursor1, UserCursor2, UserCursor3}
	// Save all UserCursors
	for _, c := range UserCursors {
		if err := SaveUserCursor(store, c); err != nil {
			t.Fatalf("SaveUserUserCursor failed: %v", err)
		}
	}

	store.Save()

	// Reload store from file
	storeReloaded, err := storage.NewJSONStorage(tmpFile)
	if err != nil {
		t.Fatalf("failed to reload JSONStorage: %v", err)
	}

	for _, c := range UserCursors {
		loaded, err := LoadUserCursor(storeReloaded, c.UserID, c.BookID)
		if err != nil {
			t.Fatalf("LoadUserUserCursor failed: %v", err)
		}
		if loaded.Cursor.Chapter != c.Cursor.Chapter || loaded.Cursor.Chunk != c.Cursor.Chunk || loaded.UserID != c.UserID || loaded.BookID != c.BookID {
			t.Errorf("Loaded UserCursor %+v; want %+v", loaded, c)
		}
	}
}
