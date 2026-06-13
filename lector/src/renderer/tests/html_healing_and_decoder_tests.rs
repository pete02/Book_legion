
#[cfg(test)]
mod html_decoder_tests {
    use crate::renderer::html_healer::*;


    #[test]
    fn can_run(){
        assert!(true);
    }

    #[test]
    fn htlm_deocder_works_with_decoded_html(){
        let html = "<div>Hello World!</div>";
        let decoded_html = decode_html(html);
        assert_eq!(decoded_html, "<div>Hello World!</div>");
    }
    #[test]
    fn htlm_deocder_strips_whitespace(){
        let html = "<div>Hello World!</div>   <div>Hello World!</div>";
        let decoded_html = decode_html(html);
        assert_eq!(decoded_html, "<div>Hello World!</div><div>Hello World!</div>");
    }
    #[test]
    fn htlm_deocder_strips_comments(){
        let html = "<div>Hello World!<!-- hidden --></div>";
        let decoded_html = decode_html(html);
        assert_eq!(decoded_html, "<div>Hello World!</div>");
    }

    #[test]
    fn test_decode_html_basic_entities() {
        let input = "&lt;div&gt;Hello &amp; goodbye&quot;&#39;&lt;/div&gt;";
        let expected = "<div>Hello & goodbye\"'</div>";

        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_decode_html_double_encoded() {
        let input = "&amp;lt;";
        let expected = "&lt;";

        // Single-pass decoding behavior
        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_decode_html_no_entities() {
        let input = "plain text";
        let expected = "plain text";

        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_decode_html_mixed_content() {
        let input = "5 &lt; 10 &amp;&amp; 10 &gt; 5";
        let expected = "5 < 10 && 10 > 5";

        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_decode_html_numeric_entity() {
        let input = "&#60;tag&#62;";
        
        // Your implementation does NOT decode numeric entities
        let expected = "<tag>";

        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_decode_html_hex_entity_not_supported() {
        let input = "&#x3C;tag&#x3E;";
        
        // Your implementation does NOT decode hex entities
        let expected = "<tag>";

        assert_eq!(decode_html(input), expected);
    }
    
    #[test]
    fn test_empty_string() {
        assert_eq!(decode_html(""), "");
    }

    #[test]
    fn test_only_entities() {
        let input = "&lt;&gt;&amp;";
        let expected = "<>&";
        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_entity_at_end_without_semicolon() {
        // Malformed entity - should pass through unchanged
        let input = "test &amp";
        assert_eq!(decode_html(input), "test &amp");
    }

    #[test]
    fn test_consecutive_entities() {
        let input = "&lt;&lt;&lt;";
        let expected = "<<<";
        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_decode_html_common_entities() {
        let input = "5 &nbsp; &copy; 2024 &mdash; test &lsquo;quote&#39;";
        let expected = "5 \u{a0} © 2024 — test ‘quote'";
        assert_eq!(decode_html(input), expected);
    }

    #[test]
    fn test_case_sensitivity() {
        // HTML entities are typically lowercase
        let input = "&LT;test&GT;";
        assert_eq!(decode_html(input), "<test>");
    }
}


#[cfg(test)]
mod html_healing_tests {
    use crate::renderer::html_healer;
    #[test]
    fn does_not_change_well_formed_html() {
        let html = "<p>Hello <em>world</em></p>";
        let healed =html_healer::heal_html(html);
        assert_eq!(healed, html);
    }

    #[test]
    fn heals_unclosed_open_tags() {
        let html = "<p>Hello <em>world";
        let healed =html_healer::heal_html(html);
        assert_eq!(healed, "<p>Hello <em>world</em></p>");
    }

    #[test]
    fn heals_orphaned_close_tags() {
        let html = "</p><p>Hello world</p>";
        let healed =html_healer::heal_html(html);
        assert_eq!(healed, "<p></p><p>Hello world</p>");
    }

    #[test]
    fn orphaned_close_tags_partner_is_at_beginning() {
        let html = "Hello</p><p>world</p>";
        let healed =html_healer::heal_html(html);
        assert_eq!(healed, "<p>Hello</p><p>world</p>");
    }


    #[test]
    fn void_elements_no_close_tags() {
        let html = "<img src='x.jpg'><p>Text";
        let healed =html_healer::heal_html(html);
        assert_eq!(healed, "<img src='x.jpg'><p>Text</p>");
    }
}

#[cfg(test)]
mod slicing_tests {
    use super::*;
    use crate::renderer::html_healer::slice_text;
    // Basic slicing
    #[test]
    fn test_basic_start_and_end() {
        assert_eq!(slice_text("hello world", Some(6), Some(11)), "world");
    }

    #[test]
    fn test_no_end_defaults_to_text_end() {
        assert_eq!(slice_text("hello world", Some(6), None), "world");
    }

    #[test]
    fn test_no_start_defaults_to_zero() {
        assert_eq!(slice_text("hello world", None, Some(5)), "hello");
    }

    #[test]
    fn test_no_start_no_end_returns_full_text() {
        assert_eq!(slice_text("hello world", None, None), "hello world");
    }

    // Multi-byte / UTF-8
    #[test]
    fn test_multibyte_clean_boundary() {
        // "café" — 'é' is 2 bytes, starts at byte 3
        assert_eq!(slice_text("café", Some(0), Some(3)), "caf");
    }

    #[test]
    fn test_multibyte_inside_char_snaps_forward() {
        // 'é' occupies bytes 3..5, so byte 4 is inside it
        // snapping forward should land at byte 5 (after 'é'), giving ""
        assert_eq!(slice_text("café", Some(4), None), "");
    }

    #[test]
    fn test_curly_quote_boundary() {
        // "it\u{2019}s" — ' is 3 bytes (e2 80 99), starts at byte 2
        let s = "it\u{2019}s";
        assert_eq!(slice_text(s, Some(0), Some(2)), "it");
    }

    #[test]
    fn test_curly_quote_mid_char_snaps_forward() {
        // byte 3 and 4 are inside the 3-byte ', snapping lands at byte 5 → "s"
        let s = "it\u{2019}s";
        assert_eq!(slice_text(s, Some(3), None), "s");
        assert_eq!(slice_text(s, Some(4), None), "s");
    }

    // Edge cases: out-of-bounds numbers
    #[test]
    fn test_start_beyond_len_returns_empty() {
        assert_eq!(slice_text("hello", Some(999), None), "");
    }

    #[test]
    fn test_end_beyond_len_clamps_to_end() {
        assert_eq!(slice_text("hello", None, Some(999)), "hello");
    }

    #[test]
    fn test_start_and_end_beyond_len_returns_empty() {
        assert_eq!(slice_text("hello", Some(999), Some(1000)), "");
    }

    // Edge cases: empty input
    #[test]
    fn test_empty_string_no_options() {
        assert_eq!(slice_text("", None, None), "");
    }

    #[test]
    fn test_empty_string_with_options() {
        assert_eq!(slice_text("", Some(0), Some(0)), "");
    }

    // Edge cases: zero-length slice
    #[test]
    fn test_start_equals_end_returns_empty() {
        assert_eq!(slice_text("hello", Some(2), Some(2)), "");
    }

    #[test]
    fn test_start_zero_end_zero_returns_empty() {
        assert_eq!(slice_text("hello", Some(0), Some(0)), "");
    }
}