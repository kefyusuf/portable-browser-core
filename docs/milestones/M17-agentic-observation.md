# M17 — Agentic purpose and the observation contract

**State: IMPLEMENTING** — ADR-019 accepted 2026-10-29; implementation on the branch, CI and independent acceptance pending.

## Deliverable

The project purpose (agents/LLMs; headless vs headed is the host's choice) becomes first-class in the docs, and the core gains the agent's eyes: `Workspace::observe_text()` — a deterministic, token-efficient, phase-derived action list per tab, exposed over the bridge.

## Acceptance criteria

- [ ] ADR-019 records the purpose and the observation v1 contract; README and `docs/architecture.md` state the agentic purpose first-class.
- [ ] `Workspace::observe_text()` renders the deterministic observation text (window/tab lines, quoted titles/favicons, phase-derived `actions=` lists, `-` for empty).
- [ ] The rendering is total and deterministic; exact bytes are pinned by a golden test over scripted scenarios (multi-window, titles, favicons, pending loads, empty strip).
- [ ] `WorkspaceBinding::observe_text()` exposes it over the bridge; the structured snapshot is unchanged.
- [ ] Multi-platform CI green bound to the final exact PR head (3-OS Rust + Ubuntu cdylib build + Node binding tests).
- [ ] Independent acceptance of ADR-019 for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No prompt templates, no structured observation objects beyond the existing snapshot, no token counting, no host headless-engine adapters until this milestone is accepted.
