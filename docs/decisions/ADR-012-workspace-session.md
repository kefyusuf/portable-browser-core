# ADR-012 — M9 workspace session format

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead, first post-roadmap scope); implemented on `feat/m9-workspace-session`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

M6 established a deterministic v1 session format for a single container. A host that restarts a whole session needs the same guarantee for the M7 workspace: every window, its full container state, the window strip order, the active window, and both id counters. The design must not fork the M6 format — two ways to serialize a tab would be a drift hazard.

## Decision

- **Embedding, not re-encoding.** The workspace format v1 is a thin envelope over the accepted M6 container format: a workspace magic line, a workspace `header` (window count, window id counter, active window id or `-`), then per window one `window<TAB><id>` line followed **verbatim** by that container's full M6 session text (magic, container header, tab lines). Both formats share escaping, title prefixes, pending encodings, and validation; there is exactly one way a tab is serialized. The binding layer (M8) is untouched — this format is a core API, exposed to JS in a later decision.
- **API.** `Workspace::to_workspace_text() -> String` and `Workspace::from_workspace_text(&str) -> Result<Workspace, WorkspaceParseError>` — the same shape as M6's container methods, on the next layer up.
- **Round-trip contract.** `from_workspace_text(to_workspace_text(w))` yields a workspace with the same `state()` view, an identical re-serialization, and therefore identical later behavior.
- **Parser totality.** `from_workspace_text` never panics. Beyond the per-container M6 validation (applied unchanged to every embedded block), the workspace layer rejects: duplicate window ids, a window count mismatch, a window id at or above the workspace counter, an active window that does not exist, an active window alongside an empty strip (or none with a non-empty strip), and any trailing content. All failures are one error: `WorkspaceParseError` (`"malformed_workspace"`).
- **No I/O, no dependencies.** In-memory text only; hosts own storage. Compatibility promises cover workspace format v1 and the embedded container format v1.
- **Determinism.** The same workspace always serializes to the same bytes; the same bytes always parse to the same workspace, on every platform.

## Explicitly out of scope

Binding-layer (M8) exposure of this format, cross-version migration, favicons, per-history-entry titles, platform adapters, and generalized browser-session compatibility claims.

## Evidence gate

Golden workspace-session scenario scripts with exact-bytes expectations (multi-window histories, titles, pending kinds, empty workspace, reordered tab strips), round-trip idempotence, and targeted negative parser tests on Linux/macOS/Windows CI plus Ubuntu WASM compile; evidence bound to the exact commit. Do not mark M9 done without passing evidence.
