# ADR-006 — M3 tab container

**Status:** Semantics accepted 2026-10-25 (maintainer go-ahead); implemented with golden container scenarios on `feat/m3-tab-container`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

M2 delivered a single-tab lifecycle and session-history machine (`TabMachine`). A host shell needs a set of tabs with stable identities: create, close, switch, and route navigation and load reports to a specific tab — including background tabs, because real loads complete independently of which tab is visible. Same constraints as M1/M2: pure, deterministic, no I/O, no clocks, no randomness; the container keeps bookkeeping and emits effect requests.

## Decision

- **Single-window scope.** M3 models exactly one ordered tab set. Multiple windows/containers are a later milestone and must not leak into this API.
- **Deterministic identities.** Tabs are identified by `u64` ids from a sequential counter starting at 1, never reused. Ids are deterministic replay artifacts: the same event sequence yields the same ids. Hosts must not assign cross-session meaning to them; persistence is out of scope.
- **Ordering.** Tabs are kept in creation order. M3 has no reorder/move events.
- **Total over inputs.** Every (event, state) combination returns an outcome. Events addressing a nonexistent tab are no-ops with reason `unknown_tab`; the container never panics.
- **Events:**
  - `NewTab(initial: Option<target>)` — appends a fresh tab at the end and **activates it**. When an initial target is given, the container routes a `Navigate` to the new machine atomically. Effect `StartLoad(url)` iff an initial target was given.
  - `CloseTab(id)` — removes the tab and its machine. If the closed tab was active, the **right neighbor activates, else the left neighbor** (Chrome-like expectation); if none remain the container becomes empty. Closing a background tab never changes the active tab. Effect `None`.
  - `Activate(id)` — makes the addressed tab active. Switching never triggers a reload; effect `None`.
  - `Tab(id, TabEvent)` — delegates the M2 event to that tab's machine. The container never re-implements M2 semantics; the inner outcome (state view, effect, no-op reason) is surfaced on the container outcome. Because load reports are id-addressed, a background tab can `Commit`/`Fail`/`Stop` while another tab is active.
- **Emptiness.** The container may become empty. The core stays neutral about what a host shows then (new-tab page, window close, etc.) — that is host policy.
- **Effects.** The container surfaces at most one `StartLoad(url)` per event — the affected tab's inner effect, or the initial navigation of `NewTab`. Close/activate produce none. Effects remain requests; a returned URL is not authorization (ADR-004 boundary).
- **View.** `ContainerView` exposes the active id (or `None`) and the ordered tabs as summaries `{ id, is_active, state: TabStateView }`, where `TabStateView` is M2's full per-tab view. The view is derivable without an event.
- **Determinism.** No clocks, no randomness, no environment reads. Replaying the same event sequence over a fresh container yields identical ids, states, and effects on every platform.

## Explicitly out of scope

Multiple windows, tab reordering/moving, pinning/muting, titles and favicons, search-engine selection (ADR-004), persistence and session restore, actual loading, WebView or platform events beyond the modeled host reports, WASM JavaScript bindings, and generalized browser tab-strip compatibility claims.

## Evidence gate

Golden container scenario fixtures with targeted tests (activation policy on close, unknown ids, empty container, background-tab load reports) on Linux/macOS/Windows CI plus Ubuntu WASM compile; evidence bound to the exact commit. Do not mark M3 done without passing evidence.
