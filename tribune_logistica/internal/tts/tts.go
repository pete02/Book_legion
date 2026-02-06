package tts

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/book_legion-tribune_logistica/internal/api"
	"github.com/book_legion-tribune_logistica/internal/epub"
	"github.com/book_legion-tribune_logistica/internal/types"
)

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

type ttsRequest struct {
	Text        string  `json:"text"`
	Voice       string  `json:"voice"`
	CFGWeight   float64 `json:"cfg_weight"`
	Temperature float64 `json:"temperature"`
}

func TTS_fetch(c types.UserCursor, api api.API, url string) (types.Chunk, bool) {
	text, err := extractChunk(c, api)
	if err != nil {
		fmt.Printf("Error extracting chunk: %v\n", err)
		return types.Chunk{}, false
	}

	payload := ttsRequest{
		Text:        text,
		Voice:       "sofia",
		CFGWeight:   0.4,
		Temperature: 0.9,
	}

	body, err := json.Marshal(payload)
	if err != nil {
		fmt.Printf("Error marshaling TTS payload: %v\n", err)
		return types.Chunk{}, false
	}

	req, err := http.NewRequest(
		http.MethodPost,
		url,
		bytes.NewReader(body),
	)
	if err != nil {
		fmt.Printf("Error creating TTS request: %v\n", err)
		return types.Chunk{}, false
	}

	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{
		Timeout: 60 * time.Second,
	}

	resp, err := client.Do(req)
	if err != nil {
		fmt.Printf("TTS request failed: %v\n", err)
		return types.Chunk{}, false
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		errBody, _ := io.ReadAll(resp.Body)
		fmt.Printf("TTS error (%d): %s\n", resp.StatusCode, string(errBody))
		return types.Chunk{}, false
	}

	audioBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		fmt.Printf("Error reading TTS audio: %v\n", err)
		return types.Chunk{}, false
	}

	ch := types.Chunk{
		ID:   c,
		Data: audioBytes,
	}

	return ch, true
}
