// Structural verification over the assembled multi-target package
// (ADR-017): every target's directory must contain its glue and `.wasm`,
// and every glue must expose the bridge's public names. This proves the
// artifact shape — browser execution belongs to the WebView adapter
// milestone. Run by CI after assembly; exits non-zero on any mismatch.
const assert = require('node:assert');
const fs = require('node:fs');
const path = require('node:path');

const pkg = path.join(__dirname, '..', 'pkg');

const targets = {
  nodejs: ['browser_core_wasm.js', 'browser_core_wasm_bg.wasm', 'browser_core_wasm.d.ts'],
  bundler: [
    'browser_core_wasm.js',
    'browser_core_wasm_bg.js',
    'browser_core_wasm_bg.wasm',
    'browser_core_wasm.d.ts',
  ],
  web: ['browser_core_wasm.js', 'browser_core_wasm_bg.wasm', 'browser_core_wasm.d.ts'],
};

// The public surface every glue flavor must carry (Rust names verbatim).
const publicNames = [
  'resolve_navigation',
  'WorkspaceBinding',
  'new_window',
  'close_window',
  'activate_window',
  'container_new_tab',
  'container_navigate',
  'container_commit',
  'container_back',
  'container_forward',
  'container_reload',
  'container_move_tab',
  'container_set_title',
  'container_set_favicon',
  'container_session_text',
  'workspace_session_text',
  'observe_text',
];

for (const [target, files] of Object.entries(targets)) {
  const dir = path.join(pkg, target);
  for (const file of files) {
    assert.ok(
      fs.existsSync(path.join(dir, file)),
      `${target}/${file} missing from the assembled package`
    );
  }
  // Flavors differ in layout: nodejs/web keep everything in one file, the
  // bundler flavor splits the class into browser_core_wasm_bg.js — so the
  // name check scans every .js file in the target directory.
  const glue = fs
    .readdirSync(dir)
    .filter((file) => file.endsWith('.js'))
    .map((file) => fs.readFileSync(path.join(dir, file), 'utf8'))
    .join('\n');
  for (const name of publicNames) {
    assert.ok(
      glue.includes(name),
      `${target} glue does not expose ${name}`
    );
  }
}

// The manifest wires each consumer to the right flavor.
const manifest = JSON.parse(fs.readFileSync(path.join(pkg, 'package.json'), 'utf8'));
assert.strictEqual(manifest.exports['.'].node, './nodejs/browser_core_wasm.js');
assert.strictEqual(manifest.exports['.'].bundler, './bundler/browser_core_wasm.js');
assert.strictEqual(manifest.exports['.'].default, './web/browser_core_wasm.js');

console.log('multi-target package structural verification passed');
