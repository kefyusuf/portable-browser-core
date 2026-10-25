//! Pure, host-independent browser decisions.
//!
//! No network requests, OS integrations, or browser side effects occur here.

pub mod navigation;
pub mod tab;

pub use navigation::{resolve_navigation, NavigationDecision, RejectionReason};
pub use tab::{NoOpReason, TabEffect, TabEvent, TabMachine, TabOutcome, TabPhase, TabStateView};
