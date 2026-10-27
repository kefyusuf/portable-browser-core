# M9 — Workspace session format

**State: IMPLEMENTING** — semantics accepted 2026-10-27; implementation and fixtures on the branch, CI and independent acceptance pending.

## Deliverable

A deterministic, dependency-free text serialization of the whole workspace and a validating parser that restores an equivalent workspace, embedding the accepted M6 per-container format verbatim so both formats stay aligned. The core performs no I/O. Golden round-trip scenarios and the same CI gates.

## Acceptance criteria

- [x] `Workspace::to_workspace_text` serializes every state component: window strip order, window ids, the window id counter, the active window, and — embedded verbatim per window — the M6 container session (tabs, titles, histories, cursors, pending kinds, per-container id counters).
- [x] `Workspace::from_workspace_text` restores an equivalent workspace: same view, identical re-serialization (round-trip idempotence), same later behavior.
- [x] The parser is total: malformed input returns `WorkspaceParseError`, never panics; structural violations (unknown active window, active window with an empty strip, duplicate window ids, window id not below the workspace counter, malformed embedded container block, trailing content) are rejected.
- [x] The embedded M6 container block is accepted unchanged: no re-encoding, no drift between the two formats.
- [x] Serialization adds no dependency (no serde, no I/O in the core).
- [x] Golden workspace-session scenarios exist with exact-count drift guards and exact-bytes expectations (4 scenarios, 19 steps; source inspection until CI).
- [ ] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile) bound to the final exact PR head.
- [ ] Independent acceptance of ADR-012 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No binding-layer (M8) exposure of the workspace format, no cross-version compatibility promises beyond v1, no platform adapters, no WASM JavaScript bindings for this format until this milestone is accepted.
