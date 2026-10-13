use browser_core::{resolve_navigation, NavigationDecision, RejectionReason};

#[test]
fn navigation_golden_matrix() {
    let mut checked = 0usize;

    for (index, line) in include_str!("golden/navigation.tsv").lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 3, "bad golden row {}", index + 1);
        let actual = resolve_navigation(fields[0]);
        let (kind, value) = match actual {
            NavigationDecision::Url(value) => ("url", value),
            NavigationDecision::Search(value) => ("search", value),
            NavigationDecision::Rejection(reason) => ("reject", reason.as_str().to_owned()),
        };
        assert_eq!(kind, fields[1], "kind at row {}", index + 1);
        assert_eq!(value, fields[2], "value at row {}", index + 1);
        checked += 1;
    }

    assert_eq!(checked, 30, "M1 fixture count drift");
}

#[test]
fn malformed_http_url_fails_closed() {
    assert_eq!(
        resolve_navigation("https://"),
        NavigationDecision::Rejection(RejectionReason::InvalidUrl)
    );
    assert_eq!(
        resolve_navigation("http:example.com"),
        NavigationDecision::Rejection(RejectionReason::InvalidUrl)
    );
    assert_eq!(
        resolve_navigation("example.com:99999"),
        NavigationDecision::Rejection(RejectionReason::InvalidUrl)
    );
}

#[test]
fn unicode_input_does_not_panic_on_scheme_detection() {
    assert_eq!(
        resolve_navigation("a😊:search"),
        NavigationDecision::Search("a😊:search".to_owned())
    );
}

#[test]
fn unsupported_ftp_is_not_a_search() {
    assert_eq!(
        resolve_navigation("ftp://example.com"),
        NavigationDecision::Rejection(RejectionReason::UnsupportedScheme)
    );
}
