//! A deterministic container over ordered single-tab machines.
//!
//! Pure bookkeeping over host-reported events: tab creation with
//! deterministic identities, closing with an explicit activation policy,
//! activation, and id-addressed routing of M2 tab events so background tabs
//! can report load outcomes. The container never performs a load.

use super::tab::{TabEffect, TabEvent, TabMachine, TabOutcome, TabStateView};

/// An event the host submits to the tab container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContainerEvent {
    /// Append a fresh tab at the end, activate it, and optionally navigate it
    /// atomically to an already-resolved target.
    NewTab(Option<String>),
    /// Remove the addressed tab and its machine.
    CloseTab(u64),
    /// Make the addressed tab active. Never reloads.
    Activate(u64),
    /// Delegate an M2 event to the addressed tab's machine.
    Tab(u64, TabEvent),
}

/// Why a container event produced no transition. The container is total:
/// invalid events are no-ops, never panics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerNoOpReason {
    /// No tab with the addressed id exists.
    UnknownTab,
}

impl ContainerNoOpReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownTab => "unknown_tab",
        }
    }
}

/// A per-tab summary in the container view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabSummary {
    pub id: u64,
    pub is_active: bool,
    pub state: TabStateView,
}

/// A read-only view of the container for hosts and fixtures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerView {
    /// The active tab's id, or `None` when the container is empty.
    pub active_id: Option<u64>,
    /// All tabs in creation order.
    pub tabs: Vec<TabSummary>,
}

/// The result of one container event: the new view, the surfaced load
/// request, the affected tab's inner outcome (when exactly one machine was
/// consulted), and the no-op reason when the event did not transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerOutcome {
    pub view: ContainerView,
    pub effect: TabEffect,
    pub tab_outcome: Option<TabOutcome>,
    pub noop: Option<ContainerNoOpReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Slot {
    id: u64,
    machine: TabMachine,
}

/// A single-window ordered tab set over M2 machines.
///
/// Deterministic: the same event sequence over a fresh container yields
/// identical ids, states, and effects on every platform. Tab ids come from a
/// sequential counter starting at 1 and are never reused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabContainer {
    slots: Vec<Slot>,
    active: Option<u64>,
    next_id: u64,
}

impl Default for TabContainer {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            active: None,
            next_id: 1,
        }
    }
}

impl TabContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> ContainerView {
        ContainerView {
            active_id: self.active,
            tabs: self
                .slots
                .iter()
                .map(|slot| TabSummary {
                    id: slot.id,
                    is_active: self.active == Some(slot.id),
                    state: slot.machine.state(),
                })
                .collect(),
        }
    }

    pub fn handle(&mut self, event: ContainerEvent) -> ContainerOutcome {
        match event {
            ContainerEvent::NewTab(initial) => self.new_tab(initial),
            ContainerEvent::CloseTab(id) => self.close_tab(id),
            ContainerEvent::Activate(id) => self.activate(id),
            ContainerEvent::Tab(id, tab_event) => self.route(id, tab_event),
        }
    }

    fn new_tab(&mut self, initial: Option<String>) -> ContainerOutcome {
        let id = self.next_id;
        self.next_id += 1;
        self.slots.push(Slot {
            id,
            machine: TabMachine::new(),
        });
        self.active = Some(id);

        match initial {
            Some(url) => {
                let index = self.slots.len() - 1;
                let outcome = self.slots[index].machine.handle(TabEvent::Navigate(url));
                self.finish(outcome.effect.clone(), Some(outcome), None)
            }
            None => {
                let index = self.slots.len() - 1;
                let tab_outcome = TabOutcome {
                    state: self.slots[index].machine.state(),
                    effect: TabEffect::None,
                    noop: None,
                };
                self.finish(TabEffect::None, Some(tab_outcome), None)
            }
        }
    }

    fn close_tab(&mut self, id: u64) -> ContainerOutcome {
        let Some(index) = self.slot_index(id) else {
            return self.finish(TabEffect::None, None, Some(ContainerNoOpReason::UnknownTab));
        };
        self.slots.remove(index);
        if self.active == Some(id) {
            // Right neighbor (the element that shifted into `index`), else
            // the left neighbor, else the container is empty.
            self.active = if index < self.slots.len() {
                Some(self.slots[index].id)
            } else if index > 0 {
                Some(self.slots[index - 1].id)
            } else {
                None
            };
        }
        self.finish(TabEffect::None, None, None)
    }

    fn activate(&mut self, id: u64) -> ContainerOutcome {
        if self.slot_index(id).is_none() {
            return self.finish(TabEffect::None, None, Some(ContainerNoOpReason::UnknownTab));
        }
        self.active = Some(id);
        self.finish(TabEffect::None, None, None)
    }

    fn route(&mut self, id: u64, tab_event: TabEvent) -> ContainerOutcome {
        let Some(index) = self.slot_index(id) else {
            return self.finish(TabEffect::None, None, Some(ContainerNoOpReason::UnknownTab));
        };
        let outcome = self.slots[index].machine.handle(tab_event);
        self.finish(outcome.effect.clone(), Some(outcome), None)
    }

    fn slot_index(&self, id: u64) -> Option<usize> {
        self.slots.iter().position(|slot| slot.id == id)
    }

    fn finish(
        &self,
        effect: TabEffect,
        tab_outcome: Option<TabOutcome>,
        noop: Option<ContainerNoOpReason>,
    ) -> ContainerOutcome {
        ContainerOutcome {
            view: self.state(),
            effect,
            tab_outcome,
            noop,
        }
    }
}
