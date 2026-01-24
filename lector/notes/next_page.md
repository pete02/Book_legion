1. Entry Point: render_next_page

This is the function that triggers when the user wants to go to the next page.

pub fn render_next_page(text_handler: &mut TextHandler)

Flow:

Check if chapter is finished:

if (text_handler.chapter_end)() == true {
    next_chapter(text_handler);
    return;
}


If the chapter has ended, it immediately moves to the next chapter via next_chapter.

Otherwise, it continues to render the next page in the current chapter.

Save the cursor (reading progress):

save_cursor(text_handler.clone());


Saves the current reading position asynchronously.

Uses the start_text as a reference to persist progress in the backend.

Determine starting point for new page:

let chapter = (text_handler.chapter)();
let start_text = (text_handler.start_text)();
let start_offset = find_sentence_offset_with_html_backtrack(&chapter, &start_text);


Finds the position in the chapter HTML where the new page should start.

Ensures it does not break HTML tags mid-way (*_with_html_backtrack).

Set visible text for the page:

let new_visible = chapter[start_offset..].to_string();
text_handler.visible_text.set(new_visible);


Schedule trimming overflowing text asynchronously:

let closure = Closure::once_into_js(move || {
    trim_overflowing_node(&mut handler_for_trim);
});
window.set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), 0);


The visible text is updated, but some may overflow the container.

Uses trim_overflowing_node to hide overflowing words and prepare the next page.

2. Saving Cursor (save_cursor)

Keeps track of where the reader left off.

Flow:

Checks if start_text has content:

If yes: Fetches cursor position from infra using get_cursor_from_text and saves it.

If no: Loads existing cursor, updates chapter index, and saves.

Async task using spawn ensures non-blocking execution.

3. Finding the Correct Start Offset

Two functions:

a. find_sentence_offset_with_html_backtrack

Calls find_sentence_offset to locate the snippet in the chapter.

Adjusts backward to avoid starting mid-HTML tag.

Returns a "safe" start index in the chapter HTML.

b. find_sentence_offset

Splits start_text into sentences.

Searches for the first sentence in the chapter HTML.

If multiple occurrences, iterates to find the one where subsequent sentences also match.

Returns index of first matched sentence.

4. Trimming Overflowing Text (trim_overflowing_node)

Ensures the visible text fits in the page container.

Flow:

Clear start_text.

Identify container element in DOM:

let container = document.get_element_by_id("book-renderer")


Identify the first child element that overflows container:

let Some(child)=first_overflowing_child(&container)


Returns (child_element, completely_outside) flag.

If completely outside:

Set start_text to the child’s full inner text for the next page.

Mark chapter as ended if no child found.

If partially overflowing:

Split child into visible and hidden text: split_node_by_visible_words

Snap to the last sentence boundary: snap_to_last_sentence_break

Split DOM node into two elements and hide overflow: split_and_hide_node_in_chapter

Set start_text to the hidden text for next page.

5. Handling DOM Overflow
a. first_overflowing_child

Iterates child nodes of container.

Checks get_bounding_client_rect to find which node exceeds container bounds.

Returns either:

Entire node is outside → completely new page.

Node partially visible → needs splitting.

b. split_node_by_visible_words

Splits a node’s text word by word.

Uses DOM Range API to check if adding a word overflows container.

Returns (visible_text, hidden_text).

c. snap_to_last_sentence_break

Adjust visible portion to the last sentence boundary.

Avoids cutting sentences mid-way.

Appends leftover text to hidden portion.

d. split_and_hide_node_in_chapter

Creates two new DOM nodes:

visible_node → remains visible.

hidden_node → hidden, used as starting point for next page.

Updates chapter HTML in text_handler.

6. Text Utilities

split_sentences: Regex-based sentence splitting.

normalize_text: Strips punctuation, lowercases for matching.

strip_html: Removes all HTML tags for text comparison.

collect_text_nodes: Helper for iterating nested text nodes.

7. Page Turn Summary Flow

Step-by-step:

User triggers next page → render_next_page

Check chapter end

If end → next_chapter → fetch new chapter → return

Save reading cursor

Determine new start offset

Find snippet in HTML

Adjust to avoid breaking HTML tags

Update visible text

Schedule DOM trimming

Trimming process (trim_overflowing_node)

Find overflowing child

Fully outside → move whole node to start_text

Partially → split node

Split visible vs hidden words

Snap visible to last sentence

Update chapter DOM

Store hidden text as start_text

Next page is ready

When user presses next → repeat

8. Key Points

Cursor management: Ensures the user can resume reading where they left off.

HTML-aware offset: Avoids breaking tags when calculating page breaks.

Word-level and sentence-level splitting: Guarantees readable pages.

DOM manipulation: Updates nodes without losing HTML structure.

Async handling: set_timeout allows rendering before measuring overflow.

Fallbacks: Handles empty chapters, missing nodes, or chapters ending.