use browser_core::{
    ContainerEvent, TabEffect, TabEvent, Workspace, WorkspaceEvent, WorkspaceNoOpReason,
    WorkspaceOutcome, WorkspaceView,
};

fn expected_option(cell: &str) -> Option<&str> {
    if cell == "-" {
        None
    } else {
        Some(cell)
    }
}

fn parse_workspace_event(event: &str, arg: &str, row: usize) -> WorkspaceEvent {
    if event == "new_window" {
        return WorkspaceEvent::NewWindow;
    }
    if let Some(id) = event.strip_prefix("close_w:") {
        return WorkspaceEvent::CloseWindow(
            id.parse()
                .unwrap_or_else(|_| panic!("bad close_w id at row {row}")),
        );
    }
    if let Some(rest) = event.strip_prefix("move_w:") {
        let (id, to_index) = rest
            .split_once(':')
            .unwrap_or_else(|| panic!("bad move_w event at row {row}"));
        return WorkspaceEvent::MoveWindow(
            id.parse()
                .unwrap_or_else(|_| panic!("bad move_w id at row {row}")),
            to_index
                .parse()
                .unwrap_or_else(|_| panic!("bad move_w index at row {row}")),
        );
    }
    if let Some(id) = event.strip_prefix("activate_w:") {
        return WorkspaceEvent::ActivateWindow(
            id.parse()
                .unwrap_or_else(|_| panic!("bad activate_w id at row {row}")),
        );
    }
    if let Some(rest) = event.strip_prefix("win:") {
        let (id, container_event) = rest
            .split_once(':')
            .unwrap_or_else(|| panic!("bad routed event at row {row}"));
        let id: u64 = id
            .parse()
            .unwrap_or_else(|_| panic!("bad window id at row {row}"));
        return WorkspaceEvent::Container(id, parse_container_event(container_event, arg, row));
    }
    panic!("unknown workspace event {event:?} at row {row}");
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
    panic!("unknown container event {event:?} at row {row}");
}

fn assert_view(view: &WorkspaceView, cell: &str, row: usize) {
    let (active_part, wins_part) = cell
        .split_once("|wins=")
        .unwrap_or_else(|| panic!("bad view cell at row {row}"));
    let expected_active = if active_part == "active_w=-" {
        None
    } else {
        Some(
            active_part
                .strip_prefix("active_w=")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or_else(|| panic!("bad active window id at row {row}")),
        )
    };
    assert_eq!(
        view.active_window_id, expected_active,
        "active_window_id at row {row}"
    );

    if wins_part == "-" {
        assert!(
            view.windows.is_empty(),
            "expected empty windows at row {row}"
        );
        return;
    }

    let cells: Vec<&str> = wins_part.split(',').collect();
    assert_eq!(cells.len(), view.windows.len(), "window count at row {row}");
    for (cell, summary) in cells.iter().zip(view.windows.iter()) {
        let (id, inner_active) = cell
            .split_once(':')
            .unwrap_or_else(|| panic!("bad window cell at row {row}"));
        assert_eq!(
            summary.id,
            id.parse::<u64>()
                .unwrap_or_else(|_| panic!("bad window id at row {row}")),
            "window id at row {row}"
        );
        let expected_inner =
            expected_option(inner_active).and_then(|value| value.parse::<u64>().ok());
        assert_eq!(
            summary.container.active_id, expected_inner,
            "inner active id at row {row}"
        );
        assert_eq!(
            summary.is_active,
            Some(summary.id) == expected_active,
            "is_active at row {row}"
        );
    }
}

fn assert_effect(outcome: &WorkspaceOutcome, cell: &str, row: usize) {
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

fn assert_noop(outcome: &WorkspaceOutcome, cell: &str, row: usize) {
    let expected = match cell {
        "-" => None,
        reason => {
            let known = [
                WorkspaceNoOpReason::UnknownWindow,
                WorkspaceNoOpReason::IndexOutOfRange,
            ]
            .into_iter()
            .find(|known| known.as_str() == reason)
            .unwrap_or_else(|| panic!("unknown noop reason {reason:?} at row {row}"));
            Some(known)
        }
    };
    assert_eq!(outcome.noop, expected, "noop at row {row}");
}

#[test]
fn workspace_golden_matrix() {
    let contents = include_str!("golden/workspace_transitions.tsv");
    let mut checked = 0usize;
    let mut scenario_count = 0usize;
    let mut current_scenario: Option<&str> = None;
    let mut workspace = Workspace::new();
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
            workspace = Workspace::new();
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

        let outcome = workspace.handle(parse_workspace_event(event, arg, index + 1));
        assert_view(&outcome.view, fields[4], index + 1);
        assert_effect(&outcome, fields[5], index + 1);
        assert_noop(&outcome, fields[6], index + 1);
        checked += 1;
    }

    assert_eq!(scenario_count, 6, "fixture scenario count drift");
    assert_eq!(checked, 32, "fixture row count drift");
}

