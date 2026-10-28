use browser_core::{
    ContainerEvent, ContainerNoOpReason, SessionParseError, TabContainer, TabEffect, TabEvent,
};

fn parse_script_event(event: &str, arg: &str, row: usize) -> ContainerEvent {
    if event == "new_tab" {
        let initial = if arg == "-" {
            None
        } else {
            Some(arg.to_owned())
        };
        return ContainerEvent::NewTab(initial);
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
        let title = if arg == "-" {
            None
        } else {
            Some(arg.to_owned())
        };
        return ContainerEvent::SetTitle(
            id.parse()
                .unwrap_or_else(|_| panic!("bad set_title id at row {row}")),
            title,
        );
    }
    panic!("unknown script event {event:?} at row {row}");
}

#[test]
fn session_roundtrip_matrix() {
    let contents = include_str!("golden/session_scripts.tsv");
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
        assert_eq!(fields.len(), 4, "bad golden row {}", index + 1);
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

        container.handle(parse_script_event(event, arg, index + 1));
        checked += 1;
    }

    // The scenario text pins the serialized bytes per scenario, so the
    // round-trip is exercised against exact expected sessions, not only
    // structurally.
    for (scenario, expected) in expected_sessions() {
        let mut container = TabContainer::new();
        for (index, line) in contents.lines().enumerate() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields[0] != scenario {
                continue;
            }
            let row = index + 1;
            container.handle(parse_script_event(fields[2], fields[3], row));
        }
        let text = container.to_session_text();
        assert_eq!(text, expected, "serialized bytes for {scenario}");

        let restored = TabContainer::from_session_text(&text)
            .unwrap_or_else(|error| panic!("{scenario} session must parse: {}", error.as_str()));
        assert_eq!(restored.state(), container.state(), "view for {scenario}");
        assert_eq!(
            restored.to_session_text(),
            text,
            "re-serialization for {scenario}"
        );
    }

    assert_eq!(scenario_count, 4, "M6 fixture scenario count drift");
    assert_eq!(checked, 18, "M6 fixture row count drift");
}

fn expected_sessions() -> Vec<(&'static str, String)> {
    vec![
        (
            "roundtrip_full_history_and_titles",
            [
                "browser-core-session\t2",
                "header\t2\t3\t2",
                "tab\t1\t1\t-\thttps://a.test/\thttps://b.test/~Example Domain",
                "tab\t2\t-\t-",
            ]
            .join("\n")
                + "\n",
        ),
        (
            "roundtrip_pending_kinds",
            [
                "browser-core-session\t2",
                "header\t2\t3\t2",
                "tab\t1\t0\t-\thttps://a.test/",
                "tab\t2\t0\treload\thttps://b.test/",
            ]
            .join("\n")
                + "\n",
        ),
        (
            "roundtrip_strip_order_and_titles",
            [
                "browser-core-session\t2",
                "header\t2\t3\t2",
                "tab\t2\t0\t-\thttps://b.test/~Moved",
                "tab\t1\t-\t-",
            ]
            .join("\n")
                + "\n",
        ),
        (
            "roundtrip_close_to_empty",
            ["browser-core-session\t2", "header\t0\t2\t-"].join("\n") + "\n",
        ),
    ]
}

#[test]
fn session_text_of_fresh_container_is_pinned() {
    let text = TabContainer::new().to_session_text();
    assert_eq!(text, "browser-core-session\t2\nheader\t0\t1\t-\n");
    let restored = TabContainer::from_session_text(&text).expect("fresh session parses");
    assert_eq!(restored.state(), TabContainer::new().state());
}

