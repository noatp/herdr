use super::*;
use crate::api::schema::AgentStatus;
use crate::protocol::{
    ClientShellAgent, ClientShellPane, ClientShellTab, ClientShellWorktree, PaneSurfacePane,
    PaneSurfaceSplit, PaneSurfaceSplitDirection, SurfaceRect,
};
use crossterm::event::MouseEvent;
mod text_editing;

#[test]
fn an_agent_tab_renders_a_colored_status_dot_before_an_intact_label() {
    use super::render::render_tab_bar;

    let mut snap = snapshot();
    snap.tabs[0].label = "build".into();
    snap.tabs[0].agent_status = AgentStatus::Blocked;
    let config = ClientShellConfig::from_config(&Config::default());

    let area = Rect::new(0, 0, 40, 1);
    let mut buffer = Buffer::empty(area);
    let mut scroll = 0usize;
    let mut reveal = false;
    let mut hits = ShellHitMap::default();
    render_tab_bar(
        &mut buffer,
        area,
        &snap,
        &config,
        &mut scroll,
        &mut reveal,
        None,
        false,
        &mut hits,
    );

    let row: String = (0..area.width)
        .map(|x| buffer[(x, 0)].symbol().to_string())
        .collect();
    assert!(
        row.contains("● build"),
        "dot must sit before an intact label, got {row:?}"
    );

    // The dot carries the status color, not the tab's label color.
    let dot_x = row.find('●').expect("dot is drawn") as u16;
    assert_eq!(
        buffer[(dot_x, 0)].fg,
        status_color(AgentStatus::Blocked, &config.palette),
        "dot must use the blocked status color"
    );

    // A tab with no agent stays unmarked.
    let mut plain = snapshot();
    plain.tabs[0].label = "shell".into();
    plain.tabs[0].agent_status = AgentStatus::Unknown;
    let mut buffer = Buffer::empty(area);
    let mut scroll = 0usize;
    let mut reveal = false;
    let mut hits = ShellHitMap::default();
    render_tab_bar(
        &mut buffer,
        area,
        &plain,
        &config,
        &mut scroll,
        &mut reveal,
        None,
        false,
        &mut hits,
    );
    let row: String = (0..area.width)
        .map(|x| buffer[(x, 0)].symbol().to_string())
        .collect();
    assert!(row.contains("shell"), "label must render, got {row:?}");
    assert!(
        !row.contains('●') && !row.contains('○') && !row.contains('·'),
        "a tab with no agent must not show a dot, got {row:?}"
    );

    // A disconnected endpoint dims its cached chrome; the dot must follow that
    // convention instead of staying vivid and implying live status.
    let mut buffer = Buffer::empty(area);
    let mut scroll = 0usize;
    let mut reveal = false;
    let mut hits = ShellHitMap::default();
    render_tab_bar(
        &mut buffer,
        area,
        &snap,
        &config,
        &mut scroll,
        &mut reveal,
        None,
        true,
        &mut hits,
    );
    let row: String = (0..area.width)
        .map(|x| buffer[(x, 0)].symbol().to_string())
        .collect();
    let dot_x = row.find('●').expect("dot is still drawn when stale") as u16;
    assert_eq!(
        buffer[(dot_x, 0)].fg,
        config.palette.overlay0,
        "a stale dot must be dimmed, not shown in its status color"
    );
}

