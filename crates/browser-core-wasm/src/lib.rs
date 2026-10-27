//! Thin, decision-free JS bridge over the browser decision core.
//!
//! Every decision stays in `browser-core`; this crate only converts types.
//! Invalid events return `{ noop }` results and never throw. Returned URLs
//! and `start_load` effects are requests, never authorizations: the JS host
//! owns DNS, fetches, redirects, and execution.

use browser_core::{
    resolve_navigation as core_resolve_navigation, ContainerEvent, NavigationDecision, TabEffect,
    TabEvent, TabPhase, Workspace, WorkspaceEvent,
};
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;

fn set_optional_string(target: &Object, key: &str, value: Option<&str>) {
    let value = match value {
        Some(value) => JsValue::from(value),
        None => JsValue::NULL,
    };
    let _ = Reflect::set(target, &key.into(), &value);
}

fn phase_str(phase: TabPhase) -> &'static str {
    match phase {
        TabPhase::Empty => "empty",
        TabPhase::Loading => "loading",
        TabPhase::Active => "active",
    }
}

fn effect_snapshot(effect: &TabEffect) -> JsValue {
    let effect_obj = Object::new();
    match effect {
        TabEffect::None => {
            let _ = Reflect::set(&effect_obj, &"type".into(), &"none".into());
        }
        TabEffect::StartLoad(url) => {
            let _ = Reflect::set(&effect_obj, &"type".into(), &"start_load".into());
            let _ = Reflect::set(&effect_obj, &"url".into(), &url.into());
        }
    }
    effect_obj.into()
}

/// The shape mirrors the Rust views 1:1 and is the bridge's pinned contract,
/// tested from JS.
fn workspace_snapshot(
    view: &browser_core::WorkspaceView,
    effect: &TabEffect,
    noop: Option<&str>,
) -> JsValue {
    let obj = Object::new();
    let active_window_id = match view.active_window_id {
        Some(id) => JsValue::from_f64(id as f64),
        None => JsValue::NULL,
    };
    let _ = Reflect::set(&obj, &"activeWindowId".into(), &active_window_id);

    let windows = Array::new();
    for window in &view.windows {
        let window_obj = Object::new();
        let _ = Reflect::set(
            &window_obj,
            &"id".into(),
            &JsValue::from_f64(window.id as f64),
        );
        let _ = Reflect::set(&window_obj, &"isActive".into(), &window.is_active.into());

        let container = Object::new();
        let active_tab_id = match window.container.active_id {
            Some(id) => JsValue::from_f64(id as f64),
            None => JsValue::NULL,
        };
        let _ = Reflect::set(&container, &"activeTabId".into(), &active_tab_id);
        let tabs = Array::new();
        for tab in &window.container.tabs {
            let tab_obj = Object::new();
            let _ = Reflect::set(&tab_obj, &"id".into(), &JsValue::from_f64(tab.id as f64));
            let _ = Reflect::set(&tab_obj, &"isActive".into(), &tab.is_active.into());
            set_optional_string(&tab_obj, "title", tab.title.as_deref());
            let _ = Reflect::set(
                &tab_obj,
                &"phase".into(),
                &phase_str(tab.state.phase).into(),
            );
            set_optional_string(&tab_obj, "currentUrl", tab.state.current_url.as_deref());
            set_optional_string(&tab_obj, "pendingUrl", tab.state.pending_url.as_deref());
            let _ = Reflect::set(&tab_obj, &"canBack".into(), &tab.state.can_back.into());
            let _ = Reflect::set(
                &tab_obj,
                &"canForward".into(),
                &tab.state.can_forward.into(),
            );
            tabs.push(&tab_obj);
        }
        let _ = Reflect::set(&container, &"tabs".into(), &tabs.into());
        let _ = Reflect::set(&window_obj, &"container".into(), &container.into());
        windows.push(&window_obj);
    }
    let _ = Reflect::set(&obj, &"windows".into(), &windows.into());
    let _ = Reflect::set(&obj, &"effect".into(), &effect_snapshot(effect));

    let noop = match noop {
        Some(reason) => JsValue::from(reason),
        None => JsValue::NULL,
    };
    let _ = Reflect::set(&obj, &"noop".into(), &noop);
    obj.into()
}

/// The classification of one address-bar input, mirroring M1 exactly.
#[wasm_bindgen]
pub struct JsDecision {
    kind: String,
    value: String,
    reason: Option<String>,
}

#[wasm_bindgen]
impl JsDecision {
    /// `"url"`, `"search"`, or `"reject"`.
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        self.kind.clone()
    }

    /// The normalized URL for `"url"`, the search text for `"search"`, and
    /// the empty string for `"reject"`.
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String {
        self.value.clone()
    }

    /// The rejection reason for `"reject"`, otherwise `null`.
    #[wasm_bindgen(getter)]
    pub fn reason(&self) -> Option<String> {
        self.reason.clone()
    }
}

