# Contributing

Thanks for your interest. This repository runs a disciplined,
evidence-bound process; contributions follow the same loop.

## Toolchain and verification

Rust is pinned via `rust-toolchain.toml` (1.98.1). From the repository root:

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo deny check
rustup target add wasm32-unknown-unknown
cargo check --workspace --locked --target wasm32-unknown-unknown
```

The `Cargo.lock` file is checked in; CI runs `--locked`. Generated outputs
are never committed — `crates/browser-core-wasm/pkg/` is assembled in CI.

## How change happens here

1. **Milestone first.** Work is organized as numbered milestones
   (`docs/milestones/M<N>-*.md`) with a deliverable, acceptance criteria, a
   baseline/provenance note, and a stop boundary. A change without a
   milestone document is out of process.
2. **ADR before semantics.** If a change alters accepted semantics, propose
   an ADR under `docs/decisions/` first (numbering continues from
   ADR-020). Documentation- or packaging-hygiene-only changes record
   "no ADR — no semantics" in the changelog entry instead.
3. **Branch and PR.** Branches are named `feat/m<N>-<slug>`; commits use
   conventional style with the milestone scope (`feat(m19): …`,
   `docs(m20): …`, `ci(m20): …`).
4. **Evidence gate.** Every acceptance criterion is bound to exact
   evidence: the green CI run on the exact PR head plus any
   milestone-specific golden or structural checks. A milestone is marked
   accepted only after the maintainer reviews that evidence; acceptance is
   recorded in the milestone document and the PR evidence comment.
5. **Determinism is the contract.** No OS APIs, no networking, no
   filesystem, no nondeterminism in `crates/browser-core` (ADR-004). Host
   execution is outside the core's boundary; do not pull it in.

## Dependency policy

Dependencies are exact-pinned and moved deliberately — see
[ADR-020](docs/decisions/ADR-020-dependency-pinning.md). The `wasm-bindgen`
trio (crate, `wasm-bindgen-test`, and CI's `wasm-bindgen-cli`) must move in
lockstep; a bump that updates one member without the others fails CI.