pub(super) fn snapshot() -> ClientShellSnapshot {
    ClientShellSnapshot {
        boot_id: "boot-1".into(),
        revision: 1,
        config_diagnostic: None,
        product_announcement: None,
        update_available: None,
        update_install_command: "herdr update".into(),
        server_keybindings_toml: None,
        latest_release_notes_available: false,
        integration_updates_available: false,
        worktree_directory: "/tmp/herdr-worktrees".into(),
        release_notes: None,
        focused_workspace_id: Some("ws_1".into()),
        focused_tab_id: Some("tab_1".into()),
        focused_pane_id: Some("pane_1".into()),
        tab_bar_right: Vec::new(),
        tab_bar_right_separator: " ".into(),
        agent_view_label: None,
        agent_order: Vec::new(),
        workspaces: vec![ClientShellWorkspace {
            workspace_id: "ws_1".into(),
            active_tab_id: "tab_1".into(),
            new_workspace_cwd: "/repo".into(),
            number: 1,
            label: "client-shell".into(),
            custom_label: false,
            branch: Some("main".into()),
            git_ahead_behind: None,
            tokens: Vec::new(),
            worktree: None,
            focused: true,
            agent_status: AgentStatus::Idle,
        }],
        tabs: vec![ClientShellTab {
            tab_id: "tab_1".into(),
            workspace_id: "ws_1".into(),
            number: 1,
            label: "1".into(),
            custom_label: false,
            zoomed: false,
            focused: true,
            agent_status: AgentStatus::Idle,
        }],
        panes: vec![ClientShellPane {
            pane_id: "pane_1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            label: None,
            cwd: Some("/repo".into()),
            foreground_cwd: Some("/repo".into()),
            focused: true,
            right_click_passthrough: false,
            pinned: false,
        }],
        agents: Vec::new(),
        commands: Vec::new(),
    }
}

fn worktree_list_result(open_workspace_id: Option<&str>) -> crate::api::schema::ResponseResult {
    crate::api::schema::ResponseResult::WorktreeList {
        source: crate::api::schema::WorktreeSourceInfo {
            repo_key: "repo-key".into(),
            repo_name: "repo".into(),
            repo_root: "/repo".into(),
            source_checkout_path: "/repo".into(),
            source_workspace_id: Some("ws_1".into()),
        },
        worktrees: vec![crate::api::schema::WorktreeInfo {
            path: "/repo-feature".into(),
            branch: Some("feature".into()),
            is_bare: false,
            is_detached: false,
            is_prunable: false,
            is_linked_worktree: true,
            open_workspace_id: open_workspace_id.map(str::to_owned),
            label: "repo".into(),
        }],
    }
}

fn surface() -> PaneSurfaceFrame {
    let surface_buffer = Buffer::with_lines(["LIVE", "PANE"]);
    PaneSurfaceFrame {
        boot_id: "boot-1".into(),
        projection_revision: 1,
        surface_revision: 1,
        frame: FrameData::from_ratatui_buffer_with_hyperlinks(
            &surface_buffer,
            Some(crate::protocol::CursorState {
                x: 1,
                y: 1,
                visible: true,
                shape: 2,
            }),
            &[],
        ),
        panes: vec![PaneSurfacePane {
            pane_id: "pane_1".into(),
            content_revision: 0,
            rect: SurfaceRect {
                x: 0,
                y: 0,
                width: 4,
                height: 2,
            },
            inner_rect: SurfaceRect {
                x: 0,
                y: 0,
                width: 4,
                height: 2,
            },
            scrollbar_rect: None,
            scroll: None,
            focused: true,
            mouse_reporting: false,
            sgr_pixel_mouse: false,
            alternate_screen_active: false,
            pixel_width: 0,
            pixel_height: 0,
        }],
        splits: Vec::new(),
        popup: None,
        graphics: crate::protocol::SurfaceGraphicsScene::default(),
    }
}

/// A pinned pane overlays the active tab's panes, so the hit map must list it
/// before the pane underneath or clicks land on the hidden pane.
#[test]
fn pane_hit_regions_are_ordered_topmost_first_so_pinned_panes_are_clickable() {
    let mut surface = surface();
    // Simulate the server's paint order: the active tab's pane first, then a
    // pinned pane from another tab drawn over its right half.
    let underneath = surface.panes[0].clone();
    let mut pinned = underneath.clone();
    pinned.pane_id = "pinned_pane".into();
    pinned.focused = false;
    pinned.rect = SurfaceRect {
        x: 2,
        y: 0,
        width: 2,
        height: 2,
    };
    pinned.inner_rect = pinned.rect;
    surface.panes = vec![underneath, pinned];

    let mut config = ClientShellConfig::from_config(&Config::default());
    config.palette = Palette::terminal();
    config.theme_runtime.auto_switch = false;
    let mut state = ClientShellState::new(config);
    state.set_snapshot(Box::new(snapshot()));
    state.set_pane_surface(surface);
    state.compose(106, 20).expect("composed frame");

    let overlap_cell = state
        .hits
        .panes
        .iter()
        .find(|hit| hit.pane_id == "pinned_pane")
        .map(|hit| (hit.inner_rect.x, hit.inner_rect.y))
        .expect("the pinned pane has a hit region");
    let topmost = state
        .hits
        .panes
        .iter()
        .find(|hit| super::contains(hit.inner_rect, overlap_cell))
        .expect("a pane covers the overlapping cell");
    assert_eq!(
        topmost.pane_id, "pinned_pane",
        "the first hit over a shared cell must be the pane drawn on top"
    );
}

