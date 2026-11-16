## What

<!-- One paragraph: what this PR delivers. -->

## Milestone / ADR

- Milestone: `docs/milestones/M<N>-*.md` (link)
- ADR (if semantics changed): `docs/decisions/ADR-*` (link) — or state "no ADR — no semantics"

## Evidence gate

- [ ] All acceptance criteria in the milestone document are checked
- [ ] `cargo fmt --all -- --check` green on the exact head
- [ ] `cargo test --workspace --locked` green on the exact head
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` green on the exact head
- [ ] `cargo deny check` green on the exact head
- [ ] CI run on the exact head is green (link the run)
- [ ] `CHANGELOG.md` entry added; generated outputs (`pkg/`) not committed

<!-- Acceptance is the maintainer's act: record the acceptance and CI run in the milestone document and the PR evidence comment before marking the milestone done. -->
