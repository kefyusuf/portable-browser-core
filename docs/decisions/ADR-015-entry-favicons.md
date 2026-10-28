# ADR-015 — M12 per-entry favicons

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead, "sıradaki iş maddesi ile devam edelim" on the candidate list); implemented on `feat/m12-entry-favicons`. Milestone evidence (multi-platform CI and independent acceptance) pending.

## Context

ADR-014 moved titles into history entries. Favicons behave identically in a browser: the icon shown on a tab belongs to the page being displayed, and going back shows the landed page's icon. The model, plumbing, and format extension therefore mirror ADR-014 exactly.

## Decision

- **Entry state.** `Entry`/`EntrySnapshot` gain `favicon: Option<String>` — a host-reported favicon reference. The core never fetches, infers, or validates it as a URL; it is opaque host data, exactly like titles.
- **`TabEvent::SetFavicon(Option<String>)` retargets by phase — the ADR-014 rule applied verbatim:** during `Loading` it attaches to the pending target (a fresh pending carries its favicon into the entry at commit; a reload pending updates the entry being reloaded); during `Active` it updates `entries[cursor].favicon`; in `Empty` it is a no-op with the new reason `nothing_to_favicon`. The displayed favicon (`TabStateView.favicon`) follows the same pending-current rule as titles.
- **Container plumbing mirrors M11.** `ContainerEvent::SetFavicon` routes into the machine; control-byte favicon references are rejected as `invalid_favicon` (the ADR-009 rule applied to favicons); the machine's `nothing_to_favicon` surfaces as `ContainerNoOpReason::NothingToFavicon`. `TabSummary.favicon` is derived from the machine.
- **Session v2 extension (amends ADR-014's v2 while it is still unreleased).** Entry fields extend to `<url>[~<title>[~<favicon>]]` — absent trailing segments are omitted, all segments escape-aware (the ADR-014 `\~` rule covers the new separator usage). Pending `fresh:` fields extend identically. Readers accept 1–3 segment fields and v1; the workspace envelope inherits everything through the embedded block.
- **M8 surface, additive.** The bridge's tab snapshot object gains `favicon`; no existing field or method changes.
- **Determinism unchanged.**

## Explicitly out of scope

Favicon fetching or inference, image handling, npm packaging, WebView adapter, and generalized browser favicon compatibility claims.

## Evidence gate

Regenerated golden fixtures (M2 favicon component + retarget scenario; M3 favicon component; M6/M9 bytes where favicons appear), Node-executed binding tests over the additive snapshot field, and full local + CI gates; evidence bound to the exact commit. Do not mark M12 done without passing evidence.
