use super::*;

fn floating(id: usize) -> Op {
    Op::AddWindow {
        params: TestWindowParams {
            is_floating: true,
            ..TestWindowParams::new(id)
        },
    }
}

fn focus_id(layout: &Layout<TestWindow>) -> Option<usize> {
    layout.focus().map(|win| *win.id())
}

fn is_visible(layout: &Layout<TestWindow>, id: usize) -> bool {
    let (_, _, ws) = layout
        .workspaces()
        .find(|(_, _, ws)| ws.has_window(&id))
        .unwrap();
    let (_, _, visible) = ws
        .tiles_with_render_positions()
        .find(|(tile, _, _)| *tile.window().id() == id)
        .unwrap();
    visible
}

#[test]
fn minimize_hides_and_moves_focus_to_next_floating() {
    let ops = [
        Op::AddOutput(1),
        floating(1),
        floating(2),
        Op::MinimizeWindow { id: None },
    ];
    let layout = check_ops(ops);

    assert!(layout.is_minimized(&2));
    assert!(!is_visible(&layout, 2));
    assert!(is_visible(&layout, 1));
    assert_eq!(focus_id(&layout), Some(1));
}

#[test]
fn activating_restores_minimized() {
    let ops = [
        Op::AddOutput(1),
        floating(1),
        floating(2),
        Op::MinimizeWindow { id: Some(1) },
        Op::FocusWindow(1),
    ];
    let layout = check_ops(ops);

    assert!(!layout.is_minimized(&1));
    assert!(is_visible(&layout, 1));
    assert_eq!(focus_id(&layout), Some(1));
}

#[test]
fn minimize_last_floating_focuses_tiling() {
    let ops = [
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams::new(1),
        },
        floating(2),
        Op::MinimizeWindow { id: None },
    ];
    let layout = check_ops(ops);

    assert_eq!(focus_id(&layout), Some(1));
}

#[test]
fn minimize_tiled_is_noop() {
    let ops = [
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams::new(1),
        },
        Op::MinimizeWindow { id: Some(1) },
    ];
    let layout = check_ops(ops);

    assert!(!layout.is_minimized(&1));
    assert_eq!(focus_id(&layout), Some(1));
}

#[test]
fn minimize_only_window_keeps_it_in_layout() {
    let ops = [
        Op::AddOutput(1),
        floating(1),
        Op::MinimizeWindow { id: None },
        Op::FocusWorkspaceDown,
        Op::FocusWorkspaceUp,
    ];
    let layout = check_ops(ops);

    assert!(layout.has_window(&1));
    assert!(layout.is_minimized(&1));
    assert_eq!(focus_id(&layout), None);
}

#[test]
fn switch_focus_floating_tiling_skips_minimized() {
    let ops = [
        Op::AddOutput(1),
        Op::AddWindow {
            params: TestWindowParams::new(1),
        },
        floating(2),
        Op::MinimizeWindow { id: None },
        Op::SwitchFocusFloatingTiling,
        Op::FocusFloating,
    ];
    let layout = check_ops(ops);

    assert_eq!(focus_id(&layout), Some(1));
}

#[test]
fn close_minimized_window() {
    let ops = [
        Op::AddOutput(1),
        floating(1),
        floating(2),
        Op::MinimizeWindow { id: Some(2) },
        Op::CloseWindow(2),
        Op::CloseWindow(1),
    ];
    check_ops(ops);
}

#[test]
fn move_minimized_window_to_workspace_and_tiling() {
    let ops = [
        Op::AddOutput(1),
        floating(1),
        floating(2),
        Op::MinimizeWindow { id: Some(1) },
        Op::MoveWindowToWorkspace {
            window_id: Some(1),
            workspace_idx: 1,
        },
        Op::MinimizeWindow { id: Some(2) },
        Op::ToggleWindowFloating { id: Some(2) },
    ];
    let layout = check_ops(ops);

    // Leaving the floating space restores the window.
    assert!(!layout.is_minimized(&1));
    assert!(!layout.is_minimized(&2));
}

#[test]
fn minimize_during_interactive_resize() {
    let ops = [
        Op::AddOutput(1),
        floating(1),
        Op::InteractiveResizeBegin {
            window: 1,
            edges: ResizeEdge::BOTTOM_RIGHT,
        },
        Op::MinimizeWindow { id: Some(1) },
        Op::InteractiveResizeUpdate {
            window: 1,
            dx: 100.,
            dy: 100.,
        },
        Op::InteractiveResizeEnd { window: 1 },
    ];
    check_ops(ops);
}

#[test]
fn remove_output_with_minimized_window() {
    let ops = [
        Op::AddOutput(1),
        Op::AddOutput(2),
        floating(1),
        Op::MinimizeWindow { id: Some(1) },
        Op::RemoveOutput(1),
        Op::FocusWindow(1),
    ];
    let layout = check_ops(ops);

    assert!(!layout.is_minimized(&1));
}

#[test]
fn minimized_window_opens_dialog() {
    let ops = [
        Op::AddOutput(1),
        floating(1),
        Op::MinimizeWindow { id: Some(1) },
        Op::AddWindowNextTo {
            params: TestWindowParams::new(2),
            next_to_id: 1,
        },
        Op::AddWindowNextTo {
            params: TestWindowParams {
                is_floating: true,
                parent_id: Some(1),
                ..TestWindowParams::new(3)
            },
            next_to_id: 1,
        },
    ];
    let layout = check_ops(ops);

    assert!(layout.is_minimized(&1));
}
