# ADR-019 — Agentic purpose and the observation contract

**Status:** Accepted and merged 2026-10-29 (PR #17, accepted head `e9afe27`, CI run 40942369390). See the M17 milestone for the evidence index.

## Context

The maintainer clarified the project's purpose: this core exists to give **agents and LLMs** a fast, effective browser experience. Whether the host runs it headless or as a full browser is the host's choice. This does not change any accepted decision — it names what the architecture was already built for and draws two consequences.

## Decision

- **Purpose, first-class.** The core is the agent's browser decision layer: deterministic, total, with reasoned no-ops as tool-error semantics an LLM can read and correct. Headless engines and browser UIs are both just *hosts* under the ADR-004 boundary; the core never knows or cares which one is executing. README and the architecture document state this purpose first-class.
- **The observation is the agent's eyes — and token efficiency is a feature.** An agent reads state every step, so the core provides a deterministic, compact **observation text** (`Workspace::observe_text()`), designed for LLM prompts:
  - Line 1: `workspace windows=<n> active=<id|->`.
  - One `window <id>[ active]` line per window in strip order, then one `  tab <id> <phase>[ url=<url>][ pending=<url>][ title="<title>"][ favicon="<favicon>"] actions=<list>` line per tab.
  - `actions=` is the deterministic, phase-derived list of what the agent may do to that tab next, in fixed order: `back`, `forward`, `reload`, `navigate`, `commit`, `fail`, `stop`, `set_title`, `set_favicon` — filtered by `can_back`, `can_forward`, and phase (`reload` and `set_*` require a non-`Empty` phase; `commit`/`fail`/`stop` require `Loading`). An empty list renders as `actions=-`.
  - Titles and favicons are double-quoted with `"` and `\` escaped (`\"`, `\\`); URLs are emitted verbatim (they contain no whitespace/control bytes by construction of the callers; M1 rejects them for its own inputs).
  - The text is deterministic and total; the same state always yields the same bytes.
- **The structured snapshot stays** (`snapshot()` and the bridge's object form) — the observation text is an additional projection for prompts, not a replacement. Both derive from the same view; neither adds policy.
- **Bridge, additive.** `WorkspaceBinding::observe_text() -> String` exposes the rendering; the snapshot object is unchanged.

## Explicitly out of scope

Structured observation objects beyond the existing snapshot, prompt templates, host headless-engine adapters, token counting, and any change to accepted event semantics.

## Evidence gate

An exact-bytes golden test over scripted workspace scenarios (multi-window, titles, favicons, pending loads, empty strip), binding exposure tested in Node, and all pre-existing gates green on the exact commit. Do not mark M17 done without passing evidence.
