# WebView integration guide

How to use `portable-browser-core` as the decision layer of a WebView-based
browser host — headed or headless. The division of labor is fixed by
[ADR-004](decisions/ADR-004-navigation-resolution-policy.md): **the core
decides, the host executes.** Nothing in the package performs I/O, touches
the network, or renders anything.

## What the package provides

The npm package (see [ADR-016](decisions/ADR-016-npm-packaging.md) and
[ADR-017](decisions/ADR-017-browser-targets.md)) ships three glue flavors
over one WASM binary:

| Flavor | Consumer |
|---|---|
| `web/` | A plain WebView loading a `<script type="module">` — no bundler needed |
| `bundler/` | A packaged application whose assets go through a bundler |
| `nodejs/` | A headless Node host (CI, automation, agent runtimes) |

All three expose the identical surface: `resolve_navigation`,
`WorkspaceBinding` (with its flat event methods), `observe_text()`, and the
session passthroughs.

## The integration loop

The contract an agent host implements is the loop demonstrated in
[`examples/agent-loop.cjs`](../examples/agent-loop.cjs):

1. **Observe** — `workspace.observe_text()` gives a compact, deterministic
   state projection (windows, tabs, phases, URLs, titles, favicons, and the
   phase-derived action list). This is what you hand to the agent.
2. **Decide** — the agent (LLM or policy) picks the next action. Address-bar
   inputs go through `resolve_navigation` first; only `Url` decisions become
   navigations. `Search` results are the host's to resolve against its own
   search provider.
3. **Act** — call the corresponding binding method
   (`container_new_tab`, `container_navigate`, `container_back`, …). Every
   call returns the full snapshot **plus the surfaced effect**
   (`start_load(url)`) and a `noop` reason for invalid actions.
4. **Execute the effects** — this is the WebView's job: `start_load` means
   the host navigates its actual engine to the URL. The core never touches
   the network. The engine then reports back what happened: `commit` on
   success, `fail`/`stop` on failure/cancel — addressed to the same tab id.
5. **Repeat** — every state change is observable again; the loop is
   synchronous and deterministic between engine callbacks.

Invalid actions never throw: they return `{ noop: reason }` results
(`at_boundary`, `unknown_tab`, `index_out_of_range`, …) that an LLM agent
can read and adapt to.

## Wiring the engine callbacks

The engine's lifecycle callbacks map 1:1 to core events:

| Engine event | Core call |
|---|---|
| Navigation started from the address bar | `container_new_tab` / `container_navigate` (the returned `start_load` tells the engine where to go) |
| Page loaded successfully | `container_commit` |
| Load failed | `container_fail` |
| Load cancelled (user stop) | `container_stop` |
| Page `<title>` parsed | `container_set_title` |
| Favicon known | `container_set_favicon` |

## Sessions

`workspace_session_text()` / `workspaceSetSessionText()` (ADR-012) persist
and restore the whole workspace — window strip, active window, per-tab
histories, titles, favicons, and both id counters — so a host can implement
crash recovery and session restore without defining its own format.

## Boundary reminders

- A returned URL or `start_load` effect is a **request**, not an
  authorization: the host owns network policy, redirects, and execution
  (ADR-004).
- Search-engine selection is a host concern (ADR-004); the core classifies
  search inputs and hands them back unresolved.
- Titles and favicons are host-reported facts (ADR-014/015); the core never
  invents them.

## What this repository deliberately does not provide

Rendering, DOM access, network I/O, a headless engine, or prompt templates
for any specific LLM. Those belong to the host — this package is the
deterministic decision layer between the agent and whichever engine the
host picks.
