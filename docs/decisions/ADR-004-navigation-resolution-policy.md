# ADR-004 — M1 navigation resolution

**Status:** Accepted for M1 reconstruction; CI verification pending.

## Context

An address bar may contain an explicit URL, a probable host, a search query, or a prohibited URI scheme. The core must classify input deterministically without owning browser/OS effects.

## Decision

- `resolve_navigation(&str)` returns exactly `Url`, `Search`, or `Rejection`.
- Only `http` and `https` URLs are allowed. Any other recognized scheme is rejected with `unsupported_scheme`.
- An explicit malformed HTTP(S) URL is rejected with `invalid_url`, not searched.
- Recognizable bare hostnames are prefixed with `https://`. `localhost` and dotted-quad IPv4 literals are prefixed with `http://`. URL parsing/normalization is delegated to the pure Rust `url` crate.
- `example.com:443/path` normalizes to `https://example.com/path`.
- Ambiguous numeric forms (`1.2.3`, `127.1`) remain search queries.
- Leading/trailing whitespace around the entire input is trimmed; ordinary free text remains a search query.
- Returning `Url` does not authorize or execute a network request. Hosts retain responsibility for network policy, redirects, and execution.

## Explicitly out of scope

Search-engine selection, DNS, networking, browser rendering, WebView integration, navigation history, tab state, redirect safety checks, platform adapters, and generalized RFC/WHATWG compatibility claims.

## Evidence gate

30 tabular cases plus targeted negative tests and Linux/macOS/Windows CI; WASM compilation on Linux. Do not mark M1 done without passing evidence bound to the actual commit.
