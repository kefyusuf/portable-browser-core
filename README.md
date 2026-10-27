# Portable Browser Core

A platform-independent browser decision core. The current executable slice is deterministic address/navigation resolution in Rust.

**Status:** M1 **DONE** — accepted and merged 2026-10-25 (PR #1, accepted head `9779823`, merge commit `17cf8c7`, [CI run 40220530561](https://github.com/kefyusuf/portable-browser-core/actions/runs/40220530561)).

## Frozen boundary

- Core: deterministic input -> decision. No OS APIs, WebView, browser UI, networking, filesystem, or persistence.
- Host/browser adapters are not part of M1.
- No tabs, history, bookmarks, rendering, or cross-platform wrappers before M1 is accepted.

## M1 API

`browser_core::resolve_navigation(&str) -> NavigationDecision`

The result is exactly one of `Url(String)`, `Search(String)`, or `Rejection(RejectionReason)`. URL resolution **does not load or navigate** to the URL. Search resolution returns a query, not a search-engine URL.

## Verify

Requires Rust 1.98.1. From repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
rustup target add wasm32-unknown-unknown
cargo check --workspace --locked --target wasm32-unknown-unknown
```

The Cargo.lock file is checked in. CI uses --locked to reject unrecorded dependency resolution changes. CI must pass on Linux, macOS and Windows, with a WASM-target compile check. See [M1 acceptance](docs/milestones/M1-navigation-core.md).

## Milestones

- **M0:** Boundaries and repository bootstrap — present, no claim of reproducing earlier ZIP exactly.
- **M1:** Navigation resolver and deterministic golden cases — **DONE** (accepted and merged 2026-10-25).
- **M2:** Tab state machine — **DONE** (accepted and merged 2026-10-25; PR #2, head `fb1a22c`).
- **M3:** Tab container — **DONE** (accepted and merged 2026-10-26; PR #3, head `8afca7f`).
- **M4:** Tab reordering — **DONE** (accepted and merged 2026-10-27; PR #4, head `81d5b72`).
- **M5:** Host-reported tab titles — **DONE** (accepted and merged 2026-10-27; PR #5, head `69fab1f`).
- **M6:** Session persistence — **DONE** (accepted and merged 2026-10-27; PR #6, head `d4dd983`).
- **M7:** Window set (workspace) — **DONE** (accepted and merged 2026-10-27; PR #7, head `caa01b9`).
- **M8:** WASM host binding — **DONE** (accepted and merged 2026-10-27; PR #8, head `fec6208`).

The a–e roadmap is complete. No further milestone is predefined; the next scope starts from the ADRs' explicitly out-of-scope lists (workspace session format, npm packaging, WebView adapter, per-entry titles, favicons).

The prior locally discussed M0/M1 archive is not mounted here. This change reconstructs the intended behavior from recorded decisions; it does **not** assert binary/hash equivalence with that archive.
