# Portable Browser Core

A platform-independent browser decision core, starting with deterministic navigation resolution in Rust.

**Status:** Bootstrap repository. The previously designed M0/M1 baseline has not yet been verified against a GitHub revision.

## Scope boundary

The core resolves commands into decisions/effects without owning OS APIs, browser UI, WebView, filesystem, or networking. This repository is **not** a browser implementation.

## Milestone sequence

- **M0:** Establish the repository and explicit core boundary.
- **M1:** Deterministic address/navigation resolution (`URL`, `Search`, `Rejection`), golden fixtures, native CI and WASM compile check.
- **M2:** Tab state machine — **blocked until M1 passes CI and is accepted**.

Feature work occurs through reviewable PRs. No M1 completion claim until the exact PR head has passing verification.
