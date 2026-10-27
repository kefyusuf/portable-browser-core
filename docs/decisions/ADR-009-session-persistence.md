# ADR-009 — M6 session persistence

**Status:** Accepted and merged 2026-10-27 (PR #6, accepted head `d4dd983`, CI run 40529844209). See the M6 milestone for the evidence index.

## Context

A host that restarts needs to restore the tab strip exactly: which tabs exist, in what order, each tab's full history, pending load, title, which tab is active, and the id counter (so future ids never collide with restored ones). The core owns none of that on disk — it provides a deterministic byte representation and a validating parser; hosts do all I/O. Zero new dependencies.

## Decision

- **Machine snapshots.** `TabMachine` gains `snapshot() -> TabSnapshot` and `from_snapshot(TabSnapshot) -> Result<TabMachine, SnapshotError>`. `TabSnapshot` exposes the complete machine state: `entries` (oldest first), `cursor`, and `pending` as `PendingSnapshot::Fresh(url)` or `PendingSnapshot::Reload` — the reload distinction is part of the persisted state because it changes what `commit` does. `from_snapshot` validates (`cursor` within `entries`; `reload` pending requires a cursor) and never panics. This is an additive, read-only exposure of M2 state; M2 behavior and its golden fixtures are unchanged.
- **Format v1** (line-based, tab-separated, escaped):
  - Line 1: `browser-core-session<TAB>1` — magic and format version.
  - Line 2: `header<TAB><tab_count><TAB><next_id><TAB><active_id or ->`.
  - One line per tab, in strip order: `tab<TAB><id><TAB><title field><TAB><cursor or -><TAB><pending field>[<TAB><entry>…]` where the title field is `-` or `t:<escaped title>`, and the pending field is `-`, `reload`, or `fresh:<escaped url>`. Entries are the full history, oldest first, one escaped URL per field.
  - **Escaping.** In every free-text component (titles, pending URLs, entries): `\` → `\\`, TAB → `\t`, CR → `\r`, LF → `\n`. Absent values are a bare `-`; the `t:` title prefix keeps a literal `-` title unambiguous.
- **Round-trip contract.** `from_session_text(to_session_text(c))` yields a container with the same `state()` view, an identical re-serialization, and therefore identical later behavior (the machine is fully determined by its snapshot and total over events).
- **Parser totality.** `from_session_text` returns `Result<TabContainer, SessionParseError>` and never panics. Beyond syntax, it rejects structural violations: unknown active id, active id present with an empty strip (or missing with a non-empty strip), cursor past the end of history, `reload` pending without a cursor, `next_id` not strictly greater than every restored id, duplicate ids, and any trailing content.
- **Title validation (amends ADR-008).** Persistence forced a decision about arbitrary title content: `SetTitle` with any ASCII control byte (C0 or DEL) in the title is now a no-op with reason `invalid_title`. Titles remain free otherwise; the escaper still guards the format.
- **No I/O, no dependencies.** The core converts containers to and from strings in memory. Reading/writing files, storage backends, and schema evolution are host concerns. Compatibility promises cover format v1 only.
- **Determinism.** The same container always serializes to the same bytes; the same bytes always parse to the same container, on every platform.

## Explicitly out of scope

File/network/storage I/O, serde or external schema libraries, multi-window sessions, favicons, per-history-entry titles, cross-version format migration, platform adapters, WASM JavaScript bindings, and generalized browser-session compatibility claims.

## Evidence gate

Golden round-trip scenario scripts (full histories, pending kinds, titles, empty strip), a pinned exact-format test, and targeted negative parser tests on Linux/macOS/Windows CI plus Ubuntu WASM compile; evidence bound to the exact commit. Do not mark M6 done without passing evidence.
