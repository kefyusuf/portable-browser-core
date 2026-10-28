# M12 — Per-entry favicons

**State: IMPLEMENTING** — ADR-015 accepted 2026-10-27; implementation on the branch, CI and independent acceptance pending.

## Deliverable

Favicons mirror the ADR-014 title model exactly: each history entry remembers its own host-reported favicon, `SetFavicon` retargets by phase like `SetTitle`, the session v2 entry field gains an optional favicon segment, and the M8 snapshot gains a favicon field additively.

## Acceptance criteria

- [x] `Entry`/`EntrySnapshot` carry `favicon: Option<String>`; `TabStateView` and `TabSummary` expose the displayed favicon with the same pending/current rule as titles.
- [x] `TabEvent::SetFavicon(Option<String>)` retargets by phase like `SetTitle`: `Loading` → pending target (fresh carries into commit, reload updates the reloaded entry); `Active` → current entry; `Empty` → no-op `nothing_to_favicon`.
- [x] The container mirrors the M11 title plumbing: control-byte favicon references are rejected as `invalid_favicon`; `ContainerNoOpReason::NothingToFavicon` surfaces the machine no-op; `ContainerEvent::SetFavicon` routes into the machine.
- [x] Session v2 entry fields extend to `<url>[~<title>[~<favicon>]]` (absent segments omitted; escape-aware `~` split reused); pending `fresh:` fields likewise. Writers emit the extended v2; readers accept 1–3 segment fields and v1 unchanged.
- [x] The M8 snapshot's tab object gains `favicon` additively; existing fields unchanged.
- [x] Golden fixtures regenerate: M2 (79 rows / 13 scenarios, favicon component + `favicon_retargets_by_phase`), M3 (favicon component), M6/M9 byte expectations unchanged (no favicons in those scenarios) with new v2 favicon-segment tests.
- [ ] Multi-platform CI green bound to the final exact PR head (3-OS Rust + Ubuntu cdylib build + Node binding tests).
- [ ] Independent acceptance of ADR-015 semantics for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No favicon fetching or inference, no image handling in the core, no npm packaging, no WebView adapter until this milestone is accepted.
