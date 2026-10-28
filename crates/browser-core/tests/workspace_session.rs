use browser_core::{
    ContainerEvent, Workspace, WorkspaceEvent, WorkspaceParseError, WorkspaceSessionError,
};

fn parse_script_event(event: &str, arg: &str, row: usize) -> WorkspaceEvent {
    if event == "new_window" {
        return WorkspaceEvent::NewWindow;
    }
    if let Some(id) = event.strip_prefix("close_w:") {
        return WorkspaceEvent::CloseWindow(
            id.parse()
                .unwrap_or_else(|_| panic!("bad close_w id at row {row}")),
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
    panic!("unknown script event {event:?} at row {row}");
}

fn parse_container_event(event: &str, arg: &str, row: usize) -> ContainerEvent {
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
                browser_core::TabEvent::Navigate(arg.to_owned())
            }
            "commit" => browser_core::TabEvent::Commit,
            "fail" => browser_core::TabEvent::Fail,
            "stop" => browser_core::TabEvent::Stop,
            "back" => browser_core::TabEvent::Back,
            "forward" => browser_core::TabEvent::Forward,
            "reload" => browser_core::TabEvent::Reload,
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
    panic!("unknown container event {event:?} at row {row}");
}

fn build_scenario(contents: &str, scenario: &str) -> Workspace {
    let mut workspace = Workspace::new();
    for (index, line) in contents.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields[0] != scenario {
            continue;
        }
        let row = index + 1;
        workspace.handle(parse_script_event(fields[2], fields[3], row));
    }
    workspace
}

#[test]
fn workspace_session_matrix() {
    let contents = include_str!("golden/workspace_session_scripts.tsv");
    let mut checked = 0usize;
    let mut seen: Vec<&str> = Vec::new();

    for (index, line) in contents.lines().enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 4, "bad golden row {}", index + 1);
        if !seen.contains(&fields[0]) {
            seen.push(fields[0]);
        }
        checked += 1;
    }

    for (scenario, expected) in expected_sessions() {
        let workspace = build_scenario(contents, scenario);
        let text = workspace.to_workspace_text();
        assert_eq!(text, expected, "serialized bytes for {scenario}");

        let restored = Workspace::from_workspace_text(&text).expect("workspace session must parse");
        assert_eq!(restored.state(), workspace.state(), "view for {scenario}");
        assert_eq!(
            restored.to_workspace_text(),
            text,
            "re-serialization for {scenario}"
        );
    }

    assert_eq!(seen.len(), 4, "M9 fixture scenario count drift");
    assert_eq!(checked, 20, "M9 fixture row count drift");
}

fn expected_sessions() -> Vec<(&'static str, String)> {
    vec![
        (
            "full_workspace_roundtrip",
            [
                "browser-core-workspace\t1",
                "header\t2\t3\t2",
                "window\t1",
                "browser-core-session\t2",
                "header\t1\t2\t1",
                "tab\t1\t0\t-\thttps://a.test/~Example Domain",
                "window\t2",
                "browser-core-session\t2",
                "header\t1\t2\t1",
                "tab\t1\t0\treload\thttps://b.test/",
            ]
            .join("\n")
                + "\n",
        ),
        (
            "empty_workspace",
            ["browser-core-workspace\t1", "header\t0\t2\t-"].join("\n") + "\n",
        ),
        (
            "reordered_tab_strip_preserved",
            [
                "browser-core-workspace\t1",
                "header\t1\t2\t1",
                "window\t1",
                "browser-core-session\t2",
                "header\t2\t3\t2",
                "tab\t2\t0\t-\thttps://b.test/~Moved",
                "tab\t1\t-\t-",
            ]
            .join("\n")
                + "\n",
        ),
        (
            "fresh_pending_while_background",
            [
                "browser-core-workspace\t1",
                "header\t2\t3\t1",
                "window\t1",
                "browser-core-session\t2",
                "header\t0\t1\t-",
                "window\t2",
                "browser-core-session\t2",
                "header\t1\t2\t1",
                "tab\t1\t-\tfresh:https://b.test/",
            ]
            .join("\n")
                + "\n",
        ),
    ]
}

#[test]
fn workspace_session_of_fresh_workspace_is_pinned() {
    let text = Workspace::new().to_workspace_text();
    assert_eq!(text, "browser-core-workspace\t1\nheader\t0\t1\t-\n");
    let restored = Workspace::from_workspace_text(&text).expect("fresh session parses");
    assert_eq!(restored.state(), Workspace::new().state());
}

#[test]
fn workspace_parser_rejects_malformed_sessions() {
    let ok = "browser-core-workspace\t1\nheader\t1\t2\t1\nwindow\t1\nbrowser-core-session\t2\nheader\t0\t1\t-\n";
    assert!(Workspace::from_workspace_text(ok).is_ok());

    // v1 container blocks remain readable inside the workspace envelope.
    let v1_embedded = "browser-core-workspace\t1\nheader\t1\t2\t1\nwindow\t1\nbrowser-core-session\t1\nheader\t1\t2\t1\ntab\t1\t-\t0\t-\thttps://a.test/\n";
    let workspace = Workspace::from_workspace_text(v1_embedded).expect("v1 block parses");
    assert_eq!(
        workspace.state().windows[0].container.tabs[0].title,
        None,
        "v1 blocks carry no titles"
    );

    let cases: [(&str, &str); 8] = [
        ("bad magic", "browser-core-workspace\t2\nheader\t0\t1\t-\n"),
        ("unknown active window", "browser-core-workspace\t1\nheader\t0\t1\t7\n"),
        (
            "active with empty strip",
            "browser-core-workspace\t1\nheader\t0\t1\t1\n",
        ),
        (
            "duplicate window ids",
            "browser-core-workspace\t1\nheader\t2\t3\t1\nwindow\t1\nbrowser-core-session\t2\nheader\t0\t1\t-\nwindow\t1\nbrowser-core-session\t2\nheader\t0\t1\t-\n",
        ),
        (
            "window id at or above counter",
            "browser-core-workspace\t1\nheader\t1\t1\t1\nwindow\t1\nbrowser-core-session\t2\nheader\t0\t1\t-\n",
        ),
        (
            "malformed embedded container",
            "browser-core-workspace\t1\nheader\t1\t2\t1\nwindow\t1\nbrowser-core-session\t2\nheader\t1\t2\t1\ntab\t1\t3\t-\thttps://a.test/\n",
        ),
        (
            "missing embedded magic",
            "browser-core-workspace\t1\nheader\t1\t2\t1\nwindow\t1\nheader\t0\t1\t-\n",
        ),
        (
            "trailing content",
            "browser-core-workspace\t1\nheader\t0\t1\t-\ngarbage\n",
        ),
    ];
    for (label, text) in cases {
        let error = Workspace::from_workspace_text(text).expect_err(label);
        assert_eq!(error.as_str(), "malformed_workspace", "{label}");
    }

    // Total: arbitrary garbage never panics.
    for garbage in ["", "\n\n\n", "window\t1", "browser-core-workspace"] {
        assert!(Workspace::from_workspace_text(garbage).is_err());
    }
    let _ = WorkspaceParseError.as_str();
    let _ = WorkspaceSessionError::UnknownWindow.as_str();
}
