// internal/buffer/types.go
package types

import (
	"errors"
	"fmt"

	"github.com/book_legion-tribune_logistica/internal/storage"
)

type TextCursor struct {
	Cursor UserCursor `json:"cursor"`
	Text   string     `json:"text"`
}
type UserCursor struct {
	UserID string `json:"user_id"`
	BookID string `json:"book_id"`
	Cursor Cursor `json:"cursor"`
}

func (a UserCursor) CompareCursor(b UserCursor) int {
	return a.Cursor.CompareCursor(b.Cursor)
}
func NewUserCursor(user string, book string, chapter int, chunk int) UserCursor {
	return UserCursor{
		UserID: user,
		BookID: book,
		Cursor: Cursor{
			Chapter: chapter,
			Index:   chunk,
		},
	}
}

// SaveUserCursor saves a user's UserCursor position for a specific book
func SaveUserCursor(store storage.Storage, c UserCursor) error {
	row := map[string]interface{}{
		"id":      c.UserID + ":" + c.BookID,
		"user_id": c.UserID,
		"book_id": c.BookID,
		"chapter": c.Cursor.Chapter,
		"chunk":   c.Cursor.Index,
	}
	return store.Insert("UserCursors", "id", row)
}

// LoadUserCursor loads a user's UserCursor for a book
func LoadUserCursor(store storage.Storage, userID, bookID string) (UserCursor, error) {
	synthID := userID + ":" + bookID

	rows, err := store.Query("UserCursors", map[string]interface{}{
		"id": synthID,
	})
	if err != nil {
		user := UserCursor{
			UserID: userID,
			BookID: bookID,
			Cursor: Cursor{Chapter: 0, Index: 0},
		}
		SaveUserCursor(store, user)
		return user, nil
	}

	if len(rows) == 0 {
		return UserCursor{
			UserID: userID,
			BookID: bookID,
			Cursor: Cursor{Chapter: 0, Index: 0},
		}, nil
	}

	row := rows[0]

	// Parse chapter
	var chapter int
	switch v := row["chapter"].(type) {
	case float64:
		chapter = int(v)
	case int:
		chapter = v
	case int64:
		chapter = int(v)
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
	case int64:
		chunk = int(v)
	default:
		return UserCursor{}, fmt.Errorf("unexpected type for chunk: %T", row["chunk"])
	}

	cursr := Cursor{
		Chapter: chapter,
		Index:   chunk,
	}

	return UserCursor{
		UserID: row["user_id"].(string),
		BookID: row["book_id"].(string),
		Cursor: cursr,
	}, nil
}

func (uc UserCursor) ValidateCursor(maxIndexs map[int]int) (UserCursor, error) {
	currentChapter := uc.Cursor.Chapter
	currentIndex := uc.Cursor.Index

	// Check if current chapter exists in maxIndexs
	maxIndex, ok := maxIndexs[currentChapter]
	if !ok {
		return uc, errors.New("chapter does not exist")
	}

	// If chunk is within the valid range, return as is
	if currentIndex <= maxIndex {
		return uc, nil
	}

	// Move to next chapter
	nextChapter := currentChapter + 1
	if _, ok := maxIndexs[nextChapter]; !ok {
		return uc, errors.New("cursor is past the last chapter")
	}

	// Return cursor at next chapter, chunk 0
	uc.Cursor.Chapter = nextChapter
	uc.Cursor.Index = 0
	return uc, nil
}

type CursorLocateRequest struct {
	SnippetHTML string `json:"snippet_html"`
}
