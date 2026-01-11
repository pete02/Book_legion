// internal/buffer/types.go
package types

import (
	"fmt"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

// Cursor represents a location in the audiobook
type Cursor struct {
	Chapter int `json:"chapter"`
	Chunk   int `json:"chunk"`
}

func (c *Cursor) Next(maxChunk int, maxChapter int) {
	if c.Chunk == maxChunk {
		if c.Chapter < maxChapter {
			c.Chapter += 1
			c.Chunk = 0
		}
	} else {
		c.Chunk += 1
	}
}

func (c *Cursor) Prev(maxChunk int, minChapter int) {
	if c.Chunk == 0 {
		if c.Chapter > minChapter {
			c.Chapter -= 1
			c.Chunk = maxChunk
		}
	} else {
		c.Chunk -= 1
	}
}

func (c Cursor) StepBack(steps int, minChapter int, maxChunks map[int]int) Cursor {
	cur := c
	for range steps {
		maxChunk, ok := maxChunks[cur.Chapter-1]
		if !ok {
			maxChunk = 0
		}
		cur.Prev(maxChunk, minChapter)
	}
	return cur
}

// compareCursor returns -1 if a < b, 0 if a == b, 1 if a > b
func (a Cursor) CompareCursor(b Cursor) int {
	if a.Chapter < b.Chapter {
		return -1
	} else if a.Chapter > b.Chapter {
		return 1
	} else { // same chapter
		if a.Chunk < b.Chunk {
			return -1
		} else if a.Chunk > b.Chunk {
			return 1
		}
		return 0
	}
}

// Chunk represents an audio Chunk
type Chunk struct {
	ID   Cursor `json:"Cursor"`
	Data []byte `json:"data"`
}

type UserCursor struct {
	UserID string `json:"UserID"`
	BookID string `json:"BookID"`
	Cursor Cursor `json:"Cursor"`
}

// SaveUserCursor saves a user's UserCursor position for a specific book
func SaveUserCursor(store storage.Storage, c UserCursor) error {
	row := map[string]interface{}{
		"user_id": c.UserID,
		"book_id": c.BookID,
		"chapter": c.Cursor.Chapter,
		"chunk":   c.Cursor.Chunk,
	}
	return store.Insert("UserCursors", row)
}

// LoadUserCursor loads a user's UserCursor for a book
func LoadUserCursor(store storage.Storage, userID, bookID string) (UserCursor, error) {
	rows, err := store.Query("UserCursors", map[string]interface{}{
		"user_id": userID,
		"book_id": bookID,
	})
	if err != nil {
		user := UserCursor{UserID: userID, BookID: bookID, Cursor: Cursor{Chapter: 0, Chunk: 0}}
		SaveUserCursor(store, user)
		return user, nil
	}

	if len(rows) == 0 {
		return UserCursor{UserID: userID, BookID: bookID, Cursor: Cursor{Chapter: 0, Chunk: 0}}, nil
	}

	row := rows[0]

	// Parse chapter
	var chapter int
	switch v := row["chapter"].(type) {
	case float64:
		chapter = int(v)
	case int:
		chapter = v
	default:
		return UserCursor{}, fmt.Errorf("unexpected type for chapter: %T", row["chapter"])
	}

	// Parse chunk
	var chunk int
	switch v := row["chunk"].(type) {
	case float64:
		chunk = int(v)
	case int:
		chunk = v
	default:
		return UserCursor{}, fmt.Errorf("unexpected type for chunk: %T", row["chunk"])
	}

	cursr := Cursor{
		Chapter: chapter,
		Chunk:   chunk,
	}

	return UserCursor{
		UserID: row["user_id"].(string),
		BookID: row["book_id"].(string),
		Cursor: cursr,
	}, nil
}
