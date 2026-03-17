use ghtui_api::pagination::parse_link_header;

#[test]
fn test_parse_full_link_header() {
    let header = r#"<https://api.github.com/repos/owner/repo/pulls?page=2>; rel="next", <https://api.github.com/repos/owner/repo/pulls?page=5>; rel="last""#;
    let pagination = parse_link_header(Some(header));
    assert!(pagination.has_next);
    assert_eq!(pagination.total, Some(5));
}

#[test]
fn test_parse_link_header_first_page() {
    let header = r#"<https://api.github.com/repos/o/r/pulls?page=2>; rel="next", <https://api.github.com/repos/o/r/pulls?page=10>; rel="last""#;
    let pagination = parse_link_header(Some(header));
    assert!(pagination.has_next);
    assert_eq!(pagination.total, Some(10));
}

#[test]
fn test_parse_link_header_last_page() {
    let header = r#"<https://api.github.com/repos/o/r/pulls?page=1>; rel="first", <https://api.github.com/repos/o/r/pulls?page=4>; rel="prev""#;
    let pagination = parse_link_header(Some(header));
    assert!(!pagination.has_next);
    assert!(pagination.total.is_none());
}

#[test]
fn test_parse_link_header_none() {
    let pagination = parse_link_header(None);
    assert!(!pagination.has_next);
    assert!(pagination.total.is_none());
}

#[test]
fn test_parse_link_header_empty() {
    let pagination = parse_link_header(Some(""));
    assert!(!pagination.has_next);
}
