# M14 — Browser target outputs

**State: DONE** — accepted and merged 2026-10-27.

## Deliverable

The npm package gains `bundler` and `web` target outputs alongside the existing nodejs output, wired through an npm `exports` map, with a structural verification script proving every target's artifacts exist and expose the bridge's public names. Browser E2E execution remains the WebView adapter milestone.

## Acceptance criteria

- [x] CI assembles `pkg/nodejs/`, `pkg/bundler/`, and `pkg/web/` from the same pinned `wasm-bindgen-cli` and cdylib build.
- [x] The checked-in manifest gains an npm `exports` map: `node` → the nodejs glue, `bundler` → the bundler glue, `default` → the web glue; `files` covers all three targets.
- [x] A checked-in structural verification script (Node) asserts, for every target, that the glue and `.wasm` exist and the glue exposes the bridge's public surface (`resolve_navigation`, `WorkspaceBinding` with the flat method set). The bundler flavor splits the class into `browser_core_wasm_bg.js`, so the name check scans every `.js` file in the target directory.
- [x] The existing nodejs smoke test keeps passing unchanged (require path moved to `pkg/nodejs/`).
- [x] No new Rust dependencies; no committed generated artifacts.
- [x] Multi-platform CI green bound to the final exact PR head (existing gates unchanged, plus the extended packaging step). *(Core CI run [40912565912](https://github.com/kefyusuf/portable-browser-core/actions/runs/40912565912) PASS on head `df83d62`; Verify multi-target package structure step success.)*
- [x] Independent acceptance of ADR-017 for the final exact HEAD. *(Maintainer accepted 2026-10-27 by approving the merge of PR #14. Semantics acceptance recorded 2026-10-27 in the design round; the local Windows verification record — including a local three-target assembly and verification run — is in the PR evidence comment.)*

## Baseline/provenance

New design — no prior archive to reconstruct. The canonical identity will be bound to the accepted PR head SHA and its green CI run, recorded in the PR evidence comment.

## Stop boundary

No headless-browser E2E, no TypeScript build configuration, no registry publishing, no WebView adapter, no license decision until this milestone is accepted.
