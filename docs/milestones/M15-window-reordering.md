# M15 — Window reordering

**State: DONE** — accepted and merged 2026-10-29.

## Deliverable

`WorkspaceEvent::MoveWindow(id, to_index)` — the ADR-007 tab-reordering rule applied at the window level: identity strictly separated from position, activation follows the id, out-of-range targets are reasoned no-ops. Golden scenarios and the same CI gates.

## Acceptance criteria

- [x] `MoveWindow(id, to_index)` repositions the addressed window; window ids never change on a move; containers (tabs, titles, favicons, load state, history) travel with the identity.
- [x] Activation follows the identity: moving the active window keeps it active; moving any window never changes which id is active.
- [x] The ADR-006 close-activation policy (and the ADR-007 close-after-reorder rule) continue to operate on current positions after a workspace reorder.
- [x] Events addressing a nonexistent window are no-ops (`unknown_window`); out-of-range target indexes are no-ops (`index_out_of_range`); moving a window to its current index is a valid no-transition.
- [x] Golden workspace scenarios extend with reordering rows and exact-count drift guards (6 scenarios / 32 steps).
- [x] Multi-platform CI green bound to the final exact PR head (3-OS Rust + Ubuntu cdylib build + Node binding tests). *(Core CI run [40918583857](https://github.com/kefyusuf/portable-browser-core/actions/runs/40918583857) PASS on head `4afe22d`.)*
- [x] Independent acceptance of ADR-018 semantics for the final exact HEAD. *(Maintainer accepted 2026-10-29 by approving the merge of PR #15. Semantics acceptance recorded 2026-10-27 in the design round; the local Windows verification record is in the PR evidence comment.)*

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No drag-and-drop gesture semantics, no workspace session format changes, no platform adapters until this milestone is accepted.
