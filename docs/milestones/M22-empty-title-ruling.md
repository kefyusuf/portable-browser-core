# M22 — Empty title/favicon ruling (ADR-022)

**State: DONE** — accepted and merged 2026-11-16 (PR #25, accepted head `05dbb73`, merge `756dc3f`; CI run 43940915127 on the exact head).

## Deliverable

The semantics ruling that resolves the M21 fuzz gate's second finding:
**an empty title or favicon is absent** — no accepted input (event or
legacy text) can produce a state the session format cannot re-express.

- **Events:** `SetTitle(id, Some(""))` / `SetFavicon(id, Some(""))`
  return `invalid_title` / `invalid_favicon` reasoned no-ops (the same
  class as control bytes; the pending-load path is guarded identically).
- **Legacy reads:** a v1 `t:` payload that unescapes to empty reads as
  `None` — lenient, matching the format's own empty-segment rule.
- **ADR-022** records the ruling; the machine layer is untouched (the
  container is the session-format boundary); format v2 unchanged.

The minimized M21 finding (v1 `t:` bare payload) becomes a pinned
round-trip regression; both events are pinned as reasoned no-ops
including the pending-load path.

## Acceptance criteria

- [x] The minimized M21 input parses to `title: None` with a stable
      canonical round-trip (pinned test).
- [x] `SetTitle("")` / `SetFavicon("")` return `invalid_title` /
      `invalid_favicon`, including during a pending load (pinned test).
- [x] All pre-existing gates green locally (fmt, 48 tests, clippy, wasm
      check, deny); the ruling changes no other accepted decision.
- [x] CI green on the exact PR head. *(Run 43940915127, 4/4 jobs, on head
      `05dbb73`.)*
- [x] M21 rebased over this ruling goes green — its session campaign is
      the standing acceptance for the ruling. *(Run 43947104997: the
      campaign that was red precisely because of this finding now passes
      crash-free — 15.19M session runs.)*

## Baseline/provenance

Derived from the M21 fuzz gate's first campaigns (finding 2, minimized
locally through the wasm binding). Semantics ruling approved in-session
by the maintainer before implementation.

## Stop boundary

No other event's validation changes, no machine-layer change, no format
change, no fuzz-infra change (M21's), no publishing or release scope.
