# ADR-008 — M5 host-reported tab titles

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead for the a–e sequence, item b); implemented on `feat/m5-tab-titles`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

ADR-005/006 explicitly placed titles outside the core ("titles are host display state"). The approved roadmap moves them into the core as container-level bookkeeping so that hosts get one deterministic, replayable snapshot of the whole tab strip — including what each tab displays — without re-implementing title rules themselves. The M2 machine's navigation bookkeeping must remain untouched.

## Decision

- **Storage and boundary.** Titles live on the container slot per tab, **not** inside `TabMachine`. `TabSummary` gains `title: Option<String>`; `TabStateView` is unchanged, so all M2 semantics and its golden fixtures remain valid as recorded. This ADR amends the ADR-006 statement that titles are out of scope: they are now in-scope container state, reported by the host.
- **Event.** `SetTitle(id, Option<String>)` sets the addressed tab's title; `None` clears it. Unknown ids are no-ops with reason `unknown_tab`. The event never produces an effect and never consults a machine.
- **Titles are host-reported facts, never inferred.** The core never derives a title from a URL or invents one. New tabs start untitled (`None`).
- **Staleness policy.** The title is simply *the last host-reported value for this tab*. Navigation events (`navigate`, `commit`, `fail`, `stop`, `back`, `forward`, `reload`) never alter or clear it: while a new page loads, the tab keeps showing its last-known title, and the host reports a new title (or clears it) when it knows one. This mirrors browser behavior deterministically without the core guessing.
- **Identity travel.** Titles belong to the tab identity: reordering (ADR-007) and activation keep titles with their id; `CloseTab` discards the title with the slot.
- **Out of scope (explicit).** Per-history-entry titles (titles remembered per back/forward entry), favicons, title inference, and multiple windows. A host that wants entry-specific titles reports `SetTitle` itself when navigation events occur.
- **Determinism.** Same event sequence over a fresh container yields the same titles, states, and effects on every platform.

## Evidence gate

Golden container scenario fixtures (set/clear, per-tab independence, reorder/close travel, unknown ids) with targeted tests on Linux/macOS/Windows CI plus Ubuntu WASM compile; evidence bound to the exact commit. Do not mark M5 done without passing evidence.
