# M3 — Tab container

**State: PROPOSED** — docs-first; implementation not started.

## Deliverable

A pure, deterministic container over ordered `TabMachine` instances: tab creation with deterministic identities, closing with an explicit activation policy, activation, and id-addressed routing of M2 tab events (so background tabs can report load outcomes). Golden container scenarios and the same CI gates.

## Acceptance criteria

- [ ] Public tab container API exists and is total: every (event, state) combination returns an outcome, never panics.
- [ ] Tab identities are deterministic (sequential, never reused) across a replayed event sequence.
- [ ] `NewTab` appends at the end and activates; with an initial target it navigates atomically.
- [ ] Closing the active tab activates its right neighbor, else its left neighbor; closing a background tab keeps the active tab.
- [ ] The container may become empty; the empty state is representable and the host owns empty-state presentation.
- [ ] Events addressing a nonexistent tab are reasoned no-ops (`unknown_tab`).
- [ ] Tab events route to the addressed tab's M2 machine without duplicating M2 semantics; the affected tab's outcome is surfaced.
- [ ] The container view exposes ordered tab summaries (id, active flag, full M2 state view) plus the active id.
- [ ] Golden container scenarios exist with exact-count drift guards (source inspection until CI).
- [ ] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head.
- [ ] Independent acceptance of ADR-006 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No multiple windows/containers, tab reordering, pinning or muting, titles/favicons, persistence or session restore, platform adapters, or WASM JavaScript bindings until this milestone is accepted.
