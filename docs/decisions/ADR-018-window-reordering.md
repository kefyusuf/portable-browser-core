# ADR-018 — M15 window reordering

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead on the candidate list); implemented on `feat/m15-window-reorder`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

ADR-007 gave tabs position-changing with immutable identity. The M7 window set is the same shape one layer up, and the same ergonomics apply: a host shell reorders windows the way it reorders tabs. The tab rule is applied verbatim — no new policy.

## Decision

- **Event.** `WorkspaceEvent::MoveWindow(id, to_index)` repositions the addressed window so it sits at `to_index` (0-based) in the workspace strip. This is a reorder of existing slots: ids, containers, and their entire states travel with the identity; no container is created, destroyed, or consulted, so the surfaced effect is always `None` and no inner outcome is produced.
- **Identity is immutable; activation follows identity.** A move never changes any window's id. Moving the active window keeps it active; moving any window never changes which id is active.
- **Bounds and invalid events.** A move addressing a nonexistent window is a no-op with reason `unknown_window`; a `to_index` of `len` or more is a no-op with reason `index_out_of_range` (no clamping); moving a window to its current index is valid and produces no transition.
- **Composition.** Close-activation (ADR-006 mirrored at the window level) keeps operating on **current positions** after any reorder; close-after-reorder semantics are pinned in the golden scenarios.
- **Determinism unchanged.**

## Explicitly out of scope

Drag-and-drop gesture interpretation, workspace session format changes, platform adapters, npm surface changes, and generalized multi-window compatibility claims.

## Evidence gate

Golden workspace scenarios extended with reordering rows (identity preservation, activation stability, close-after-reorder, invalid events) with exact-count drift guards, plus targeted tests; full local + CI gates; evidence bound to the exact commit. Do not mark M15 done without passing evidence.
