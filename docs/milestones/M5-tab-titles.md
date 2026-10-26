# M5 — Host-reported tab titles

**State: IMPLEMENTING** — semantics accepted 2026-10-27; implementation and fixtures on the branch, CI and independent acceptance pending.

## Deliverable

An extension of the M3 container: per-tab, host-reported display titles held as container state. The M2 machine and its semantics are untouched. Golden container scenarios and the same CI gates.

## Acceptance criteria

- [x] `SetTitle(id, Option<String>)` sets or clears the addressed tab's title; unknown ids are `unknown_tab` no-ops.
- [x] Titles live on the container slot, not the M2 machine: `TabMachine` and its golden fixtures are unchanged; `TabSummary` gains `title`.
- [x] The title is the last host-reported value: `navigate`, `commit`, `fail`, `stop`, `back`, `forward`, and `reload` never alter or clear it; only `SetTitle` does.
- [x] Titles travel with tab identity: reorder and activation keep titles with their id; closing a tab discards its title with the slot.
- [x] New tabs start untitled; the core never invents a title.
- [x] Golden container scenarios cover set/clear, per-tab independence, reorder/close travel, and unknown ids, with exact-count drift guards (13 scenarios, 59 steps; source inspection until CI).
- [ ] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head.
- [ ] Independent acceptance of ADR-008 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No per-history-entry titles, favicons, favicon/title inference from URLs, multiple windows, persistence, platform adapters, or WASM JavaScript bindings until this milestone is accepted.
