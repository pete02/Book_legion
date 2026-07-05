#[cfg(test)]
mod tests {
    use crate::domain::text::strip_color_from_css;
    #[test]
    fn test_removes_color_property() {
        let css = "p { color: red; margin: 12px; }";
        let result = strip_color_from_css(css);
        assert!(!result.contains("color: red"));
        assert!(result.contains("margin : 12px"));
    }

    #[test]
    fn test_removes_color_with_hex_value() {
        let css = "p { color: #ff0000; }";
        let result = strip_color_from_css(css);
        assert!(!result.contains("color"));
    }

    #[test]
    fn test_removes_background_color() {
        let css = "p { background-color: blue; margin: 12px; }";
        let result = strip_color_from_css(css);
        assert!(!result.contains("background-color"));
        assert!(result.contains("margin: 12px"));
    }

    #[test]
    fn test_preserves_non_color_properties() {
        let css = "p { margin: 10px; }";
        let result = strip_color_from_css(css);
        assert_eq!(result, css);
    }

    #[test]
    fn test_empty_css_returns_empty() {
        assert_eq!(strip_color_from_css(""), "");
    }

    #[test]
    fn test_removes_color_case_insensitive() {
        let css = "p { Color: red; BACKGROUND-COLOR: blue; }";
        let result = strip_color_from_css(css);
        assert!(!result.to_lowercase().contains("color"));
    }

    #[test]
    fn test_removes_font_size_property() {
        let css = "p { font-size: 12px; margin: 10px; }";
        let result = strip_color_from_css(css);
        assert!(!result.contains("font-size"));
        assert!(result.contains("margin: 10px"));
    }

    #[test]
    fn test_removes_font_size_with_em_value() {
        let css = "p { font-size: 1.2em; }";
        let result = strip_color_from_css(css);
        assert!(!result.contains("font-size"));
    }

    #[test]
    fn test_removes_font_size_case_insensitive() {
        let css = "p { FONT-SIZE: 14px; }";
        let result = strip_color_from_css(css);
        assert!(!result.to_lowercase().contains("font-size"));
    }

    #[test]
    fn test_removes_font_size_but_preserves_font_family() {
        let css = "p { font-size: 12px; font-family: Arial; }";
        let result = strip_color_from_css(css);
        assert!(!result.contains("font-size"));
        assert!(result.contains("font-family: Arial"));
    }
}

