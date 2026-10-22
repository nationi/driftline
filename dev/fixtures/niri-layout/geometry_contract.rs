use super::*;
use niri_config::PresetSize;

fn configured_layout(width: PresetSize) -> Layout<TestWindow> {
    let mut options = Options::default();
    options.layout.gaps = 16.;
    options.layout.border.off = false;
    options.layout.border.width = 4.;
    options.layout.default_column_width = Some(width);
    options.layout.struts = Struts {
        left: FloatOrInt(40.),
        right: FloatOrInt(40.),
        top: FloatOrInt(20.),
        bottom: FloatOrInt(20.),
    };
    let mut layout = check_ops_with_options(options, [Op::AddOutput(1)]);
    layout.add_window(
        TestWindow::new(TestWindowParams::new(1)),
        AddWindowTarget::Auto,
        Some(width),
        None,
        false,
        false,
        ActivateWindow::default(),
    );
    check_ops_on_layout(&mut layout, [Op::Communicate(1)]);
    layout
}

fn assert_single_size(layout: &Layout<TestWindow>, content: (i32, i32), frame: (f64, f64)) {
    let (_, _, ws) = layout
        .workspaces()
        .find(|(_, _, ws)| ws.windows().next().is_some())
        .unwrap();
    let tiles: Vec<_> = ws.tiles_with_ipc_layouts().collect();
    assert_eq!(tiles.len(), 1);
    let (_, geometry) = &tiles[0];
    assert_eq!(geometry.window_size, content);
    assert_eq!(geometry.tile_size, frame);
    assert_eq!(geometry.window_offset_in_tile, (4., 4.));
    assert_eq!(geometry.pos_in_scrolling_layout, Some((1, 1)));
}

#[test]
fn fixed_width_is_content_width_with_border_added_outside() {
    // 720 - 40 struts - 32 gaps - 8 border = 640 content height.
    let layout = configured_layout(PresetSize::Fixed(300));
    assert_single_size(&layout, (300, 640), (308., 648.));
}

#[test]
fn half_width_divides_work_area_after_inter_column_gap() {
    // 1200 usable width accommodates two 576 frames and three 16 gaps.
    let layout = configured_layout(PresetSize::Proportion(0.5));
    assert_single_size(&layout, (568, 640), (576., 648.));
}

#[test]
fn explicit_fixed_resize_uses_content_width() {
    let mut layout = configured_layout(PresetSize::Proportion(0.5));
    check_ops_on_layout(
        &mut layout,
        [
            Op::SetColumnWidth(SizeChange::SetFixed(400)),
            Op::Communicate(1),
        ],
    );
    assert_single_size(&layout, (400, 640), (408., 648.));
}

#[test]
fn struts_shift_frame_origin_beyond_outer_gaps() {
    let mut layout = configured_layout(PresetSize::Fixed(300));
    check_ops_on_layout(&mut layout, [Op::AdvanceAnimations { msec_delta: 10000 }]);
    let (_, _, ws) = layout
        .workspaces()
        .find(|(_, _, ws)| ws.windows().next().is_some())
        .unwrap();
    let (_, pos, _) = ws.tiles_with_render_positions().next().unwrap();
    assert_eq!((pos.x, pos.y), (56., 36.));
}

#[test]
fn consume_splits_frame_height_and_preserves_gap() {
    let mut layout = configured_layout(PresetSize::Fixed(300));
    check_ops_on_layout(
        &mut layout,
        [
            Op::AddWindow {
                params: TestWindowParams::new(2),
            },
            Op::Communicate(2),
            Op::FocusColumnLeft,
            Op::ConsumeWindowIntoColumn,
            Op::Communicate(1),
            Op::Communicate(2),
            Op::AdvanceAnimations { msec_delta: 10000 },
        ],
    );
    let (_, _, ws) = layout
        .workspaces()
        .find(|(_, _, ws)| ws.windows().next().is_some())
        .unwrap();
    let tiles: Vec<_> = ws.tiles_with_ipc_layouts().collect();
    assert_eq!(tiles.len(), 2);
    for (index, (tile, geometry)) in tiles.iter().enumerate() {
        assert_eq!(*tile.window().id(), index + 1);
        assert_eq!(geometry.pos_in_scrolling_layout, Some((1, index + 1)));
        // 680 usable height minus three gaps leaves two 316-high frames.
        assert_eq!(geometry.tile_size, (308., 316.));
        assert_eq!(geometry.window_size, (300, 308));
    }
    let mut positions: Vec<_> = ws
        .tiles_with_render_positions()
        .map(|(tile, pos, _)| (*tile.window().id(), pos.x, pos.y))
        .collect();
    positions.sort_by_key(|entry| entry.0);
    assert_eq!(positions, vec![(1, 56., 36.), (2, 56., 368.)]);
}
