# M8 — WASM host binding

**State: IMPLEMENTING** — ADR-011 accepted 2026-10-27; implementation on the branch, CI and independent acceptance pending.

## Deliverable

A thin, decision-free JavaScript bridge over the existing core: a new `crates/browser-core-wasm` cdylib exposing M1 navigation decisions and the M7 workspace to JS via wasm-bindgen, with no policy of its own. The pure core crate stays dependency-pure (only `url`).

## Acceptance criteria

- [x] New crate `crates/browser-core-wasm` with `crate-type = ["cdylib", "rlib"]`; new dependencies exact-pinned (`wasm-bindgen =0.2.129`, `js-sys =0.3.106`) and committed to `Cargo.lock`.
- [x] `browser-core` crate itself gains **no** new dependencies and no wasm-specific code.
- [x] `resolve_navigation` exposed to JS as a total function returning `{ kind, value, reason? }` — never throws on any input.
- [x] A `WorkspaceBinding` class exposing flat 1:1 methods for every M7 event path (windows, container events incl. tabs, moves, titles), each returning the full workspace snapshot as a plain JS object; invalid events return a `{ noop }` result instead of throwing.
- [x] Surfaced effects are returned as `{ type: "start_load", url } | { type: "none" }`; a returned URL remains a request, never an authorization (ADR-004 boundary restated for JS).
- [x] Session text passthrough exposed (per window: `containerSessionText`/`containerSetSessionText`) so M6 persistence works unchanged over the bridge.
- [x] The binding layer contains zero policy: every decision remains in `browser-core`; JS glue only converts types.
- [x] wasm-bindgen-test suite running in Node on Ubuntu CI covers: navigation classification, workspace lifecycle, effect surfacing, no-op reasons (innermost reason surfaced), and session passthrough.
- [ ] CI green bound to the final exact PR head: existing 3-OS Rust gates unchanged, plus the new Ubuntu wasm steps (cdylib build + Node test run).
- [ ] Independent acceptance of ADR-011 for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No npm publishing or packaging, no WebView/browser adapter, no async/HTTP APIs over the bridge, no serde/JSON-schema layer, no TypeScript-specific API beyond wasm-bindgen's generated definitions, no framework integrations until this milestone is accepted.
