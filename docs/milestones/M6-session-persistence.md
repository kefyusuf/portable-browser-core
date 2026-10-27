# M6 — Session persistence (snapshot and restore)

**State: IMPLEMENTING** — semantics accepted 2026-10-27; implementation and fixtures on the branch, CI and independent acceptance pending.

## Deliverable

A deterministic, dependency-free text serialization of the whole container (`TabContainer::to_session_text`) and a validating parser that restores an equivalent container (`TabContainer::from_session_text`). The core performs no I/O: hosts read and write the bytes. Golden round-trip scenarios and the same CI gates.

## Acceptance criteria

- [x] `TabMachine` exposes a complete read-only snapshot (`TabSnapshot` with entries, cursor, and pending-target kind) and a validated `from_snapshot` constructor.
- [x] `to_session_text` serializes every state component: strip order, ids, titles, full per-tab history, cursor, pending target kind (`fresh:`/`reload`), active id, and the id counter.
- [x] `from_session_text` restores an equivalent container: same view, same subsequent serialization (round-trip idempotence), same later behavior.
- [x] The parser is total: malformed input returns `SessionParseError`, never panics; structural violations (unknown active id, cursor past history, `reload` pending without a cursor, non-increasing id counter, duplicate ids, trailing content) are rejected.
- [x] The format is unambiguous under free-text titles and URLs: backslash, tab, CR, and LF are escaped; absent values are marked `-`; titles carry a `t:` prefix so a literal `-` title round-trips.
- [x] Serialization adds no dependency (no serde, no I/O in the core).
- [x] `SetTitle` now rejects control-character titles (C0/DEL) as `invalid_title` no-ops — a persistence prerequisite documented in ADR-009.
- [x] Golden round-trip scenarios exist with exact-count drift guards, plus a pinned exact-format test (4 scenarios, 17 steps; source inspection until CI).
- [ ] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head.
- [ ] Independent acceptance of ADR-009 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No file/network I/O, no serde or external schema, no cross-version format compatibility promises beyond v1, no multiple-window sessions, no platform adapters, no WASM JavaScript bindings until this milestone is accepted.
