# M13 — npm packaging (preparation)

**State: IMPLEMENTING** — ADR-016 accepted 2026-10-27; implementation on the branch (pkg assembly + smoke test verified locally), CI and independent acceptance pending.

## Deliverable

A reproducible npm package assembly for the M8 bridge: a checked-in package manifest, a `wasm-bindgen --target nodejs` generation step, and a Node smoke test that imports the assembled package and exercises it. Publishing to a registry is explicitly out of scope.

## Acceptance criteria

- [x] A checked-in `package.json` template for the bridge (name `portable-browser-core`, provisional version, `UNLICENSED` until a licensing decision) with `main`/`types` pointing at the generated glue.
- [x] CI (Ubuntu) assembles `pkg/`: `wasm-bindgen --target nodejs` over the pinned cli, the manifest copied in, and the directory verified to contain glue, `.wasm`, and types.
- [x] A Node smoke test imports the assembled package and exercises it end-to-end: navigation classification, a workspace lifecycle (window → tab → commit → title), and an effect surfacing — failing the build on any mismatch.
- [x] The generated `pkg/` directory is git-ignored; nothing generated is committed.
- [x] No new Rust dependencies; the only new tool usage is the already-pinned `wasm-bindgen-cli`.
- [ ] Multi-platform CI green bound to the final exact PR head (existing gates unchanged, plus the packaging step).
- [ ] Independent acceptance of ADR-016 for the final exact HEAD.

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No registry publishing, no bundler/web target outputs, no TypeScript build setup beyond the generated definitions, no scoped package naming decisions, no WebView adapter until this milestone is accepted.
