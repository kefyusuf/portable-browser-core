# ADR-010 — M7 window set (workspace)

**Status:** Accepted and merged 2026-10-27 (PR #7, accepted head `caa01b9`, CI run 40549969877). See the M7 milestone for the evidence index.

## Context

ADR-006 modeled exactly one ordered tab set and deferred "multiple windows". A host now needs an ordered set of containers — windows — with the same guarantees: pure, deterministic, no I/O, id-addressed routing, reasoned no-ops instead of panics.

## Decision

- **Naming and scope.** The type is `Workspace`: exactly one ordered window set for a browser session. A window is an M3 `TabContainer` plus its identity. Window geometry, monitors, and z-order are host concerns and do not exist in the core.
- **Deterministic identities.** Windows are identified by `u64` ids from a sequential counter starting at 1, never reused — the same rule as tab ids (ADR-006). Ids are replay artifacts with no cross-session meaning.
- **Ordering.** Windows are kept in creation order; M7 has no window-reorder events.
- **Total over inputs.** Every (event, state) combination returns an outcome. Events addressing a nonexistent window are no-ops with reason `unknown_window`; the workspace never panics.
- **Events:**
  - `NewWindow` — appends a fresh window (empty container) at the end and **activates it**. Hosts compose `NewWindow` + routed `NewTab` to open a window with content; the core does not conflate the two. Effect `None`.
  - `CloseWindow(id)` — removes the window and its container. If the closed window was active, the **right neighbor activates, else the left neighbor** — the exact ADR-006 policy mirrored at the window level. The workspace may become empty; the core stays neutral about what a host shows then.
  - `ActivateWindow(id)` — makes the addressed window active; switching never triggers loads; effect `None`.
  - `Container(id, ContainerEvent)` — routes a full M3–M5 container event to the addressed window's container. The workspace never re-implements container semantics; the inner `ContainerOutcome` (view, effect, inner outcome, no-op reason) is surfaced on the workspace outcome. Window-level bookkeeping (ids, activation) is untouched by routed events.
- **Effects.** The workspace surfaces at most one `StartLoad(url)` per event — the routed container's inner effect. Window-level events produce none. A returned URL remains a request, never authorization (ADR-004 boundary).
- **View.** `WorkspaceView` exposes the active window id (or `None`) and the ordered windows as summaries `{ id, is_active, container: ContainerView }`. The view is derivable without an event.
- **Determinism.** Same event sequence over a fresh workspace yields identical ids, states, and effects on every platform; no clocks, randomness, or environment reads.

## Explicitly out of scope

Workspace-level session serialization (M6 covers single containers; serializing a workspace is future work), window reordering, geometry/screens, titles at the window level, platform adapters, WASM JavaScript bindings, and generalized multi-window browser compatibility claims.

## Evidence gate

Golden workspace scenario fixtures (window lifecycle and close policy, routed navigation across windows, unknown ids, close-to-empty) with targeted tests on Linux/macOS/Windows CI plus Ubuntu WASM compile; evidence bound to the exact commit. Do not mark M7 done without passing evidence.
