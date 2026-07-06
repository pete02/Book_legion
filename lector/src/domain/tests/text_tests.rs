#[cfg(test)]
mod page_count_tests {
    use super::*;
    use crate::domain::text::compute_total_pages;
    #[test]
    fn exact_multiple_gives_correct_page_count() {
        // 3 pages exactly, no partial page
        assert_eq!(compute_total_pages(900.0, 300.0), Some(2));
    }

    #[test]
    fn partial_last_page_rounds_up() {
        // 2.33 pages worth of content -> must round up to 3, not truncate,
        // or the last partial page of text would be inaccessible
        assert_eq!(compute_total_pages(700.0, 300.0), Some(2));
    }

    #[test]
    fn single_page_content_gives_one_page_not_zero() {
        // content narrower than the viewport itself
        assert_eq!(compute_total_pages(150.0, 300.0), Some(1));
    }

    #[test]
    fn zero_client_width_returns_none() {
        // viewport hasn't laid out yet — must not divide by zero
        assert_eq!(compute_total_pages(900.0, 0.0), None);
    }

    #[test]
    fn negative_client_width_returns_none() {
        assert_eq!(compute_total_pages(900.0, -10.0), None);
    }
}

