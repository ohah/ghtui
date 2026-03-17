use ghtui_core::types::Pagination;

pub fn parse_link_header(header: Option<&str>) -> Pagination {
    let mut pagination = Pagination {
        page: 1,
        per_page: 30,
        has_next: false,
        total: None,
    };

    let Some(header) = header else {
        return pagination;
    };

    for part in header.split(',') {
        let sections: Vec<&str> = part.split(';').collect();
        if sections.len() != 2 {
            continue;
        }

        let rel = sections[1].trim();
        if rel == "rel=\"next\"" {
            pagination.has_next = true;

            // Extract page number from URL
            let url = sections[0].trim().trim_matches(|c| c == '<' || c == '>');
            if let Some(page) = extract_page_param(url) {
                pagination.page = page.saturating_sub(1); // current page is prev of next
            }
        } else if rel == "rel=\"last\"" {
            let url = sections[0].trim().trim_matches(|c| c == '<' || c == '>');
            if let Some(page) = extract_page_param(url) {
                pagination.total = Some(page);
            }
        }
    }

    pagination
}

fn extract_page_param(url: &str) -> Option<u32> {
    url.split('?')
        .nth(1)?
        .split('&')
        .find(|p| p.starts_with("page="))?
        .strip_prefix("page=")?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link_header() {
        let header = r#"<https://api.github.com/repos/owner/repo/pulls?page=2>; rel="next", <https://api.github.com/repos/owner/repo/pulls?page=5>; rel="last""#;
        let pagination = parse_link_header(Some(header));
        assert!(pagination.has_next);
        assert_eq!(pagination.total, Some(5));
    }

    #[test]
    fn test_parse_link_header_no_next() {
        let header = r#"<https://api.github.com/repos/owner/repo/pulls?page=1>; rel="first""#;
        let pagination = parse_link_header(Some(header));
        assert!(!pagination.has_next);
    }

    #[test]
    fn test_parse_link_header_none() {
        let pagination = parse_link_header(None);
        assert!(!pagination.has_next);
    }
}