fn frame_rows(frame: &FrameData) -> Vec<String> {
    frame
        .cells
        .chunks(frame.width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol.as_str()).collect())
        .collect()
}

/// Absolute cell position of `needle` inside `area`, for style assertions.
fn cell_symbol_position(frame: &FrameData, area: Rect, needle: &str) -> (u16, u16) {
    let rows = frame_rows(frame);
    for y in area.y..area.bottom().min(frame.height) {
        let row = &rows[y as usize];
        let slice = row
            .chars()
            .skip(area.x as usize)
            .take(area.width as usize)
            .collect::<String>();
        if let Some(byte) = slice.find(needle) {
            let column = slice[..byte].chars().count() as u16 + area.x;
            return (column, y);
        }
    }
    let visible = (area.y..area.bottom().min(frame.height))
        .map(|y| {
            rows[y as usize]
                .chars()
                .skip(area.x as usize)
                .take(area.width as usize)
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    panic!("symbol {needle:?} not found in {area:?}: {visible:?}");
}

fn pane_scroll_result(
    offset_from_bottom: u64,
    max_offset_from_bottom: u64,
    viewport_rows: u64,
) -> crate::api::schema::ResponseResult {
    crate::api::schema::ResponseResult::PaneInfo {
        pane: crate::api::schema::PaneInfo {
            pane_id: "pane_1".into(),
            terminal_id: "terminal_1".into(),
            workspace_id: "ws_1".into(),
            tab_id: "tab_1".into(),
            focused: true,
            cwd: None,
            foreground_cwd: None,
            restore_error: None,
            label: None,
            agent: None,
            title: None,
            terminal_title: None,
            terminal_title_stripped: None,
            display_agent: None,
            agent_status: crate::api::schema::AgentStatus::Unknown,
            state_labels: HashMap::new(),
            tokens: HashMap::new(),
            agent_session: None,
            scroll: Some(crate::api::schema::PaneScrollInfo {
                offset_from_bottom,
                max_offset_from_bottom,
                viewport_rows,
            }),
            revision: 0,
        },
    }
}

fn copy_search_result(
    matches: Vec<crate::api::schema::PaneTextRange>,
    current: Option<u32>,
) -> crate::api::schema::ResponseResult {
    let total = matches.len() as u64;
    crate::api::schema::ResponseResult::PaneCopySearch {
        pane_id: "pane_1".into(),
        content_revision: 0,
        matches,
        total,
        current,
        current_global: current.map(u64::from),
    }
}

fn surface_with_popup() -> PaneSurfaceFrame {
    let mut surface = surface();
    let popup_buffer = Buffer::with_lines(["popup-live", "", ""]);
    surface.popup = Some(Box::new(crate::protocol::ClientShellPopupSurface {
        terminal_id: "terminal-popup".into(),
        title: "popup title".into(),
        width: Some(crate::protocol::ClientShellPopupSize::Cells(12)),
        height: Some(crate::protocol::ClientShellPopupSize::Cells(5)),
        frame: FrameData::from_ratatui_buffer_with_hyperlinks(
            &popup_buffer,
            Some(crate::protocol::CursorState {
                x: 2,
                y: 1,
                visible: true,
                shape: 1,
            }),
            &[],
        ),
        mouse_reporting: true,
        sgr_pixel_mouse: false,
        pixel_width: 0,
        pixel_height: 0,
    }));
    surface
}

mod agents_worktrees_notifications;
mod chrome_context;
mod close_tab;
mod copy;
mod endpoint_requests;
mod endpoints;
mod graphics;
#[path = "input.rs"]
mod input_domain;
mod keybindings_settings;
mod link_hover;
mod mobile;
mod mouse_selection;
mod popup_focus_projection;
mod startup_overlays;
