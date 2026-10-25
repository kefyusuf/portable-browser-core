//! Single-tab navigation lifecycle and session history.
//!
//! Pure bookkeeping over host-reported events. The machine never performs a
//! load; `StartLoad` is a request the host executes. It consumes
//! already-resolved targets and does not re-parse them.

/// An event the host submits to the tab machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TabEvent {
    /// Navigate to an already-resolved target (an M1 `Url` value or a
    /// host-resolved search result). The machine does not re-parse targets.
    Navigate(String),
    /// Host report: the pending load succeeded.
    Commit,
    /// Host report: the pending load failed.
    Fail,
    /// Host report: the pending load was cancelled.
    Stop,
    /// Move one entry back in session history.
    Back,
    /// Move one entry forward in session history.
    Forward,
    /// Re-emit the current entry as a pending load.
    Reload,
}

/// Why an event produced no transition. The machine is total: invalid
/// (event, state) combinations are no-ops, never panics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NoOpReason {
    /// The event is not valid in the current phase (for example, `commit` while `Empty`).
    UnexpectedEvent,
    /// Session history has no entry in the requested direction.
    AtBoundary,
    /// There is no committed entry to reload.
    NothingToReload,
}

impl NoOpReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnexpectedEvent => "unexpected_event",
            Self::AtBoundary => "at_boundary",
            Self::NothingToReload => "nothing_to_reload",
        }
    }
}

/// An effect request. The machine never executes effects; hosts own
/// network policy, redirects, and execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TabEffect {
    None,
    StartLoad(String),
}

/// The observable phase of the machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabPhase {
    /// No committed history.
    Empty,
    /// A target is awaiting a host load report.
    Loading,
    /// A committed entry is current.
    Active,
}

/// A read-only view of the machine state for hosts and fixtures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabStateView {
    pub phase: TabPhase,
    /// The URL of the current committed entry, if any.
    pub current_url: Option<String>,
    /// The URL awaiting a load report, if any. During a reload this is the
    /// reloaded entry's URL.
    pub pending_url: Option<String>,
    pub can_back: bool,
    pub can_forward: bool,
}

/// The result of one event: the new state view, at most one effect request,
/// and the no-op reason when the event did not transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabOutcome {
    pub state: TabStateView,
    pub effect: TabEffect,
    pub noop: Option<NoOpReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PendingTarget {
    Fresh(String),
    Reload,
}

/// A single tab's navigation lifecycle and session history.
///
/// Deterministic: the same event sequence over the same initial state yields
/// identical states and effects on every platform. No clocks, no randomness,
/// no environment reads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TabMachine {
    entries: Vec<String>,
    cursor: Option<usize>,
    pending: Option<PendingTarget>,
}

impl TabMachine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> TabStateView {
        let phase = if self.pending.is_some() {
            TabPhase::Loading
        } else if self.entries.is_empty() {
            TabPhase::Empty
        } else {
            TabPhase::Active
        };
        let current_url = self.cursor.map(|cursor| self.entries[cursor].clone());
        let pending_url = match &self.pending {
            Some(PendingTarget::Fresh(url)) => Some(url.clone()),
            // During a reload the cursor still points at the reloaded entry.
            Some(PendingTarget::Reload) => current_url.clone(),
            None => None,
        };
        TabStateView {
            phase,
            can_back: matches!(self.cursor, Some(cursor) if cursor > 0),
            can_forward: matches!(self.cursor, Some(cursor) if cursor + 1 < self.entries.len()),
            current_url,
            pending_url,
        }
    }

    pub fn handle(&mut self, event: TabEvent) -> TabOutcome {
        match event {
            TabEvent::Navigate(url) => {
                self.pending = Some(PendingTarget::Fresh(url.clone()));
                self.finish(TabEffect::StartLoad(url), None)
            }
            TabEvent::Commit => match self.pending.take() {
                None => self.finish(TabEffect::None, Some(NoOpReason::UnexpectedEvent)),
                // Truncation of the forward stack happens here, at commit,
                // so a failed navigation cannot destroy reachable history.
                Some(PendingTarget::Fresh(url)) => {
                    if let Some(cursor) = self.cursor {
                        self.entries.truncate(cursor + 1);
                    }
                    self.entries.push(url);
                    self.cursor = Some(self.entries.len() - 1);
                    self.finish(TabEffect::None, None)
                }
                // A committed reload must not duplicate the entry.
                Some(PendingTarget::Reload) => self.finish(TabEffect::None, None),
            },
            TabEvent::Fail | TabEvent::Stop => {
                if self.pending.take().is_none() {
                    return self.finish(TabEffect::None, Some(NoOpReason::UnexpectedEvent));
                }
                self.finish(TabEffect::None, None)
            }
            TabEvent::Back => self.step_history(-1),
            TabEvent::Forward => self.step_history(1),
            TabEvent::Reload => {
                if self.pending.is_some() {
                    return self.finish(TabEffect::None, Some(NoOpReason::UnexpectedEvent));
                }
                match self.cursor.map(|cursor| self.entries[cursor].clone()) {
                    Some(url) => {
                        self.pending = Some(PendingTarget::Reload);
                        self.finish(TabEffect::StartLoad(url), None)
                    }
                    None => self.finish(TabEffect::None, Some(NoOpReason::NothingToReload)),
                }
            }
        }
    }

    fn step_history(&mut self, delta: isize) -> TabOutcome {
        if self.pending.is_some() {
            return self.finish(TabEffect::None, Some(NoOpReason::UnexpectedEvent));
        }
        let Some(cursor) = self.cursor else {
            return self.finish(TabEffect::None, Some(NoOpReason::AtBoundary));
        };
        let target = cursor as isize + delta;
        if target < 0 {
            return self.finish(TabEffect::None, Some(NoOpReason::AtBoundary));
        }
        let target = target as usize;
        if target >= self.entries.len() {
            return self.finish(TabEffect::None, Some(NoOpReason::AtBoundary));
        }
        self.cursor = Some(target);
        let url = self.entries[target].clone();
        self.finish(TabEffect::StartLoad(url), None)
    }

    fn finish(&self, effect: TabEffect, noop: Option<NoOpReason>) -> TabOutcome {
        TabOutcome {
            state: self.state(),
            effect,
            noop,
        }
    }
}
