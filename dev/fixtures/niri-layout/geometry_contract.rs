use super::*;
use niri_config::PresetSize;

fn configured_layout(width: PresetSize) -> Layout<TestWindow> {
    configured_scaled_layout(width, 1., 16., 4., [40., 40., 20., 20.])
}

fn configured_scaled_layout(
    width: PresetSize,
    scale: f64,
    gaps: f64,
    border: f64,
    struts: [f64; 4],
) -> Layout<TestWindow> {
    let mut options = Options::default();
    options.layout.gaps = gaps;
    options.layout.border.off = false;
    options.layout.border.width = border;
    options.layout.default_column_width = Some(width);
    options.layout.struts = Struts {
        left: FloatOrInt(struts[0]),
        right: FloatOrInt(struts[1]),
        top: FloatOrInt(struts[2]),
        bottom: FloatOrInt(struts[3]),
    };
    let mut layout = check_ops_with_options(
        options,
        [Op::AddScaledOutput {
            id: 1,
            scale,
            layout_config: None,
        }],
    );
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

fn assert_near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "actual {actual}, expected {expected}"
    );
}

fn assert_fractional_geometry(
    scale: f64,
    content: (i32, i32),
    frame: (f64, f64),
    offset: f64,
    origin: (f64, f64),
) {
    let mut layout = configured_scaled_layout(
        PresetSize::Proportion(0.5),
        scale,
        3.,
        1.,
        [10.1, 9.9, 5.1, 4.9],
    );
    check_ops_on_layout(&mut layout, [Op::AdvanceAnimations { msec_delta: 10000 }]);
    let (_, _, ws) = layout
        .workspaces()
        .find(|(_, _, ws)| ws.windows().next().is_some())
        .unwrap();
    let (_, geometry) = ws.tiles_with_ipc_layouts().next().unwrap();
    assert_eq!(geometry.window_size, content);
    assert_near(geometry.tile_size.0, frame.0);
    assert_near(geometry.tile_size.1, frame.1);
    assert_near(geometry.window_offset_in_tile.0, offset);
    assert_near(geometry.window_offset_in_tile.1, offset);
    let (_, pos, _) = ws.tiles_with_render_positions().next().unwrap();
    assert_near(pos.x, origin.0);
    assert_near(pos.y, origin.1);
}

#[test]
fn quarter_scale_rounds_gap_border_and_strut_origin_in_physical_pixels() {
    // 1024×576 logical; gap 4 physical, border 1 physical; width floors while auto height rounds.
    assert_fractional_geometry(1.25, (495, 558), (496.8, 560.), 0.8, (13.6, 8.8));
}

#[test]
fn half_scale_rounds_half_pixel_gap_and_border_up() {
    // Gap 4.5→5 physical and border 1.5→2 physical.
    assert_fractional_geometry(
        1.5,
        (408, 460),
        (1232. / 3., 1388. / 3.),
        4. / 3.,
        (14., 26. / 3.),
    );
}

#[test]
fn double_scale_rounds_auto_height_after_fractional_struts() {
    assert_fractional_geometry(2., (303, 342), (305., 344.), 1., (13.5, 8.5));
}

fn settle_three_windows(layout: &mut Layout<TestWindow>) {
    check_ops_on_layout(
        layout,
        [
            Op::Communicate(1),
            Op::Communicate(2),
            Op::Communicate(3),
            Op::AdvanceAnimations { msec_delta: 10000 },
        ],
    );
}

fn fractional_three_columns() -> Layout<TestWindow> {
    let mut layout =
        configured_scaled_layout(PresetSize::Fixed(300), 1.25, 3., 1., [10.1, 9.9, 5.1, 4.9]);
    check_ops_on_layout(
        &mut layout,
        [
            Op::AddWindow {
                params: TestWindowParams::new(2),
            },
            Op::AddWindow {
                params: TestWindowParams::new(3),
            },
            Op::FocusColumnLeft,
            Op::FocusColumnLeft,
        ],
    );
    settle_three_windows(&mut layout);
    layout
}

