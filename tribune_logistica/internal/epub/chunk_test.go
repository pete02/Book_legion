package epub

import (
	"fmt"
	"strings"
	"testing"
)

func TestTextChunk(t *testing.T) {
	tests := []struct {
		name        string
		text        string
		ChunkPolicy ChunkPolicy
		wantChunks  []string
	}{
		{
			name:        "happy path simple split",
			text:        "Hello world. This is a test. Split nicely.",
			ChunkPolicy: ChunkPolicy{TargetSize: 12, MaxSize: 20},
			wantChunks: []string{
				"Hello world.",
				"This is a test.",
				"Split nicely.",
			},
		},
		{
			name:        "sentence smaller than target size",
			text:        "Hi. Bye. Go.",
			ChunkPolicy: ChunkPolicy{TargetSize: 10, MaxSize: 20},
			wantChunks: []string{
				"Hi. Bye.",
				"Go.",
			},
		},
		{
			name:        "last sentence smaller than min size",
			text:        "Sentence one. Last.",
			ChunkPolicy: ChunkPolicy{TargetSize: 15, MinSize: 10, MaxSize: 20},
			wantChunks: []string{
				"Sentence one.",
				"Last.",
			},
		},
		{
			name:        "sentence exceeds max size",
			text:        "Short one. This sentence is extremely long and exceeds maximum chunk size easily. Short one.",
			ChunkPolicy: ChunkPolicy{TargetSize: 20, MaxSize: 27},
			wantChunks: []string{
				"Short one.",
				"This sentence is extremely",
				"long and exceeds maximum",
				"chunk size easily.",
				"Short one.",
			},
		},
		{
			name:        "sentence exactly target size",
			text:        "1234567890.1234567890.",
			ChunkPolicy: ChunkPolicy{TargetSize: len("1234567890.1234567890."), MaxSize: 20},
			wantChunks: []string{
				"1234567890.1234567890.",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			linear := LinearText{FullText: tt.text}
			chunks := TextChunk(linear, tt.ChunkPolicy)
			prettychunks := PrettyChunks(chunks, linear)
			fmt.Printf("chunks: %v\n", chunks)
			fmt.Printf("chunks text: [%v]\n", strings.Join(prettychunks, ";"))
			fmt.Printf("expected: [%v]\n", strings.Join(tt.wantChunks, ";"))

			if len(chunks) != len(tt.wantChunks) {
				t.Fatalf("number of chunks mismatch: want %d, got %d", len(tt.wantChunks), len(chunks))
			}
			for i, c := range chunks {
				got := linear.FullText[c.Start:c.End]
				want := tt.wantChunks[i]
				if got != want {
					t.Errorf("chunk %d mismatch:\nwant: %q\ngot:  %q", i, want, got)
				}
			}

			// Verify chunks cover the entire text
			var combined = strings.Join(prettychunks, " ")
			if combined != tt.text {
				t.Errorf("combined chunks do not match original text:\nwant: %q\ngot:  %q", tt.text, combined)
			}
		})
	}
}

func TestSplitSentenceByWords(t *testing.T) {
	tests := []struct {
		name     string
		text     string
		start    int
		end      int
		maxSize  int
		expected []string
	}{
		{
			name:     "short sentence, no split",
			text:     "Hello world.",
			start:    0,
			end:      12,
			maxSize:  20,
			expected: []string{"Hello world."},
		},
		{
			name:     "sentence slightly longer than max",
			text:     "Hello world this is a test.",
			start:    0,
			end:      27,
			maxSize:  12,
			expected: []string{"Hello world", "this is a", "test."},
		},
		{
			name:    "very long sentence",
			text:    "This is a very long sentence that should be split correctly into multiple chunks.",
			start:   0,
			end:     len("This is a very long sentence that should be split correctly into multiple chunks."),
			maxSize: 21,
			expected: []string{
				"This is a very long",
				"sentence that should",
				"be split correctly",
				"into multiple chunks.",
			},
		},
		{
			name:    "single long word",
			text:    "Supercalifragilisticexpialidocious",
			start:   0,
			end:     len("Supercalifragilisticexpialidocious"),
			maxSize: 10,
			expected: []string{
				"Supercalif",
				"ragilistic",
				"expialidoc",
				"ious",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			chunks := SplitSentenceByWords(tt.text, tt.start, tt.end, tt.maxSize)
			linear := LinearText{FullText: tt.text}
			pretty := PrettyChunks(chunks, linear)
			fmt.Printf("expected: %v\n", list_to_txt(tt.expected))
			fmt.Printf("got: %v\n", list_to_txt(pretty))
			if len(chunks) != len(tt.expected) {
				t.Fatalf("number of chunks mismatch: want %d, got %d", len(tt.expected), len(chunks))
			}

			for i, c := range chunks {
				got := strings.TrimSpace(tt.text[c.Start:c.End])
				want := tt.expected[i]
				if got != want {
					t.Errorf("chunk %d mismatch:\nwant: %q\ngot:  %q", i, want, got)
				}
			}
		})
	}
}

func list_to_txt(s []string) string {
	return strings.Join(s, ";")
}
