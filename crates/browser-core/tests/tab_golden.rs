use browser_core::{NoOpReason, TabEffect, TabEvent, TabMachine, TabOutcome, TabPhase};

fn expected_option(cell: &str) -> Option<&str> {
    if cell == "-" {
        None
    } else {
        Some(cell)
    }
}

fn assert_state(outcome: &TabOutcome, cell: &str, row: usize) {
    let parts: Vec<&str> = cell.split('|').collect();
    assert_eq!(parts.len(), 6, "state cell arity at row {row}");
    let expected_phase = match parts[0] {
        "empty" => TabPhase::Empty,
        "loading" => TabPhase::Loading,
        "active" => TabPhase::Active,
        other => panic!("unknown phase {other:?} at row {row}"),
    };
    assert_eq!(outcome.state.phase, expected_phase, "phase at row {row}");
    assert_eq!(
        outcome.state.pending_url.as_deref(),
        expected_option(parts[1]),
        "pending_url at row {row}"
    );
    assert_eq!(
        outcome.state.current_url.as_deref(),
        expected_option(parts[2]),
        "current_url at row {row}"
    );
    assert_eq!(
        outcome.state.can_back,
        parts[3] == "1",
        "can_back at row {row}"
    );
    assert_eq!(
        outcome.state.can_forward,
        parts[4] == "1",
        "can_forward at row {row}"
    );
    assert_eq!(
        outcome.state.title.as_deref(),
        expected_option(parts[5]),
        "title at row {row}"
    );
}

fn assert_effect(outcome: &TabOutcome, cell: &str, row: usize) {
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

fn assert_noop(outcome: &TabOutcome, cell: &str, row: usize) {
    let expected = match cell {
        "-" => None,
        reason => {
            let known = [
                NoOpReason::UnexpectedEvent,
                NoOpReason::AtBoundary,
                NoOpReason::NothingToReload,
                NoOpReason::NothingToTitle,
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
fn tab_golden_matrix() {
    let contents = include_str!("golden/tab_transitions.tsv");
    let mut checked = 0usize;
    let mut scenario_count = 0usize;
    let mut current_scenario: Option<&str> = None;
    let mut machine = TabMachine::new();
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
            machine = TabMachine::new();
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

        let event = match event {
            "navigate" => {
                assert_ne!(arg, "-", "navigate needs an arg at row {}", index + 1);
                TabEvent::Navigate(arg.to_owned())
            }
            "commit" => TabEvent::Commit,
            "fail" => TabEvent::Fail,
            "stop" => TabEvent::Stop,
            "back" => TabEvent::Back,
            "forward" => TabEvent::Forward,
            "reload" => TabEvent::Reload,
            "set_title" => TabEvent::SetTitle(expected_option(arg).map(str::to_owned)),
            other => panic!("unknown event {other:?} at row {}", index + 1),
        };

        let outcome = machine.handle(event);
        assert_state(&outcome, fields[4], index + 1);
        assert_effect(&outcome, fields[5], index + 1);
        assert_noop(&outcome, fields[6], index + 1);
        checked += 1;
    }

    assert_eq!(scenario_count, 12, "M2 fixture scenario count drift");
    assert_eq!(checked, 70, "M2 fixture row count drift");
}

#[test]
fn can_flags_track_cursor_across_history() {
    let mut machine = TabMachine::new();
    machine.handle(TabEvent::Navigate("https://a.test/".into()));
    machine.handle(TabEvent::Commit);
    machine.handle(TabEvent::Navigate("https://b.test/".into()));
    machine.handle(TabEvent::Commit);
    let view = machine.state();
    assert_eq!((view.can_back, view.can_forward), (true, false));

    machine.handle(TabEvent::Back);
    let view = machine.state();
    assert_eq!((view.can_back, view.can_forward), (false, true));
    assert_eq!(view.current_url.as_deref(), Some("https://a.test/"));
}

#[test]
fn targets_are_consumed_verbatim_without_reparsing() {
    // ADR-005: the machine takes already-resolved targets; hosts own resolution.
    let mut machine = TabMachine::new();
    let outcome = machine.handle(TabEvent::Navigate("not a url".into()));
    assert_eq!(outcome.state.pending_url.as_deref(), Some("not a url"));
    assert_eq!(outcome.effect, TabEffect::StartLoad("not a url".into()));
}

#[test]
fn forward_stack_survives_failed_navigation_from_middle() {
    let mut machine = TabMachine::new();
    for url in ["https://a.test/", "https://b.test/"] {
        machine.handle(TabEvent::Navigate(url.into()));
        machine.handle(TabEvent::Commit);
    }
    machine.handle(TabEvent::Back);
    // A navigation attempted from the middle of history is pending.
    machine.handle(TabEvent::Navigate("https://c.test/".into()));
    machine.handle(TabEvent::Fail);
    // Truncation happens at commit, so the forward entry is still reachable.
    let view = machine.state();
    assert!(view.can_forward);
    let outcome = machine.handle(TabEvent::Forward);
    assert_eq!(
        outcome.effect,
        TabEffect::StartLoad("https://b.test/".into())
    );
}