fn assert_fractional_stack(
    layout: &Layout<TestWindow>,
    expected: &[(usize, usize, usize, i32, f64, f64)],
) {
    let (_, _, ws) = layout
        .workspaces()
        .find(|(_, _, ws)| ws.windows().next().is_some())
        .unwrap();
    let tiles: Vec<_> = ws.tiles_with_ipc_layouts().collect();
    assert_eq!(tiles.len(), expected.len());
    assert_eq!(*ws.active_window().unwrap().id(), 1);
    for ((tile, geometry), &(id, column, row, height, frame_height, y)) in
        tiles.iter().zip(expected)
    {
        assert_eq!(*tile.window().id(), id);
        assert_eq!(geometry.pos_in_scrolling_layout, Some((column, row)));
        assert_eq!(geometry.window_size, (300, height));
        assert_near(geometry.tile_size.0, 301.6);
        assert_near(geometry.tile_size.1, frame_height);
        assert_near(geometry.window_offset_in_tile.0, 0.8);
        assert_near(geometry.window_offset_in_tile.1, 0.8);
        let (_, pos, _) = ws
            .tiles_with_render_positions()
            .find(|(tile, _, _)| *tile.window().id() == id)
            .unwrap();
        assert_near(pos.x, 13.6 + (column - 1) as f64 * 304.8);
        assert_near(pos.y, y);
    }
}

fn consume_three_windows() -> Layout<TestWindow> {
    let mut layout = fractional_three_columns();
    check_ops_on_layout(&mut layout, [Op::ConsumeWindowIntoColumn]);
    settle_three_windows(&mut layout);
    check_ops_on_layout(&mut layout, [Op::ConsumeWindowIntoColumn]);
    settle_three_windows(&mut layout);
    layout
}

#[test]
fn sequential_consume_distributes_fractional_height_remainder() {
    let mut layout = fractional_three_columns();
    check_ops_on_layout(&mut layout, [Op::ConsumeWindowIntoColumn]);
    settle_three_windows(&mut layout);
    // The remaining third column still has its original client-chosen width.
    let (_, _, ws) = layout
        .workspaces()
        .find(|(_, _, ws)| ws.windows().next().is_some())
        .unwrap();
    let sizes: Vec<_> = ws
        .tiles_with_ipc_layouts()
        .map(|(tile, geometry)| (*tile.window().id(), geometry.window_size))
        .collect();
    assert_eq!(
        sizes,
        vec![(1, (300, 276)), (2, (300, 277)), (3, (100, 558))]
    );
    check_ops_on_layout(&mut layout, [Op::ConsumeWindowIntoColumn]);
    settle_three_windows(&mut layout);
    assert_fractional_stack(
        &layout,
        &[
            (1, 1, 1, 183, 184.8, 8.8),
            (2, 1, 2, 182, 184., 196.8),
            (3, 1, 3, 183, 184.8, 384.),
        ],
    );
}

#[test]
fn expel_moves_last_window_and_reconsume_restores_fractional_stack() {
    let mut layout = consume_three_windows();
    check_ops_on_layout(&mut layout, [Op::ExpelWindowFromColumn]);
    settle_three_windows(&mut layout);
    assert_fractional_stack(
        &layout,
        &[
            (1, 1, 1, 276, 277.6, 8.8),
            (2, 1, 2, 277, 278.4, 289.6),
            (3, 2, 1, 558, 560., 8.8),
        ],
    );
    check_ops_on_layout(&mut layout, [Op::ConsumeWindowIntoColumn]);
    settle_three_windows(&mut layout);
    assert_fractional_stack(
        &layout,
        &[
            (1, 1, 1, 183, 184.8, 8.8),
            (2, 1, 2, 182, 184., 196.8),
            (3, 1, 3, 183, 184.8, 384.),
        ],
    );
}

#[test]
fn fixed_height_and_reset_redistribute_fractional_auto_remainder() {
    let mut layout = consume_three_windows();
    check_ops_on_layout(
        &mut layout,
        [Op::SetWindowHeight {
            id: Some(2),
            change: SizeChange::SetFixed(100),
        }],
    );
    settle_three_windows(&mut layout);
    assert_fractional_stack(
        &layout,
        &[
            (1, 1, 1, 224, 225.6, 8.8),
            (2, 1, 2, 100, 101.6, 237.6),
            (3, 1, 3, 224, 225.6, 342.4),
        ],
    );
    check_ops_on_layout(&mut layout, [Op::ResetWindowHeight { id: Some(2) }]);
    settle_three_windows(&mut layout);
    assert_fractional_stack(
        &layout,
        &[
            (1, 1, 1, 183, 184.8, 8.8),
            (2, 1, 2, 182, 184., 196.8),
            (3, 1, 3, 183, 184.8, 384.),
        ],
    );
}
