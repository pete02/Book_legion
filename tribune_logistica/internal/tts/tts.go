package tts

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"strings"

	"github.com/book_legion-tribune_logistica/internal/manager"
	"github.com/book_legion-tribune_logistica/internal/types"
)

// ttsRequest mirrors the Sanic /tts handler's expected JSON body.
// cfg_weight, exaggeration, and temperature are all optional server-side
// (the Python handler defaults them with req.get(..., default)), so we
// only send them if you actually want to override the defaults.
type ttsRequest struct {
	Text  string `json:"text"`
	Voice string `json:"voice"`
}

func MockAudioFetcher() manager.AudioFetcher {
	return func(ctx context.Context, chunk types.TextChunk) (types.AudioChunk, bool) {
		if err := ctx.Err(); err != nil {
			return types.AudioChunk{}, false
		}
		return types.AudioChunk{
			Id:   chunk.Id,
			Data: []byte(chunk.Data),
		}, true
	}
}

func RealAudioFetcher(baseURL string) manager.AudioFetcher {
	client := &http.Client{}
	url := strings.TrimRight(baseURL, "/") + "/tts"

	return func(ctx context.Context, chunk types.TextChunk) (types.AudioChunk, bool) {
		payload, err := json.Marshal(ttsRequest{
			Text:  chunk.Data,
			Voice: "sofia",
		})
		if err != nil {
			log.Printf("[tts] failed to marshal request for chunk %s: %v", chunk.Id, err)
			return types.AudioChunk{}, false
		}

		req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(payload))
		if err != nil {
			log.Printf("[tts] failed to build request for chunk %s: %v", chunk.Id, err)
			return types.AudioChunk{}, false
		}
		req.Header.Set("Content-Type", "application/json")

		resp, err := client.Do(req)
		if err != nil {
			if ctx.Err() != nil {
				log.Printf("[tts] chunk %s cancelled: %v", chunk.Id, ctx.Err())
			} else {
				log.Printf("[tts] request failed for chunk %s: %v", chunk.Id, err)
			}
			return types.AudioChunk{}, false
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusOK {
			msg, _ := io.ReadAll(resp.Body)
			log.Printf("[tts] backend returned %d for chunk %s: %s", resp.StatusCode, chunk.Id, msg)
			return types.AudioChunk{}, false
		}

		data, err := io.ReadAll(resp.Body)
		if err != nil {
			log.Printf("[tts] failed to read audio for chunk %s: %v", chunk.Id, err)
			return types.AudioChunk{}, false
		}

		return types.AudioChunk{
			Id:   chunk.Id,
			Data: data,
		}, true
	}
}
