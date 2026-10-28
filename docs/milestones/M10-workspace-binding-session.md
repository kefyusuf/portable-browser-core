# M10 — Workspace session over the JS bridge

**State: IMPLEMENTING** — semantics accepted 2026-10-27; implementation on the branch (Node suite extended), CI and independent acceptance pending.

## Deliverable

Two methods extending the accepted M8 `WorkspaceBinding` surface: `workspaceSessionText()` and `workspaceSetSessionText(text)`, exposing the ADR-012 workspace format over the JS bridge 1:1. No core changes, no new dependencies.

## Acceptance criteria

- [x] `workspaceSessionText()` returns the ADR-012 session text of the whole workspace.
- [x] `workspaceSetSessionText(text)` replaces the whole workspace from session text (ids and counters come from the text) and returns the full snapshot; malformed text is rejected as `"malformed_workspace"` — a result, never an exception.
- [x] The binding adds zero policy: 1:1 passthrough of ADR-012 semantics; no new dependencies.
- [x] Node-executed tests cover: round-trip identity over the bridge (build → text → restore → identical text), restored state inspection (active window, tab title/URL), and malformed-text rejection.
- [ ] Multi-platform CI green bound to the final exact PR head: existing gates unchanged (3-OS Rust + Ubuntu cdylib build + Node binding tests).
- [ ] Independent acceptance of ADR-013 for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No npm packaging, no WebView adapter, no per-entry titles, no favicons until this milestone is accepted.
