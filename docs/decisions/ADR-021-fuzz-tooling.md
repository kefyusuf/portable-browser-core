# ADR-021 — Fuzz tooling and surfaces

**Status:** Proposed (2026-11-16) — see the M21 milestone for the scope draft. Semantics untouched; this ADR records a tooling and evidence-gate decision, in the style of ADR-011's tooling choice.

## Context

The core's totality contract — every input yields a reasoned outcome,
never a panic — is enforced today by golden fixtures and targeted tests
over curated inputs. The 2026-11-16 industry-norm review flagged that the
text-input surfaces (`resolve_navigation`, the session-format parsers)
take untrusted host text, and no coverage-guided fuzzing exists. Industry
norm for parsers of untrusted input is fuzzing; the M20 stop boundary
deferred it to a dedicated milestone.

## Decision

- **Tool: `cargo-fuzz` (libFuzzer backend).** The official rust-fuzz
  tool; coverage-guided, lowest in-repo setup. AFLplus (heavier setup),
  OSS-Fuzz (continuous external infrastructure — out of scope), and
  `proptest` (not coverage-guided; complementary, not chosen here) are
  rejected for this milestone.
- **Isolation: `fuzz/` stays outside the workspace.** A separate crate
  with a path dependency on `browser-core`; its `libfuzzer-sys` dev
  dependencies never enter the shipped crates' `Cargo.lock`. The main
  workspace resolves identically under `--locked`.
- **Nightly pinned by date.** `cargo-fuzz` requires nightly; the fuzz
  job pins `nightly-<YYYY-MM-DD>` so toolchain drift cannot flip a green
  gate — same reproducibility discipline as the pinned 1.98.1 stable.
- **Surfaces and oracles.** Three harnesses over the three text surfaces
  (navigation classification, container session parse, workspace session
  parse). Primary oracle: totality (no panic/hang/unbounded growth).
  Secondary oracle on the two session parsers: canonical round-trip
  through `to_text`/`from_text` plus a panic-free `observe_text()`
  render. Oracles assert the documented contract only.
- **Corpus: golden fixtures as seeds.** The existing TSV golden rows and
  checked-in session fixtures seed the corpus; campaign artifacts are
  git-ignored, seeds are committed.
- **CI: ubuntu-only bounded campaign.** `cargo-fuzz` does not support the
  Windows host (libFuzzer backend), so — like the wasm and npm steps —
  the evidence gate runs on ubuntu in CI, with a bounded `-max_total_time`
  campaign per harness and crash artifacts uploaded on failure.
- **Windows caveat recorded.** Local fuzzing on the maintainer's Windows
  host is not available; local verification covers compilation-adjacent
  checks only, CI carries the campaign evidence.

## Explicitly out of scope

Semantics changes of any kind, OSS-Fuzz integration, continuous/long-run
fuzzing infrastructure, coverage thresholds, snapshot-JSON surface (the
bridge's structured snapshot path — a later surface if a real consumer
demands it), and property-based testing (`proptest`) as a separate
complementary milestone.

## Evidence gate

Fuzz harnesses compile and run the bounded campaign crash-free in CI on
the exact head, with the campaign log as evidence; all pre-existing gates
green; main workspace lockfile unchanged. Do not mark M21 done without
passing evidence.
