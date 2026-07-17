package types

type ChunkIdentifier struct {
	ID          string `json:"BookId"`
	Chapter     int    `json:"Chapter"`
	StartOffset int    `json:"StartOffset"`
	EndOffset   int    `json:"EndOffset"`
}

type TextChunk struct {
	Id   ChunkIdentifier
	Data string
}

type AudioChunk struct {
	Id   ChunkIdentifier
	Data []byte
}
