# ADR-014 — M11 per-history-entry titles

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead); implemented on `feat/m11-per-entry-titles`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

M5 gave each tab a single host-reported title: "the last reported value", untouched by navigation. Real browsers remember a title **per history entry**: going back shows the title of the page you land on, and committing a new page shows that page's title once known — not the previous page's forever. This milestone moves titles into history, correcting the M5 approximation, and carries the change through the session format.

## Decision

- **Titles are entry state.** `TabMachine` entries become `Entry { url, title: Option<String> }`. The displayed title (`TabStateView.title`): in `Active`, the current entry's title; in `Loading`, the pending title if already reported, else the current entry's title (the page being left) — matching browser behavior during loads.
- **`SetTitle` retargets by phase (amends ADR-008).** During `Active`, it updates `entries[cursor].title`. During `Loading`, it attaches to the pending target: a fresh pending carries its title into the entry at `commit` (a host that learns the title mid-load does not lose it), and a reload pending updates the entry being reloaded. In `Empty` there is no page to title: the event is a no-op with the new reason `nothing_to_title` — correcting M5, where a title could be set on an entryless tab and persisted.
- **Post-commit correction (amends ADR-008).** After committing a new page, the displayed title is the **new entry's title** — `None` until the host reports it. M5's "navigation never alters the title" persisted the previous page's title after commit, which real browsers do not do. `back`/`forward` now show each entry's remembered title.
- **Container title state removed.** `ContainerEvent::SetTitle` routes into the machine; the M5 slot-level title disappears; `TabSummary.title` is derived from the machine's displayed title. The M8 snapshot shape is unchanged (title stays on the tab summary), and the M8 session passthroughs transparently carry the new format.
- **Session format v2 (amends ADR-009).** Container sessions are written as `browser-core-session\t2`: the tab line drops the tab-level title field, and each entry field carries its title as `<escaped url>~<escaped title>`. The escaper additionally escapes `~` (`\~`), so free-text titles and URLs containing the separator round-trip. Readers accept **v1 and v2** (v1: entries are bare escaped URLs, all titles `None`); writers emit v2 only. The M9 workspace envelope is unchanged and inherits v2 through the embedded block magic. `TabSnapshot` carries per-entry titles.
- **Fixture regeneration is part of the acceptance.** The M2 golden gains a title component (55 rows); M5-era scenarios that set titles on entryless tabs are corrected to the new flow; M6/M9 exact-byte expectations regenerate at v2. A Node-side round-trip test keeps the bridge honest over v2.
- **Determinism unchanged.** Same event sequence over the same initial state yields identical states, titles, and effects on every platform.

## Explicitly out of scope

Favicons, v1 writing support (read-only compatibility), cross-version migration tooling, npm packaging, WebView adapter, and generalized browser title-memory compatibility claims.

## Evidence gate

Regenerated golden fixtures with exact-byte expectations at v2 (including a v1-compat parse test), Node-executed binding tests over v2, targeted tests for retargeted `SetTitle` (pending-title travel, reload retarget, `nothing_to_title`), and full local + CI gates; evidence bound to the exact commit. Do not mark M11 done without passing evidence.
