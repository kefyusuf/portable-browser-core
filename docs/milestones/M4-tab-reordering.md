# M4 — Tab reordering

**State: IMPLEMENTING** — semantics accepted 2026-10-27; implementation and fixtures on the branch, CI and independent acceptance pending.

## Deliverable

An extension of the M3 container: host-initiated repositioning of a tab within the strip, with tab identity strictly separated from tab position. Golden container scenarios and the same CI gates.

## Acceptance criteria

- [x] `MoveTab(id, to_index)` repositions the addressed tab; tab ids never change on a move.
- [x] Activation follows the tab identity, not the position: moving the active tab keeps it active; moving any tab never changes which id is active.
- [x] Closing after a reorder still uses the ADR-006 right-else-left neighbor policy on current positions.
- [x] Events addressing a nonexistent tab are no-ops (`unknown_tab`); out-of-range target indexes are no-ops (`index_out_of_range`); moving a tab to its current index is a valid no-transition.
- [x] No interaction with load state: reordering a loading tab neither cancels nor restarts its load.
- [x] Golden reordering scenarios exist with exact-count drift guards (9 scenarios, 44 steps total; source inspection until CI).
- [ ] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head.
- [ ] Independent acceptance of ADR-007 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No multiple windows, tab pinning/muting, drag-and-drop gesture semantics, titles, persistence, platform adapters, or WASM JavaScript bindings until this milestone is accepted.
