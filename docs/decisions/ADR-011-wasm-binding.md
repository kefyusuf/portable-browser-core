# ADR-011 — M8 WASM host binding

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead); implemented with a Node-executed `wasm-bindgen-test` suite on `feat/m8-wasm-binding`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

The core is pure Rust: deterministic decisions, no I/O, WASM-compilable since M1. Item (e) of the approved roadmap opens the first bridge to a real host runtime — JavaScript — without letting host concerns leak into the core. The binding must stay as thin as the repo's boundary discipline demands: it converts types, never makes decisions.

## Decision

- **Separate crate, pinned dependencies.** New `crates/browser-core-wasm` with `crate-type = ["cdylib", "rlib"]`, depending on `browser-core`, `wasm-bindgen =0.2.129`, and `js-sys` on the same `0.2.x` track — exact-pinned and committed to `Cargo.lock`, matching the repo's reproducibility discipline. The `browser-core` crate itself gains **no** new dependencies and no wasm-specific code: the bridge is a host adapter, exactly the category earlier ADRs kept out of the core.
- **Thin, total, decision-free surface.**
  - `resolve_navigation(input) -> { kind: "url" | "search" | "reject", value, reason? }` — a total function; any input yields a decision, never an exception.
  - `WorkspaceBinding` — one flat method per M7 event path (`newWindow`, `closeWindow`, `activateWindow`, `containerNewTab(w, url?)`, `containerCloseTab`, `containerActivateTab`, `containerNavigate(w, tab, url)`, `containerBack/Forward/Reload/Commit/Fail/Stop`, `containerMoveTab(w, tab, to)`, `containerSetTitle(w, tab, title?)`), plus `sessionText` get/set passthrough to M6. Every mutating method returns the **full workspace snapshot** as a plain JS object: `{ activeWindowId, windows: [{ id, isActive, container: { activeTabId, tabs: [{ id, isActive, title, phase, currentUrl, pendingUrl, canBack, canForward }] } }], effect: { type: "start_load", url } | { type: "none" }, noop: reason? }`.
  - Invalid events return `{ noop }` results, never throw — the core's total no-op semantics carried across the bridge verbatim.
  - The binding layer contains zero policy. If a line of JS-bridge code could be described as a decision, it belongs in `browser-core`.
- **Snapshots as plain objects, no serde.** Snapshot objects are built directly with `js_sys` types. This avoids pulling `serde`/`serde_json`/`serde-wasm-bindgen` into the tree for a format the golden fixtures already pin in Rust. The shape mirrors the Rust views 1:1 and is the bridge's own pinned contract, tested from JS.
- **Boundary restated for the host language.** `resolve_navigation` returning a URL and `effect: { type: "start_load" }` are **requests**. The JS host owns DNS, fetches, redirects, and execution — exactly as ADR-004 states for Rust hosts. Nothing in the binding performs I/O.
- **CI extension.** The existing 3-OS Rust gates stay untouched. On Ubuntu, Core CI additionally: (1) builds the cdylib for `wasm32-unknown-unknown`; (2) installs `wasm-bindgen-cli` pinned to the same `0.2.129` and runs a `wasm-bindgen-test` suite in **Node** covering navigation classification, workspace lifecycle, effect surfacing, no-op reasons, and session passthrough. WASM moves from *compilation-only* evidence to *executed* evidence for the binding layer.
- **Determinism unchanged.** The bridge is synchronous and stateless between calls; the same method sequence on a fresh `WorkspaceBinding` yields identical snapshots on every platform and in every JS runtime.
- **Implementation refinements (recorded 2026-10-27 at implementation time):** the session passthrough is exposed per window as `containerSessionText(window)` / `containerSetSessionText(window, text)` because the M6 format is per-container; a workspace-level session format remains future work. On routed events the surfaced `noop` is the **innermost** reason that explains the non-transition — tab machine (`at_boundary`, `unexpected_event`, `nothing_to_reload`), then container (`unknown_tab`, `index_out_of_range`, `invalid_title`), then workspace (`unknown_window`) — so the bridge's total semantics hold at every layer.

## Explicitly out of scope

npm publishing/packaging, WebView or browser extensions adapters, async APIs, fetch/HTTP over the bridge, serde/JSON-schema layers, framework integrations (React/Vue/etc.), TypeScript-specific surface beyond wasm-bindgen's generated `.d.ts`, workspace-level session format (M6 covers single containers), and generalized JS compatibility claims.

## Evidence gate

wasm-bindgen-test suite executed in Node on CI (not merely compiled), exact-pinned `Cargo.lock`, all pre-existing Rust gates green, evidence bound to the exact commit. Do not mark M8 done without passing evidence.
