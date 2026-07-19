package types

import (
	"errors"
	"strings"
	"testing"
)

func TestBuildTextChunk_BasicSentenceSplit(t *testing.T) {
	html := "<p>The cat sat.</p><p>The dog ran fast today.</p>"
	cfg := ChunkConfig{TargetWords: 3, MinSplitWords: 1}
	id := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: 0}

	chunk, err := BuildTextChunk(html, id, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk: %v", err)
	}
	if chunk.Data != "The cat sat." {
		t.Fatalf("got Data %q, want %q", chunk.Data, "The cat sat.")
	}
	if chunk.Id.EndOffset != 19 {
		t.Fatalf("Got: %v, wanted 19", chunk.Id.EndOffset)
	}

	// Continue from where this chunk left off; should pick up the second
	// sentence cleanly, with the intervening tags stripped and ignored.
	next := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: chunk.Id.EndOffset}
	chunk2, err := BuildTextChunk(html, next, ChunkConfig{TargetWords: 5, MinSplitWords: 1})
	if err != nil {
		t.Fatalf("BuildTextChunk (2nd): %v", err)
	}
	if chunk2.Data != "The dog ran fast today." {
		t.Fatalf("got Data %q, want %q", chunk2.Data, "The dog ran fast today.")
	}
	if chunk2.Id.StartOffset != 19 {
		t.Fatalf("Got Start offset: %v, wanted 19", chunk2.Id.StartOffset)
	}

	if chunk2.Id.EndOffset != 49 {
		t.Fatalf("Got End offset: %v, wanted 49", chunk2.Id.EndOffset)
	}
}

func TestBuildTextChunk_MinSplitWordsRejectsShortCandidate(t *testing.T) {
	// "Hi." alone is only 1 word — below MinSplitWords(5) — so it must be
	// rejected as a split point even though it's a sentence end, and
	// scanning must continue into the next sentence to find a usable split.
	html := "<p>Hi. This is a test.</p>"
	cfg := ChunkConfig{TargetWords: 3, MinSplitWords: 5}
	id := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: 0}

	chunk, err := BuildTextChunk(html, id, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk: %v", err)
	}
	want := "Hi. This is a test."
	if chunk.Data != want {
		t.Fatalf("got Data %q, want %q (the too-short 'Hi.' split should have been skipped)", chunk.Data, want)
	}

	if chunk.Id.EndOffset != 26 {
		t.Fatalf("Got: %v, wanted 23", chunk.Id.EndOffset)
	}
}

func TestBuildTextChunk_FailsafeScansPastTargetForValidSplit(t *testing.T) {
	// No sentence-ending punctuation until word 7, well past TargetWords(3).
	// Scanning must continue past target rather than cutting mid-sentence,
	// and terminate immediately once a valid split appears.
	html := "<p>one two three four five six seven.</p>"
	cfg := ChunkConfig{TargetWords: 3, MinSplitWords: 1}
	id := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: 0}

	chunk, err := BuildTextChunk(html, id, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk: %v", err)
	}
	want := "one two three four five six seven."
	if chunk.Data != want {
		t.Fatalf("got Data %q, want %q", chunk.Data, want)
	}
}

func TestBuildTextChunk_BlockBoundarySplitsWithoutPunctuation(t *testing.T) {
	// List items have no terminal punctuation, but a closing </li> should
	// still count as a valid split point on its own.
	html := "<ul><li>Apple</li><li>Banana</li><li>Cherry</li></ul>"
	cfg := ChunkConfig{TargetWords: 1, MinSplitWords: 1}
	id := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: 0}

	chunk, err := BuildTextChunk(html, id, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk: %v", err)
	}
	if chunk.Data != "Apple" {
		t.Fatalf("got Data %q, want %q", chunk.Data, "Apple")
	}

	next := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: chunk.Id.EndOffset}
	chunk2, err := BuildTextChunk(html, next, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk (2nd): %v", err)
	}
	if chunk2.Data != "Banana" {
		t.Fatalf("got Data %q, want %q", chunk2.Data, "Banana")
	}
}

