
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

mod html_healing_tests {
    use crate::renderer::*;
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
    fn drops_orphaned_close_tags() {
        let html = "</p><p>Hello world</p>";
        let healed =html_healer::heal_html(html);
        assert_eq!(healed, "<p>Hello world</p>");
    }

    #[test]
    fn void_elements_no_close_tags() {
        let html = "<img src='x.jpg'><p>Text";
        let healed =html_healer::heal_html(html);
        assert_eq!(healed, "<img src='x.jpg'><p>Text</p>");
    }
}