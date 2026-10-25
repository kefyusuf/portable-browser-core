# M2 — Tab state machine

**State: PROPOSED** — docs-first; implementation not started.

## Deliverable

A pure, deterministic, single-tab navigation lifecycle and session-history machine. It consumes already-resolved navigation targets (M1 `Url` decisions or host-resolved search results) and host-reported load outcomes, and emits host effects. Golden transition fixtures and the same CI gates as M1.

## Acceptance criteria

- [ ] Public tab machine API exists and is total: every (event, state) combination returns an outcome, never panics.
- [ ] States `Empty` / `Loading` / `Active` with host-reported `commit`, `fail`, and `stop` transitions.
- [ ] Back/forward move the history cursor within bounds and are documented no-ops at the boundaries.
- [ ] Forward-stack truncation happens at commit only; a failed navigation leaves no phantom entry.
- [ ] Re-navigate while loading replaces the pending target (last-pending-wins).
- [ ] Reload re-emits the current entry without duplicating it.
- [ ] Effects are requests only (`StartLoad`); the core never executes loads. Search-engine selection stays with the host per ADR-004.
- [ ] Golden transition fixtures exist with an exact-count drift guard (source inspection until CI).
- [ ] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head.
- [ ] Independent acceptance of ADR-005 semantics for the final exact HEAD.

## Baseline/provenance

New design — there is no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No multi-tab container or window model, tab switching, titles/favicons, persistence or session restore, platform adapters, or WASM JavaScript bindings until this milestone is accepted.
