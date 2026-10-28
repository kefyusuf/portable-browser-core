#![cfg(target_arch = "wasm32")]
//! Node-executed tests for the JS bridge, run via wasm-bindgen-test.
//! Empty on host targets; see ADR-011's evidence gate.

use browser_core_wasm::{resolve_navigation, WorkspaceBinding};
use js_sys::{Array, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

fn get(value: &JsValue, key: &str) -> JsValue {
    Reflect::get(value, &key.into()).expect("property read")
}

fn str_of(value: &JsValue, key: &str) -> Option<String> {
    let property = get(value, key);
    if property.is_null() || property.is_undefined() {
        None
    } else {
        Some(property.as_string().expect("string property"))
    }
}

fn num_of(value: &JsValue, key: &str) -> f64 {
    get(value, key).as_f64().expect("number property")
}

fn f64_of(value: &JsValue) -> f64 {
    value.as_f64().expect("number value")
}

fn window_at(snapshot: &JsValue, index: usize) -> JsValue {
    get(snapshot, "windows")
        .dyn_into::<Array>()
        .expect("windows array")
        .get(index as u32)
}

fn tab_at(snapshot: &JsValue, window_index: usize, tab_index: usize) -> JsValue {
    let container = get(&window_at(snapshot, window_index), "container");
    get(&container, "tabs")
        .dyn_into::<Array>()
        .expect("tabs array")
        .get(tab_index as u32)
}

#[wasm_bindgen_test]
fn resolve_navigation_classifies_all_kinds() {
    let decision = resolve_navigation("example.com");
    assert_eq!(decision.kind(), "url");
    assert_eq!(decision.value(), "https://example.com/");
    assert_eq!(decision.reason(), None);

    let decision = resolve_navigation("rust language");
    assert_eq!(decision.kind(), "search");
    assert_eq!(decision.value(), "rust language");
    assert_eq!(decision.reason(), None);

    let decision = resolve_navigation("rust:tips");
    assert_eq!(decision.kind(), "reject");
    assert_eq!(decision.value(), "");
    assert_eq!(decision.reason(), Some("unsupported_scheme".to_owned()));

    let decision = resolve_navigation("https://");
    assert_eq!(decision.kind(), "reject");
    assert_eq!(decision.reason(), Some("invalid_url".to_owned()));
}

#[wasm_bindgen_test]
fn workspace_lifecycle_and_effect_surfacing() {
    let mut workspace = WorkspaceBinding::new();
    let snapshot = workspace.new_window();
    assert_eq!(num_of(&snapshot, "activeWindowId"), 1.0);
    assert_eq!(
        str_of(&get(&snapshot, "effect"), "type"),
        Some("none".into())
    );
    assert!(get(&snapshot, "noop").is_null());

    let snapshot = workspace.container_new_tab(1, Some("https://a.test/".into()));
    assert_eq!(
        str_of(&get(&snapshot, "effect"), "type"),
        Some("start_load".into())
    );
    assert_eq!(
        Reflect::get(&get(&snapshot, "effect"), &"url".into())
            .expect("effect url")
            .as_string()
            .expect("effect url string"),
        "https://a.test/"
    );
    let tab = tab_at(&snapshot, 0, 0);
    assert_eq!(str_of(&tab, "phase"), Some("loading".into()));
    assert_eq!(str_of(&tab, "pendingUrl"), Some("https://a.test/".into()));

    let snapshot = workspace.container_commit(1, 1);
    let tab = tab_at(&snapshot, 0, 0);
    assert_eq!(str_of(&tab, "phase"), Some("active".into()));
    assert_eq!(str_of(&tab, "currentUrl"), Some("https://a.test/".into()));
    assert_eq!(str_of(&tab, "title"), None);

    let snapshot = workspace.container_set_title(1, 1, Some("Example Domain".into()));
    let tab = tab_at(&snapshot, 0, 0);
    assert_eq!(str_of(&tab, "title"), Some("Example Domain".into()));
}

#[wasm_bindgen_test]
fn invalid_events_return_noop_results_never_throw() {
    let mut workspace = WorkspaceBinding::new();
    // Unknown window: unknown_window, no throw.
    let snapshot = workspace.container_new_tab(9, Some("https://a.test/".into()));
    assert_eq!(str_of(&snapshot, "noop"), Some("unknown_window".into()));
    assert!(get(&snapshot, "activeWindowId").is_null());

    workspace.new_window();
    workspace.container_new_tab(1, Some("https://a.test/".into()));

    // Out-of-range move on an existing tab: index_out_of_range (container layer).
    let snapshot = workspace.container_move_tab(1, 1, 5);
    assert_eq!(str_of(&snapshot, "noop"), Some("index_out_of_range".into()));

    // Control-character title: invalid_title (container layer).
    let snapshot = workspace.container_set_title(1, 1, Some("bad\nline".into()));
    assert_eq!(str_of(&snapshot, "noop"), Some("invalid_title".into()));

    // Back at the history edge: at_boundary (tab-machine layer, surfaced
    // through the container outcome).
    workspace.container_commit(1, 1);
    let snapshot = workspace.container_back(1, 1);
    assert_eq!(str_of(&snapshot, "noop"), Some("at_boundary".into()));
}

#[wasm_bindgen_test]
fn window_level_semantics_mirror_the_core() {
    let mut workspace = WorkspaceBinding::new();
    workspace.new_window();
    workspace.new_window();
    // Closing the active window (2) activates the left neighbor (1): the
    // ADR-006 policy mirrored at the window level.
    let snapshot = workspace.close_window(2);
    assert_eq!(num_of(&snapshot, "activeWindowId"), 1.0);
    assert_eq!(f64_of(&get(&window_at(&snapshot, 0), "id")), 1.0);

    // Each window keeps an independent container: routing to window 1 does
    // not change the fact that the workspace has one window left.
    let snapshot = workspace.container_new_tab(1, Some("https://b.test/".into()));
    assert_eq!(
        str_of(&get(&snapshot, "effect"), "type"),
        Some("start_load".into())
    );
    assert_eq!(f64_of(&get(&snapshot, "activeWindowId")), 1.0);
}

#[wasm_bindgen_test]
fn session_passthrough_round_trips() {
    let mut workspace = WorkspaceBinding::new();
    workspace.new_window();
    workspace.container_new_tab(1, Some("https://a.test/".into()));
    workspace.container_commit(1, 1);
    workspace.container_set_title(1, 1, Some("Example Domain".into()));

    let text = workspace
        .container_session_text(1)
        .expect("session text for window 1");
    assert!(text.starts_with("browser-core-session\t2"));

    // Restore into a second window; the serialized state must be identical.
    workspace.new_window();
    workspace
        .container_set_session_text(2, &text)
        .expect("session text is well-formed");
    let restored = workspace
        .container_session_text(2)
        .expect("session text for window 2");
    assert_eq!(restored, text);

    // Malformed text is rejected as a result, not an exception.
    let error = workspace
        .container_set_session_text(1, "garbage")
        .expect_err("malformed session");
    assert_eq!(error.as_string(), Some("malformed_session".to_owned()));

    // Unknown window errors, never throws.
    let error = workspace
        .container_session_text(42)
        .expect_err("unknown window");
    assert_eq!(error.as_string(), Some("unknown_window".to_owned()));
}

#[wasm_bindgen_test]
fn favicon_flows_through_the_bridge() {
    let mut workspace = WorkspaceBinding::new();
    workspace.new_window();
    workspace.container_new_tab(1, Some("https://a.test/".into()));
    workspace.container_commit(1, 1);
    let snapshot = workspace.container_set_favicon(1, 1, Some("https://a.test/icon.ico".into()));
    assert!(get(&snapshot, "noop").is_null());
    let tab = tab_at(&snapshot, 0, 0);
    assert_eq!(
        str_of(&tab, "favicon"),
        Some("https://a.test/icon.ico".into())
    );

    // Session passthrough carries the favicon segment (v2): untitled entry
    // first, then the three-segment form once the title is reported.
    let text = workspace
        .container_session_text(1)
        .expect("session text for window 1");
    assert!(text.contains("https://a.test/~~https://a.test/icon.ico"));
    workspace.container_set_title(1, 1, Some("Example Domain".into()));
    let text = workspace
        .container_session_text(1)
        .expect("session text for window 1");
    assert!(text.contains("https://a.test/~Example Domain~https://a.test/icon.ico"));
}

#[wasm_bindgen_test]
fn workspace_session_passthrough_round_trips() {
    let mut workspace = WorkspaceBinding::new();
    workspace.new_window();
    workspace.container_new_tab(1, Some("https://a.test/".into()));
    workspace.container_commit(1, 1);
    workspace.container_set_title(1, 1, Some("Example Domain".into()));

    let text = workspace.workspace_session_text();
    assert!(text.starts_with("browser-core-workspace\t1"));

    // Restore into a fresh binding from the same text; identity holds.
    let mut fresh = WorkspaceBinding::new();
    fresh
        .workspace_set_session_text(&text)
        .expect("workspace text is well-formed");
    assert_eq!(fresh.workspace_session_text(), text);
    let snapshot = fresh.snapshot();
    assert_eq!(num_of(&snapshot, "activeWindowId"), 1.0);
    let tab = tab_at(&snapshot, 0, 0);
    assert_eq!(str_of(&tab, "title"), Some("Example Domain".into()));
    assert_eq!(str_of(&tab, "currentUrl"), Some("https://a.test/".into()));

    // Malformed text is rejected as a result, not an exception.
    let error = fresh
        .workspace_set_session_text("garbage")
        .expect_err("malformed workspace");
    assert_eq!(error.as_string(), Some("malformed_workspace".to_owned()));
}
