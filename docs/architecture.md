# Architecture

The repository is a **platform-independent browser decision core**: it
resolves commands into deterministic decisions and effect *requests* — it
owns no OS APIs, no UI, no WebView, no filesystem, no networking. Hosts
execute; the core decides. Every layer is pure, synchronous, and
deterministic: the same event sequence over the same initial state yields
identical states and effects on every platform.

## Crates

```
crates/
  browser-core/        The pure decision core (dependency: url only)
    src/navigation.rs  M1  address-bar input → Url | Search | Rejection
    src/tab.rs         M2  TabMachine — per-tab lifecycle + session history
    src/container.rs   M3–M6  TabContainer — ordered tab set, titles,
                       session format v2 (M6)
    src/workspace.rs   M7+M15  Workspace — ordered window set, reordering
  browser-core-wasm/   The JS bridge (wasm-bindgen; the only crate allowed
                       host dependencies)
```

## Layer diagram

```
Workspace (M7/M15)
  └── ordered TabContainers (M3–M6)
        └── TabMachine (M2)
              └── consumes targets resolved by resolve_navigation (M1)
```

Each layer is total over its events: every (event, state) combination
returns a reasoned outcome — invalid combinations are no-ops with reasons
(`unexpected_event`, `at_boundary`, `unknown_tab`, `index_out_of_range`,
`invalid_title`, `invalid_favicon`, `nothing_to_reload`, `nothing_to_title`,
`nothing_to_favicon`, `unknown_window`), never panics. Routing is
id-addressed at every level, so background tabs and windows receive load
reports while something else is visible.

## Core API surface

| Layer | Entry points | Notes |
|---|---|---|
| Navigation (M1) | `resolve_navigation(&str) -> NavigationDecision` | `Url(String)` is a decision, **not** authorization |
| Tab (M2) | `TabMachine::handle(TabEvent) -> TabOutcome`, `snapshot()`/`from_snapshot()` | `Navigate`/`Commit`/`Fail`/`Stop`/`Back`/`Forward`/`Reload`/`SetTitle`/`SetFavicon` |
| Container (M3–M6) | `TabContainer::handle(ContainerEvent)`, `to_session_text()`/`from_session_text()` | events address tabs by id; titles/favicons are per history entry (ADR-014/015) |
| Workspace (M7/M15) | `Workspace::handle(WorkspaceEvent)`, `to_workspace_text()`/`from_workspace_text()` | `MoveWindow` (ADR-018); the workspace format embeds container sessions verbatim (ADR-012) |

Outcomes carry at most one effect request: `TabEffect::StartLoad(url)` — a
request the host executes under its own network policy. A returned URL is
never an authorization (ADR-004).

## JS bridge (M8, M10, M13, M14)

`crates/browser-core-wasm` exposes the same semantics over wasm-bindgen with
the Rust names verbatim (`resolve_navigation`, `WorkspaceBinding`,
`new_window`, `container_navigate`, …). Invalid events return `{ noop }`
results and never throw. Sessions (container and workspace formats) pass
through verbatim. The npm package assembles three glue flavors (`nodejs`,
`bundler`, `web`) behind an `exports` map (ADR-016/017).

## Session formats

- Container v2 (`browser-core-session\t2`): per-tab lines with
  `url[~title[~favicon]]` entries; readers accept v1 (no titles). (ADR-009,
  ADR-014, ADR-015.)
- Workspace v1 (`browser-core-workspace\t1`): a header plus per-window
  embedded container blocks verbatim. (ADR-012.)

Both are validated and total: malformed input yields `malformed_session` /
`malformed_workspace`, never a panic. Writers emit v2 / v1 respectively.

## Verification

Local (Rust 1.98.1, pinned in `rust-toolchain.toml`):

```sh
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
rustup target add wasm32-unknown-unknown --toolchain 1.98.1
cargo check --workspace --locked --target wasm32-unknown-unknown
```

CI (`.github/workflows/core-ci.yml`) runs the same gates on Linux, macOS,
and Windows; Ubuntu additionally builds the cdylib, runs the Node
wasm-bindgen-test suite, assembles the npm package, verifies its structure,
and runs the package smoke test. Golden fixtures pin exact bytes and views
per step; count drift guards protect every fixture file.

## Decision records

ADR-004 through ADR-018 under `docs/decisions/` capture every accepted
policy, each with its milestone evidence under `docs/milestones/`. New work
starts docs-first: milestone document + ADR, maintainer acceptance, then
implementation with golden fixtures and revision-bound CI evidence.
