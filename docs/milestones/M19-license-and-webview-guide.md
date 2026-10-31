# M19 — License and WebView integration guide

**State: IMPLEMENTING** — maintainer decisions taken 2026-10-31 (MIT; publishing deferred; integration guide over adapter project). Implementation on the branch, CI and independent acceptance pending.

## Deliverable

The maintainer's three decisions applied: the MIT license adopted across the repository, npm publishing kept deferred (`private: true` stays), and the WebView integration guide added as documentation — the last in-repo item from the post-roadmap decision list.

## Acceptance criteria

- [x] `LICENSE` (MIT, copyright Yusuf KEF 2026) at the repository root.
- [x] Crate manifests declare `license = "MIT"`; the npm manifest declares `"license": "MIT"` and stays `private: true` (publishing remains deferred).
- [x] `docs/webview-integration.md` — the WebView/agent host guide: package flavors, the observe → decide → act → execute loop, engine-callback mapping, session restore, and boundary reminders. Documentation only; no adapter code.
- [x] README gains License and WebView integration sections; CHANGELOG gains the M19 entry.
- [ ] Multi-platform CI green bound to the final exact PR head (existing gates unchanged).
- [ ] Independent acceptance for the final exact HEAD.

## Baseline/provenance

Governance + documentation — no semantics, no ADR required (M16 precedent). The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No registry publishing, no engine adapter code, no favicon/title inference, no new dependencies.
