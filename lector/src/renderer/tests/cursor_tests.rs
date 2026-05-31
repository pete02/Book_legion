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

#[cfg(test)]
mod end_tests {
    use crate::renderer::get_save_slice;
    #[test]
    fn test_returns_at_most_1000_chars() {
        let html = "a".repeat(2000);
        let result = get_save_slice(&html,  0);
        assert!(result.len() <= 1000);
    }

    #[test]
    fn test_returns_less_if_html_shorter_than_1000() {
        let html = "<p>short</p>".to_string();
        let result = get_save_slice(&html,  0);
        assert_eq!(result, "<p>short</p>");
    }

    #[test]
    fn test_does_not_cut_inside_opening_tag() {
        // 1000 chars lands inside <p class="x">
        let padding = "a".repeat(995);
        let html = format!("{}<p class=\"x\">hello</p>", padding);
        let result = get_save_slice(&html,  0);
        assert!(!result.contains("<p class="));
    }
    #[test]
    fn test_empty_html_returns_empty() {
        let result = get_save_slice("",  0);
        assert_eq!(result, "");
    }

    #[test]
    fn test_start_beyond_html_returns_empty() {
        let result = get_save_slice("<p>hi</p>",  999);
        assert_eq!(result, "");
    }
}