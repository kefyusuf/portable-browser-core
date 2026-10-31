# Portable Browser Core

A platform-independent browser decision core for **agents and LLMs**:
deterministic navigation resolution, per-tab lifecycle and history, a tab
strip with session persistence, and a window set — pure and synchronous in
Rust, bridged to JavaScript via WASM. Whether the host runs it headless or
as a full browser is the host's choice; the core returns decisions and
effect requests, hosts execute (ADR-004). The agent's eyes are the
observation text (`observe_text()`, ADR-019). See [docs/architecture.md](docs/architecture.md) for
the crate map and layer diagram, and [CHANGELOG.md](CHANGELOG.md) for the
milestone history.

**Status:** M1–M17 **DONE** — the in-repo milestone sequence (navigation core → tab machine → tab strip → sessions → window set → JS bridge → packaging → docs → agentic observation) is accepted, merged, and CI-verified. Per-milestone evidence lives in each milestone document and its PR; next scopes (WebView adapter, license, publishing) await maintainer decisions.

## Frozen boundary

- Core: deterministic input -> decision. No OS APIs, WebView, browser UI, networking, filesystem, or persistence.
- Host/browser adapters are not part of M1.
- No tabs, history, bookmarks, rendering, or cross-platform wrappers before M1 is accepted.

## M1 API

`browser_core::resolve_navigation(&str) -> NavigationDecision`

The result is exactly one of `Url(String)`, `Search(String)`, or `Rejection(RejectionReason)`. URL resolution **does not load or navigate** to the URL. Search resolution returns a query, not a search-engine URL.

## WebView integration

Using the core as the decision layer of a headless or WebView-based host —
the observe → decide → act → execute loop, engine callback mapping, and
session restore — is documented in
[docs/webview-integration.md](docs/webview-integration.md).

## License

MIT — see [LICENSE](LICENSE).

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
- **M9:** Workspace session format — **DONE** (accepted and merged 2026-10-27; PR #9, head `f97e108`).
- **M10:** Workspace session over the JS bridge — **DONE** (accepted and merged 2026-10-27; PR #10, head `1a4a83b`).
- **M11:** Per-history-entry titles — **DONE** (accepted and merged 2026-10-27; PR #11, head `b32dcdb`).
- **M12:** Per-entry favicons — **DONE** (accepted and merged 2026-10-27; PR #12, head `62f0cbf`).
- **M13:** npm packaging (preparation) — **DONE** (accepted and merged 2026-10-27; PR #13, head `c45be01`).
- **M14:** Browser target outputs — **DONE** (accepted and merged 2026-10-27; PR #14, head `df83d62`).
- **M15:** Window reordering — **DONE** (accepted and merged 2026-10-29; PR #15, head `4afe22d`).
- **M16:** Documentation consolidation — **DONE** (accepted and merged 2026-10-29; PR #16, head `c642b29`).
- **M17:** Agentic purpose and observation text — **DONE** (accepted and merged 2026-10-29; PR #17, head `e9afe27`).
- **M18:** Agent loop example — **DONE** (accepted and merged 2026-10-31; PR #18, head `faa4b0e`).

The prior locally discussed M0/M1 archive is not mounted here. This change reconstructs the intended behavior from recorded decisions; it does **not** assert binary/hash equivalence with that archive.
