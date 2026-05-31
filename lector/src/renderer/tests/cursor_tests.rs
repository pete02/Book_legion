#[cfg(test)]
mod tests {
    use crate::renderer::find_start_offset;
    #[test]
    fn test_empty_cursor_text_returns_zero() {
        assert_eq!(find_start_offset("<p>some html</p>", ""), 0);
    }

    #[test]
    fn test_cursor_text_found_returns_its_position() {
        assert_eq!(find_start_offset("<p>hello</p><p>world</p>", "<p>world</p>"), 12);
    }

    #[test]
    fn test_cursor_text_not_found_returns_zero() {
        assert_eq!(find_start_offset("<p>hello</p>", "not in here"), 0);
    }

    #[test]
    fn test_cursor_text_is_entire_html() {
        assert_eq!(find_start_offset("<p>hello</p>", "<p>hello</p>"), 0);
    }
}