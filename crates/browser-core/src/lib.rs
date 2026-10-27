//! Pure, host-independent browser decisions.
//!
//! No network requests, OS integrations, or browser side effects occur here.

pub mod container;
pub mod navigation;
pub mod tab;
pub mod workspace;

pub use container::{
    ContainerEvent, ContainerNoOpReason, ContainerOutcome, ContainerView, SessionParseError,
    TabContainer, TabSummary,
};
pub use navigation::{resolve_navigation, NavigationDecision, RejectionReason};
pub use tab::{
    NoOpReason, PendingSnapshot, SnapshotError, TabEffect, TabEvent, TabMachine, TabOutcome,
    TabPhase, TabSnapshot, TabStateView,
};
pub use workspace::{
    WindowSummary, Workspace, WorkspaceEvent, WorkspaceNoOpReason, WorkspaceOutcome, WorkspaceView,
};
