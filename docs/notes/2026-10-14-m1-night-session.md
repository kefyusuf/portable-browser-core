# M1 session evidence — 2026-10-14 (Europe/Istanbul)

## Scope guard
- Repository: `kefyusuf/portable-browser-core`
- PR: #1, `feat/m1-navigation-baseline` -> `main`
- Milestone: M1 only, deterministic URL/Search/Rejection decisions
- Excluded: platform integration, network calls, M2 tab state, release, merge
- Original M0/M1 archive unavailable; no claim that the reconstructed source is byte-equivalent

## Observations before this update
- Exact branch head: `a7f64a01f0dd760d905e6a354a77a9630a8b1d7f`; PR remained draft and unmerged
- Run #11 on `cec9be5`: Linux/macOS/Windows + Linux WASM compile: PASS
- No submitted independent reviews or unresolved review threads in PR #1
- `Cargo.lock` was missing, although the direct dependency `url = "=2.5.4"` was pinned

## Reproducibility remediation
- A temporary diagnostic step in GitHub Actions run #12 (run ID `38471007257`) printed the Cargo-generated lockfile from the Ubuntu runner using Rust 1.98.1
- The run completed green on three hosts
- Extracted the generated lockfile without manual dependency-version selection (33 package records; `url@2.5.4`)
- Added `Cargo.lock` and `--locked` on CI cargo test/clippy/check commands
- Removed temporary diagnostic log emission
- The final new HEAD requires a fresh CI run before green can be claimed

## Review/acceptance pending
- Independent review of behavior and host/core boundary
- Original M0/M1 archive comparison **or** explicit adoption of the new reconstructed baseline
- No merge, no M2 until those gates close

## Method
All source-of-truth changes go through the M1 feature branch. Record follow-up commit hashes, CI run IDs, and anomalies in PR #1 comments. Do not treat a previous green revision as current-head evidence.
