//! A deterministic container over ordered single-tab machines.
//!
//! Pure bookkeeping over host-reported events: tab creation with
//! deterministic identities, closing with an explicit activation policy,
//! activation, and id-addressed routing of M2 tab events so background tabs
//! can report load outcomes. The container never performs a load.

use super::tab::{
    EntrySnapshot, NoOpReason, PendingSnapshot, TabEffect, TabEvent, TabMachine, TabOutcome,
    TabSnapshot, TabStateView,
};

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
    /// Reposition the addressed tab so it sits at the target index
    /// (0-based) afterwards. Ids, load state, and history are untouched.
    MoveTab(u64, usize),
    /// Set or clear the addressed tab's host-reported display title. The
    /// last reported value is kept until the next report; navigation never
    /// alters it.
    SetTitle(u64, Option<String>),
}

/// Why a container event produced no transition. The container is total:
/// invalid events are no-ops, never panics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerNoOpReason {
    /// No tab with the addressed id exists.
    UnknownTab,
    /// The requested strip position does not exist; the core never clamps.
    IndexOutOfRange,
    /// The reported title contains an ASCII control byte (C0 or DEL).
    InvalidTitle,
    /// There is no history entry to attach a title to (ADR-014).
    NothingToTitle,
}

impl ContainerNoOpReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownTab => "unknown_tab",
            Self::IndexOutOfRange => "index_out_of_range",
            Self::InvalidTitle => "invalid_title",
            Self::NothingToTitle => "nothing_to_title",
        }
    }
}

/// A per-tab summary in the container view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabSummary {
    pub id: u64,
    pub is_active: bool,
    pub state: TabStateView,
    /// The displayed title, derived from the machine (ADR-014): the current
    /// entry's title, or a title reported for the load in flight.
    pub title: Option<String>,
}

