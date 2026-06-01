#[cfg(test)]
mod tests {
    use crate::renderer::place_finder::find_plain_text_start;

    #[test]
    fn finds_text_without_nested_tags() {
        let html =
            "<p>hello world</p><p>this is a test</p>";

        let result =
            find_plain_text_start(html, "this is a test")
                .unwrap();

        assert_eq!(
            result,
            "<p>hello world</p>".len()
        );
    }

    #[test]
    fn finds_text_across_nested_tags() {
        let html =
            "<p>hello world</p><p>this is a <em>test</em></p>";

        let result =
            find_plain_text_start(html, "this is a test")
                .unwrap();

        assert_eq!(
            result,
            "<p>hello world</p>".len()
        );
    }

    #[test]
    fn returns_start_of_immediately_preceding_tag() {
        let html =
            "<div><p>this is a test</p></div>";

        let result =
            find_plain_text_start(html, "this is a test")
                .unwrap();

        assert_eq!(result, 5);
    }

    #[test]
    fn does_not_return_outer_tag_when_inner_tag_is_closer() {
        let html =
            "<div><p>this is a test</p></div>";

        let result =
            find_plain_text_start(html, "this is a test")
                .unwrap();

        assert_eq!(
            &html[result..result + 3],
            "<p>"
        );
    }

    #[test]
    fn returns_none_when_text_not_found() {
        let html = "<p>hello world</p>";

        let result =
            find_plain_text_start(html, "goodbye");

        assert_eq!(result, None);
    }

    #[test]
    fn matches_book_example_with_headings_before_paragraph() {
        let html = concat!(
            "<h1>CHAPTER 1</h1>",
            "<h1>IA</h1>",
            "<p>SHE WAS SEVENTEEN, and her life was about to...</p>"
        );

        let result = find_plain_text_start(
            html,
            "SHE WAS SEVENTEEN, and her life was about to..."
        )
        .unwrap();

        assert_eq!(
            &html[result..result + 3],
            "<p>"
        );
    }

    #[test]
    fn matches_when_every_word_is_separated_by_tags() {
        let html =
            "<p>this <b>is</b> a <em>test</em></p>";

        let result =
            find_plain_text_start(html, "this is a test")
                .unwrap();

        assert_eq!(result, 0);
    }

    #[test]
    fn ignores_multiple_spaces_in_needle() {
        let html = "<p>hello world</p><p>this is a test</p><p>goodbye world</p>";

        let result =
            find_plain_text_start(
                html,
                "this   is    a     test",
            );

        assert!(result.is_some());
    }

        #[test]
    fn ignores_spaces_spaces_in_needle() {
        let html = "<p>hello world</p><p>this is a test</p><p>goodbye world</p>";

        let result =
            find_plain_text_start(
                html,
                "a test goodbye world",
            );

        assert!(result.is_some());
    }

}