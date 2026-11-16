//! Totality + canonical round-trip oracle over the workspace envelope
//! (ADR-012, ADR-021): any host-provided text parses or reasons — never
//! panics — and accepted text keeps its view through serialize →
//! deserialize, while the observation text (ADR-019) renders without
//! panicking on any restored state.
#![no_main]

use browser_core::Workspace;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let Ok(workspace) = Workspace::from_workspace_text(text) else {
        return;
    };
    let view = workspace.state();
    assert!(
        !workspace.observe_text().is_empty(),
        "observation text must render for any restored workspace"
    );
    let serialized = workspace.to_workspace_text();
    let reparsed = Workspace::from_workspace_text(&serialized)
        .expect("workspace re-parse must accept own bytes");
    assert_eq!(
        reparsed.state(),
        view,
        "workspace round-trip changed the state view"
    );
});
