# M21 — Fuzz targets

**State: DONE** — accepted and merged 2026-11-16 (PR #24, accepted head `0e27262`, merge `6f3d289`; standing-acceptance CI run [43947104997](https://github.com/kefyusuf/portable-browser-core/actions/runs/43947104997): all three campaigns crash-free — 1.39M / 15.19M / 14.09M runs — after the M22 ruling landed).

## Deliverable

Coverage-guided fuzzing over the core's text-input surfaces, proving the
totality contract ("parsing accepts or reasons, never panics") against
arbitrary host-provided text — the deferred P3 finding of the 2026-11-16
industry-norm review and an explicit item in the M20 stop boundary.

Three text surfaces take untrusted input (see SECURITY.md scope):

1. `resolve_navigation(&str)` — address-bar classification (ADR-004)
2. `TabContainer::from_session_text(&str)` — session format v2, escaping
   and structural validation (ADR-009, ADR-014, ADR-015)
3. `Workspace::from_workspace_text(&str)` — envelope embedding container
   sessions verbatim (ADR-012)

For each surface a fuzz harness asserts **no panic, no hang, no unbounded
growth**. For surfaces 2–3, accepted inputs additionally feed a
round-trip oracle: `to_text(from_text(x))` re-parses to an equivalent
state, and `observe_text()` renders without panicking. Any round-trip
instability found is triaged as a finding, not silently accepted — the
oracles test the documented contract, they do not add semantics.

Tooling: `cargo-fuzz` (libFuzzer backend) under a `fuzz/` crate kept
**outside** the workspace, so the shipped crates' `Cargo.lock` is
untouched; the nightly toolchain is pinned by date (ADR-020 spirit);
golden TSV fixtures seed the corpus; CI runs a bounded campaign
(ubuntu-only — `cargo-fuzz` does not support the local Windows host, the
same evidence-gate precedent as the wasm steps).

## Acceptance criteria

- [x] `fuzz/` targets for all three surfaces compile and run under the
      pinned nightly on ubuntu CI.
- [x] A bounded campaign (≥ 60 s per target) is crash-free on the exact
      head; any finding is triaged before the milestone can be marked
      done. *(Two findings triaged: the allocation abort fixed in this
      milestone; the empty-title break ruled in M22 — the red gate in
      between was the evidence.)*
- [x] Golden fixtures wired as corpus seeds; seeds checked in, campaign
      artifacts git-ignored.
- [x] All pre-existing gates green; accepted semantics unchanged; the
      main workspace's `Cargo.lock` unchanged. *(The shipped lockfile is
      untouched; the ruling itself rode with M22 per the maintainer's
      in-session approval.)*
- [x] CI green (3-OS matrix + cargo-deny + cargo-fuzz jobs) on the exact
      PR head, with the fuzz job's campaign log as evidence. *(Run
      43947104997, 5/5 jobs, on head `0e27262`.)*

## Triage (first campaigns, runs 43934990697 / 43936926830)

Two findings from the first bounded campaigns — both minimized and
root-caused; the gate worked as designed:

1. **Allocation abort from untrusted header counts** (both parsers).
   `Vec::with_capacity(tab_count / window_count)` trusted the declared
   count; a hostile header (e.g. `header\t67440737191031\t…`) aborted with
   allocation-size-too-big — a totality violation. **Fixed in this
   milestone:** the pre-allocation hint is removed (memory stays
   proportional to the input; the existing count checks keep validation
   total); hostile-count rejection cases pinned in both golden test
   files. Semantics-neutral — no accepted decision changes.
2. **The session format cannot represent an empty title.**
   `SetTitle("")` is a valid event (only control bytes are rejected), but
   format v2 reads an empty segment as `None` — so a live state with
   `title: Some("")` saves as `url~` and restores as `title: None`. The
   v1 parser reaches the same state through an empty `t:` payload
   (minimized input: `tab\t1\tt:\t0\t-\thttps://a.test/`). A round-trip
   break over event-reachable states: a semantics ruling is required
   (empty ≡ absent everywhere, event-layer rejection, or a format
   change). **Pending maintainer decision** — the session campaign stays
   red until it lands.

## Baseline/provenance

Derived from the 2026-11-16 industry-norm review (P3: parser surfaces
without fuzzing) and the M20 stop boundary. No prior archive; no
semantics change is proposed — fuzzing is test infrastructure.

## Stop boundary

No change to accepted semantics, no new runtime dependencies, no public
API change (harnesses are `#[cfg(fuzzing)]`-gated inside `fuzz/`), no
OSS-Fuzz continuous integration, no coverage thresholds, no new decision
areas.
