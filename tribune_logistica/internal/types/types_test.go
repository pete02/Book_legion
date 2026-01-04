package types_test

import (
	"testing"

	types "github.com/book_legion-tribune_logistica/internal/types"
)

func TestCursorNext(t *testing.T) {
	cases := []struct {
		name       string
		start      types.Cursor
		maxChunk   int
		maxChapter int
		want       types.Cursor
	}{
		{"increment chunk", types.Cursor{Chapter: 1, Chunk: 0}, 3, 5, types.Cursor{Chapter: 1, Chunk: 1}},
		{"rollover chapter", types.Cursor{Chapter: 1, Chunk: 3}, 3, 5, types.Cursor{Chapter: 2, Chunk: 0}},
		{"max chapter limit", types.Cursor{Chapter: 5, Chunk: 3}, 3, 5, types.Cursor{Chapter: 5, Chunk: 3}},
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
		start      types.Cursor
		maxChunk   int
		minChapter int
		want       types.Cursor
	}{
		{"decrement chunk", types.Cursor{Chapter: 1, Chunk: 2}, 3, 0, types.Cursor{Chapter: 1, Chunk: 1}},
		{"rollover chapter", types.Cursor{Chapter: 2, Chunk: 0}, 3, 0, types.Cursor{Chapter: 1, Chunk: 3}},
		{"min chapter limit", types.Cursor{Chapter: 0, Chunk: 0}, 3, 0, types.Cursor{Chapter: 0, Chunk: 0}},
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
		start      types.Cursor
		steps      int
		minChapter int
		want       types.Cursor
	}{
		{"single chapter", types.Cursor{0, 2}, 2, 0, types.Cursor{0, 0}},
		{"multi chapter", types.Cursor{2, 1}, 4, 0, types.Cursor{1, 1}},
		{"past min chapter", types.Cursor{0, 1}, 5, 0, types.Cursor{0, 0}},
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
