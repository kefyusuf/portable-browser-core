# M12 — Per-entry favicons

**State: DONE** — accepted and merged 2026-10-27.

## Deliverable

Favicons mirror the ADR-014 title model exactly: each history entry remembers its own host-reported favicon, `SetFavicon` retargets by phase like `SetTitle`, the session v2 entry field gains an optional favicon segment, and the M8 snapshot gains a favicon field additively.

## Acceptance criteria

- [x] `Entry`/`EntrySnapshot` carry `favicon: Option<String>`; `TabStateView` and `TabSummary` expose the displayed favicon with the same pending/current rule as titles.
- [x] `TabEvent::SetFavicon(Option<String>)` retargets by phase like `SetTitle`: `Loading` → pending target (fresh carries into commit, reload updates the reloaded entry); `Active` → current entry; `Empty` → no-op `nothing_to_favicon`.
- [x] The container mirrors the M11 title plumbing: control-byte favicon references are rejected as `invalid_favicon`; `ContainerNoOpReason::NothingToFavicon` surfaces the machine no-op; `ContainerEvent::SetFavicon` routes into the machine.
- [x] Session v2 entry fields extend to `<url>[~<title>[~<favicon>]]` (absent segments omitted; escape-aware `~` split reused); pending `fresh:` fields likewise. Writers emit the extended v2; readers accept 1–3 segment fields and v1 unchanged.
- [x] The M8 snapshot's tab object gains `favicon` additively; existing fields unchanged.
- [x] Golden fixtures regenerate: M2 (79 rows / 13 scenarios, favicon component + `favicon_retargets_by_phase`), M3 (favicon component), M6/M9 byte expectations unchanged (no favicons in those scenarios) with new v2 favicon-segment tests.
- [x] Multi-platform CI green bound to the final exact PR head (3-OS Rust + Ubuntu cdylib build + Node binding tests). *(Core CI run [40773534771](https://github.com/kefyusuf/portable-browser-core/actions/runs/40773534771) PASS on head `62f0cbf`.)*
- [x] Independent acceptance of ADR-015 semantics for the final exact HEAD. *(Maintainer accepted 2026-10-27 by approving the merge of PR #12. Semantics acceptance recorded 2026-10-27 in the design round; the local Windows verification record — including a local Node run — is in the PR evidence comment.)*

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No favicon fetching or inference, no image handling in the core, no npm packaging, no WebView adapter until this milestone is accepted.
