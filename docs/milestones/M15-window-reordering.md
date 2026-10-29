# M15 — Window reordering

**State: IMPLEMENTING** — ADR-018 accepted 2026-10-27; implementation on the branch, CI and independent acceptance pending.

## Deliverable

`WorkspaceEvent::MoveWindow(id, to_index)` — the ADR-007 tab-reordering rule applied at the window level: identity strictly separated from position, activation follows the id, out-of-range targets are reasoned no-ops. Golden scenarios and the same CI gates.

## Acceptance criteria

- [x] `MoveWindow(id, to_index)` repositions the addressed window; window ids never change on a move; containers (tabs, titles, favicons, load state, history) travel with the identity.
- [x] Activation follows the identity: moving the active window keeps it active; moving any window never changes which id is active.
- [x] The ADR-006 close-activation policy (and the ADR-007 close-after-reorder rule) continue to operate on current positions after a workspace reorder.
- [x] Events addressing a nonexistent window are no-ops (`unknown_window`); out-of-range target indexes are no-ops (`index_out_of_range`); moving a window to its current index is a valid no-transition.
- [x] Golden workspace scenarios extend with reordering rows and exact-count drift guards (6 scenarios / 32 steps).
- [ ] Multi-platform CI green bound to the final exact PR head (3-OS Rust + Ubuntu cdylib build + Node binding tests).
- [ ] Independent acceptance of ADR-018 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No drag-and-drop gesture semantics, no workspace session format changes, no platform adapters until this milestone is accepted.