#[test]
fn v1_sessions_still_parse_with_titles_mapped_to_the_current_entry() {
    // ADR-014: readers accept v1; the v1 tab-level title maps onto the
    // current entry, and entryless tabs lose it (nothing_to_title).
    let v1 = "browser-core-session\t1\nheader\t1\t2\t1\ntab\t1\tt:Example Domain\t0\t-\thttps://a.test/\n";
    let container = TabContainer::from_session_text(v1).expect("v1 session parses");
    let view = container.state();
    let tab = view.tabs.first().expect("tab exists");
    assert_eq!(tab.state.current_url.as_deref(), Some("https://a.test/"));
    assert_eq!(tab.title.as_deref(), Some("Example Domain"));

    let v1_entryless = "browser-core-session\t1\nheader\t1\t2\t1\ntab\t1\tt:Lost\t-\t-\n";
    let container = TabContainer::from_session_text(v1_entryless).expect("v1 session parses");
    let view = container.state();
    let tab = view.tabs.first().expect("tab exists");
    assert_eq!(tab.title, None);
}

#[test]
fn escaping_round_trips_hostile_titles_and_urls() {
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(Some("https://a~b.test".into())));
    container.handle(ContainerEvent::SetTitle(
        1,
        Some("back\\slash and - dash".into()),
    ));
    let text = container.to_session_text();
    // The `~` in the URL and the title are escaped, so the entry field
    // carries exactly one separator.
    assert!(text.contains("https://a\\~b.test~back\\\\slash and - dash"));
    let restored = TabContainer::from_session_text(&text).expect("hostile session parses");
    assert_eq!(restored.state(), container.state());
    assert_eq!(restored.to_session_text(), text);
    let view = restored.state();
    let tab = view.tabs.first().expect("tab exists");
    assert_eq!(tab.title.as_deref(), Some("back\\slash and - dash"));
    assert_eq!(tab.state.pending_url.as_deref(), Some("https://a~b.test"));
}

#[test]
fn favicon_segments_round_trip_in_v2() {
    // ADR-015: entry fields extend to url~title~favicon; a favicon without
    // a title serializes as an empty middle segment; a favicon reported for
    // a load travels into the entry at commit.
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(Some("https://a.test/".into())));
    container.handle(ContainerEvent::SetFavicon(
        1,
        Some("https://a.test/icon.ico".into()),
    ));
    let text = container.to_session_text();
    // Loading, untitled: the pending field carries the favicon segment.
    assert!(text.contains("fresh:https://a.test/~~https://a.test/icon.ico"));

    container.handle(ContainerEvent::Tab(1, TabEvent::Commit));
    let text = container.to_session_text();
    // Committed, untitled entry: the empty middle segment is explicit.
    assert!(text.contains("tab	1	0	-	https://a.test/~~https://a.test/icon.ico"));

    let restored = TabContainer::from_session_text(&text).expect("favicon session parses");
    assert_eq!(restored.state(), container.state());
    let view = restored.state();
    let tab = view.tabs.first().expect("tab exists");
    assert_eq!(tab.favicon.as_deref(), Some("https://a.test/icon.ico"));
    assert_eq!(tab.title, None);

    // Titled and faviconed entries carry three segments.
    container.handle(ContainerEvent::SetTitle(1, Some("Example".into())));
    let text = container.to_session_text();
    assert!(text.contains("https://a.test/~Example~https://a.test/icon.ico"));
    let restored = TabContainer::from_session_text(&text).expect("three-segment parses");
    let view = restored.state();
    let tab = view.tabs.first().expect("tab exists");
    assert_eq!(tab.title.as_deref(), Some("Example"));
    assert_eq!(tab.favicon.as_deref(), Some("https://a.test/icon.ico"));
}

#[test]
fn set_favicon_mirrors_the_title_plumbing() {
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(None));
    // Control-byte references are rejected before routing.
    let outcome = container.handle(ContainerEvent::SetFavicon(
        1,
        Some(
            "bad
line"
                .into(),
        ),
    ));
    assert_eq!(outcome.noop, Some(ContainerNoOpReason::InvalidFavicon));
    // Entryless tabs have nothing to attach a favicon to.
    let outcome = container.handle(ContainerEvent::SetFavicon(1, Some("icon".into())));
    assert_eq!(outcome.noop, Some(ContainerNoOpReason::NothingToFavicon));
    // After the tab has an entry, the report lands on it.
    container.handle(ContainerEvent::Tab(
        1,
        TabEvent::Navigate("https://a.test/".into()),
    ));
    container.handle(ContainerEvent::Tab(1, TabEvent::Commit));
    let outcome = container.handle(ContainerEvent::SetFavicon(1, Some("icon".into())));
    assert_eq!(outcome.noop, None);
    assert_eq!(
        outcome
            .view
            .tabs
            .first()
            .expect("tab exists")
            .favicon
            .as_deref(),
        Some("icon")
    );
}

