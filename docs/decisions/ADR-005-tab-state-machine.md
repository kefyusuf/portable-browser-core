# ADR-005 — M2 tab state machine

**Status:** Semantics accepted 2026-10-25 (maintainer go-ahead); implemented with golden transition fixtures on `feat/m2-tab-state-machine`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

After M1, the core classifies address-bar input into `Url`, `Search`, or `Rejection` decisions. A host now needs a per-tab model of navigation lifecycle and session history (back/forward) under the same constraints as M1: pure, deterministic, no I/O, no clocks, no randomness. The core cannot observe real page loads, so lifecycle progress must arrive as host-reported events; the core keeps the bookkeeping and emits effect *requests* the host executes.

## Decision

- **Single-tab scope.** The machine models one tab. A multi-tab container/window model is a later milestone and must not leak into this API.
- **Total over inputs.** Every (event, state) combination returns an outcome. Invalid combinations (for example, `commit` while `Empty`) are no-ops with a reason, never panics or errors — the machine fails closed as a bookkeeper.
- **State.** `Empty` (no history), `Loading { pending_url }` (a target is awaiting a host load report), `Active` (an entry is current). Internally: an ordered `Vec<Entry>` plus a cursor. `Entry` carries the normalized target URL string only. Page titles, favicons, and scroll state are host display state, not core state in M2.
- **Events** (each returns an outcome: resulting state view plus at most one effect):
  - `navigate(url)` — valid from any state. Sets the pending target (last-pending-wins if already `Loading`), produces `StartLoad(url)`. It performs **no** history mutation: no append, no truncation.
  - `commit()` — valid only from `Loading`. The host reports the load succeeded and the pending target is cleared. If the pending target is a fresh navigation, the forward stack is truncated here (at commit, not at navigate), `Entry { url }` is appended, the cursor moves to the new last entry, and state becomes `Active`. If the pending target is a reload, the bookkeeping is unchanged: the entry already exists and must not be duplicated. Effect `None` in both cases. Rationale: truncating at navigate would destroy reachable forward history when the load ultimately fails.
  - `fail()` / `stop()` — valid only from `Loading`. The pending target is discarded and the state reverts to what it was before the navigation (`Active` at the unchanged cursor, or `Empty`), effect `None`. Error pages are host concerns and are not history entries.
  - `back()` / `forward()` — valid only from `Active`. Move the cursor one step within `[0, len-1]` and produce `StartLoad(entry_url)`; at a boundary the event is a no-op with reason `at_boundary` and effect `None`. Invoking them while `Loading` is a no-op with reason `unexpected_event`; a host that wants browser-like behavior composes `stop` → `back` itself.
  - `reload()` — valid only from `Active`. Re-emits the current entry's URL as a pending target: state `Loading`, effect `StartLoad(current_url)`. The pending target is marked as a reload so that a later `commit` does not append a duplicate entry. In `Empty` it is a no-op with reason `nothing_to_reload`.
- **Effects are requests.** The only effect in M2 is `StartLoad(url)`. The core never performs a load and returning a URL is not authorization; hosts own network policy, redirects, and execution (same boundary as ADR-004).
- **Composition with M1.** The machine takes already-resolved targets and does not re-parse them. `Url` decisions from `resolve_navigation` are valid targets; a `Search` decision requires the host to pick its search engine (out of core scope per ADR-004) and submit the resulting URL; `Rejection` never reaches the machine. No M1 behavior is duplicated inside M2.
- **Determinism.** No clocks, no randomness, no environment reads. Replaying the same event sequence over the same initial state yields identical states and effects on every platform.

## Explicitly out of scope

Multi-tab containers and window models, tab switching, titles/favicons, search-engine selection, persistence and session restore, actual loading, error pages, WebView or platform events beyond the modeled host reports, WASM JavaScript bindings, and generalized browser-history compatibility claims.

## Evidence gate

Golden transition scenario fixtures with targeted negative tests (invalid events, boundaries, truncation, pending-target races) on Linux/macOS/Windows CI plus Ubuntu WASM compile; evidence bound to the exact commit. Do not mark M2 done without passing evidence.