#[test]
fn routed_load_surfaces_effect_and_inner_outcome() {
    let mut workspace = Workspace::new();
    workspace.handle(WorkspaceEvent::NewWindow);
    let outcome = workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::NewTab(Some("https://a.test/".into())),
    ));
    assert_eq!(
        outcome.effect,
        TabEffect::StartLoad("https://a.test/".into())
    );
    let container_outcome = outcome
        .container_outcome
        .expect("routed events surface the container outcome");
    assert_eq!(container_outcome.effect, outcome.effect);
    assert_eq!(container_outcome.view.active_id, Some(1));
    assert_eq!(
        container_outcome
            .view
            .tabs
            .first()
            .expect("tab exists")
            .state
            .pending_url
            .as_deref(),
        Some("https://a.test/")
    );
}

#[test]
fn window_ids_are_sequential_and_never_reused() {
    let mut workspace = Workspace::new();
    workspace.handle(WorkspaceEvent::NewWindow);
    workspace.handle(WorkspaceEvent::NewWindow);
    workspace.handle(WorkspaceEvent::CloseWindow(1));
    let outcome = workspace.handle(WorkspaceEvent::NewWindow);
    // The fresh window gets id 3, not the recycled id 1.
    assert_eq!(outcome.view.active_window_id, Some(3));
    let ids: Vec<u64> = outcome.view.windows.iter().map(|w| w.id).collect();
    assert_eq!(ids, vec![2, 3]);
}

#[test]
fn workspace_can_become_empty_and_recover() {
    let mut workspace = Workspace::new();
    workspace.handle(WorkspaceEvent::NewWindow);
    let outcome = workspace.handle(WorkspaceEvent::CloseWindow(1));
    assert_eq!(outcome.view.active_window_id, None);
    assert!(outcome.view.windows.is_empty());
    let outcome = workspace.handle(WorkspaceEvent::NewWindow);
    assert_eq!(outcome.view.active_window_id, Some(2));
}

#[test]
fn moving_active_window_keeps_activation() {
    let mut workspace = Workspace::new();
    for _ in 0..3 {
        workspace.handle(WorkspaceEvent::NewWindow);
    }
    workspace.handle(WorkspaceEvent::ActivateWindow(2));
    let outcome = workspace.handle(WorkspaceEvent::MoveWindow(2, 0));
    // Activation follows the id, not the position.
    assert_eq!(outcome.view.active_window_id, Some(2));
    let first = outcome.view.windows.first().expect("strip not empty");
    assert_eq!(first.id, 2);
    assert!(first.is_active);
    let ids: Vec<u64> = outcome.view.windows.iter().map(|w| w.id).collect();
    assert_eq!(ids, vec![2, 1, 3]);
}

#[test]
fn containers_are_independent_across_windows() {
    let mut workspace = Workspace::new();
    workspace.handle(WorkspaceEvent::NewWindow);
    workspace.handle(WorkspaceEvent::NewWindow);
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::NewTab(Some("https://a.test/".into())),
    ));
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::Tab(1, TabEvent::Commit),
    ));
    workspace.handle(WorkspaceEvent::Container(
        2,
        ContainerEvent::NewTab(Some("https://b.test/".into())),
    ));

    let view = workspace.state();
    let first = view.windows.iter().find(|w| w.id == 1).expect("window 1");
    let second = view.windows.iter().find(|w| w.id == 2).expect("window 2");
    assert_eq!(
        first.container.tabs[0].state.current_url.as_deref(),
        Some("https://a.test/")
    );
    assert_eq!(
        second.container.tabs[0].state.pending_url.as_deref(),
        Some("https://b.test/")
    );
    // Navigating in window 1 again does not touch window 2.
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::Tab(1, TabEvent::Navigate("https://c.test/".into())),
    ));
    let view = workspace.state();
    let second = view.windows.iter().find(|w| w.id == 2).expect("window 2");
    assert_eq!(
        second.container.tabs[0].state.pending_url.as_deref(),
        Some("https://b.test/")
    );
}
