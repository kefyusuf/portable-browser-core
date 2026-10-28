//! A deterministic ordered set of containers ("windows").
//!
//! Pure bookkeeping over host-reported events: window creation with
//! deterministic identities, closing with an explicit activation policy,
//! activation, and id-addressed routing of M3 container events. The
//! workspace never performs a load.

use super::container::{ContainerEvent, ContainerOutcome, ContainerView, TabContainer};
use super::tab::TabEffect;

/// An event the host submits to the workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkspaceEvent {
    /// Append a fresh window (empty container) at the end and activate it.
    NewWindow,
    /// Remove the addressed window and its container.
    CloseWindow(u64),
    /// Make the addressed window active. Never triggers loads.
    ActivateWindow(u64),
    /// Delegate a container event to the addressed window's container.
    Container(u64, ContainerEvent),
}

/// Why a workspace event produced no transition. The workspace is total:
/// invalid events are no-ops, never panics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceNoOpReason {
    /// No window with the addressed id exists.
    UnknownWindow,
}

impl WorkspaceNoOpReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownWindow => "unknown_window",
        }
    }
}

/// A per-window summary in the workspace view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowSummary {
    pub id: u64,
    pub is_active: bool,
    pub container: ContainerView,
}

/// A read-only view of the workspace for hosts and fixtures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceView {
    /// The active window's id, or `None` when the workspace is empty.
    pub active_window_id: Option<u64>,
    /// All windows in creation order.
    pub windows: Vec<WindowSummary>,
}

/// The result of one workspace event: the new view, the surfaced load
/// request, the routed container's outcome (when exactly one container was
/// consulted), and the no-op reason when the event did not transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceOutcome {
    pub view: WorkspaceView,
    pub effect: TabEffect,
    pub container_outcome: Option<ContainerOutcome>,
    pub noop: Option<WorkspaceNoOpReason>,
}

/// Why a window-scoped session operation failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceSessionError {
    /// No window with the addressed id exists.
    UnknownWindow,
    /// The session text was rejected by the M6 parser.
    MalformedSession,
}

impl WorkspaceSessionError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownWindow => "unknown_window",
            Self::MalformedSession => "malformed_session",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WindowSlot {
    id: u64,
    container: TabContainer,
}

/// An ordered window set over M3 containers.
///
/// Deterministic: the same event sequence over a fresh workspace yields
/// identical ids, states, and effects on every platform. Window ids come
/// from a sequential counter starting at 1 and are never reused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    windows: Vec<WindowSlot>,
    active: Option<u64>,
    next_id: u64,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            active: None,
            next_id: 1,
        }
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> WorkspaceView {
        WorkspaceView {
            active_window_id: self.active,
            windows: self
                .windows
                .iter()
                .map(|slot| WindowSummary {
                    id: slot.id,
                    is_active: self.active == Some(slot.id),
                    container: slot.container.state(),
                })
                .collect(),
        }
    }

    pub fn handle(&mut self, event: WorkspaceEvent) -> WorkspaceOutcome {
        match event {
            WorkspaceEvent::NewWindow => self.new_window(),
            WorkspaceEvent::CloseWindow(id) => self.close_window(id),
            WorkspaceEvent::ActivateWindow(id) => self.activate_window(id),
            WorkspaceEvent::Container(id, container_event) => self.route(id, container_event),
        }
    }

    fn new_window(&mut self) -> WorkspaceOutcome {
        let id = self.next_id;
        self.next_id += 1;
        self.windows.push(WindowSlot {
            id,
            container: TabContainer::new(),
        });
        self.active = Some(id);
        self.finish(TabEffect::None, None, None)
    }

    fn close_window(&mut self, id: u64) -> WorkspaceOutcome {
        let Some(index) = self.window_index(id) else {
            return self.finish(
                TabEffect::None,
                None,
                Some(WorkspaceNoOpReason::UnknownWindow),
            );
        };
        self.windows.remove(index);
        if self.active == Some(id) {
            // The ADR-006 policy mirrored at the window level: right
            // neighbor, else left neighbor, else the workspace is empty.
            self.active = if index < self.windows.len() {
                Some(self.windows[index].id)
            } else if index > 0 {
                Some(self.windows[index - 1].id)
            } else {
                None
            };
        }
        self.finish(TabEffect::None, None, None)
    }

    fn activate_window(&mut self, id: u64) -> WorkspaceOutcome {
        if self.window_index(id).is_none() {
            return self.finish(
                TabEffect::None,
                None,
                Some(WorkspaceNoOpReason::UnknownWindow),
            );
        }
        self.active = Some(id);
        self.finish(TabEffect::None, None, None)
    }

    fn route(&mut self, id: u64, container_event: ContainerEvent) -> WorkspaceOutcome {
        let Some(index) = self.window_index(id) else {
            return self.finish(
                TabEffect::None,
                None,
                Some(WorkspaceNoOpReason::UnknownWindow),
            );
        };
        let outcome = self.windows[index].container.handle(container_event);
        self.finish(outcome.effect.clone(), Some(outcome), None)
    }

    /// The M6 session text of the addressed window's container, or `None`
    /// when the window does not exist. Pure access for host bridges.
    pub fn container_session_text(&self, window: u64) -> Option<String> {
        let index = self.window_index(window)?;
        Some(self.windows[index].container.to_session_text())
    }

    /// Replaces the addressed window's container with one parsed from M6
    /// session text. Pure; the new container starts with no pending outcome.
    pub fn set_container_session_text(
        &mut self,
        window: u64,
        text: &str,
    ) -> Result<(), WorkspaceSessionError> {
        let index = self
            .window_index(window)
            .ok_or(WorkspaceSessionError::UnknownWindow)?;
        let container = TabContainer::from_session_text(text)
            .map_err(|_| WorkspaceSessionError::MalformedSession)?;
        self.windows[index].container = container;
        Ok(())
    }

    fn window_index(&self, id: u64) -> Option<usize> {
        self.windows.iter().position(|slot| slot.id == id)
    }

    fn finish(
        &self,
        effect: TabEffect,
        container_outcome: Option<ContainerOutcome>,
        noop: Option<WorkspaceNoOpReason>,
    ) -> WorkspaceOutcome {
        WorkspaceOutcome {
            view: self.state(),
            effect,
            container_outcome,
            noop,
        }
    }
}

