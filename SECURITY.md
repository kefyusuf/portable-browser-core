# Security Policy

## Reporting a vulnerability

Do **not** open a public issue for a suspected vulnerability. Use GitHub's
**private vulnerability reporting** for this repository
(**Security → Report a vulnerability**), which notifies the maintainer
directly and privately. Include a minimal reproduction and, if possible,
the affected input surface.

## Scope

**In scope** — surfaces that parse untrusted or host-provided text:

- `resolve_navigation(&str)` — address-bar classification (ADR-004)
- `TabContainer::from_session_text` / `Workspace::from_workspace_text` —
  session format parsing and escaping (ADR-009, ADR-012)
- `Workspace::observe_text` rendering (ADR-019), insofar as
  host-controlled titles, favicons, or URLs flow into it
- the WASM/JS bridge surface in `crates/browser-core-wasm`

**Out of scope** — by the ADR-004 frozen boundary the core executes
nothing: any issue in a *host's* execution of a returned decision (network
policy, WebView, engine, filesystem) belongs to that host. A returned
`Url` is a decision, never an authorization.

## Supported versions

The `main` branch and `0.1.x` snapshots built from it. There is no release
channel yet (publishing is a pending maintainer decision, ADR-016).

## Expectations

The core's contract is totality: parsing accepts or reasons, and never
panics on input. A panic, hang, or unbounded memory growth reachable from
the text surfaces above is treated as a security-relevant defect.
