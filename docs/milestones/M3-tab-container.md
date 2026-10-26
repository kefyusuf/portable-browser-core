# M3 — Tab container

**State: DONE** — accepted and merged 2026-10-26.

## Deliverable

A pure, deterministic container over ordered `TabMachine` instances: tab creation with deterministic identities, closing with an explicit activation policy, activation, and id-addressed routing of M2 tab events (so background tabs can report load outcomes). Golden container scenarios and the same CI gates.

## Acceptance criteria

- [x] Public tab container API exists and is total: every (event, state) combination returns an outcome, never panics.
- [x] Tab identities are deterministic (sequential, never reused) across a replayed event sequence.
- [x] `NewTab` appends at the end and activates; with an initial target it navigates atomically.
- [x] Closing the active tab activates its right neighbor, else its left neighbor; closing a background tab keeps the active tab.
- [x] The container may become empty; the empty state is representable and the host owns empty-state presentation.
- [x] Events addressing a nonexistent tab are reasoned no-ops (`unknown_tab`).
- [x] Tab events route to the addressed tab's M2 machine without duplicating M2 semantics; the affected tab's outcome is surfaced.
- [x] The container view exposes ordered tab summaries (id, active flag, full M2 state view) plus the active id.
- [x] Golden container scenarios exist with exact-count drift guards (6 scenarios, 28 steps; source inspection until CI).
- [x] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head. *(Core CI run [40397253703](https://github.com/kefyusuf/portable-browser-core/actions/runs/40397253703) PASS on head `8afca7f`.)*
- [x] Independent acceptance of ADR-006 semantics for the final exact HEAD. *(Maintainer accepted 2026-10-26 by approving the merge of PR #3. Semantics acceptance recorded 2026-10-25 before implementation; local Windows verification (pinned 1.98.1: fmt, 23 locked tests, Clippy, WASM check) and the golden-fixture audit are recorded in the PR evidence comment.)*

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No multiple windows/containers, tab reordering, pinning or muting, titles/favicons, persistence or session restore, platform adapters, or WASM JavaScript bindings until this milestone is accepted.
