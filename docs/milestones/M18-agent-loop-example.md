# M18 — Agent loop example

**State: IMPLEMENTING** — docs-by-code under the clarified purpose (ADR-019); no semantics, no ADR required (M16 precedent). Example verified locally against the assembled package; CI and independent acceptance pending.

## Deliverable

A checked-in, deterministic Node example (`examples/agent-loop.cjs`) demonstrating the intended agent integration pattern over the assembled npm package: **observe → decide → act → host effects**, with the core's total semantics shown as tool errors the agent reads and adapts to.

## Acceptance criteria

- [x] The example imports the assembled `pkg/nodejs/` glue and runs the loop end-to-end: address-bar classification (all three decision kinds), a workspace lifecycle (window → tab → commit → title), back/forward over per-tab history, a reasoned no-op the "agent" adapts to, and a session save/restore.
- [x] The example prints the observation text after each state change — demonstrating that `observe_text()` is the agent's eyes (ADR-019).
- [x] Every effect is printed as a **request** ("host would execute") — the ADR-004 boundary is demonstrated, not blurred. No network, no engine, no prompt templates.
- [x] The example is deterministic and asserts its key expectations (exits non-zero on any mismatch), so CI can run it as evidence.
- [x] CI runs the example after the package assembly step.
- [ ] Multi-platform CI green bound to the final exact PR head (existing gates unchanged).
- [ ] Independent acceptance for the final exact HEAD.

## Baseline/provenance

New documentation-by-code — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No real headless-engine adapter, no LLM/API calls, no prompt templates, no new dependencies, no core changes until a new milestone accepts them.