func TestBuildTextChunk_StripsHTMLTags(t *testing.T) {
	html := `<p>Hello <em>brave</em> new <strong>world</strong>.</p>`
	cfg := ChunkConfig{TargetWords: 10, MinSplitWords: 1}
	id := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: 0}

	// Runs off the end of this tiny chapter, so it comes back via the
	// end-of-content path rather than hitting a target-driven split — that's
	// fine, this test only cares about tag stripping.
	chunk, err := BuildTextChunk(html, id, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk: %v", err)
	}
	want := "Hello brave new world."
	if chunk.Data != want {
		t.Fatalf("got Data %q, want %q", chunk.Data, want)
	}
}

func TestBuildTextChunk_EndOfChapter(t *testing.T) {
	html := "<p>Short ending.</p>"
	cfg := ChunkConfig{TargetWords: 100, MinSplitWords: 1} // unreachable target
	id := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: 0}

	chunk, err := BuildTextChunk(html, id, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk: %v", err)
	}
	if chunk.Data != "Short ending." {
		t.Fatalf("got Data %q, want %q", chunk.Data, "Short ending.")
	}

	// The next call, starting exactly where the chapter's content ended,
	// must signal end-of-chapter rather than returning an empty chunk.
	next := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: chunk.Id.EndOffset}
	_, err = BuildTextChunk(html, next, cfg)
	if !errors.Is(err, ErrEndOfChapter) {
		t.Fatalf("expected ErrEndOfChapter, got %v", err)
	}
}

func TestBuildTextChunk_StartOffsetPastEnd(t *testing.T) {
	html := "<p>Hi.</p>"
	id := ChunkIdentifier{ID: "b1", Chapter: 0, StartOffset: 1000}

	_, err := BuildTextChunk(html, id, DefaultChunkConfig())
	if !errors.Is(err, ErrEndOfChapter) {
		t.Fatalf("expected ErrEndOfChapter, got %v", err)
	}
}

func TestDefaultChunkConfig(t *testing.T) {
	cfg := DefaultChunkConfig()
	if cfg.TargetWords != 35 {
		t.Fatalf("got TargetWords %d, want 35 (~15s at ~140 WPM)", cfg.TargetWords)
	}
	if cfg.MinSplitWords != 15 {
		t.Fatalf("got MinSplitWords %d, want 15", cfg.MinSplitWords)
	}
}

// --- NearestAllowedSplit, tested directly as its own reusable primitive ---

func TestNearestAllowedSplit_SentenceBoundary(t *testing.T) {
	html := "<p>Hi. This is a test.</p>"
	seg := NearestAllowedSplit(html, 0, 1)
	if seg.EndOfContent {
		t.Fatalf("expected a real split, got EndOfContent")
	}
	if seg.Text != "Hi." || seg.Words != 1 {
		t.Fatalf("got {%q, %d words}, want {%q, 1 word}", seg.Text, seg.Words, "Hi.")
	}
}

func TestNearestAllowedSplit_SkipsTooShortCandidates(t *testing.T) {
	html := "<p>Hi. This is a test.</p>"
	seg := NearestAllowedSplit(html, 0, 5)
	if seg.Text != "Hi. This is a test." || seg.Words != 5 {
		t.Fatalf("got {%q, %d words}, want {%q, 5 words}", seg.Text, seg.Words, "Hi. This is a test.")
	}
}

func TestNearestAllowedSplit_StartMidwayThroughContent(t *testing.T) {
	html := "<p>Hi. This is a test.</p>"
	first := NearestAllowedSplit(html, 0, 1)
	// Resuming from the first split's offset should pick up the next
	// sentence cleanly — this is exactly the access pattern BuildTextChunk
	// (and, later, any seek-snap caller resuming a scan) relies on.
	second := NearestAllowedSplit(html, first.Offset, 1)
	if second.Text != "This is a test." {
		t.Fatalf("got %q, want %q", second.Text, "This is a test.")
	}
}

