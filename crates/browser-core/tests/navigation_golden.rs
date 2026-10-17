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
fn colon_slashes_outside_leading_scheme_are_not_unsupported() {
    assert_eq!(
        resolve_navigation("how do I use https://example.com"),
        NavigationDecision::Search("how do I use https://example.com".to_owned())
    );
    assert_eq!(
        resolve_navigation("example.com/guide://section"),
        NavigationDecision::Url("https://example.com/guide://section".to_owned())
    );
    assert_eq!(
        resolve_navigation("example.com:443/guide://section"),
        NavigationDecision::Url("https://example.com/guide://section".to_owned())
    );
    assert_eq!(
        resolve_navigation("mailto://user@example.com"),
        NavigationDecision::Rejection(RejectionReason::UnsupportedScheme)
    );
}

#[test]
fn at_sign_in_bare_host_path_query_or_fragment_is_not_userinfo() {
    for (input, expected) in [
        (
            "example.com/contact@support",
            "https://example.com/contact@support",
        ),
        (
            "example.com/?contact=staff@example.com",
            "https://example.com/?contact=staff@example.com",
        ),
        ("example.com/#team@home", "https://example.com/#team@home"),
        (
            "localhost:8000/contact@support",
            "http://localhost:8000/contact@support",
        ),
    ] {
        assert_eq!(
            resolve_navigation(input),
            NavigationDecision::Url(expected.to_owned()),
            "unexpected classification for {input:?}"
        );
    }
}

#[test]
fn userinfo_in_bare_host_authority_does_not_infer_url() {
    assert_eq!(
        resolve_navigation("user@example.com/path"),
        NavigationDecision::Search("user@example.com/path".to_owned())
    );
    assert_eq!(
        resolve_navigation("example.com@evil.test/path"),
        NavigationDecision::Search("example.com@evil.test/path".to_owned())
    );
}

#[test]
fn leading_zero_ipv4_quads_remain_searches() {
    // WHATWG IPv4 parsing can treat leading-zero components as octal.
    // A bare numeric host must not silently resolve to another address.
    for input in [
        "010.0.0.1",
        "001.2.3.4",
        "127.000.0.1",
        "08.0.0.1",
        "000.0.0.0",
    ] {
        assert_eq!(
            resolve_navigation(input),
            NavigationDecision::Search(input.to_owned()),
            "unexpected numeric host classification for {input:?}"
        );
    }
}

#[test]
fn hexadecimal_ending_ipv4_like_hosts_remain_searches() {
    // WHATWG URL parsing recognizes 0x-prefixed host suffixes as IPv4 numbers.
    for input in [
        "127.0.0.0x",
        "127.0.0.0X",
        "127.0.0.0x1",
        "127.0.0.0X1",
        "example.0x",
        "example.0xA",
    ] {
        assert_eq!(
            resolve_navigation(input),
            NavigationDecision::Search(input.to_owned()),
            "unexpected numeric-like hostname classification for {input:?}"
        );
    }
}

#[test]
fn ordinary_alphabetic_hostnames_still_infer_https() {
    assert_eq!(
        resolve_navigation("example.technology"),
        NavigationDecision::Url("https://example.technology/".to_owned())
    );
}

#[test]
fn unambiguous_ipv4_quads_still_infer_http() {
    for (input, expected) in [
        ("0.0.0.0", "http://0.0.0.0/"),
        ("10.0.0.1", "http://10.0.0.1/"),
        ("192.168.1.1", "http://192.168.1.1/"),
    ] {
        assert_eq!(
            resolve_navigation(input),
            NavigationDecision::Url(expected.to_owned()),
            "unexpected numeric host classification for {input:?}"
        );
    }
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
fn embedded_ascii_controls_fail_closed() {
    for input in [
        "https://exa\nmple.com",
        "https://exa\tmple.com",
        "example.com/pa\rth",
        "exa\tmple.com",
        "foo\nbar",
        "https://example.com/\u{0}data",
        "https://example.com/\u{7f}data",
    ] {
        assert_eq!(
            resolve_navigation(input),
            NavigationDecision::Rejection(RejectionReason::InvalidUrl),
            "unexpected decision for {input:?}"
        );
    }
}

#[test]
fn outer_whitespace_remains_trimmable() {
    assert_eq!(
        resolve_navigation("  https://example.com/  "),
        NavigationDecision::Url("https://example.com/".to_owned())
    );
}

#[test]
fn unsupported_ftp_is_not_a_search() {
    assert_eq!(
        resolve_navigation("ftp://example.com"),
        NavigationDecision::Rejection(RejectionReason::UnsupportedScheme)
    );
}
