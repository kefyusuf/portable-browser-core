# M20 — Industry-norm alignment

**State: PROPOSED** — PR #20 (2026-11-16).

## Deliverable

An industry-norm review (2026-11-16) found the engineering discipline above
average and the gaps concentrated in open-source publishing norms. This
milestone closes the mechanical gaps without touching accepted semantics:

- **Community standards.** `SECURITY.md` (private vulnerability reporting as
  the only channel; the parsing/session surfaces in scope, the ADR-004
  host-execution boundary out of scope), `CONTRIBUTING.md` (the repo's own
  loop: milestone → ADR → branch/PR → evidence gate), `CODE_OF_CONDUCT.md`
  (Contributor Covenant 2.1), issue forms and a PR evidence-gate template.
- **Supply-chain CI.** A `cargo-deny` job (RustSec advisories, licenses,
  bans, sources; `deny.toml`), SHA-pinned GitHub Actions, `rust-cache`
  caching, workflow concurrency, and `.github/dependabot.yml` (cargo +
  github-actions, weekly, grouped).
- **Cargo workspace norms.** `[workspace.package]` inheritance
  (version/edition/rust-version/license) across both crate manifests;
  `wasm-bindgen-test` exact-pinned. ADR-020 records the dependency pinning
  policy, including the wasm-bindgen trio lockstep (`0.2.N` ↔ `0.2.N` ↔
  `0.3.(N-50)`).
- **Packaging norms.** The npm manifest's `exports` map gains a nested
  `types` condition per flavor (TypeScript bundler resolution);
  `keywords`/`bugs`/`homepage` added; `verify-targets.cjs` asserts the new
  shape.
- **Evidence hygiene.** README status paragraph refreshed to M1–M19;
  CHANGELOG head hashes corrected (M6 → `d4dd983`, M7 → `caa01b9`,
  M8 → `fec6208`) and the missing M18 entry added; `.mimosa/` ignored.

## Acceptance criteria

- [ ] No accepted semantics change: core behavior identical; the
      `--locked` commands pass unchanged after the workspace-inheritance
      refactor (no lockfile change).
- [ ] `cargo deny check` passes locally and in CI on the exact head.
- [ ] CI green (3-OS matrix + cargo-deny job) on the exact PR head.
- [ ] The assembled package passes `verify-targets.cjs` and the Node smoke
      test with the nested-`types` exports map.
- [ ] Community files present; `SECURITY.md` points at private
      vulnerability reporting only.
- [ ] CHANGELOG head hashes verified against git history for every M1–M19
      entry; the M18 entry present.

## Baseline/provenance

Derived from the 2026-11-16 industry-norm review of this repository; no
prior archive to reconstruct. Existing conventions are preserved
(evidence-bound milestones, ADR discipline, pinned toolchain).

## Stop boundary

No semantics change, no new dependencies in `crates/browser-core`, no
publishing, no release automation, no npm scoped-name decision (ADR-016
pending), no fuzz targets (a future milestone), no edition bump (a future
milestone).
