# M1 — Navigation decision core

**State: VERIFYING** (not DONE).

## Deliverable

A pure Rust resolver with three outcomes, a 30-row deterministic fixture, and a constrained toolchain. This is a **reconstruction** of the prior planned M1 baseline, not an assertion that the earlier external ZIP has been imported verbatim.

## Acceptance criteria

- [x] Public decision API exists with `Url`, `Search`, `Rejection`.
- [x] Rejection for all non-HTTP(S) URI schemes.
- [x] Inferred HTTPS for ordinary dotted hostnames, HTTP for localhost/IPv4.
- [x] `1.2.3` and `127.1` remain searches.
- [x] 30 golden fixtures exist in the branch (source inspection only).
- [ ] Latest PR head passes format/test/clippy on Linux, macOS and Windows, and `wasm32-unknown-unknown` compilation on Linux. (Previously green at `80e1f2e`; **new head must be rerun**.)
- [ ] Independent review checks negative input cases and boundary fidelity.
- [ ] The original M0/M1 ZIP is compared **or** reconstructed behavior is explicitly accepted as the new canonical baseline; an identical-source claim is not permitted without the original archive.

## Stop boundary

No M2 tab state machine work, browser adapters, OS APIs, or WASM JavaScript binding until this milestone is accepted. CI is an acceptance gate, not proof of behavior not covered by the test cases.
