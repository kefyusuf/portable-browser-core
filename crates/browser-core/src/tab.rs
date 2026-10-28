//! A single tab's navigation lifecycle and session history.
//!
//! Deterministic: the same event sequence over the same initial state yields
//! identical states and effects on every platform. No clocks, no randomness,
//! no environment reads.

/// The pending target of a snapshot, preserving the fresh/reload
/// distinction that changes what a later `commit` does.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PendingSnapshot {
    Fresh {
        url: String,
        /// A title reported while loading travels into the entry at commit.
        title: Option<String>,
    },
    Reload,
}

/// A complete, read-only snapshot of a tab machine's state, sufficient to
/// restore an equivalent machine.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TabSnapshot {
    /// Full history, oldest first.
    pub entries: Vec<EntrySnapshot>,
    pub cursor: Option<usize>,
    pub pending: Option<PendingSnapshot>,
}

/// One persisted history entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntrySnapshot {
    pub url: String,
    pub title: Option<String>,
}

/// Why a snapshot could not be restored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotError;

impl SnapshotError {
    pub const fn as_str(self) -> &'static str {
        "invalid_snapshot"
    }
}

/// One history entry: the normalized target URL and the host-reported title
/// remembered for that page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    pub url: String,
    pub title: Option<String>,
}

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
    /// Report the title of the page the event addresses: the current entry
    /// in `Active`, the pending target in `Loading`. Nothing to title in
    /// `Empty`.
    SetTitle(Option<String>),
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
    /// There is no history entry to attach a title to.
    NothingToTitle,
}

impl NoOpReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnexpectedEvent => "unexpected_event",
            Self::AtBoundary => "at_boundary",
            Self::NothingToReload => "nothing_to_reload",
            Self::NothingToTitle => "nothing_to_title",
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
    /// The displayed title: during a load the pending title if already
    /// reported, else the current entry's title (the page being left).
    pub title: Option<String>,
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
    Fresh { url: String, title: Option<String> },
    Reload,
}

/// A single tab's navigation lifecycle and session history.
///
/// Deterministic: the same event sequence over the same initial state yields
/// identical states and effects on every platform. No clocks, no randomness,
/// no environment reads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TabMachine {
    entries: Vec<Entry>,
    cursor: Option<usize>,
    pending: Option<PendingTarget>,
}

impl TabMachine {
    pub fn new() -> Self {
        Self::default()
    }

    /// The complete machine state. Serialization needs internals the view
    /// intentionally hides (history beyond the cursor, pending kind/title).
    pub fn snapshot(&self) -> TabSnapshot {
        TabSnapshot {
            entries: self
                .entries
                .iter()
                .map(|entry| EntrySnapshot {
                    url: entry.url.clone(),
                    title: entry.title.clone(),
                })
                .collect(),
            cursor: self.cursor,
            pending: self.pending.as_ref().map(|pending| match pending {
                PendingTarget::Fresh { url, title } => PendingSnapshot::Fresh {
                    url: url.clone(),
                    title: title.clone(),
                },
                PendingTarget::Reload => PendingSnapshot::Reload,
            }),
        }
    }

    /// Restores a machine from a snapshot. Validated, never panics.
    pub fn from_snapshot(snapshot: TabSnapshot) -> Result<Self, SnapshotError> {
        if let Some(cursor) = snapshot.cursor {
            if cursor >= snapshot.entries.len() {
                return Err(SnapshotError);
            }
        }
        if matches!(snapshot.pending, Some(PendingSnapshot::Reload)) && snapshot.cursor.is_none() {
            return Err(SnapshotError);
        }
        Ok(Self {
            entries: snapshot
                .entries
                .into_iter()
                .map(|entry| Entry {
                    url: entry.url,
                    title: entry.title,
                })
                .collect(),
            cursor: snapshot.cursor,
            pending: snapshot.pending.map(|pending| match pending {
                PendingSnapshot::Fresh { url, title } => PendingTarget::Fresh { url, title },
                PendingSnapshot::Reload => PendingTarget::Reload,
            }),
        })
    }

    pub fn state(&self) -> TabStateView {
        let phase = if self.pending.is_some() {
            TabPhase::Loading
        } else if self.entries.is_empty() {
            TabPhase::Empty
        } else {
            TabPhase::Active
        };
        let current_entry = self.cursor.map(|cursor| &self.entries[cursor]);
        let current_url = current_entry.map(|entry| entry.url.clone());
        let pending_url = match &self.pending {
            Some(PendingTarget::Fresh { url, .. }) => Some(url.clone()),
            // During a reload the cursor still points at the reloaded entry.
            Some(PendingTarget::Reload) => current_url.clone(),
            None => None,
        };
        // Displayed title: a title reported for the loading page wins, else
        // the page being left (or the current entry) shows its own.
        let title = match &self.pending {
            Some(PendingTarget::Fresh {
                title: Some(title), ..
            }) => Some(title.clone()),
            Some(PendingTarget::Fresh { title: None, .. }) | Some(PendingTarget::Reload) => {
                current_entry.and_then(|entry| entry.title.clone())
            }
            None => current_entry.and_then(|entry| entry.title.clone()),
        };
        TabStateView {
            phase,
            can_back: matches!(self.cursor, Some(cursor) if cursor > 0),
            can_forward: matches!(self.cursor, Some(cursor) if cursor + 1 < self.entries.len()),
            current_url,
            pending_url,
            title,
        }
    }

    pub fn handle(&mut self, event: TabEvent) -> TabOutcome {
        match event {
            TabEvent::Navigate(url) => {
                self.pending = Some(PendingTarget::Fresh {
                    url: url.clone(),
                    title: None,
                });
                self.finish(TabEffect::StartLoad(url), None)
            }
            TabEvent::Commit => match self.pending.take() {
                None => self.finish(TabEffect::None, Some(NoOpReason::UnexpectedEvent)),
                // Truncation of the forward stack happens here, at commit,
                // so a failed navigation cannot destroy reachable history.
                Some(PendingTarget::Fresh { url, title }) => {
                    if let Some(cursor) = self.cursor {
                        self.entries.truncate(cursor + 1);
                    }
                    self.entries.push(Entry { url, title });
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
                match self.cursor.map(|cursor| self.entries[cursor].url.clone()) {
                    Some(url) => {
                        self.pending = Some(PendingTarget::Reload);
                        self.finish(TabEffect::StartLoad(url), None)
                    }
                    None => self.finish(TabEffect::None, Some(NoOpReason::NothingToReload)),
                }
            }
            TabEvent::SetTitle(title) => {
                // A title reported during a load belongs to the page being
                // loaded: it travels into the entry at commit. A reload
                // retitles the entry being reloaded. Otherwise the current
                // entry is updated. An entryless tab has nothing to title.
                match &mut self.pending {
                    Some(PendingTarget::Fresh { title: field, .. }) => {
                        *field = title;
                        return self.finish(TabEffect::None, None);
                    }
                    Some(PendingTarget::Reload) => {
                        let cursor = self.cursor.expect("reload pending implies a cursor");
                        self.entries[cursor].title = title;
                        return self.finish(TabEffect::None, None);
                    }
                    None => {}
                }
                match self.cursor {
                    Some(cursor) => {
                        self.entries[cursor].title = title;
                        self.finish(TabEffect::None, None)
                    }
                    None => self.finish(TabEffect::None, Some(NoOpReason::NothingToTitle)),
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
        let url = self.entries[target].url.clone();
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
