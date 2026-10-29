use browser_core::{ContainerEvent, TabEvent, Workspace, WorkspaceEvent};

fn rich_workspace() -> Workspace {
    let mut workspace = Workspace::new();
    workspace.handle(WorkspaceEvent::NewWindow);
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::NewTab(Some("https://a.test/".into())),
    ));
    workspace.handle(WorkspaceEvent::Container(
        1,
        TabEvent::Commit.into_tab_event(1),
    ));
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::SetTitle(1, Some("Example Domain".into())),
    ));
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::SetFavicon(1, Some("https://a.test/icon.ico".into())),
    ));
    workspace.handle(WorkspaceEvent::NewWindow);
    workspace.handle(WorkspaceEvent::Container(
        2,
        ContainerEvent::NewTab(Some("https://b.test/".into())),
    ));
    workspace.handle(WorkspaceEvent::Container(
        2,
        TabEvent::Commit.into_tab_event(1),
    ));
    workspace.handle(WorkspaceEvent::Container(
        2,
        ContainerEvent::Tab(1, TabEvent::Navigate("https://c.test/".into())),
    ));
    workspace.handle(WorkspaceEvent::ActivateWindow(1));
    workspace
}

trait IntoTabEvent {
    fn into_tab_event(self, tab: u64) -> ContainerEvent;
}

impl IntoTabEvent for TabEvent {
    fn into_tab_event(self, tab: u64) -> ContainerEvent {
        ContainerEvent::Tab(tab, self)
    }
}

#[test]
fn observation_text_pins_the_contract() {
    let workspace = rich_workspace();
    let text = workspace.observe_text();
    let expected = [
        "workspace windows=2 active=1",
        "window 1 [active]",
        "  tab 1 active url=https://a.test/ title=\"Example Domain\" favicon=\"https://a.test/icon.ico\" actions=reload,navigate,set_title,set_favicon",
        "window 2",
        "  tab 1 loading url=https://b.test/ pending=https://c.test/ actions=navigate,commit,fail,stop,set_title,set_favicon",
    ]
    .join("\n")
        + "\n";
    assert_eq!(text, expected);
    // Determinism: the same state always yields the same bytes.
    assert_eq!(rich_workspace().observe_text(), text);
}

#[test]
fn observation_text_reflects_boundaries_and_quoting() {
    let mut workspace = Workspace::new();
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
        1,
        ContainerEvent::NewTab(Some("https://b.test/".into())),
    ));
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::Tab(2, TabEvent::Commit),
    ));
    // Give tab 2 its own two-entry history and land on its first entry:
    // forward is available, back is not.
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::Tab(2, TabEvent::Navigate("https://c.test/".into())),
    ));
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::Tab(2, TabEvent::Commit),
    ));
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::Tab(2, TabEvent::Back),
    ));
    // A title with quotes and backslashes exercises the observation escaping.
    workspace.handle(WorkspaceEvent::Container(
        1,
        ContainerEvent::SetTitle(1, Some("he said \"hi\" \\ ok".into())),
    ));

    let expected = [
        "workspace windows=1 active=1",
        "window 1 [active]",
        "  tab 1 active url=https://a.test/ title=\"he said \\\"hi\\\" \\\\ ok\" actions=reload,navigate,set_title,set_favicon",
        "  tab 2 active url=https://b.test/ actions=forward,reload,navigate,set_title,set_favicon",
    ]
    .join("\n")
        + "\n";
    assert_eq!(workspace.observe_text(), expected);
}

#[test]
fn observation_text_covers_degenerate_states() {
    // Empty workspace.
    assert_eq!(
        Workspace::new().observe_text(),
        "workspace windows=0 active=-\n"
    );

    // An entryless tab in an otherwise populated workspace: no url, no
    // title/favicon segments, and only `navigate` is available.
    let mut workspace = Workspace::new();
    workspace.handle(WorkspaceEvent::NewWindow);
    workspace.handle(WorkspaceEvent::Container(1, ContainerEvent::NewTab(None)));
    let expected = [
        "workspace windows=1 active=1",
        "window 1 [active]",
        "  tab 1 empty actions=navigate",
    ]
    .join("\n")
        + "\n";
    assert_eq!(workspace.observe_text(), expected);
}
