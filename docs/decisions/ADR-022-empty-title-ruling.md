# ADR-022 — Empty title and favicon ≡ absent

**Status:** Proposed (PR #25, 2026-11-16). Ruling surfaced by the M21 fuzz campaign; the maintainer approved the "empty ≡ absent" direction in-session before implementation. See the M22 milestone for the evidence index.

## Context

The M21 fuzz gate's first campaign proved the session format cannot
represent an empty title or favicon: format v2 reads an empty segment as
`None`, but the event layer admitted `Some("")` — a live state with
`title: Some("")` saves as `url~` and restores as `title: None`, losing
state on every save → restore. The v1 legacy parser reaches the same
state through a bare `t:` payload. Both paths were minimized and
verified against the wasm binding.

## Decision

- **Empty ≡ absent, everywhere.** No state reachable by any accepted
  input — event or legacy text — may carry `Some("")` for a title or
  favicon, because the format cannot re-express it.
- **Events: reasoned no-ops.** `SetTitle(id, Some(""))` and
  `SetFavicon(id, Some(""))` return `invalid_title` / `invalid_favicon`,
  the same class as control-byte rejections. Totality holds: the agent
  reads a reason and corrects, per ADR-019's tool-error semantics.
- **Legacy reads stay lenient.** A v1 `t:` payload that unescapes to an
  empty string reads as `None` — the same rule every optional segment in
  the format follows. Old texts keep parsing; nothing in the wild gains
  a new rejection.
- **The machine layer is untouched.** The ruling guards the container —
  the session-format boundary — where the existing control-byte checks
  live. A directly-driven `TabMachine` (no text-format contract) keeps
  its current behavior.
- **Format v2 unchanged.** No version bump, no new marker, no migration:
  the ruling makes event-reachable states a subset of what the format
  already expresses.

## Explicitly out of scope

A distinct empty-string marker (format v3), machine-level validation for
direct `TabMachine` users, `SetTitle(None)`/`SetFavicon(None)` semantics
(already "clear"), and any other event's validation rules.

## Evidence gate

Regression tests pin both sides of the ruling: the events return
`invalid_title` / `invalid_favicon` (including the pending-load path),
and the minimized v1 input parses to `title: None` with a stable
canonical round-trip. All pre-existing gates green; the M21 session
campaign goes green on a rebase over this ruling. Do not mark M22 done
without passing evidence.
