# ADR-013 — M10 workspace session over the JS bridge

**Status:** Accepted and merged 2026-10-27 (PR #10, accepted head `1a4a83b`, CI run 40740275502). See the M10 milestone for the evidence index.

## Context

The persistence stack is complete in the core: M6 (container format), M9 (workspace envelope embedding M6 verbatim). The M8 bridge exposes only the per-container passthrough, so a JS host cannot yet save or restore a whole session. ADR-012 explicitly deferred bridge exposure to a later decision — this is that decision.

## Decision

- **Two methods, zero policy.** `WorkspaceBinding` gains `workspaceSessionText() -> String` and `workspaceSetSessionText(text) -> Result<JsValue, JsValue>`, a 1:1 passthrough of ADR-012's `to_workspace_text`/`from_workspace_text`. `workspaceSetSessionText` **replaces the whole workspace** from the text — window ids, tab ids, and both id counters come from the text, exactly as in the core — and returns the full snapshot on success. Malformed text is rejected as the result `"malformed_workspace"`, never an exception; the bridge's total semantics extend to this surface.
- **No core changes, no new dependencies.** The core API from ADR-012 is already public; the binding only forwards.
- **Testing.** The Node suite gains a round-trip test: build a workspace through bridge methods, serialize, restore into a fresh binding, assert identical re-serialized bytes and inspect the restored state (active window, tab title/URL); malformed text is rejected as a result.

## Explicitly out of scope

npm packaging, WebView adapter, per-history-entry titles, favicons, and cross-version format migration.

## Evidence gate

Node-executed round-trip and rejection tests on CI (Ubuntu), all pre-existing gates green, evidence bound to the exact commit. Do not mark M10 done without passing evidence.
