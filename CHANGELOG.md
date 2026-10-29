# Changelog

All notable changes to the portable browser decision core. Milestone evidence
(PR number, accepted head, CI run) is recorded per entry and in each
milestone document under `docs/milestones/`.

## M15 — Window reordering (2026-10-29)

`WorkspaceEvent::MoveWindow(id, to_index)` — the tab-reordering rule
(ADR-007) applied verbatim at the window level: identity separated from
position, activation follows the id, `unknown_window`/`index_out_of_range`
reasoned no-ops, close-activation on current positions after a reorder.
(PR #15, head `4afe22d`, ADR-018.)

## M14 — Browser target outputs (2026-10-29)

The npm package assembles `pkg/nodejs/`, `pkg/bundler/`, and `pkg/web/` from
one cdylib build; the manifest gains the standard conditional `exports` map;
a structural verification script proves every target's artifact shape. Browser
E2E stays with the WebView adapter milestone. (PR #14, head `df83d62`,
ADR-017.)

## M13 — npm packaging preparation (2026-10-29)

The bridge becomes a real npm artifact: checked-in manifest, `pkg/` assembly
via the pinned `wasm-bindgen-cli` (`--target nodejs`), and a Node smoke test
over the assembled package. Binding id parameters narrowed to `u32` (plain JS
Numbers); the JS surface uses the Rust names verbatim. (PR #13, head
`c45be01`, ADR-016.)

## M12 — Per-entry favicons (2026-10-29)

Favicons mirror the ADR-014 title model: `Entry.favicon`,
`SetFavicon` with identical phase retargeting, session v2 entry fields
extended to `url[~title[~favicon]]`, M8 snapshot gains `favicon` additively.
(PR #12, head `62f0cbf`, ADR-015.)

## M11 — Per-history-entry titles (2026-10-29)

Titles move into history: `Entry { url, title }`; `SetTitle` retargets by
phase (pending title travels into commit; `nothing_to_title` in `Empty`);
post-commit the displayed title is the new entry's until reported; session
format v2 (`browser-core-session\t2`, entries as `url~title`, readers accept
v1). Corrects the M5 approximation over accepted semantics. (PR #11, head
`b32dcdb`, ADR-014.)

## M10 — Workspace session over the JS bridge (2026-10-28)

`WorkspaceBinding.workspaceSessionText()` / `workspaceSetSessionText(text)` —
the ADR-012 workspace format exposed 1:1 over the bridge. (PR #10, head
`1a4a83b`, ADR-013.)

## M9 — Workspace session format (2026-10-28)

`Workspace::to_workspace_text` / `from_workspace_text` — the whole workspace
serialized as a thin envelope embedding the M6 container format verbatim.
(PR #9, head `f97e108`, ADR-012.)

## M8 — WASM host binding (2026-10-27)

`crates/browser-core-wasm`: `resolve_navigation` and a flat `WorkspaceBinding`
over `wasm-bindgen`, total semantics carried across (`{ noop }` results, never
throws); CI gains a Node-executed wasm-bindgen-test suite and packaging steps.
First host adapter — extends the core's stop-boundary. (PR #8, head
`f1b11aa`/M8 design, ADR-011.)

## M7 — Window set (2026-10-27)

`Workspace` — an ordered set of containers with deterministic window ids,
close-activation (right-else-left), and id-addressed routing of container
events. (PR #7, head `4afe22d`—see also M15, ADR-010.)

## M6 — Session persistence (2026-10-27)

`TabContainer::to_session_text` / `from_session_text` — format v1 with
escaping and structural validation; `TabSnapshot`/`from_snapshot` expose full
machine state; `SetTitle` rejects control-byte titles. (PR #6, head
`f97e108`, ADR-009.)

## M5 — Host-reported tab titles (2026-10-27)

`ContainerEvent::SetTitle` — host-reported display titles as container state;
the last reported value persists across navigation (superseded by M11's
per-entry model). (PR #5, head `69fab1f`, ADR-008.)

## M4 — Tab reordering (2026-10-27)

`ContainerEvent::MoveTab(id, to_index)` — identity separated from position,
activation follows the id, `index_out_of_range` no-ops, close-activation on
current positions. (PR #4, head `81d5b72`, ADR-007.)

## M3 — Tab container (2026-10-26)

`TabContainer` — an ordered set of M2 machines: deterministic sequential ids,
`NewTab`/`CloseTab`/`Activate`/routed `Tab(id, event)` events, id-addressed
load reports, full `ContainerView`. (PR #3, head `8afca7f`, ADR-006.)

## M2 — Tab state machine (2026-10-26)

`TabMachine` — per-tab lifecycle (`Empty`/`Loading`/`Active`) and session
history (back/forward/reload) over host-reported events; total event
handling; golden transition scenarios. (PR #2, head `fb1a22c`, ADR-005.)

## M1 — Navigation decision core (2026-10-25)

`resolve_navigation(&str)` → `Url`/`Search`/`Rejection`: deterministic
address-bar classification with WHATWG hazard guards; 30-row golden fixture;
3-OS CI + WASM compile check. (PR #1, head `9779823`, ADR-004.)

## M0 — Repository bootstrap (pre-2026-10-25)

Scope boundary and ADR discipline established: the core resolves commands
into decisions/effects and owns no OS APIs, UI, WebView, filesystem, or
networking.
