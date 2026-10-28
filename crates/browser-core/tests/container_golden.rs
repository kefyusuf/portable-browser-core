use browser_core::{
    ContainerEvent, ContainerNoOpReason, ContainerOutcome, ContainerView, TabContainer, TabEffect,
    TabEvent, TabPhase,
};

fn expected_option(cell: &str) -> Option<&str> {
    if cell == "-" {
        None
    } else {
        Some(cell)
    }
}

fn assert_view(view: &ContainerView, cell: &str, row: usize) {
    let (active_part, tabs_part) = cell
        .split_once("|tabs=")
        .unwrap_or_else(|| panic!("bad view cell at row {row}"));
    let expected_active = if active_part == "active=-" {
        None
    } else {
        Some(
            active_part
                .strip_prefix("active=")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or_else(|| panic!("bad active id at row {row}")),
        )
    };
    assert_eq!(view.active_id, expected_active, "active_id at row {row}");

    if tabs_part == "-" {
        assert!(view.tabs.is_empty(), "expected empty tabs at row {row}");
        return;
    }

    let cells: Vec<&str> = tabs_part.split(',').collect();
    assert_eq!(cells.len(), view.tabs.len(), "tab count at row {row}");
    for (cell, summary) in cells.iter().zip(view.tabs.iter()) {
        let parts: Vec<&str> = cell.split('|').collect();
        assert_eq!(parts.len(), 7, "tab cell arity at row {row}");
        assert_eq!(
            summary.id,
            parts[0]
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("bad tab id at row {row}")),
            "tab id at row {row}"
        );
        let expected_phase = match parts[1] {
            "empty" => TabPhase::Empty,
            "loading" => TabPhase::Loading,
            "active" => TabPhase::Active,
            other => panic!("unknown phase {other:?} at row {row}"),
        };
        assert_eq!(summary.state.phase, expected_phase, "phase at row {row}");
        assert_eq!(
            summary.state.pending_url.as_deref(),
            expected_option(parts[2]),
            "pending_url at row {row}"
        );
        assert_eq!(
            summary.state.current_url.as_deref(),
            expected_option(parts[3]),
            "current_url at row {row}"
        );
        assert_eq!(
            summary.state.can_back,
            parts[4] == "1",
            "can_back at row {row}"
        );
        assert_eq!(
            summary.state.can_forward,
            parts[5] == "1",
            "can_forward at row {row}"
        );
        assert_eq!(
            summary.is_active,
            Some(summary.id) == expected_active,
            "is_active at row {row}"
        );
        assert_eq!(
            summary.title.as_deref(),
            expected_option(parts[6]),
            "title at row {row}"
        );
    }
}

fn assert_effect(outcome: &ContainerOutcome, cell: &str, row: usize) {
    if cell == "none" {
        assert_eq!(outcome.effect, TabEffect::None, "effect at row {row}");
    } else if let Some(url) = cell.strip_prefix("start_load|") {
        assert_eq!(
            outcome.effect,
            TabEffect::StartLoad(url.to_owned()),
            "effect at row {row}"
        );
    } else {
        panic!("unknown effect cell {cell:?} at row {row}");
    }
}

fn assert_noop(outcome: &ContainerOutcome, cell: &str, row: usize) {
    let expected = match cell {
        "-" => None,
        reason => {
            let known = [
                ContainerNoOpReason::UnknownTab,
                ContainerNoOpReason::IndexOutOfRange,
                ContainerNoOpReason::InvalidTitle,
                ContainerNoOpReason::NothingToTitle,
            ]
            .into_iter()
            .find(|known| known.as_str() == reason)
            .unwrap_or_else(|| panic!("unknown noop reason {reason:?} at row {row}"));
            Some(known)
        }
    };
    assert_eq!(outcome.noop, expected, "noop at row {row}");
}