func TestNearestAllowedSplit_EndOfContent(t *testing.T) {
	html := "<p>Short ending.</p>"
	// minWords high enough that no in-content candidate can qualify.
	seg := NearestAllowedSplit(html, 0, 100)
	if !seg.EndOfContent {
		t.Fatalf("expected EndOfContent, got a real split at %+v", seg)
	}
	if seg.Text != "Short ending." || seg.Words != 2 {
		t.Fatalf("got {%q, %d words}, want {%q, 2 words}", seg.Text, seg.Words, "Short ending.")
	}
}

func TestNearestAllowedSplit_StartPastEnd(t *testing.T) {
	html := "<p>Hi.</p>"
	seg := NearestAllowedSplit(html, 1000, 1)
	if !seg.EndOfContent || seg.Words != 0 {
		t.Fatalf("got %+v, want EndOfContent with 0 words", seg)
	}
}
func TestNearestPrecedingSplit_MidSentenceBacktracksToStart(t *testing.T) {
	html := "<p>Hi. Hello world.</p>"
	// "Hello" starts right after "Hi. " — landing mid-word inside "Hello"
	// should backtrack to where "Hello" begins.
	start := indexOf(t, html, "Hello")
	mid := start + 2 // inside "Hello"

	got := NearestPrecedingSplit(html, mid)
	if got != start {
		t.Fatalf("got offset %d, want %d (start of \"Hello\")", got, start)
	}
}

func TestNearestPrecedingSplit_AlreadyAtSentenceStartIsUnchanged(t *testing.T) {
	html := "<p>Hi. Hello world.</p>"
	start := indexOf(t, html, "Hello")

	got := NearestPrecedingSplit(html, start)
	if got != start {
		t.Fatalf("got offset %d, want %d (already at sentence start)", got, start)
	}
}

func TestNearestPrecedingSplit_WithinFirstSentenceReturnsChapterStart(t *testing.T) {
	html := "<p>Hello world today.</p>"
	mid := indexOf(t, html, "world")

	got := NearestPrecedingSplit(html, mid)
	want := indexOf(t, html, "Hello")
	if got != want {
		t.Fatalf("got offset %d, want %d (chapter's first content char)", got, want)
	}
}

func TestNearestPrecedingSplit_BlockTagBoundary(t *testing.T) {
	html := "<ul><li>Apple</li><li>Banana pie today</li></ul>"
	mid := indexOf(t, html, "pie")

	got := NearestPrecedingSplit(html, mid)
	want := indexOf(t, html, "Banana")
	if got != want {
		t.Fatalf("got offset %d, want %d (start of the Banana <li>)", got, want)
	}
}

func TestNearestPrecedingSplit_ConsistentWithForwardSplit(t *testing.T) {
	// Round-trip check: find a forward split, then confirm backtracking
	// from any point within the following sentence lands back on exactly
	// that split's start-of-content offset — i.e. the two functions agree
	// on where sentences begin, since they share the same boundary rules.
	html := "<p>First sentence here. Second sentence right here. Third one.</p>"
	firstSplit := NearestAllowedSplit(html, 0, 1)
	secondSentenceStart := skipNonContent([]rune(html), firstSplit.Offset)

	midSecond := secondSentenceStart + 3 // a few runes into the 2nd sentence
	got := NearestPrecedingSplit(html, midSecond)
	if got != secondSentenceStart {
		t.Fatalf("got offset %d, want %d (forward/backward disagree on sentence start)", got, secondSentenceStart)
	}
}

func TestNearestPrecedingSplit_OffsetPastEndClamps(t *testing.T) {
	html := "<p>Hi. Hello world.</p>"
	// Offset way past the string clamps to len(runes). "world." is
	// immediately followed by the closing tag with no trailing space, so
	// that closing tag boundary — which sits exactly at the end of the
	// string — is itself the nearest valid preceding split; there's no
	// unclosed content left to backtrack into.
	got := NearestPrecedingSplit(html, 10000)
	want := len([]rune(html))
	if got != want {
		t.Fatalf("got offset %d, want %d (end of content)", got, want)
	}
}

