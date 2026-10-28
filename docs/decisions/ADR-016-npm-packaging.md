# ADR-016 — M13 npm packaging (preparation)

**Status:** Semantics accepted 2026-10-27 (maintainer go-ahead on the candidate list); implemented on `feat/m13-npm-packaging`. Milestone evidence (multi-platform CI and independent acceptance) pending.

**Implementation refinements (recorded 2026-10-27):** the generated glue exposes the Rust names verbatim (snake_case — `resolve_navigation`, `new_window`, …); camelCase aliasing stays a later decision for when a consumer exists. Binding id parameters are narrowed to **u32** (plain JS Numbers), widened losslessly to the core's `u64` — the packaging smoke test surfaced the BigInt friction of u64 parameters, and Numbers keep the JS surface symmetric (ids also serialize as Numbers in snapshots).

## Context

The M8 bridge is tested in Node but is not yet shaped as something a JavaScript project can consume: there is no package manifest, no generated-glue layout, and no test that the *assembled artifact* (not just the raw wasm harness) works. Publishing decisions — registry, scope, versioning, licensing — belong to the maintainer and stay out of scope; this milestone makes the artifact real and verified so publishing becomes a mechanical follow-up.

## Decision

- **Tooling: the pinned `wasm-bindgen-cli`, not wasm-pack.** The repo already pins `wasm-bindgen-cli 0.2.129` in CI; adding wasm-pack would introduce a second build tool with its own version drift for no capability this milestone needs. Package assembly is: `wasm-bindgen --target nodejs --out-dir pkg` over the built cdylib, plus the checked-in manifest copied in.
- **Manifest.** A checked-in `crates/browser-core-wasm/npm/package.json`: name `portable-browser-core` (provisional — a scoped/renamed decision accompanies publishing), version `0.1.0`, `main`/`types` pointing at the generated glue, `license: "UNLICENSED"` until a licensing decision, `private: true` so no tooling can accidentally publish.
- **Smoke test over the artifact.** A checked-in Node script imports the assembled `pkg/` via `require` and exercises the bridge end-to-end: `resolveNavigation` classification (all three kinds), and a workspace lifecycle (window → tab → commit → title) asserting the returned snapshot. It runs in CI after assembly and fails the build on any mismatch. This tests the **package layout**, complementing the wasm-bindgen-test suite (which tests the raw binding).
- **Generated outputs are never committed.** `pkg/` is git-ignored; every build regenerates it from source.
- **Node target first.** `--target nodejs` is what the evidence gate can actually execute in CI. Browser targets (`bundler`, `web`, `no-modules`) are mechanical additions once a real consumer exists — deferred, not forgotten.

## Explicitly out of scope

Registry publishing, scoped package naming, bundler/web/no-modules target outputs, TypeScript build configuration, license selection, WebView adapter, and CI publishing workflows.

## Evidence gate

CI assembles the package and the smoke test passes in Node on the exact commit; all pre-existing gates green; evidence bound to the exact commit. Do not mark M13 done without passing evidence.