fn parse_container_event(event: &str, arg: &str, row: usize) -> ContainerEvent {
    if event == "new_tab" {
        return ContainerEvent::NewTab(expected_option(arg).map(str::to_owned));
    }
    if let Some(id) = event.strip_prefix("close:") {
        return ContainerEvent::CloseTab(
            id.parse()
                .unwrap_or_else(|_| panic!("bad close id at row {row}")),
        );
    }
    if let Some(id) = event.strip_prefix("activate:") {
        return ContainerEvent::Activate(
            id.parse()
                .unwrap_or_else(|_| panic!("bad activate id at row {row}")),
        );
    }
    if let Some(rest) = event.strip_prefix("tab:") {
        let (id, inner) = rest
            .split_once(':')
            .unwrap_or_else(|| panic!("bad tab event at row {row}"));
        let id: u64 = id
            .parse()
            .unwrap_or_else(|_| panic!("bad tab id at row {row}"));
        let tab_event = match inner {
            "navigate" => {
                assert_ne!(arg, "-", "routed navigate needs an arg at row {row}");
                TabEvent::Navigate(arg.to_owned())
            }
            "commit" => TabEvent::Commit,
            "fail" => TabEvent::Fail,
            "stop" => TabEvent::Stop,
            "back" => TabEvent::Back,
            "forward" => TabEvent::Forward,
            "reload" => TabEvent::Reload,
            other => panic!("unknown inner event {other:?} at row {row}"),
        };
        return ContainerEvent::Tab(id, tab_event);
    }
    if let Some(rest) = event.strip_prefix("move:") {
        let (id, to_index) = rest
            .split_once(':')
            .unwrap_or_else(|| panic!("bad move event at row {row}"));
        return ContainerEvent::MoveTab(
            id.parse()
                .unwrap_or_else(|_| panic!("bad move id at row {row}")),
            to_index
                .parse()
                .unwrap_or_else(|_| panic!("bad move index at row {row}")),
        );
    }
    if let Some(id) = event.strip_prefix("set_title:") {
        return ContainerEvent::SetTitle(
            id.parse()
                .unwrap_or_else(|_| panic!("bad set_title id at row {row}")),
            expected_option(arg).map(str::to_owned),
        );
    }
    panic!("unknown container event {event:?} at row {row}");
}

#[test]
fn container_golden_matrix() {
    let contents = include_str!("golden/container_transitions.tsv");
    let mut checked = 0usize;
    let mut scenario_count = 0usize;
    let mut current_scenario: Option<&str> = None;
    let mut container = TabContainer::new();
    let mut expected_step = 1usize;

    for (index, line) in contents.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 7, "bad golden row {}", index + 1);
        let (scenario, step, event, arg) = (fields[0], fields[1], fields[2], fields[3]);

        if current_scenario != Some(scenario) {
            scenario_count += 1;
            current_scenario = Some(scenario);
            container = TabContainer::new();
            expected_step = 1;
        }
        let step_number: usize = step
            .parse()
            .unwrap_or_else(|_| panic!("bad step number at row {}", index + 1));
        assert_eq!(
            step_number,
            expected_step,
            "step order at row {}",
            index + 1
        );
        expected_step += 1;

        let outcome = container.handle(parse_container_event(event, arg, index + 1));
        assert_view(&outcome.view, fields[4], index + 1);
        assert_effect(&outcome, fields[5], index + 1);
        assert_noop(&outcome, fields[6], index + 1);
        checked += 1;
    }

    assert_eq!(scenario_count, 13, "fixture scenario count drift");
    assert_eq!(checked, 60, "fixture row count drift");
}

#[test]
fn closing_active_tab_prefers_right_neighbor() {
    let mut container = TabContainer::new();
    for _ in 0..3 {
        container.handle(ContainerEvent::NewTab(None));
    }
    container.handle(ContainerEvent::Activate(2));
    let outcome = container.handle(ContainerEvent::CloseTab(2));
    // Right neighbor (id 3) activates; id 2 is gone.
    assert_eq!(outcome.view.active_id, Some(3));
    let ids: Vec<u64> = outcome.view.tabs.iter().map(|tab| tab.id).collect();
    assert_eq!(ids, vec![1, 3]);
}

#[test]
fn ids_are_sequential_and_never_reused() {
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(None));
    container.handle(ContainerEvent::NewTab(None));
    container.handle(ContainerEvent::CloseTab(1));
    let outcome = container.handle(ContainerEvent::NewTab(None));
    // The fresh tab gets id 3, not the recycled id 1.
    assert_eq!(outcome.view.active_id, Some(3));
    let ids: Vec<u64> = outcome.view.tabs.iter().map(|tab| tab.id).collect();
    assert_eq!(ids, vec![2, 3]);
}

#[test]
fn container_can_become_empty_and_recover() {
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(None));
    let outcome = container.handle(ContainerEvent::CloseTab(1));
    assert_eq!(outcome.view.active_id, None);
    assert!(outcome.view.tabs.is_empty());
    // The neutral empty state still accepts new tabs with a continuing counter.
    let outcome = container.handle(ContainerEvent::NewTab(Some("https://a.test/".into())));
    assert_eq!(outcome.view.active_id, Some(2));
    assert_eq!(
        outcome.effect,
        TabEffect::StartLoad("https://a.test/".into())
    );
}

