use url::Url;

/// A decision only. Host runtimes own all actual side effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NavigationDecision {
    Url(String),
    Search(String),
    Rejection(RejectionReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectionReason {
    EmptyInput,
    UnsupportedScheme,
    InvalidUrl,
}

impl RejectionReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EmptyInput => "empty_input",
            Self::UnsupportedScheme => "unsupported_scheme",
            Self::InvalidUrl => "invalid_url",
        }
    }
}

/// Classify an address-bar input without performing I/O.
///
/// Explicit HTTP(S) URLs and recognizable bare hostnames produce URLs.
/// Ambiguous numeric dot sequences (e.g. 127.1) remain search queries.
/// Other URI schemes are denied rather than sent to a search provider.
pub fn resolve_navigation(raw: &str) -> NavigationDecision {
    let input = raw.trim();
    if input.is_empty() {
        return NavigationDecision::Rejection(RejectionReason::EmptyInput);
    }

    // URL parsers may silently strip embedded ASCII controls (e.g. newlines).
    // Reject them before URL classification or normalization.
    if input.bytes().any(|byte| byte.is_ascii_control()) {
        return NavigationDecision::Rejection(RejectionReason::InvalidUrl);
    }

    if let Some((scheme, _)) = input.split_once("://") {
        // Only a leading valid URI-scheme token makes this an explicit scheme.
        // A colon-slash sequence in search text or a bare-host path is not one.
        if is_scheme_name(scheme) {
            if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
                return NavigationDecision::Rejection(RejectionReason::UnsupportedScheme);
            }
            return parse_http_url(input);
        }
    }

    // A malformed explicit HTTP URL is never silently downgraded to a search.
    if starts_with_scheme(input, "http") || starts_with_scheme(input, "https") {
        return NavigationDecision::Rejection(RejectionReason::InvalidUrl);
    }

    if looks_like_bare_host(input) {
        let authority = input.split(['/', '?', '#']).next().unwrap_or(input);
        let host = authority.split(':').next().unwrap_or(authority);
        let scheme = if host.eq_ignore_ascii_case("localhost") || is_ipv4_quad(host) {
            "http"
        } else {
            "https"
        };
        return parse_http_url(&format!("{scheme}://{input}"));
    }

    if has_scheme_prefix(input) {
        return NavigationDecision::Rejection(RejectionReason::UnsupportedScheme);
    }

    NavigationDecision::Search(input.to_owned())
}

fn parse_http_url(candidate: &str) -> NavigationDecision {
    match Url::parse(candidate) {
        Ok(url) if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() => {
            NavigationDecision::Url(url.to_string())
        }
        _ => NavigationDecision::Rejection(RejectionReason::InvalidUrl),
    }
}

fn starts_with_scheme(input: &str, scheme: &str) -> bool {
    input.get(..scheme.len()).is_some_and(|prefix| {
        prefix.eq_ignore_ascii_case(scheme)
            && input
                .get(scheme.len()..)
                .is_some_and(|suffix| suffix.starts_with(':'))
    })
}

fn has_scheme_prefix(input: &str) -> bool {
    input
        .split_once(':')
        .is_some_and(|(prefix, _)| is_scheme_name(prefix))
}

fn is_scheme_name(prefix: &str) -> bool {
    let mut chars = prefix.bytes();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.'))
}

fn looks_like_bare_host(input: &str) -> bool {
    if input.chars().any(char::is_whitespace) {
        return false;
    }

    let authority = input.split(['/', '?', '#']).next().unwrap_or(input);
    // Guard the authority, not the path/query/fragment, against userinfo.
    if authority.contains('@') {
        return false;
    }
    let host = authority.split(':').next().unwrap_or(authority);

    if host.eq_ignore_ascii_case("localhost") || is_ipv4_quad(host) {
        return true;
    }

    // Do not interpret shortened IPv4 spellings like 127.1 as domain names.
    if !host.is_ascii() || !host.contains('.') {
        return false;
    }

    let mut last_label = "";
    for label in host.split('.') {
        if label.is_empty()
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return false;
        }
        last_label = label;
    }

    // WHATWG can interpret hexadecimal-ending host labels as IPv4 numbers.
    // Do not infer a URL that silently normalizes to a different address.
    let hexadecimal_suffix = last_label
        .strip_prefix("0x")
        .or_else(|| last_label.strip_prefix("0X"))
        .is_some_and(|digits| {
            digits.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
    if hexadecimal_suffix {
        return false;
    }

    last_label.len() >= 2 && last_label.bytes().any(|c| c.is_ascii_alphabetic())
}

fn is_ipv4_quad(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|c| c.is_ascii_digit())
                // WHATWG URL parsing may interpret leading-zero values as octal.
                && (*part == "0" || !part.starts_with('0'))
                && part.parse::<u8>().is_ok()
        })
}
