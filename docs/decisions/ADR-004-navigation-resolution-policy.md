# ADR-004 — M1 navigation resolution

**Status:** Proposed for M1 reconstruction. Independent policy acceptance and baseline-provenance decision pending.

## Context

An address bar may contain an explicit URL, a probable host, a search query, or a prohibited URI scheme. The core must classify input deterministically without owning browser/OS effects.

## Decision

- `resolve_navigation(&str)` returns exactly `Url`, `Search`, or `Rejection`.
- Only `http` and `https` URLs are allowed. Any other recognized scheme is rejected with `unsupported_scheme`.
- An explicit malformed HTTP(S) URL is rejected with `invalid_url`, not searched.
- A `://` sequence is treated as a URI scheme delimiter only if the prefix is a syntactically valid scheme token at the **start** of the input. Occurrences within ordinary search text or the path of a bare hostname must not cause `unsupported_scheme`.
- Recognizable bare hostnames are prefixed with `https://`. `localhost` and dotted-quad IPv4 literals are prefixed with `http://`. URL parsing/normalization is delegated to the pure Rust `url` crate.
- For inferred bare-host URLs, `@` is disallowed in the authority (userinfo-like syntax), but is permitted in the path, query, or fragment.
- `example.com:443/path` normalizes to `https://example.com/path`.
- Ambiguous numeric forms (`1.2.3`, `127.1`) remain search queries.
- Bare IPv4-looking quads with leading-zero multi-digit components (for example, `010.0.0.1`) remain search queries rather than being octal-normalized to a different destination. A single `0` component remains valid.
- Leading/trailing whitespace around the entire input is trimmed; ordinary free text remains a search query.
- Any remaining embedded ASCII control byte (C0 or DEL) is rejected as `invalid_url` before URL parsing, including within what otherwise looks like search text. This prevents silent stripping of tabs/newlines by URL parsers. It is deliberately strict because M1 classifies single-line address-bar input.
- Returning `Url` does not authorize or execute a network request. Hosts retain responsibility for network policy, redirects, and execution.

## Explicitly out of scope

Search-engine selection, DNS, networking, browser rendering, WebView integration, navigation history, tab state, redirect safety checks, platform adapters, and generalized RFC/WHATWG compatibility claims.

## Evidence gate

30 tabular cases plus targeted negative tests and Linux/macOS/Windows CI; WASM compilation on Linux. Do not mark M1 done without passing evidence bound to the actual commit.
