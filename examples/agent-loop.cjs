// Agent loop example (ADR-019, M18): the intended integration pattern for
// agents/LLMs over the assembled npm package.
//
//   observe → decide → act → host effects → observe …
//
// The core decides; the "host" (this script) executes effects — here it
// only prints them, which is where a headless engine or a browser would
// act. Deterministic: no network, no randomness, no LLM calls. The
// "decisions" are scripted so the loop is reproducible in CI.
const assert = require('node:assert');
const { resolve_navigation, WorkspaceBinding } = require('../crates/browser-core-wasm/pkg/nodejs/browser_core_wasm.js');

const log = (...parts) => console.log(...parts);

// ── Step 1: the agent classifies an address-bar input (M1). ──────────────
for (const input of ['example.com', 'rust language', 'rust:tips']) {
  const decision = resolve_navigation(input);
  log(`[decide] address bar ${JSON.stringify(input)} -> ${decision.kind}` +
    (decision.kind === 'reject' ? ` (${decision.reason})` : ` ${decision.value}`));
}

// ── Step 2: the agent opens a tab with a Url decision. ───────────────────
const decision = resolve_navigation('example.com');
assert.strictEqual(decision.kind, 'url');
const workspace = new WorkspaceBinding();
const opened = workspace.new_window();
log(`[observe] ${opened.activeWindowId === 1 ? 'window 1 open' : 'unexpected'}`);

const loading = workspace.container_new_tab(1, decision.value);
log(`[act] container_new_tab -> effect ${loading.effect.type} ${loading.effect.url}`);
log('[host] would execute the load now (headless engine or browser) -> commit');

// ── Step 3: observe is the agent's eyes (ADR-019). ───────────────────────
let observation = workspace.observe_text();
log(`[observe]\n${observation.trimEnd()}`);
assert.ok(
  observation.includes('tab 1 loading pending=https://example.com/'),
  'observation shows the loading tab and its destination'
);

// ── Step 4: the host reports the load succeeded; the agent reads state. ──
const committed = workspace.container_commit(1, 1);
log('[act] commit (host report)');
observation = workspace.observe_text();
assert.ok(
  observation.includes('tab 1 active url=https://example.com/'),
  'observation shows the committed page'
);
log(`[observe]\n${observation.trimEnd()}`);

// ── Step 5: navigate deeper, then back over per-tab history. ─────────────
workspace.container_navigate(1, 1, 'https://example.com/docs');
const committedDocs = workspace.container_commit(1, 1);
assert.strictEqual(
  committedDocs.windows[0].container.tabs[0].currentUrl,
  'https://example.com/docs'
);

const back = workspace.container_back(1, 1);
assert.strictEqual(back.effect.type, 'start_load');
assert.strictEqual(back.effect.url, 'https://example.com/');
log('[act] back -> effect start_load https://example.com/');
log('[observe] back landed on the remembered entry');

// ── Step 6: a reasoned no-op is a tool result the agent adapts to. ───────
const atBoundary = workspace.container_back(1, 1);
assert.strictEqual(atBoundary.noop, 'at_boundary');
log(`[tool-error] back -> ${atBoundary.noop} (the agent picks a different action)`);

// ── Step 7: the session survives — save and restore over the bridge. ─────
const session = workspace.workspace_session_text();
const restored = new WorkspaceBinding();
restored.workspace_set_session_text(session);
assert.strictEqual(restored.observe_text(), workspace.observe_text());
log('[session] saved and restored; observation identical');

log('agent loop example passed');