#[test]
fn background_tab_commits_while_another_is_active() {
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(Some("https://a.test/".into())));
    container.handle(ContainerEvent::Tab(1, TabEvent::Commit));
    container.handle(ContainerEvent::NewTab(Some("https://b.test/".into())));
    container.handle(ContainerEvent::Activate(1));

    // The background tab (id 2) navigates and commits while tab 1 is visible.
    let outcome = container.handle(ContainerEvent::Tab(
        2,
        TabEvent::Navigate("https://c.test/".into()),
    ));
    assert_eq!(
        outcome.effect,
        TabEffect::StartLoad("https://c.test/".into())
    );
    let outcome = container.handle(ContainerEvent::Tab(2, TabEvent::Commit));
    assert_eq!(outcome.view.active_id, Some(1));
    let background = outcome
        .view
        .tabs
        .iter()
        .find(|tab| tab.id == 2)
        .expect("tab 2 exists");
    assert_eq!(background.state.phase, TabPhase::Active);
    assert_eq!(
        background.state.current_url.as_deref(),
        Some("https://c.test/")
    );
}

#[test]
fn moving_active_tab_keeps_activation() {
    let mut container = TabContainer::new();
    for _ in 0..3 {
        container.handle(ContainerEvent::NewTab(None));
    }
    container.handle(ContainerEvent::Activate(2));
    let outcome = container.handle(ContainerEvent::MoveTab(2, 0));
    // Activation follows the id, not the position.
    assert_eq!(outcome.view.active_id, Some(2));
    let first = outcome.view.tabs.first().expect("strip not empty");
    assert_eq!(first.id, 2);
    assert!(first.is_active);
    let ids: Vec<u64> = outcome.view.tabs.iter().map(|tab| tab.id).collect();
    assert_eq!(ids, vec![2, 1, 3]);
}

#[test]
fn reorder_moves_load_state_with_the_identity() {
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(Some("https://a.test/".into())));
    container.handle(ContainerEvent::NewTab(None));
    let outcome = container.handle(ContainerEvent::MoveTab(1, 1));
    // The loading tab's machine state travels with its id, untouched.
    assert_eq!(outcome.view.active_id, Some(2));
    let moved = outcome
        .view
        .tabs
        .iter()
        .find(|tab| tab.id == 1)
        .expect("tab 1 exists");
    assert_eq!(moved.state.phase, TabPhase::Loading);
    assert_eq!(moved.state.pending_url.as_deref(), Some("https://a.test/"));
}

#[test]
fn title_resets_on_commit_until_reported() {
    // ADR-014: the title belongs to the entry. A title reported mid-load
    // travels into the pending entry at commit; after committing a NEW page
    // the displayed title is the new entry's title until reported; back
    // shows the landed entry's remembered title.
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(Some("https://a.test/".into())));
    container.handle(ContainerEvent::Tab(1, TabEvent::Commit));
    container.handle(ContainerEvent::SetTitle(1, Some("Example Domain".into())));

    container.handle(ContainerEvent::Tab(
        1,
        TabEvent::Navigate("https://b.test/".into()),
    ));
    container.handle(ContainerEvent::SetTitle(1, Some("Pending Title".into())));
    let outcome = container.handle(ContainerEvent::Tab(1, TabEvent::Commit));
    let tab = outcome.view.tabs.first().expect("tab 1 exists");
    assert_eq!(tab.state.current_url.as_deref(), Some("https://b.test/"));
    assert_eq!(tab.title.as_deref(), Some("Pending Title"));

    container.handle(ContainerEvent::Tab(
        1,
        TabEvent::Navigate("https://c.test/".into()),
    ));
    let outcome = container.handle(ContainerEvent::Tab(1, TabEvent::Commit));
    let tab = outcome.view.tabs.first().expect("tab 1 exists");
    assert_eq!(tab.state.current_url.as_deref(), Some("https://c.test/"));
    assert_eq!(tab.title, None);

    let outcome = container.handle(ContainerEvent::Tab(1, TabEvent::Back));
    let tab = outcome.view.tabs.first().expect("tab 1 exists");
    assert_eq!(tab.state.current_url.as_deref(), Some("https://b.test/"));
    assert_eq!(tab.title.as_deref(), Some("Pending Title"));
}
