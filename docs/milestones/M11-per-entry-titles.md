# M11 — Per-history-entry titles

**State: IMPLEMENTING** — ADR-014 accepted 2026-10-27; implementation and regenerated fixtures on the branch, CI and independent acceptance pending.

## Deliverable

Titles become part of navigation history: each M2 entry remembers its own host-reported title, back/forward show the landed entry's title, and the session format moves to v2 (reading v1 for compatibility). All golden fixtures and exact-byte expectations regenerate under the new semantics.

## Acceptance criteria

- [x] `TabMachine` entries become `Entry { url, title }`; the displayed title is the current entry's title, or during `Loading` the pending title if already reported, else the current entry's title.
- [x] `TabEvent::SetTitle(Option<String>)` retargets by phase: during `Active` it updates `entries[cursor]`; during `Loading` it attaches to the pending target (a fresh pending carries its title into the entry at commit; a reload pending updates the entry being reloaded); in `Empty` it is a no-op with the new reason `nothing_to_title`.
- [x] Post-commit correction over M5: after committing a new page the displayed title is the new entry's title — `None` until reported. Back/forward show each entry's remembered title.
- [x] Container title state is removed: `TabSummary.title` is derived from the machine; `ContainerEvent::SetTitle` routes into the machine. The M8 snapshot shape is unchanged (title stays on the tab summary).
- [x] Session format v2: container sessions are written as `browser-core-session\t2` — the tab line drops the tab-level title field and each entry field carries its title as `<escaped url>~<escaped title>` (escaper extended with `\~`). Readers accept v1 (all titles `None`) and v2; writers emit v2. Workspace embedding inherits v2 through the block magic, unchanged.
- [x] `TabSnapshot` carries per-entry titles; `TabStateView` gains `title`.
- [x] Golden fixtures regenerate under the new semantics with exact-count drift guards: M2 (70 rows / 12 scenarios, title component), M3 (60 rows, corrected title scenarios), M6 (18 steps at v2), M9 (20 steps at v2).
- [ ] Multi-platform CI green (Linux/macOS/Windows `fmt`, locked tests, Clippy; Ubuntu locked WASM compile + Node binding tests) bound to the final exact PR head.
- [ ] Independent acceptance of ADR-014 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No favicons, no v1 writing support (read-only compatibility), no cross-version migration tooling, no npm packaging, no WebView adapter until this milestone is accepted.