#[test]
fn set_title_rejects_control_characters() {
    let mut container = TabContainer::new();
    container.handle(ContainerEvent::NewTab(None));
    for hostile in ["bad\nline", "col\tumn", "ret\rurn", "del\u{7f}"] {
        let outcome = container.handle(ContainerEvent::SetTitle(1, Some(hostile.into())));
        assert_eq!(
            outcome.noop,
            Some(ContainerNoOpReason::InvalidTitle),
            "expected invalid_title for {hostile:?}"
        );
        assert_eq!(outcome.effect, TabEffect::None);
    }
    // A valid title on an entryless tab is `nothing_to_title` now (ADR-014).
    let outcome = container.handle(ContainerEvent::SetTitle(1, Some("good".into())));
    assert_eq!(
        outcome.noop,
        Some(ContainerNoOpReason::NothingToTitle),
        "entryless tabs have nothing to title"
    );
    // Once the tab has an entry, the same report lands.
    container.handle(ContainerEvent::Tab(
        1,
        TabEvent::Navigate("https://a.test/".into()),
    ));
    container.handle(ContainerEvent::Tab(1, TabEvent::Commit));
    let outcome = container.handle(ContainerEvent::SetTitle(1, Some("good".into())));
    assert_eq!(outcome.noop, None);
    assert_eq!(
        outcome
            .view
            .tabs
            .first()
            .expect("tab exists")
            .title
            .as_deref(),
        Some("good")
    );
}

#[test]
fn parser_rejects_malformed_sessions() {
    let ok = "browser-core-session\t1\nheader\t1\t2\t1\ntab\t1\t-\t0\t-\thttps://a.test/\n";
    assert!(TabContainer::from_session_text(ok).is_ok());

    let cases: [(&str, &str); 8] = [
        ("bad magic", "browser-core-session\t3\nheader\t0\t1\t-\n"),
        ("missing header", "browser-core-session\t1\n"),
        (
            "unknown active id",
            "browser-core-session\t1\nheader\t0\t1\t7\n",
        ),
        (
            "active with empty strip",
            "browser-core-session\t1\nheader\t0\t1\t1\n",
        ),
        (
            "cursor past history",
            "browser-core-session\t1\nheader\t1\t2\t1\ntab\t1\t-\t1\t-\thttps://a.test/\n",
        ),
        (
            "reload without cursor",
            "browser-core-session\t1\nheader\t1\t2\t1\ntab\t1\t-\t-\treload\thttps://a.test/\n",
        ),
        (
            "next_id not above ids",
            "browser-core-session\t1\nheader\t1\t1\t1\ntab\t1\t-\t0\t-\thttps://a.test/\n",
        ),
        (
            "trailing content",
            "browser-core-session\t1\nheader\t1\t2\t1\ntab\t1\t-\t0\t-\thttps://a.test/\ngarbage\n",
        ),
    ];
    for (label, text) in cases {
        let error = TabContainer::from_session_text(text).expect_err(label);
        assert_eq!(error.as_str(), "malformed_session", "{label}");
    }

    let duplicate = "browser-core-session\t1\nheader\t2\t3\t1\ntab\t1\t-\t-\t-\ntab\t1\t-\t-\t-\n";
    assert!(TabContainer::from_session_text(duplicate).is_err());
    let truncated = "browser-core-session\t1\nheader\t2\t3\t1\ntab\t1\t-\t-\t-\n";
    assert!(TabContainer::from_session_text(truncated).is_err());

    // The error type is total: no panic on arbitrary garbage.
    for garbage in ["", "\n\n\n", "tab\t1", "browser-core-session"] {
        assert!(TabContainer::from_session_text(garbage).is_err());
    }
    let _ = SessionParseError.as_str();
}