const WORKSPACE_MAGIC: &str = "browser-core-workspace\t1";

/// Why workspace text could not be parsed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkspaceParseError;

impl WorkspaceParseError {
    pub const fn as_str(self) -> &'static str {
        "malformed_workspace"
    }
}

impl Workspace {
    /// Serializes the complete workspace state to deterministic text,
    /// embedding each window's M6 container session verbatim. The core
    /// performs no I/O; hosts own reading and writing the bytes.
    pub fn to_workspace_text(&self) -> String {
        let mut out = String::new();
        out.push_str(WORKSPACE_MAGIC);
        out.push('\n');
        out.push_str("header\t");
        out.push_str(&self.windows.len().to_string());
        out.push('\t');
        out.push_str(&self.next_id.to_string());
        out.push('\t');
        match self.active {
            Some(id) => out.push_str(&id.to_string()),
            None => out.push('-'),
        }
        out.push('\n');
        for slot in &self.windows {
            out.push_str("window\t");
            out.push_str(&slot.id.to_string());
            out.push('\n');
            out.push_str(&slot.container.to_session_text());
        }
        out
    }

    /// Parses workspace text produced by `to_workspace_text`, validating the
    /// envelope and every embedded M6 container block. Total: malformed
    /// input yields `WorkspaceParseError`, never a panic.
    pub fn from_workspace_text(text: &str) -> Result<Self, WorkspaceParseError> {
        let mut lines: Vec<&str> = text.split('\n').collect();
        if lines.last() == Some(&"") {
            lines.pop();
        }

        if lines.first() != Some(&WORKSPACE_MAGIC) {
            return Err(WorkspaceParseError);
        }

        let header: Vec<&str> = lines
            .get(1)
            .ok_or(WorkspaceParseError)?
            .split('\t')
            .collect();
        if header.len() != 4 || header[0] != "header" {
            return Err(WorkspaceParseError);
        }
        let window_count: usize = header[1].parse().map_err(|_| WorkspaceParseError)?;
        let next_id: u64 = header[2].parse().map_err(|_| WorkspaceParseError)?;
        let active: Option<u64> = match header[3] {
            "-" => None,
            value => Some(value.parse().map_err(|_| WorkspaceParseError)?),
        };

        let mut windows = Vec::with_capacity(window_count);
        let mut cursor = 2usize;
        for _ in 0..window_count {
            let line = lines.get(cursor).ok_or(WorkspaceParseError)?;
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != 2 || fields[0] != "window" {
                return Err(WorkspaceParseError);
            }
            let id: u64 = fields[1].parse().map_err(|_| WorkspaceParseError)?;
            if windows.iter().any(|slot: &WindowSlot| slot.id == id) || id >= next_id {
                return Err(WorkspaceParseError);
            }

            // The embedded container block is exactly the M6 session text:
            // magic (v1 or v2 — readers accept both per ADR-014), container
            // header (with the tab count), and tab lines.
            let block_start = cursor + 1;
            let magic_ok = matches!(
                lines.get(block_start),
                Some(&super::container::SESSION_MAGIC) | Some(&super::container::SESSION_MAGIC_V1)
            );
            if !magic_ok {
                return Err(WorkspaceParseError);
            }
            let container_header: Vec<&str> = lines
                .get(block_start + 1)
                .ok_or(WorkspaceParseError)?
                .split('\t')
                .collect();
            if container_header.len() != 4 || container_header[0] != "header" {
                return Err(WorkspaceParseError);
            }
            let tab_count: usize = container_header[1]
                .parse()
                .map_err(|_| WorkspaceParseError)?;
            let block_end = block_start + 2 + tab_count;
            if block_end > lines.len() {
                return Err(WorkspaceParseError);
            }
            let block = lines[block_start..block_end].join("\n");
            let container =
                TabContainer::from_session_text(&block).map_err(|_| WorkspaceParseError)?;

            windows.push(WindowSlot { id, container });
            cursor = block_end;
        }

        if cursor != lines.len() {
            return Err(WorkspaceParseError);
        }
        match active {
            None if !windows.is_empty() => return Err(WorkspaceParseError),
            Some(id) if !windows.iter().any(|slot| slot.id == id) => {
                return Err(WorkspaceParseError);
            }
            _ => {}
        }

        Ok(Self {
            windows,
            active,
            next_id,
        })
    }
}