// indexOf returns the rune offset of the first occurrence of substr in s.
func indexOf(t *testing.T, s, substr string) int {
	t.Helper()
	byteIdx := strings.Index(s, substr)
	if byteIdx < 0 {
		t.Fatalf("substring %q not found in %q", substr, s)
	}
	return len([]rune(s[:byteIdx]))
}

const wantSentence = "She laughed, bumping him again before leaning in and resting her head on his shoulder as she ate. It was a bit uncomfortable, and it made the whole process longer and harder, but she reveled in the contact."

const wellFormedHTML = `<p class="cnNiOTRjODMxNTNiZDQyMDZhNTUxZDVlNzcxM2M5NWIx">
                <span style="font-weight: 400">` + wantSentence + `</span>
            </p>`

// jsonEscapedHTML is the same markup, but with the class attribute's
// closing quote and the tag's closing '>' leaked through as literal JSON
// escape sequences (\" and \u003e) instead of real characters - exactly
// what was reported.
const jsonEscapedHTML = `<p class=\"cnNiOTRjODMxNTNiZDQyMDZhNTUxZDVlNzcxM2M5NWIx\"\u003e` +
	`<span style="font-weight: 400">` + wantSentence + `</span></p>`

// The two source encodings differ only in how the <p> tag's tail is
// written; BuildTextChunk must produce identical output for both.
func TestBuildTextChunk_EncodingsProduceIdenticalOutput(t *testing.T) {
	cfg := ChunkConfig{TargetWords: 1000, MinSplitWords: 1}

	for name, html := range map[string]string{
		"well-formed":  wellFormedHTML,
		"json-escaped": jsonEscapedHTML,
	} {
		t.Run(name, func(t *testing.T) {
			chunk, err := BuildTextChunk(html, ChunkIdentifier{StartOffset: 0}, cfg)
			if err != nil {
				t.Fatalf("BuildTextChunk: %v", err)
			}
			if chunk.Data != wantSentence {
				t.Fatalf("got Data %q, want %q", chunk.Data, wantSentence)
			}
		})
	}
}

// Regression test for the reported bug: if StartOffset happens to land in
// the middle of the class attribute (e.g. because it was computed against
// a differently-encoded copy of this html), the leftover attribute text
// must NOT leak into Data - regardless of how the tag's tail is encoded.
func TestBuildTextChunk_MidTagStartOffset_NoLeakage(t *testing.T) {
	cfg := ChunkConfig{TargetWords: 1000, MinSplitWords: 1}

	tests := []struct {
		name string
		html string
	}{
		{"well-formed", wellFormedHTML},
		{"json-escaped", jsonEscapedHTML},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Start partway through the class attribute value - a position
			// that has no business ever being treated as the start of real
			// content, but which is exactly where a mismatched offset from
			// another encoding of this string could land.
			start := indexOf(t, tc.html, "DZhNTUxZDVlNzcxM2M5NWIx")

			chunk, err := BuildTextChunk(tc.html, ChunkIdentifier{StartOffset: start}, cfg)
			if err != nil {
				t.Fatalf("BuildTextChunk: %v", err)
			}
			if chunk.Data != wantSentence {
				t.Fatalf("attribute leaked into output:\ngot:  %q\nwant: %q", chunk.Data, wantSentence)
			}
		})
	}
}

// Same idea, but starting from inside the *style* attribute of the <span>
// tag instead of the <p>'s class attribute, to make sure the fix isn't
// accidentally specific to one tag or attribute.
func TestBuildTextChunk_MidTagStartOffset_SpanAttribute(t *testing.T) {
	html := wellFormedHTML
	start := indexOf(t, html, "font-weight")
	cfg := ChunkConfig{TargetWords: 1000, MinSplitWords: 1}

	chunk, err := BuildTextChunk(html, ChunkIdentifier{StartOffset: start}, cfg)
	if err != nil {
		t.Fatalf("BuildTextChunk: %v", err)
	}
	if chunk.Data != wantSentence {
		t.Fatalf("attribute leaked into output:\ngot:  %q\nwant: %q", chunk.Data, wantSentence)
	}
}
