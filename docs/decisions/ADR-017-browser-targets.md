# ADR-017 — M14 browser target outputs

**Status:** Accepted and merged 2026-10-27 (PR #14, accepted head `df83d62`, CI run 40912565912). See the M14 milestone for the evidence index.

## Context

M13 ships the bridge as an npm artifact with a single `nodejs` target. A WebView or bundler-based consumer needs the browser flavors: `bundler` (ES module importing the `.wasm` — for webpack/Vite-style consumers) and `web` (standalone ES module fetching the `.wasm` — for a plain `<script type="module">`, i.e. the WebView adapter's shape). npm's `exports` map exists precisely to hand each consumer the right flavor.

## Decision

- **Three targets, one cdylib.** CI assembles `pkg/nodejs/`, `pkg/bundler/`, and `pkg/web/` from the same pinned `wasm-bindgen-cli` and the same cdylib build — no extra compilation, three glue flavors over one binary.
- **npm `exports` map.** The checked-in manifest gains the standard conditional exports: `node` → the nodejs glue, `bundler` → the bundler glue, `default` → the web glue; `files` covers all three directories. Consumers get the right flavor without configuration.
- **Structural verification, honestly scoped.** The evidence for browser targets in CI is a checked-in Node script that asserts, per target: the glue and `.wasm` files exist, and the glue's source contains the bridge's public names (`resolve_navigation`, `WorkspaceBinding`, and the flat method set). This proves the artifact shape, **not** browser execution — a headless-browser E2E belongs to the WebView adapter milestone and is deliberately deferred.
- **The nodejs smoke test stays unchanged** — the nodejs target remains the executable evidence in CI.
- **No new dependencies, no committed generated artifacts** (`pkg/` stays git-ignored).

## Explicitly out of scope

Headless-browser E2E, TypeScript build configuration, registry publishing, license decision, WebView adapter, and generalized bundler compatibility claims.

## Evidence gate

CI assembles all three targets, the structural verification passes in Node, the nodejs smoke test still passes, and all pre-existing gates stay green on the exact commit. Do not mark M14 done without passing evidence.
