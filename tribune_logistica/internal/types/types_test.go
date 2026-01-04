package types

import (
	"testing"
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