/// Classify an address-bar input. Total: any input yields a decision.
#[wasm_bindgen]
pub fn resolve_navigation(input: &str) -> JsDecision {
    match core_resolve_navigation(input) {
        NavigationDecision::Url(url) => JsDecision {
            kind: "url".to_owned(),
            value: url,
            reason: None,
        },
        NavigationDecision::Search(text) => JsDecision {
            kind: "search".to_owned(),
            value: text,
            reason: None,
        },
        NavigationDecision::Rejection(reason) => JsDecision {
            kind: "reject".to_owned(),
            value: String::new(),
            reason: Some(reason.as_str().to_owned()),
        },
    }
}

/// A thin, synchronous bridge over one M7 workspace.
#[wasm_bindgen]
pub struct WorkspaceBinding {
    workspace: Workspace,
}

impl Default for WorkspaceBinding {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WorkspaceBinding {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WorkspaceBinding {
        WorkspaceBinding {
            workspace: Workspace::new(),
        }
    }

    /// The full workspace snapshot without mutating anything.
    pub fn snapshot(&self) -> JsValue {
        let view = self.workspace.state();
        workspace_snapshot(&view, &TabEffect::None, None)
    }

    pub fn new_window(&mut self) -> JsValue {
        self.handle(WorkspaceEvent::NewWindow)
    }

    pub fn close_window(&mut self, window: u64) -> JsValue {
        self.handle(WorkspaceEvent::CloseWindow(window))
    }

    pub fn activate_window(&mut self, window: u64) -> JsValue {
        self.handle(WorkspaceEvent::ActivateWindow(window))
    }

    pub fn container_new_tab(&mut self, window: u64, initial: Option<String>) -> JsValue {
        self.route(window, ContainerEvent::NewTab(initial))
    }

    pub fn container_close_tab(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::CloseTab(tab))
    }

    pub fn container_activate_tab(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::Activate(tab))
    }

    pub fn container_navigate(&mut self, window: u64, tab: u64, url: &str) -> JsValue {
        self.route(
            window,
            ContainerEvent::Tab(tab, TabEvent::Navigate(url.to_owned())),
        )
    }

    pub fn container_commit(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::Tab(tab, TabEvent::Commit))
    }

    pub fn container_fail(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::Tab(tab, TabEvent::Fail))
    }

    pub fn container_stop(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::Tab(tab, TabEvent::Stop))
    }

    pub fn container_back(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::Tab(tab, TabEvent::Back))
    }

    pub fn container_forward(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::Tab(tab, TabEvent::Forward))
    }

    pub fn container_reload(&mut self, window: u64, tab: u64) -> JsValue {
        self.route(window, ContainerEvent::Tab(tab, TabEvent::Reload))
    }

    pub fn container_move_tab(&mut self, window: u64, tab: u64, to_index: usize) -> JsValue {
        self.route(window, ContainerEvent::MoveTab(tab, to_index))
    }

    pub fn container_set_title(&mut self, window: u64, tab: u64, title: Option<String>) -> JsValue {
        self.route(window, ContainerEvent::SetTitle(tab, title))
    }

    /// The M6 session text of the addressed window's container. Errors with
    /// `"unknown_window"` when the window does not exist.
    pub fn container_session_text(&self, window: u64) -> Result<String, JsValue> {
        match self.workspace.container_session_text(window) {
            Some(text) => Ok(text),
            None => Err("unknown_window".into()),
        }
    }

    /// Restores the addressed window's container from M6 session text.
    /// Errors with `"unknown_window"` or `"malformed_session"`; never throws
    /// for structurally invalid input.
    pub fn container_set_session_text(
        &mut self,
        window: u64,
        text: &str,
    ) -> Result<JsValue, JsValue> {
        match self.workspace.set_container_session_text(window, text) {
            Ok(()) => Ok(self.snapshot()),
            Err(error) => Err(error.as_str().into()),
        }
    }

    fn handle(&mut self, event: WorkspaceEvent) -> JsValue {
        let outcome = self.workspace.handle(event);
        workspace_snapshot(
            &outcome.view,
            &outcome.effect,
            outcome.noop.map(|reason| reason.as_str()),
        )
    }

    fn route(&mut self, window: u64, event: ContainerEvent) -> JsValue {
        let outcome = self
            .workspace
            .handle(WorkspaceEvent::Container(window, event));
        // The surfaced no-op reason is the innermost one that explains the
        // non-transition: tab machine, then container, then workspace.
        let noop = outcome.noop.map(|reason| reason.as_str()).or_else(|| {
            let container = outcome.container_outcome.as_ref()?;
            container.noop.map(|reason| reason.as_str()).or_else(|| {
                container
                    .tab_outcome
                    .as_ref()?
                    .noop
                    .map(|reason| reason.as_str())
            })
        });
        workspace_snapshot(&outcome.view, &outcome.effect, noop)
    }
}
