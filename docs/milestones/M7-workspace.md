# M7 — Window set (workspace)

**State: DONE** — accepted and merged 2026-10-27.

## Deliverable

A pure, deterministic ordered set of M3 containers ("windows") with deterministic window identities, closing with the same activation policy as tabs, window activation, and id-addressed routing of container events. Golden workspace scenarios and the same CI gates.

## Acceptance criteria

- [x] Public workspace API exists and is total: every (event, state) combination returns an outcome, never panics.
- [x] Window identities are deterministic (sequential `u64` from 1, never reused) across a replayed event sequence.
- [x] `NewWindow` appends at the end and activates; windows start with an empty container.
- [x] Closing the active window activates its right neighbor, else its left neighbor; closing a background window keeps the active window; the workspace may become empty.
- [x] `Container(id, event)` routes to the addressed window's M3 container without duplicating container semantics; the inner container outcome is surfaced and its effect (at most one `StartLoad`) surfaces on the workspace outcome.
- [x] Events addressing a nonexistent window are reasoned no-ops (`unknown_window`); window activation never triggers loads.
- [x] The workspace view exposes ordered window summaries (id, active flag, full container view) plus the active window id.
- [x] Golden workspace scenarios exist with exact-count drift guards (4 scenarios, 19 steps; source inspection until CI).
- [x] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head. *(Core CI run [40549969877](https://github.com/kefyusuf/portable-browser-core/actions/runs/40549969877) PASS on head `caa01b9`.)*
- [x] Independent acceptance of ADR-010 semantics for the final exact HEAD. *(Maintainer accepted 2026-10-27 by approving the merge of PR #7. Semantics acceptance recorded 2026-10-27 as item (d) of the approved a–e sequence; the local Windows verification record is in the PR evidence comment.)*

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No workspace session serialization (M6 covers single containers), window resizing/geometry, screen/monitor concepts, platform adapters, or WASM JavaScript bindings until this milestone is accepted.