/// A read-only view of the container for hosts and fixtures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContainerView {
    /// The active tab's id, or `None` when the container is empty.
    pub active_id: Option<u64>,
    /// All tabs in strip order (creation order, as adjusted by reordering).
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
                .map(|slot| {
                    let state = slot.machine.state();
                    TabSummary {
                        id: slot.id,
                        is_active: self.active == Some(slot.id),
                        title: state.title.clone(),
                        state,
                    }
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
            ContainerEvent::MoveTab(id, to_index) => self.move_tab(id, to_index),
            ContainerEvent::SetTitle(id, title) => self.set_title(id, title),
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

    fn move_tab(&mut self, id: u64, to_index: usize) -> ContainerOutcome {
        let Some(from) = self.slot_index(id) else {
            return self.finish(TabEffect::None, None, Some(ContainerNoOpReason::UnknownTab));
        };
        if to_index >= self.slots.len() {
            return self.finish(
                TabEffect::None,
                None,
                Some(ContainerNoOpReason::IndexOutOfRange),
            );
        }
        if from != to_index {
            // Position is the only thing that changes: identity, load state,
            // and history move with the slot, and activation follows the id.
            let slot = self.slots.remove(from);
            self.slots.insert(to_index, slot);
        }
        self.finish(TabEffect::None, None, None)
    }

    fn set_title(&mut self, id: u64, title: Option<String>) -> ContainerOutcome {
        let Some(index) = self.slot_index(id) else {
            return self.finish(TabEffect::None, None, Some(ContainerNoOpReason::UnknownTab));
        };
        if let Some(title) = &title {
            if title.bytes().any(|byte| byte.is_ascii_control()) {
                return self.finish(
                    TabEffect::None,
                    None,
                    Some(ContainerNoOpReason::InvalidTitle),
                );
            }
        }
        // ADR-014: the title belongs to the machine's history now; the
        // machine decides which entry (or pending target) it addresses.
        let outcome = self.slots[index].machine.handle(TabEvent::SetTitle(title));
        // SetTitle only no-ops as `nothing_to_title` at the machine layer.
        let noop = outcome
            .noop
            .filter(|reason| *reason == NoOpReason::NothingToTitle)
            .map(|_| ContainerNoOpReason::NothingToTitle);
        self.finish(TabEffect::None, Some(outcome), noop)
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

pub(crate) const SESSION_MAGIC_V1: &str = "browser-core-session\t1";
pub(crate) const SESSION_MAGIC: &str = "browser-core-session\t2";

/// Why session text could not be parsed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionParseError;

impl SessionParseError {
    pub const fn as_str(self) -> &'static str {
        "malformed_session"
    }
}

fn escape_field(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => out.push_str("\\\\"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\n' => out.push_str("\\n"),
            '~' => out.push_str("\\~"),
            other => out.push(other),
        }
    }
    out
}

fn unescape_field(value: &str) -> Option<String> {
    let mut out = String::with_capacity(value.len());
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match characters.next()? {
            '\\' => out.push('\\'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            'n' => out.push('\n'),
            '~' => out.push('~'),
            _ => return None,
        }
    }
    Some(out)
}

/// One entry field in format v2: `<escaped url>` or
/// `<escaped url>~<escaped title>`.
fn write_entry_field(out: &mut String, entry: &EntrySnapshot) {
    out.push_str(&escape_field(&entry.url));
    if let Some(title) = &entry.title {
        out.push('~');
        out.push_str(&escape_field(title));
    }
}

/// Splits a v2 field at the first *unescaped* `~`: escaped separators
/// (`\~`) belong to the content, not to the structure.
fn split_unescaped_tilde(field: &str) -> (&str, Option<&str>) {
    let mut characters = field.char_indices();
    while let Some((index, character)) = characters.next() {
        match character {
            '\\' => {
                characters.next();
            }
            '~' => return (&field[..index], Some(&field[index + 1..])),
            _ => {}
        }
    }
    (field, None)
}

/// The v1 tab-level title field: `-` or `t:<escaped title>`.
fn parse_v1_title_field(field: &str) -> Result<Option<String>, SessionParseError> {
    match field {
        "-" => Ok(None),
        rest => {
            let escaped = rest.strip_prefix("t:").ok_or(SessionParseError)?.to_owned();
            Ok(Some(unescape_field(&escaped).ok_or(SessionParseError)?))
        }
    }
}

fn parse_entry_field(field: &str) -> Result<EntrySnapshot, SessionParseError> {
    let (url, title) = split_unescaped_tilde(field);
    Ok(EntrySnapshot {
        url: unescape_field(url).ok_or(SessionParseError)?,
        title: match title {
            None => None,
            Some(title) => Some(unescape_field(title).ok_or(SessionParseError)?),
        },
    })
}

impl TabContainer {
    /// Serializes the complete container state to deterministic text (format
    /// v2). The core performs no I/O; hosts own reading and writing the bytes.
    pub fn to_session_text(&self) -> String {
        let mut out = String::new();
        out.push_str(SESSION_MAGIC);
        out.push('\n');
        out.push_str("header\t");
        out.push_str(&self.slots.len().to_string());
        out.push('\t');
        out.push_str(&self.next_id.to_string());
        out.push('\t');
        match self.active {
            Some(id) => out.push_str(&id.to_string()),
            None => out.push('-'),
        }
        out.push('\n');
        for slot in &self.slots {
            let snapshot = slot.machine.snapshot();
            out.push_str("tab\t");
            out.push_str(&slot.id.to_string());
            out.push('\t');
            match snapshot.cursor {
                Some(cursor) => out.push_str(&cursor.to_string()),
                None => out.push('-'),
            }
            out.push('\t');
            match snapshot.pending {
                None => out.push('-'),
                Some(PendingSnapshot::Reload) => out.push_str("reload"),
                Some(PendingSnapshot::Fresh { url, title }) => {
                    out.push_str("fresh:");
                    out.push_str(&escape_field(&url));
                    if let Some(title) = &title {
                        out.push('~');
                        out.push_str(&escape_field(title));
                    }
                }
            }
            for entry in &snapshot.entries {
                out.push('\t');
                write_entry_field(&mut out, entry);
            }
            out.push('\n');
        }
        out
    }

    /// Parses session text produced by `to_session_text` (format v2) or by
    /// the M6 v1 writer (entries are bare URLs; all titles are `None`).
    /// Validating and total: malformed input yields `SessionParseError`,
    /// never a panic.
    pub fn from_session_text(text: &str) -> Result<Self, SessionParseError> {
        let mut lines: Vec<&str> = text.split('\n').collect();
        if lines.last() == Some(&"") {
            lines.pop();
        }

        let version = match lines.first() {
            Some(&SESSION_MAGIC) => 2,
            Some(&SESSION_MAGIC_V1) => 1,
            _ => return Err(SessionParseError),
        };

        let header: Vec<&str> = lines.get(1).ok_or(SessionParseError)?.split('\t').collect();
        if header.len() != 4 || header[0] != "header" {
            return Err(SessionParseError);
        }
        let tab_count: usize = header[1].parse().map_err(|_| SessionParseError)?;
        let next_id: u64 = header[2].parse().map_err(|_| SessionParseError)?;
        let active: Option<u64> = match header[3] {
            "-" => None,
            value => Some(value.parse().map_err(|_| SessionParseError)?),
        };

        let mut slots = Vec::with_capacity(tab_count);
        for line in lines.iter().skip(2) {
            let fields: Vec<&str> = line.split('\t').collect();
            // v1: `tab <id> <title> <cursor> <pending> [entries…]` (≥5);
            // v2: `tab <id> <cursor> <pending> [entries…]` (≥4).
            let min_fields = if version == 1 { 5 } else { 4 };
            if fields.len() < min_fields || fields[0] != "tab" {
                return Err(SessionParseError);
            }
            let id: u64 = fields[1].parse().map_err(|_| SessionParseError)?;
            if slots.iter().any(|slot: &Slot| slot.id == id) || id >= next_id {
                return Err(SessionParseError);
            }
            // v1: `tab <id> <title> <cursor> <pending> <entries…>`;
            // v2: the title field moved into the entries.
            let (title, cursor_field, pending_field, entry_fields) = match version {
                1 => (
                    parse_v1_title_field(fields[2])?,
                    fields[3],
                    fields[4],
                    &fields[5..],
                ),
                _ => (None, fields[2], fields[3], &fields[4..]),
            };
            let cursor: Option<usize> = match cursor_field {
                "-" => None,
                value => Some(value.parse().map_err(|_| SessionParseError)?),
            };
            let pending: Option<PendingSnapshot> = match pending_field {
                "-" => None,
                "reload" => Some(PendingSnapshot::Reload),
                value => {
                    let rest = value
                        .strip_prefix("fresh:")
                        .ok_or(SessionParseError)?
                        .to_owned();
                    let (url, title) = split_unescaped_tilde(&rest);
                    Some(PendingSnapshot::Fresh {
                        url: unescape_field(url).ok_or(SessionParseError)?,
                        title: match title {
                            None => None,
                            Some(title) => Some(unescape_field(title).ok_or(SessionParseError)?),
                        },
                    })
                }
            };
            let entries: Vec<EntrySnapshot> = entry_fields
                .iter()
                .map(|field| {
                    if version == 1 {
                        Ok(EntrySnapshot {
                            url: unescape_field(field).ok_or(SessionParseError)?,
                            title: None,
                        })
                    } else {
                        parse_entry_field(field)
                    }
                })
                .collect::<Result<_, _>>()?;
            // v1 compatibility: the tab-level title maps onto the current
            // entry (it was "the last reported title"); with no entry it is
            // dropped, matching the new `nothing_to_title` semantics.
            let mut entries = entries;
            if version == 1 {
                if let (Some(title), Some(cursor)) = (title, cursor) {
                    if let Some(entry) = entries.get_mut(cursor) {
                        entry.title = Some(title);
                    }
                }
            }
            let machine = TabMachine::from_snapshot(TabSnapshot {
                entries,
                cursor,
                pending,
            })
            .map_err(|_| SessionParseError)?;
            slots.push(Slot { id, machine });
        }

        if slots.len() != tab_count {
            return Err(SessionParseError);
        }
        match active {
            None if !slots.is_empty() => return Err(SessionParseError),
            Some(id) if !slots.iter().any(|slot| slot.id == id) => {
                return Err(SessionParseError);
            }
            _ => {}
        }

        Ok(Self {
            slots,
            active,
            next_id,
        })
    }
}
