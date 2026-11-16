# ADR-020 — Dependency pinning policy

**Status:** Proposed (PR #20, 2026-11-16). See the M20 milestone for the evidence index.

## Context

The workspace pins its dependencies exactly (`url = "=2.5.4"`,
`wasm-bindgen = "=0.2.129"`, `js-sys = "=0.3.106"`,
`wasm-bindgen-test = "=0.3.79"`) and enforces reproducibility with a
checked-in `Cargo.lock` and `--locked` CI. An industry-norm review (M20)
flagged the tension: exact manifest pins are an application/reproducibility
norm; published libraries normally declare caret ranges so dependents can
unify versions across the graph. The policy below records the current
practice and the transition rule so the choice is deliberate, not
accidental, and so future bumps keep a fragile trio in sync.

## Decision

- **While unpublished: exact pins in manifests, truth in the lockfile.**
  Both crates are `publish = false`; `Cargo.lock` is checked in and CI runs
  `--locked`. Exact manifest pins make the reproducibility intent visible
  and keep local, CI, and contributor environments byte-identical.
- **At the crates.io publishing decision: libraries move to caret ranges.**
  `browser-core` (and any published bridge manifest) switches from
  `url = "=2.5.4"` to `url = "2.5.4"` — dropping `=`, not choosing a newer
  floor. The workspace's own builds stay byte-pinned by `Cargo.lock`; the
  caret lets dependents unify versions. The npm manifest's version policy
  accompanies the publishing ADR, not this one.
- **The wasm-bindgen trio moves in lockstep.** The `wasm-bindgen` crate and
  the `wasm-bindgen-cli` pinned in CI (`core-ci.yml`) share the same
  `0.2.N`; `wasm-bindgen-test` is `0.3.(N-50)` — today `0.2.129` ↔
  `0.2.129` ↔ `0.3.79`, with `0.2.130` ↔ `0.3.80` as the next pairing. A
  Dependabot bump of one member must update the other two in the same
  change; a mismatched trio fails CI's wasm-bindgen-test step.
- **The toolchain is pinned independently.** `rust-toolchain.toml`
  (1.98.1) and the manifest `rust-version` (MSRV floor 1.98) version
  separately from dependency pins; neither follows dependency bumps.

## Explicitly out of scope

Publishing itself (registry, scope, package naming — ADR-016 pending),
release automation and versioning of releases, fuzz targets, edition
selection, and any concrete dependency version change; this ADR records
policy, not versions.

## Evidence gate

The workspace resolves identically under `--locked` after the
`[workspace.package]` inheritance refactor (no lockfile change), all
pre-existing gates green, and `cargo deny check` introduced in M20 passes.
Do not merge without passing evidence.
