// Smoke test over the assembled npm package (ADR-016): the artifact in
// pkg/ must import and behave like the bridge. Run by CI after assembly;
// exits non-zero on any mismatch. The JS surface uses the Rust names
// verbatim (snake_case); camelCase aliasing is a later decision.
const assert = require('node:assert');
const { resolve_navigation, WorkspaceBinding } = require('../pkg/browser_core_wasm.js');

// Navigation classification, all three kinds.
const url = resolve_navigation('example.com');
assert.strictEqual(url.kind, 'url');
assert.strictEqual(url.value, 'https://example.com/');

const search = resolve_navigation('rust language');
assert.strictEqual(search.kind, 'search');
assert.strictEqual(search.value, 'rust language');

const rejected = resolve_navigation('rust:tips');
assert.strictEqual(rejected.kind, 'reject');
assert.strictEqual(rejected.reason, 'unsupported_scheme');

// Workspace lifecycle: window → tab → commit → title, over the snapshot.
// Binding ids are u32 (plain JS Numbers), widened to the core's u64.
const workspace = new WorkspaceBinding();
const created = workspace.new_window();
assert.strictEqual(created.activeWindowId, 1);

const loading = workspace.container_new_tab(1, 'https://a.test/');
assert.strictEqual(loading.effect.type, 'start_load');
assert.strictEqual(loading.effect.url, 'https://a.test/');
assert.strictEqual(loading.windows[0].container.tabs[0].phase, 'loading');

const committed = workspace.container_commit(1, 1);
assert.strictEqual(committed.windows[0].container.tabs[0].phase, 'active');
assert.strictEqual(committed.windows[0].container.tabs[0].currentUrl, 'https://a.test/');

const titled = workspace.container_set_title(1, 1, 'Example Domain');
assert.strictEqual(titled.windows[0].container.tabs[0].title, 'Example Domain');

// Total semantics: invalid events return results, never throw.
const unknown = workspace.container_new_tab(99, 'https://x.test/');
assert.strictEqual(unknown.noop, 'unknown_window');

console.log('npm package smoke test passed');
