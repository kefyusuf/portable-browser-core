# ADR-007 — M4 tab reordering

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead for the a–e sequence); implemented on `feat/m4-tab-reorder`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

ADR-006 fixed the tab strip in creation order with no reorder events. A host shell needs to reposition tabs (drag-and-drop, keyboard shortcuts, menus). The core must keep the strip deterministic while separating two concepts that browsers visually conflate: a tab's **identity** (its `u64` id, load state, history) and its **position** (where it sits in the strip).

## Decision

- **Event.** `MoveTab(id, to_index)` repositions the addressed tab so that it sits at `to_index` (0-based) in the strip afterwards. This is a reorder of the existing slots: ids, load state, and history are untouched; no machine is created, destroyed, or consulted, so the surfaced effect is always `None` and no inner outcome is produced.
- **Identity is immutable.** A move never changes any tab's id. Ids stay sequential and never reused (ADR-006); position is the only thing that changes.
- **Activation follows identity.** Moving the active tab keeps it active (at its new position). Moving any other tab never changes which id is active. There is no "activation moves with position" rule.
- **Bounds and invalid events.** A move addressing a nonexistent id is a no-op with reason `unknown_tab`. A `to_index` of `len` or more is a no-op with reason `index_out_of_range` — the core does not clamp; hosts decide what a past-the-end drag means. Moving a tab to its current index is valid and produces no transition (no-op reason `None`).
- **Load-state neutrality.** Reordering a tab that is `Loading` neither cancels nor restarts its load; the pending target and the machine state move with the id.
- **Composition with ADR-006.** The close-activation policy (active close → right neighbor, else left) continues to operate on **current positions** after any reorder; it does not remember pre-move order. The container-may-be-empty rule is unchanged.
- **Determinism.** Same event sequence over a fresh container yields the same ids, positions, and effects on every platform; no clocks, randomness, or environment reads.

## Explicitly out of scope

Multiple windows, pinning/muting, drag-and-drop gesture interpretation (the core sees only the resulting target index), titles, persistence/session restore, platform adapters, WASM JavaScript bindings, and generalized tab-strip compatibility claims.

## Evidence gate

Golden reordering scenario fixtures with targeted tests (identity stability, activation stability, close-after-reorder, invalid events) on Linux/macOS/Windows CI plus Ubuntu WASM compile; evidence bound to the exact commit. Do not mark M4 done without passing evidence.
