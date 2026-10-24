use driftline::config::{Action, BTN_LEFT, BTN_MIDDLE, BTN_RIGHT, Direction};
use smithay::reexports::wayland_server::Resource;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::seat::WaylandFocus;

use super::input_backend::{FakeDevice, key_press, key_release, pointer_to, press, release};
use super::{
    Fixture, ack_but_keep_size, adopt_last_configure, configure_count, last_configured, map_window,
    window_by_app_id, window_position,
};

const KEY_LEFTALT: u32 = 56;
const KEY_LEFTMETA: u32 = 125;
const KEY_F: u32 = 33;
const KEY_LEFT: u32 = 105;
const KEY_RIGHT: u32 = 106;

#[test]
fn remapping_a_tiled_window_restores_other_windows_before_placing_it() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 300));
    let b = map_window(&mut f, client, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let before_b = window_position(&mut f, &wb);
    f.state().set_camera((100.0, -200.0).into());
    f.state().set_zoom(0.6);
    f.state().execute_action(&Action::ToggleTiling);
    f.state().map_window(wa.clone(), (900, -500).into(), true);
    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.tiled_columns().is_empty());
    assert_eq!(window_position(&mut f, &wa), (900, -500).into());
    assert_eq!(window_position(&mut f, &wb), before_b);
    assert_eq!(last_configured(&mut f, client, &a), (400, 300));
    assert_eq!(last_configured(&mut f, client, &b), (350, 250));
    assert_eq!(f.state().camera(), (100.0, -200.0).into());
    assert_eq!(f.state().zoom(), 0.6);
}

#[test]
fn alt_drag_swaps_tiled_windows_without_leaving_tiling() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (400, 300));
    let a = window_by_app_id(&mut f, "a").unwrap();
    let b = window_by_app_id(&mut f, "b").unwrap();
    let aid = f.state().stage.id_of(&a).unwrap();
    let bid = f.state().stage.id_of(&b).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);

    let a_column = f.state().stage.column_of(aid).unwrap();
    let b_column = f.state().stage.column_of(bid).unwrap();
    assert_ne!(a_column, b_column);
    let a_position = window_position(&mut f, &a);
    let b_position = window_position(&mut f, &b);
    let a_center = a_position
        + smithay::utils::Point::from((a.geometry().size.w / 2, a.geometry().size.h / 2));
    let b_center = b_position
        + smithay::utils::Point::from((b.geometry().size.w / 2, b.geometry().size.h / 2));
    let mouse = FakeDevice::mouse();

    pointer_to(&mut f, &mouse, a_center.to_f64());
    key_press(&mut f, KEY_LEFTALT);
    press(&mut f, &mouse, BTN_LEFT);
    pointer_to(&mut f, &mouse, b_center.to_f64());
    let pointer_position = f.state().seat.get_pointer().unwrap().current_location();
    let hit = f
        .state()
        .topmost_client_under(pointer_position)
        .and_then(|window| f.state().stage.id_of(&window));
    assert_eq!(hit, Some(bid), "drop point should hit the target tile");
    release(&mut f, &mouse, BTN_LEFT);
    key_release(&mut f, KEY_LEFTALT);
    f.pump(1);

    assert_eq!(f.state().stage.column_of(aid), Some(b_column));
    assert_eq!(f.state().stage.column_of(bid), Some(a_column));
    assert_eq!(f.state().stage.active_tile(), Some(aid));
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn canvas_mouse_resize_and_cluster_move_bindings_do_not_leave_tiling() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (400, 300));
    let window = window_by_app_id(&mut f, "a").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);

    let pointer = FakeDevice::mouse();
    let location = window_position(&mut f, &window)
        + smithay::utils::Point::from((window.geometry().size.w / 2, window.geometry().size.h / 2));
    pointer_to(&mut f, &pointer, location.to_f64());
    key_press(&mut f, KEY_LEFTALT);
    press(&mut f, &pointer, BTN_RIGHT);
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().interactive_move.is_empty());
    release(&mut f, &pointer, BTN_RIGHT);
    press(&mut f, &pointer, BTN_MIDDLE);
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().interactive_move.is_empty());
    release(&mut f, &pointer, BTN_MIDDLE);
    key_release(&mut f, KEY_LEFTALT);

    key_press(&mut f, KEY_LEFTALT);
    key_press(&mut f, 42); // Left Shift
    press(&mut f, &pointer, BTN_RIGHT);
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().interactive_move.is_empty());
    release(&mut f, &pointer, BTN_RIGHT);
    press(&mut f, &pointer, BTN_MIDDLE);
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().interactive_move.is_empty());
    release(&mut f, &pointer, BTN_MIDDLE);

    press(&mut f, &pointer, BTN_LEFT);
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().interactive_move.is_empty());
    release(&mut f, &pointer, BTN_LEFT);
    key_release(&mut f, 42);
    key_release(&mut f, KEY_LEFTALT);
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn super_left_drag_scrolls_columns_horizontally_in_tiling() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (400, 300));
    let window = window_by_app_id(&mut f, "a").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    f.state().cancel_animations_everywhere();

    let position = window_position(&mut f, &window);
    let start = position
        + smithay::utils::Point::from((window.geometry().size.w / 2, window.geometry().size.h / 2));
    let pointer = FakeDevice::mouse();
    pointer_to(&mut f, &pointer, start.to_f64());
    let camera_before = f.state().camera();

    key_press(&mut f, KEY_LEFTMETA);
    press(&mut f, &pointer, BTN_LEFT);
    pointer_to(
        &mut f,
        &pointer,
        (start + smithay::utils::Point::from((120, 0))).to_f64(),
    );
    release(&mut f, &pointer, BTN_LEFT);
    key_release(&mut f, KEY_LEFTMETA);

    assert!(f.state().tiling_view.is_some());
    assert!(f.state().camera().x < camera_before.x);
    assert_eq!(f.state().camera().y, camera_before.y);
}

#[test]
fn remapping_after_a_tiling_resize_supersedes_the_restore_recenter() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let surface = map_window(&mut f, client, "a", (400, 300));
    let window = window_by_app_id(&mut f, "a").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let tiled_size = last_configured(&mut f, client, &surface);
    adopt_last_configure(&mut f, client, &surface);
    assert_eq!(window.geometry().size, tiled_size.into());
    let destination = (900, -500).into();
    f.state().map_window(window.clone(), destination, true);
    assert!(f.state().tiling_view.is_none());
    assert_eq!(last_configured(&mut f, client, &surface), (400, 300));
    adopt_last_configure(&mut f, client, &surface);
    assert_eq!(window.geometry().size, (400, 300).into());
    assert_eq!(window_position(&mut f, &window), destination);
    assert_eq!(f.state().stage.id_of(&window), Some(id));
    assert_eq!(f.state().tiling_pending_configures(), 0);
}

#[test]
fn direct_fit_restores_the_canvas_without_action_dispatch() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    let b = map_window(&mut f, client, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let before_b = window_position(&mut f, &wb);
    f.state().execute_action(&Action::ToggleTiling);
    f.state().fit_window(&wa);
    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.tiled_columns().is_empty());
    assert!(f.state().stage.is_fit(&wa));
    assert_eq!(window_position(&mut f, &wb), before_b);
    assert_eq!(last_configured(&mut f, client, &b), (350, 250));
}

#[test]
fn refusing_a_resize_reserves_the_live_width_without_configure_loops() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (1000, 300));
    let b = map_window(&mut f, client, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    ack_but_keep_size(&mut f, client, &a);
    ack_but_keep_size(&mut f, client, &b);
    let count = configure_count(&mut f, client, &a);
    for _ in 0..5 {
        f.state().refresh_and_flush_clients();
    }
    assert_eq!(configure_count(&mut f, client, &a), count);
    let a_frame = f
        .state()
        .element_chrome(&wa)
        .frame_loc(window_position(&mut f, &wa));
    let b_frame = f
        .state()
        .element_chrome(&wb)
        .frame_loc(window_position(&mut f, &wb));
    assert!(b_frame.x >= a_frame.x + 1000);
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn reentering_before_canvas_restore_commit_keeps_the_canvas_size() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let surface = map_window(&mut f, client, "a", (400, 300));
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    adopt_last_configure(&mut f, client, &surface);
    f.state().execute_action(&Action::ToggleTiling);
    f.state().execute_action(&Action::ToggleTiling);
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(last_configured(&mut f, client, &surface), (400, 300));
}

#[test]
fn tiling_restores_canvas_rectangles_and_view_even_before_clients_ack() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 300));
    let b = map_window(&mut f, client, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let before_a = window_position(&mut f, &wa);
    let before_b = window_position(&mut f, &wb);
    f.state().set_camera((100.0, -200.0).into());
    f.state().set_zoom(0.6);
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(f.state().stage.tiled_columns().len(), 2);
    assert_eq!(f.state().zoom(), 1.0);
    let first_size = last_configured(&mut f, client, &a);
    assert!(first_size.0 > 0 && first_size.0 <= 600);
    assert!(first_size.1 > 700);
    f.state().execute_action(&Action::ToggleTiling);
    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.tiled_columns().is_empty());
    assert_eq!(window_position(&mut f, &wa), before_a);
    assert_eq!(window_position(&mut f, &wb), before_b);
    assert_eq!(last_configured(&mut f, client, &a), (400, 300));
    assert_eq!(last_configured(&mut f, client, &b), (350, 250));
    assert_eq!(f.state().camera(), (100.0, -200.0).into());
    assert_eq!(f.state().zoom(), 0.6);
}

#[test]
fn reflow_does_not_raise_windows_or_send_repeated_size_requests() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (350, 250));
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    let before: Vec<_> = f.state().stage.windows().cloned().collect();
    let count = configure_count(&mut f, client, &a);
    for _ in 0..10 {
        f.state().refresh_and_flush_clients();
    }
    assert_eq!(configure_count(&mut f, client, &a), count);
    assert_eq!(
        f.state().stage.windows().cloned().collect::<Vec<_>>(),
        before
    );
    adopt_last_configure(&mut f, client, &a);
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn focus_and_width_actions_follow_columns() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    let aid = f.state().stage.id_of(&wa).unwrap();
    assert_eq!(f.state().stage.active_tile(), Some(aid));
    let half = last_configured(&mut f, client, &a).0;
    f.state().execute_action(&Action::ToggleWideColumn);
    let wide = last_configured(&mut f, client, &a).0;
    assert!(wide > half);
    f.state().execute_action(&Action::ToggleWideColumn);
    assert_eq!(last_configured(&mut f, client, &a).0, half);
    f.state().execute_action(&Action::CycleColumnWidth);
    assert!(last_configured(&mut f, client, &a).0 > half);
    f.state()
        .execute_action(&Action::CenterNearest(Direction::Right));
    assert_eq!(f.state().stage.active_tile(), Some(aid));
}

#[test]
fn super_left_right_navigate_tiling_columns_with_sloppy_focus_enabled() {
    let mut f = Fixture::with_config(super::config("focus_follows_mouse = true\n"));
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (400, 300));
    map_window(&mut f, client, "c", (400, 300));
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);

    let columns = f.state().stage.tiled_columns();
    let left = columns[0].selected;
    let middle = columns[1].selected;
    let left_window = f
        .state()
        .stage
        .window_by_id(left)
        .unwrap()
        .client()
        .unwrap()
        .clone();
    f.state()
        .raise_and_focus(&left_window, SERIAL_COUNTER.next_serial());
    f.state().stage.select_tile(left);
    f.state().refresh_tiling();

    key_press(&mut f, KEY_LEFTMETA);
    key_press(&mut f, KEY_RIGHT);
    key_release(&mut f, KEY_RIGHT);
    assert_eq!(f.state().stage.active_tile(), Some(middle));
    assert!(!f.state().hover_follows_camera);

    key_press(&mut f, KEY_LEFT);
    key_release(&mut f, KEY_LEFT);
    key_release(&mut f, KEY_LEFTMETA);
    assert_eq!(f.state().stage.active_tile(), Some(left));
}

#[test]
fn new_windows_join_after_the_focused_column_and_close_prunes_membership() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.state().raise_and_focus(&wa, SERIAL_COUNTER.next_serial());
    f.state().refresh_tiling();
    let c = map_window(&mut f, client, "c", (400, 300));
    let wc = window_by_app_id(&mut f, "c").unwrap();
    let expected = [&wa, &wc, &wb].map(|w| f.state().stage.id_of(w).unwrap());
    assert_eq!(
        f.state()
            .stage
            .tiled_columns()
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        expected
    );
    f.client(client).window(&c).destroy();
    f.double_roundtrip(client);
    assert_eq!(f.state().stage.tiled_columns().len(), 2);
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn multiple_outputs_create_independent_layouts_on_tiling_entry() {
    use driftline::layout::scrolling::transaction::ConfigureProgress;
    let mut f = Fixture::new();
    let first = f.add_output(1, (1200, 800));
    let second = f.add_output(2, (1920, 1080));
    crate::state::output_state(&second).camera = (5000.0, 5000.0).into();
    f.state().focused_output = Some(first.clone());
    let client = f.add_client();
    let first_surface = map_window(&mut f, client, "first-window", (400, 300));
    let first_window = window_by_app_id(&mut f, "first-window").unwrap();
    let first_window_id = f.state().stage.id_of(&first_window).unwrap();
    f.state().focused_output = Some(second.clone());
    let second_surface = map_window(&mut f, client, "second-window", (400, 300));
    let second_window = window_by_app_id(&mut f, "second-window").unwrap();
    let second_window_id = f.state().stage.id_of(&second_window).unwrap();
    let first_camera = crate::state::output_state(&first).camera;
    let second_camera = crate::state::output_state(&second).camera;
    f.state().execute_action(&Action::ToggleTiling);

    assert!(f.state().tiling_view.is_some());
    assert!(
        f.state()
            .stage
            .tiling_layout_id_for_output(&first.name())
            .is_some()
    );
    assert!(
        f.state()
            .stage
            .tiling_layout_id_for_output(&second.name())
            .is_some()
    );
    assert_eq!(
        f.state().tiling_canvas_view(&first),
        Some((first_camera, 1.0))
    );
    assert_eq!(
        f.state().tiling_canvas_view(&second),
        Some((second_camera, 1.0))
    );
    let first_owner = f
        .state()
        .stage
        .tiling_output_for_window(first_window_id)
        .unwrap()
        .to_owned();
    let second_owner = f
        .state()
        .stage
        .tiling_output_for_window(second_window_id)
        .unwrap()
        .to_owned();
    assert_eq!(first_owner, first.name());
    assert_eq!(second_owner, second.name());
    assert_eq!(
        f.state()
            .stage
            .tiled_windows_for_output(&first_owner)
            .map(|tile| tile.id)
            .collect::<Vec<_>>(),
        vec![first_window_id]
    );
    assert_eq!(
        f.state()
            .stage
            .tiled_windows_for_output(&second_owner)
            .map(|tile| tile.id)
            .collect::<Vec<_>>(),
        vec![second_window_id]
    );
    let first_size = last_configured(&mut f, client, &first_surface);
    let second_size = last_configured(&mut f, client, &second_surface);
    assert!(
        second_size.0 > first_size.0,
        "each output reflows against its own width: {first_size:?} vs {second_size:?}"
    );
    assert_eq!(
        f.state().tiling_configure_progress(first_window_id),
        Some(ConfigureProgress::Pending)
    );
    assert_eq!(
        f.state().tiling_configure_progress(second_window_id),
        Some(ConfigureProgress::Pending)
    );
    adopt_last_configure(&mut f, client, &first_surface);
    assert_eq!(
        f.state().tiling_configure_progress(first_window_id),
        Some(ConfigureProgress::Applied)
    );
    assert_eq!(
        f.state().tiling_configure_progress(second_window_id),
        Some(ConfigureProgress::Pending),
        "a buffer on one output cannot complete the other output's configure"
    );
    adopt_last_configure(&mut f, client, &second_surface);
    assert_eq!(
        f.state().tiling_configure_progress(second_window_id),
        Some(ConfigureProgress::Applied)
    );

    f.state().execute_action(&Action::ToggleTiling);
    assert!(f.state().tiling_view.is_none());
    assert!(!f.state().stage.is_tiled(first_window_id));
    assert!(!f.state().stage.is_tiled(second_window_id));
}

#[test]
fn tiling_reentry_restores_column_order_stacking_width_and_output() {
    let mut f = Fixture::new();
    let first = f.add_output(1, (1200, 800));
    let second = f.add_output(2, (1920, 1080));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    map_window(&mut f, client, "b", (400, 300));
    let a = window_by_app_id(&mut f, "a").unwrap();
    let b = window_by_app_id(&mut f, "b").unwrap();
    let aid = f.state().stage.id_of(&a).unwrap();
    let bid = f.state().stage.id_of(&b).unwrap();

    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::MoveColumn(Direction::Left));
    f.state().execute_action(&Action::ToggleWideColumn);
    assert!(f.state().move_tiled_window_to_output(aid, &second));

    let before_first = f
        .state()
        .stage
        .tiled_columns_for_output(&first.name())
        .unwrap()
        .to_vec();
    let before_second = f
        .state()
        .stage
        .tiled_columns_for_output(&second.name())
        .unwrap()
        .to_vec();
    f.state().execute_action(&Action::ToggleTiling);
    assert!(f.state().stage.tiled_columns().is_empty());
    f.state().execute_action(&Action::ToggleTiling);

    assert_eq!(
        f.state()
            .stage
            .tiled_columns_for_output(&first.name())
            .unwrap(),
        before_first
    );
    assert_eq!(
        f.state()
            .stage
            .tiled_columns_for_output(&second.name())
            .unwrap(),
        before_second
    );
    assert_eq!(
        f.state().stage.tiling_output_for_window(aid),
        Some(second.name().as_str())
    );
    assert_eq!(
        f.state().stage.tiling_output_for_window(bid),
        Some(first.name().as_str())
    );
}

#[test]
fn fullscreen_keeps_tiling_active_across_outputs_until_the_window_closes() {
    let mut f = Fixture::new();
    let output = f.add_output(1, (1200, 800));
    let second_output = f.add_output(2, (1600, 900));
    crate::state::output_state(&second_output).camera = (5000.0, 5000.0).into();
    let client = f.add_client();
    f.state().focused_output = Some(output.clone());
    let fullscreen_surface = map_window(&mut f, client, "fullscreen", (400, 300));
    f.state().focused_output = Some(second_output.clone());
    map_window(&mut f, client, "survivor", (400, 300));
    let fullscreen_window = window_by_app_id(&mut f, "fullscreen").unwrap();
    let survivor = window_by_app_id(&mut f, "survivor").unwrap();
    let fullscreen_id = f.state().stage.id_of(&fullscreen_window).unwrap();
    let survivor_id = f.state().stage.id_of(&survivor).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(
        f.state().stage.tiling_output_for_window(fullscreen_id),
        Some(output.name().as_str())
    );
    assert_eq!(
        f.state().stage.tiling_output_for_window(survivor_id),
        Some(second_output.name().as_str())
    );
    f.state()
        .enter_fullscreen(&fullscreen_window, Some(output.clone()));
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().stage.is_tiled(fullscreen_id));
    assert!(f.state().stage.is_tiled(survivor_id));
    assert!(f.state().stage.is_tiled(survivor_id));

    f.client(client).window(&fullscreen_surface).destroy();
    f.double_roundtrip(client);

    assert!(f.state().tiling_view.is_some());
    assert!(!f.state().stage.is_tiled(fullscreen_id));
    assert!(f.state().stage.is_tiled(survivor_id));
}

#[test]
fn win_f_enters_and_exits_fullscreen_without_leaving_tiling() {
    let mut f = Fixture::new();
    let output = f.add_output(1, (1200, 800));
    let client = f.add_client();
    let surface = map_window(&mut f, client, "fullscreen", (400, 300));
    let window = window_by_app_id(&mut f, "fullscreen").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();

    f.state().execute_action(&Action::ToggleTiling);
    key_press(&mut f, KEY_LEFTMETA);
    key_press(&mut f, KEY_F);
    key_release(&mut f, KEY_F);
    key_release(&mut f, KEY_LEFTMETA);
    ack_but_keep_size(&mut f, client, &surface);
    assert!(f.state().stage.fullscreen_on(&output.name()).is_some());
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().stage.is_tiled(id));

    key_press(&mut f, KEY_LEFTMETA);
    key_press(&mut f, KEY_F);
    key_release(&mut f, KEY_F);
    key_release(&mut f, KEY_LEFTMETA);
    ack_but_keep_size(&mut f, client, &surface);
    assert!(f.state().stage.fullscreen_on(&output.name()).is_none());
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().stage.is_tiled(id));
}

#[test]
fn fullscreen_restore_commit_keeps_tiling_active() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "fullscreen", (400, 300));
    let window = window_by_app_id(&mut f, "fullscreen").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();

    f.state().execute_action(&Action::ToggleTiling);
    assert!(f.state().stage.is_tiled(id));
    f.state().pending_recenter.insert(
        window.wl_surface().unwrap().id(),
        crate::state::PendingRecenter {
            target_center: (600.0, 400.0).into(),
            pre_exit_size: window.geometry().size,
        },
    );

    f.state().map_recentered_window(&window, (900, 100).into());

    assert!(f.state().tiling_view.is_some());
    assert!(f.state().stage.is_tiled(id));
    assert_eq!(window_position(&mut f, &window), (900, 100).into());
}

#[test]
fn a_window_opening_fullscreen_does_not_switch_other_outputs_to_canvas() {
    let mut f = Fixture::with_config(super::config(
        r#"
[[window_rules]]
app_id = "fullscreen"
fullscreen = true
output = "HEADLESS-1"
"#,
    ));
    let first = f.add_output(1, (1200, 800));
    let second = f.add_output(2, (1600, 900));
    crate::state::output_state(&second).camera = (5000.0, 5000.0).into();
    let client = f.add_client();
    f.state().focused_output = Some(second.clone());
    map_window(&mut f, client, "survivor", (400, 300));
    let survivor = window_by_app_id(&mut f, "survivor").unwrap();
    let survivor_id = f.state().stage.id_of(&survivor).unwrap();

    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(
        f.state().stage.tiling_output_for_window(survivor_id),
        Some(second.name().as_str())
    );

    let fullscreen_surface = map_window(&mut f, client, "fullscreen", (400, 300));
    let fullscreen_window = window_by_app_id(&mut f, "fullscreen").unwrap();
    let fullscreen_id = f.state().stage.id_of(&fullscreen_window).unwrap();
    assert!(f.state().stage.fullscreen_on(&first.name()).is_some());
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().stage.is_tiled(survivor_id));
    assert!(!f.state().stage.is_tiled(fullscreen_id));

    f.client(client).window(&fullscreen_surface).destroy();
    f.double_roundtrip(client);
    assert!(f.state().tiling_view.is_some());
    assert!(f.state().stage.is_tiled(survivor_id));
}

#[test]
fn a_window_mapped_on_another_output_joins_that_outputs_layout() {
    let mut f = Fixture::new();
    let first = f.add_output(1, (1200, 800));
    let second = f.add_output(2, (1920, 1080));
    crate::state::output_state(&second).camera = (5000.0, 5000.0).into();
    let client = f.add_client();

    f.state().focused_output = Some(first.clone());
    let first_surface = map_window(&mut f, client, "first", (400, 300));
    let first_window = window_by_app_id(&mut f, "first").unwrap();
    let first_id = f.state().stage.id_of(&first_window).unwrap();
    f.state().focused_output = Some(second.clone());
    let second_surface = map_window(&mut f, client, "second", (400, 300));
    let second_window = window_by_app_id(&mut f, "second").unwrap();
    let second_id = f.state().stage.id_of(&second_window).unwrap();

    f.state().execute_action(&Action::ToggleTiling);
    let first_tiled_size = last_configured(&mut f, client, &first_surface);
    let second_tiled_size = last_configured(&mut f, client, &second_surface);
    assert!(first_tiled_size.0 > 0 && second_tiled_size.0 > first_tiled_size.0);
    adopt_last_configure(&mut f, client, &first_surface);
    adopt_last_configure(&mut f, client, &second_surface);

    f.state().focused_output = Some(second.clone());
    let new_surface = map_window(&mut f, client, "late", (300, 200));
    let new_window = window_by_app_id(&mut f, "late").unwrap();
    let new_id = f.state().stage.id_of(&new_window).unwrap();
    f.state().refresh_tiling();

    assert_eq!(
        f.state().stage.tiling_output_for_window(new_id),
        Some(second.name().as_str()),
        "the focused output owns a window mapped after tiling entry"
    );
    assert_eq!(
        f.state()
            .stage
            .tiled_windows_for_output(&first.name())
            .map(|tile| tile.id)
            .collect::<Vec<_>>(),
        vec![first_id]
    );
    assert_eq!(
        f.state()
            .stage
            .tiled_windows_for_output(&second.name())
            .map(|tile| tile.id)
            .collect::<Vec<_>>(),
        vec![second_id, new_id]
    );
    let new_size = last_configured(&mut f, client, &new_surface);
    assert!(
        new_size.0 > first_tiled_size.0,
        "late mapped window is configured against output two's wider area: {new_size:?} vs {first_tiled_size:?}"
    );

    let second_layout = f
        .state()
        .stage
        .tiling_layout_id_for_output(&second.name())
        .unwrap();
    f.state().focused_output = Some(first.clone());
    f.state().refresh_tiling();
    f.state()
        .navigate_to_window_on(&second_window, &second, crate::state::NavZoom::Reset);
    assert_eq!(f.state().stage.tiling_layout_id(), Some(second_layout));
    assert!(f.state().tiling_view.is_some());
    f.state().execute_action(&Action::ToggleTiling);
}

#[test]
fn canvas_actions_are_ignored_until_tiling_is_switched_off() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    f.state().execute_action(&Action::ToggleTiling);
    f.state().execute_action(&Action::FitWindow);
    assert!(f.state().tiling_view.is_some());
    let window = window_by_app_id(&mut f, "a").unwrap();
    assert!(!f.state().stage.is_fit(&window));
    f.state().execute_action(&Action::ToggleTiling);
    f.state().execute_action(&Action::FitWindow);
    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.is_fit(&window));
}

#[test]
fn a_dialog_remains_floating_and_does_not_replace_its_parent_column() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let parent = map_window(&mut f, client, "parent", (400, 300));
    f.state().execute_action(&Action::ToggleTiling);
    let parent_role = f.client(client).window(&parent).xdg_toplevel.clone();
    let dialog = f.client(client).create_window();
    let surface = dialog.surface.clone();
    dialog.set_parent(Some(&parent_role));
    dialog.set_app_id("dialog");
    dialog.commit();
    f.roundtrip(client);
    assert_eq!(last_configured(&mut f, client, &surface), (0, 0));
    let dialog = f.client(client).window(&surface);
    dialog.set_size(200, 100);
    dialog.attach_new_buffer();
    dialog.ack_last_and_commit();
    f.double_roundtrip(client);
    assert!(f.state().tiling_view.is_some());
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    let dialog_window = window_by_app_id(&mut f, "dialog").unwrap();
    let dialog_id = f.state().stage.id_of(&dialog_window).unwrap();
    assert!(!f.state().stage.is_tiled(dialog_id));
}

#[test]
fn restoring_after_a_tiled_commit_settles_at_the_original_canvas_position() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let surface = map_window(&mut f, client, "a", (400, 300));
    let window = window_by_app_id(&mut f, "a").unwrap();
    let before = window_position(&mut f, &window);
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    adopt_last_configure(&mut f, client, &surface);
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    adopt_last_configure(&mut f, client, &surface);
    assert_eq!(window_position(&mut f, &window), before);
}

#[test]
fn grouped_windows_share_height_navigate_and_restore_their_original_rectangles() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let b = map_window(&mut f, client, "b", (400, 250));
    let c = map_window(&mut f, client, "c", (400, 300));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let wc = window_by_app_id(&mut f, "c").unwrap();
    let saved: Vec<_> = [&wa, &wb, &wc]
        .iter()
        .map(|w| window_position(&mut f, w))
        .collect();
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::MoveIntoColumn(Direction::Left));
    assert_eq!(f.state().stage.tiled_columns().len(), 2);
    let b_size = last_configured(&mut f, client, &b);
    let c_size = last_configured(&mut f, client, &c);
    assert!((b_size.1 - c_size.1).abs() <= 1);
    assert!(b_size.1 < 500);
    assert_eq!(b_size.0, c_size.0);
    adopt_last_configure(&mut f, client, &a);
    adopt_last_configure(&mut f, client, &b);
    adopt_last_configure(&mut f, client, &c);
    assert_eq!(
        window_position(&mut f, &wb).x,
        window_position(&mut f, &wc).x
    );
    assert!(window_position(&mut f, &wb).y < window_position(&mut f, &wc).y);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Up));
    let bid = f.state().stage.id_of(&wb).unwrap();
    assert_eq!(f.state().stage.active_tile(), Some(bid));
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Right));
    assert_eq!(f.state().stage.active_tile(), Some(bid));
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Down));
    let cid = f.state().stage.id_of(&wc).unwrap();
    assert_eq!(f.state().stage.active_tile(), Some(cid));
    f.state().execute_action(&Action::ToggleTiling);
    for (surface, window, expected) in [
        (&a, &wa, (400, 200)),
        (&b, &wb, (400, 250)),
        (&c, &wc, (400, 300)),
    ] {
        assert_eq!(last_configured(&mut f, client, surface), expected);
        adopt_last_configure(&mut f, client, surface);
        assert!(saved.contains(&window_position(&mut f, window)));
    }
    assert_eq!(
        [&wa, &wb, &wc]
            .iter()
            .map(|w| window_position(&mut f, w))
            .collect::<Vec<_>>(),
        saved
    );
}

#[test]
fn expelling_and_closing_grouped_windows_keeps_membership_consistent() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    map_window(&mut f, client, "b", (400, 250));
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::MoveIntoColumn(Direction::Left));
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    f.state().execute_action(&Action::ExpelFromColumn);
    assert_eq!(f.state().stage.tiled_columns().len(), 2);
    f.state()
        .execute_action(&Action::MoveIntoColumn(Direction::Left));
    f.client(client).window(&a).destroy();
    f.double_roundtrip(client);
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    assert_eq!(f.state().stage.tiled_windows().count(), 1);
    let b = window_by_app_id(&mut f, "b").unwrap();
    let id = f.state().stage.id_of(&b).unwrap();
    assert_eq!(f.state().stage.active_tile(), Some(id));
    let c = map_window(&mut f, client, "c", (400, 200));
    assert_eq!(f.state().stage.tiled_columns().len(), 2);
    assert!(last_configured(&mut f, client, &c).1 > 700);
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn a_delayed_vertical_resize_reserves_live_height_without_repeated_requests() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 700));
    let b = map_window(&mut f, client, "b", (400, 700));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::MoveIntoColumn(Direction::Left));
    f.double_roundtrip(client);
    ack_but_keep_size(&mut f, client, &a);
    ack_but_keep_size(&mut f, client, &b);
    assert!(window_position(&mut f, &wb).y >= window_position(&mut f, &wa).y + 700);
    let count = configure_count(&mut f, client, &a);
    for _ in 0..5 {
        f.state().refresh_and_flush_clients();
    }
    assert_eq!(configure_count(&mut f, client, &a), count);
    let (width, height) = last_configured(&mut f, client, &a);
    let window = f.client(client).window(&a);
    window.set_size(width as u16, height as u16);
    window.attach_new_buffer();
    window.commit();
    f.double_roundtrip(client);
    let (width, height) = last_configured(&mut f, client, &b);
    let window = f.client(client).window(&b);
    window.set_size(width as u16, height as u16);
    window.attach_new_buffer();
    window.commit();
    f.double_roundtrip(client);
    assert!(window_position(&mut f, &wb).y < window_position(&mut f, &wa).y + 700);
}

#[test]
fn removing_a_left_column_preserves_the_active_column_screen_position_and_flight() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let b = map_window(&mut f, client, "b", (400, 200));
    let c = map_window(&mut f, client, "c", (400, 200));
    let wc = window_by_app_id(&mut f, "c").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    for surface in [&a, &b, &c] {
        adopt_last_configure(&mut f, client, surface);
    }
    f.state().cancel_animations_everywhere();
    let column = f.state().stage.active_column().unwrap();
    let before = window_position(&mut f, &wc);
    f.state()
        .set_camera((f64::from(before.x) - 100.0, 0.0).into());
    let camera = f.state().camera();
    let target = camera + smithay::utils::Point::from((40.0, 0.0));
    f.state().set_camera_target(Some(target));
    let pointer_before = f.state().seat.get_pointer().unwrap().current_location();
    f.client(client).window(&a).destroy();
    f.double_roundtrip(client);
    let after = window_position(&mut f, &wc);
    let delta = f64::from(after.x) - f64::from(before.x);
    assert!(delta < 0.0);
    assert_eq!(f.state().stage.active_column(), Some(column));
    assert_eq!(f.state().camera().x, camera.x + delta);
    assert_eq!(f.state().camera_target().unwrap().x, target.x + delta);
    assert_eq!(
        f.state().seat.get_pointer().unwrap().current_location().x,
        pointer_before.x + delta
    );
    f.state().refresh_tiling();
    assert_eq!(f.state().camera().x, camera.x + delta);
}

#[test]
fn reorder_actions_keep_the_focused_window_and_column_identity() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 200));
    map_window(&mut f, client, "b", (400, 200));
    map_window(&mut f, client, "c", (400, 200));
    let wc = window_by_app_id(&mut f, "c").unwrap();
    let id = f.state().stage.id_of(&wc).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let column = f.state().stage.column_of(id).unwrap();
    f.state()
        .execute_action(&Action::MoveColumn(Direction::Left));
    assert_eq!(f.state().stage.tiled_columns()[1].id, column);
    assert_eq!(f.state().stage.active_tile(), Some(id));
    f.state()
        .execute_action(&Action::MoveIntoColumn(Direction::Right));
    let target = f.state().stage.active_column().unwrap();
    f.state()
        .execute_action(&Action::MoveTiledWindow(Direction::Up));
    assert_eq!(f.state().stage.column(target).unwrap().windows[0].id, id);
    f.state()
        .execute_action(&Action::MoveTiledWindow(Direction::Down));
    assert_eq!(f.state().stage.column(target).unwrap().windows[1].id, id);
    assert_eq!(f.state().stage.active_tile(), Some(id));
    let focused = f.state().focused_element().unwrap();
    assert_eq!(f.state().stage.id_of(&focused), Some(id));
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn reference_consume_and_expel_keep_keyboard_focus_in_the_source_column() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let b = map_window(&mut f, client, "b", (400, 200));
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let aid = f.state().stage.id_of(&wa).unwrap();
    let bid = f.state().stage.id_of(&wb).unwrap();
    let column = f.state().stage.active_column();
    f.state().execute_action(&Action::ConsumeWindowIntoColumn);
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    assert_eq!(f.state().stage.active_tile(), Some(aid));
    assert_eq!(f.state().stage.active_column(), column);
    let size_a = last_configured(&mut f, client, &a);
    let size_b = last_configured(&mut f, client, &b);
    assert_eq!(size_a, size_b);
    assert!(size_a.1 < 800);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Down));
    assert_eq!(f.state().stage.active_tile(), Some(bid));
    f.state().execute_action(&Action::ExpelWindowFromColumn);
    assert_eq!(f.state().stage.tiled_columns().len(), 2);
    assert_eq!(f.state().stage.active_column(), column);
    assert_eq!(f.state().stage.active_tile(), Some(aid));
    let focused = f.state().focused_element().unwrap();
    assert_eq!(f.state().stage.id_of(&focused), Some(aid));
    assert!(last_configured(&mut f, client, &a).1 > size_a.1);
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn empty_tiling_keeps_its_layout_identity_until_toggle_off() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    f.state().execute_action(&Action::ToggleTiling);
    let first = f.state().stage.tiling_layout_id().unwrap();
    let a = map_window(&mut f, client, "a", (400, 200));
    assert_eq!(f.state().stage.tiling_layout_id(), Some(first));
    f.client(client).window(&a).destroy();
    f.double_roundtrip(client);
    assert!(f.state().stage.tiled_columns().is_empty());
    assert!(f.state().tiling_view.is_some());
    assert_eq!(f.state().stage.tiling_layout_id(), Some(first));
    map_window(&mut f, client, "b", (400, 200));
    assert_eq!(f.state().stage.tiling_layout_id(), Some(first));
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(f.state().stage.tiling_layout_id(), None);
    f.state().execute_action(&Action::ToggleTiling);
    assert_ne!(f.state().stage.tiling_layout_id(), Some(first));
}

#[test]
fn grouped_configures_follow_reference_sequential_height_rounding() {
    let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1200, 604));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 100));
    let b = map_window(&mut f, client, "b", (400, 100));
    let c = map_window(&mut f, client, "c", (400, 100));
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    f.state().execute_action(&Action::ConsumeWindowIntoColumn);
    f.state().execute_action(&Action::ConsumeWindowIntoColumn);
    assert_eq!(last_configured(&mut f, client, &a), (600, 201));
    assert_eq!(last_configured(&mut f, client, &b), (600, 202));
    assert_eq!(last_configured(&mut f, client, &c), (600, 201));
    for surface in [&a, &b, &c] {
        adopt_last_configure(&mut f, client, surface);
    }
    let first = window_by_app_id(&mut f, "a").unwrap();
    let origin = window_position(&mut f, &first);
    for (name, y) in [("a", 0), ("b", 201), ("c", 403)] {
        let window = window_by_app_id(&mut f, name).unwrap();
        assert_eq!(
            window_position(&mut f, &window),
            origin + smithay::utils::Point::from((0, y))
        );
    }
}

#[test]
fn closing_selected_middle_tile_or_column_focuses_the_next_window() {
    for grouped in [false, true] {
        let mut f = Fixture::new();
        f.add_output(1, (1200, 800));
        let client = f.add_client();
        map_window(&mut f, client, "a", (400, 200));
        let b = map_window(&mut f, client, "b", (400, 200));
        map_window(&mut f, client, "c", (400, 200));
        let wc = window_by_app_id(&mut f, "c").unwrap();
        let cid = f.state().stage.id_of(&wc).unwrap();
        f.state().execute_action(&Action::ToggleTiling);
        f.state()
            .execute_action(&Action::FocusColumn(Direction::Left));
        if grouped {
            f.state()
                .execute_action(&Action::FocusColumn(Direction::Left));
            f.state().execute_action(&Action::ConsumeWindowIntoColumn);
            f.state().execute_action(&Action::ConsumeWindowIntoColumn);
            f.state()
                .execute_action(&Action::FocusColumn(Direction::Down));
        }
        f.client(client).window(&b).destroy();
        f.double_roundtrip(client);
        assert_eq!(f.state().stage.active_tile(), Some(cid));
        let focused = f.state().focused_element().unwrap();
        assert_eq!(f.state().stage.id_of(&focused), Some(cid));
        assert!(f.state().tiling_view.is_some());
    }
}

#[test]
fn closing_a_new_interior_column_returns_until_another_column_is_focused() {
    for change_focus in [false, true] {
        let mut f = Fixture::new();
        f.add_output(1, (1200, 800));
        let client = f.add_client();
        map_window(&mut f, client, "a", (400, 200));
        map_window(&mut f, client, "b", (400, 200));
        let wa = window_by_app_id(&mut f, "a").unwrap();
        let wb = window_by_app_id(&mut f, "b").unwrap();
        f.state().execute_action(&Action::ToggleTiling);
        f.state()
            .execute_action(&Action::FocusColumn(Direction::Left));
        let c = map_window(&mut f, client, "c", (400, 200));
        if change_focus {
            f.state()
                .execute_action(&Action::FocusColumn(Direction::Right));
            f.state()
                .execute_action(&Action::FocusColumn(Direction::Left));
        }
        f.client(client).window(&c).destroy();
        f.double_roundtrip(client);
        let expected = if change_focus { &wb } else { &wa };
        let id = f.state().stage.id_of(expected).unwrap();
        assert_eq!(f.state().stage.active_tile(), Some(id));
        let focused = f.state().focused_element().unwrap();
        assert_eq!(f.state().stage.id_of(&focused), Some(id));
    }
}

#[test]
fn crashing_the_selected_middle_column_recovers_structural_focus() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let survivor = f.add_client();
    let crashing = f.add_client();
    map_window(&mut f, survivor, "a", (400, 200));
    map_window(&mut f, crashing, "b", (400, 200));
    map_window(&mut f, survivor, "c", (400, 200));
    let wc = window_by_app_id(&mut f, "c").unwrap();
    let cid = f.state().stage.id_of(&wc).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    f.kill_client(crashing);
    f.double_roundtrip(survivor);
    assert_eq!(f.state().stage.active_tile(), Some(cid));
    let focused = f.state().focused_element().unwrap();
    assert_eq!(f.state().stage.id_of(&focused), Some(cid));
}

#[test]
fn a_background_close_and_an_intentional_focus_clear_do_not_steal_focus() {
    for clear_focus in [false, true] {
        let mut f = Fixture::new();
        f.add_output(1, (1200, 800));
        let client = f.add_client();
        let a = map_window(&mut f, client, "a", (400, 200));
        let b = map_window(&mut f, client, "b", (400, 200));
        let wb = window_by_app_id(&mut f, "b").unwrap();
        let bid = f.state().stage.id_of(&wb).unwrap();
        f.state().execute_action(&Action::ToggleTiling);
        if clear_focus {
            f.state()
                .clear_focus_to_empty_canvas(SERIAL_COUNTER.next_serial());
            f.client(client).window(&b).destroy();
        } else {
            f.client(client).window(&a).destroy();
        }
        f.double_roundtrip(client);
        if clear_focus {
            assert!(f.state().focused_element().is_none());
        } else {
            let focused = f.state().focused_element().unwrap();
            assert_eq!(f.state().stage.id_of(&focused), Some(bid));
        }
    }
}

#[test]
fn closing_a_new_column_restores_the_stationary_view_target_once() {
    for (resized, crash) in [(false, false), (true, false), (false, true)] {
        let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
        let mut f = Fixture::with_config(config);
        f.add_output(1, (1200, 800));
        let client = f.add_client();
        let a = map_window(&mut f, client, "a", (400, 200));
        let b = map_window(&mut f, client, "b", (400, 200));
        let wa = window_by_app_id(&mut f, "a").unwrap();
        f.state().execute_action(&Action::ToggleTiling);
        f.double_roundtrip(client);
        for surface in [&a, &b] {
            adopt_last_configure(&mut f, client, surface);
        }
        f.state()
            .execute_action(&Action::FocusColumn(Direction::Left));
        let x = f64::from(window_position(&mut f, &wa).x);
        let y = f.state().camera().y;
        f.state().set_camera((x - 200.0, y).into());
        f.state().set_camera_target(Some((x - 100.0, y).into()));
        let opening = f.add_client();
        let c = map_window(&mut f, opening, "c", (400, 200));
        if resized {
            f.client(client).window(&a).set_min_size(1150, 0);
            f.client(client).window(&a).commit();
            f.double_roundtrip(client);
        }
        let in_flight = (x + 50.0, y).into();
        f.state().set_camera(in_flight);
        f.state().set_camera_target(Some((x + 600.0, y).into()));
        if crash {
            f.kill_client(opening);
            f.double_roundtrip(client);
        } else {
            f.client(opening).window(&c).destroy();
            f.double_roundtrip(opening);
        }
        if resized {
            last_configured(&mut f, client, &a);
            adopt_last_configure(&mut f, client, &a);
        }
        assert_eq!(f.state().stage.tiled_columns().len(), 2);
        assert_eq!(f.state().camera(), in_flight);
        let expected = if resized { x - 50.0 } else { x - 100.0 };
        assert_eq!(
            f.state().camera_target().unwrap().x,
            expected,
            "resized={resized}, crash={crash}"
        );
        for _ in 0..3 {
            f.state().refresh_and_flush_clients();
        }
        assert_eq!(
            f.state().camera_target().unwrap().x,
            expected,
            "resized={resized}, crash={crash}"
        );
        assert_eq!(f.state().camera(), in_flight);
    }
}

#[test]
fn switching_columns_cancels_the_saved_return_view() {
    let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let b = map_window(&mut f, client, "b", (400, 200));
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    for surface in [&a, &b] {
        adopt_last_configure(&mut f, client, surface);
    }
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    let camera = f.state().camera();
    f.state()
        .set_camera_target(Some(camera + smithay::utils::Point::from((-100.0, 0.0))));
    let c = map_window(&mut f, client, "c", (400, 200));
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Right));
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    f.state().set_camera(camera);
    f.client(client).window(&c).destroy();
    f.double_roundtrip(client);
    assert_eq!(
        f.state().camera_target(),
        Some(camera.to_i32_round::<i32>().to_f64())
    );
}

#[test]
fn tiling_ack_and_empty_commit_do_not_complete_a_resize_without_matching_geometry() {
    use driftline::layout::scrolling::transaction::ConfigureProgress;
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let id = f.state().stage.id_of(&wa).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let size = last_configured(&mut f, client, &a);
    assert_eq!(
        f.state().tiling_configure_progress(id),
        Some(ConfigureProgress::Pending)
    );
    f.client(client).window(&a).ack_last();
    f.double_roundtrip(client);
    assert_eq!(
        f.state().tiling_configure_progress(id),
        Some(ConfigureProgress::Pending)
    );
    f.client(client).window(&a).commit();
    f.double_roundtrip(client);
    assert_eq!(
        f.state().tiling_configure_progress(id),
        Some(ConfigureProgress::Pending)
    );
    f.client(client)
        .window(&a)
        .set_size(size.0 as u16, size.1 as u16);
    f.client(client).window(&a).attach_new_buffer();
    f.client(client).window(&a).commit();
    f.double_roundtrip(client);
    assert_eq!(
        f.state().tiling_configure_progress(id),
        Some(ConfigureProgress::Applied)
    );
    assert_eq!(f.state().tiling_pending_configures(), 0);
}

#[test]
fn an_old_tiling_reply_cannot_complete_the_latest_size_or_survive_mode_exit() {
    use driftline::layout::scrolling::transaction::ConfigureProgress;
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let id = f.state().stage.id_of(&wa).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let first = last_configured(&mut f, client, &a);
    let first_serial = f
        .client(client)
        .window(&a)
        .configures_received
        .last()
        .unwrap()
        .0;
    f.state().execute_action(&Action::CycleColumnWidth);
    let latest = last_configured(&mut f, client, &a);
    assert_ne!(first, latest);
    {
        let window = f.client(client).window(&a);
        window.xdg_surface.ack_configure(first_serial);
        window.set_size(first.0 as u16, first.1 as u16);
        window.attach_new_buffer();
        window.commit();
    }
    f.double_roundtrip(client);
    assert_eq!(
        f.state().tiling_configure_progress(id),
        Some(ConfigureProgress::Pending)
    );
    let count = configure_count(&mut f, client, &a);
    f.state().refresh_and_flush_clients();
    assert_eq!(configure_count(&mut f, client, &a), count);
    adopt_last_configure(&mut f, client, &a);
    assert_eq!(
        f.state().tiling_configure_progress(id),
        Some(ConfigureProgress::Applied)
    );
    f.state().execute_action(&Action::CycleColumnWidth);
    let old = last_configured(&mut f, client, &a);
    let serial = f
        .client(client)
        .window(&a)
        .configures_received
        .last()
        .unwrap()
        .0;
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    {
        let window = f.client(client).window(&a);
        window.xdg_surface.ack_configure(serial);
        window.set_size(old.0 as u16, old.1 as u16);
        window.attach_new_buffer();
        window.commit();
    }
    f.double_roundtrip(client);
    assert!(f.state().tiling_view.is_none());
    assert_eq!(f.state().tiling_configure_progress(id), None);
}

#[test]
fn new_tiled_windows_receive_their_content_size_in_the_first_configure() {
    use driftline::layout::scrolling::transaction::ConfigureProgress;
    for (decorations, expected) in [
        ("client", (600, 800)),
        ("server", (596, 771)),
        ("none", (600, 800)),
    ] {
        let default = if decorations == "server" {
            "client"
        } else {
            decorations
        };
        let rule = if decorations == "server" {
            "[[window_rules]]\napp_id = \"initial-size\"\ndecoration = \"server\"\n"
        } else {
            ""
        };
        let config = driftline::config::Config::from_toml(&format!(
            "[snap]\ngap = 0\n[decorations]\ndefault_mode = \"{default}\"\nborder_width = {}\ntitle_bar_height = 25\n{rule}",
            if decorations == "server" { 2 } else { 0 },
        )).unwrap();
        let mut f = Fixture::with_config(config);
        f.add_output(1, (1200, 800));
        let client = f.add_client();
        f.state().execute_action(&Action::ToggleTiling);
        let window = f.client(client).create_window();
        let surface = window.surface.clone();
        window.set_app_id("initial-size");
        window.commit();
        f.double_roundtrip(client);
        assert_eq!(last_configured(&mut f, client, &surface), expected);
        assert_eq!(configure_count(&mut f, client, &surface), 1);
        assert!(f.state().stage.tiled_columns().is_empty());
        assert!(f.state().focused_element().is_none());
        assert_eq!(f.state().tiling_pending_configures(), 1);
        adopt_last_configure(&mut f, client, &surface);
        let window = window_by_app_id(&mut f, "initial-size").unwrap();
        let id = f.state().stage.id_of(&window).unwrap();
        assert!(f.state().stage.is_tiled(id));
        assert_eq!(
            f.state().tiling_configure_progress(id),
            Some(ConfigureProgress::Applied)
        );
        assert!(
            f.client(client)
                .window(&surface)
                .configures_received
                .iter()
                .all(|(_, c)| c.size == expected)
        );
    }
}

#[test]
fn initial_tiling_configure_respects_client_size_limits() {
    let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    f.state().execute_action(&Action::ToggleTiling);
    let window = f.client(client).create_window();
    let surface = window.surface.clone();
    window.set_app_id("limited-initial");
    window.set_min_size(750, 0);
    window.set_max_size(0, 500);
    window.commit();
    f.double_roundtrip(client);
    assert_eq!(last_configured(&mut f, client, &surface), (750, 500));
    adopt_last_configure(&mut f, client, &surface);
    assert_eq!(last_configured(&mut f, client, &surface), (750, 500));
}

#[test]
fn exiting_before_the_first_buffer_releases_the_initial_tiling_size_offer() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    f.state().execute_action(&Action::ToggleTiling);
    let window = f.client(client).create_window();
    let surface = window.surface.clone();
    window.set_app_id("late-first-buffer");
    window.commit();
    f.double_roundtrip(client);
    assert!(last_configured(&mut f, client, &surface).0 > 0);
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(last_configured(&mut f, client, &surface), (0, 0));
    assert_eq!(f.state().tiling_pending_configures(), 0);
    let window = f.client(client).window(&surface);
    window.set_size(400, 200);
    window.attach_new_buffer();
    window.ack_last_and_commit();
    f.double_roundtrip(client);
    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.tiled_columns().is_empty());
    assert_eq!(last_configured(&mut f, client, &surface), (400, 200));
}

#[test]
fn explicit_window_rules_keep_their_initial_sizing_policy_in_tiling_mode() {
    for (rule, expected) in [
        ("widget = true", (0, 0)),
        ("size = [400, 250]", (400, 250)),
        ("pinned_to_screen = true", (0, 0)),
    ] {
        let config = driftline::config::Config::from_toml(&format!(
            "[snap]\ngap = 0\n[[window_rules]]\napp_id = \"explicit-initial\"\n{rule}\n"
        ))
        .unwrap();
        let mut f = Fixture::with_config(config);
        f.add_output(1, (1200, 800));
        let client = f.add_client();
        f.state().execute_action(&Action::ToggleTiling);
        let window = f.client(client).create_window();
        let surface = window.surface.clone();
        window.set_app_id("explicit-initial");
        window.commit();
        f.double_roundtrip(client);
        assert_eq!(
            last_configured(&mut f, client, &surface),
            expected,
            "{rule}"
        );
        assert_eq!(f.state().tiling_pending_configures(), 0);
    }
}

#[test]
fn a_refused_initial_offer_keeps_the_first_buffer_size_for_canvas_restore() {
    let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    f.state().execute_action(&Action::ToggleTiling);
    let surface = map_window(&mut f, client, "preferred-size", (400, 200));
    assert_eq!(last_configured(&mut f, client, &surface), (600, 800));
    f.state().execute_action(&Action::ToggleTiling);
    assert_eq!(last_configured(&mut f, client, &surface), (400, 200));
}

#[test]
fn becoming_a_dialog_after_initial_configure_retires_the_early_tiling_request() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let parent = map_window(&mut f, client, "parent", (400, 200));
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    adopt_last_configure(&mut f, client, &parent);
    let parent_role = f.client(client).window(&parent).xdg_toplevel.clone();
    let window = f.client(client).create_window();
    let surface = window.surface.clone();
    window.set_app_id("late-dialog");
    window.commit();
    f.double_roundtrip(client);
    assert_eq!(f.state().tiling_pending_configures(), 1);
    f.client(client)
        .window(&surface)
        .set_parent(Some(&parent_role));
    adopt_last_configure(&mut f, client, &surface);
    let window = window_by_app_id(&mut f, "late-dialog").unwrap();
    let id = f.state().stage.id_of(&window).unwrap();
    assert!(!f.state().stage.is_tiled(id));
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert!(f.state().tiling_view.is_some());
}

#[test]
fn reference_mapping_order_follows_first_buffers_without_reserving_empty_columns() {
    let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    adopt_last_configure(&mut f, client, &a);
    let origin = window_position(&mut f, &wa);
    let mut surfaces = Vec::new();
    for name in ["b", "c"] {
        let window = f.client(client).create_window();
        surfaces.push(window.surface.clone());
        window.set_app_id(name);
        window.commit();
        f.double_roundtrip(client);
    }
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    let focused = f.state().focused_element().unwrap();
    assert_eq!(f.state().stage.id_of(&focused), f.state().stage.id_of(&wa));
    adopt_last_configure(&mut f, client, &surfaces[1]);
    let wc = window_by_app_id(&mut f, "c").unwrap();
    let cid = f.state().stage.id_of(&wc).unwrap();
    let column = f.state().stage.column_of(cid).unwrap();
    let before = window_position(&mut f, &wc);
    assert_eq!(before.x, origin.x + 600);
    adopt_last_configure(&mut f, client, &surfaces[0]);
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let bid = f.state().stage.id_of(&wb).unwrap();
    let order = [&wa, &wc, &wb].map(|w| f.state().stage.id_of(w).unwrap());
    assert_eq!(
        f.state()
            .stage
            .tiled_columns()
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        order
    );
    assert_eq!(f.state().stage.column_of(cid), Some(column));
    assert_eq!(window_position(&mut f, &wc), before);
    assert_eq!(window_position(&mut f, &wb).x, origin.x + 1200);
    assert_eq!(f.state().stage.active_tile(), Some(bid));
}

#[test]
fn a_delayed_first_buffer_uses_the_column_focused_when_it_maps() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 200));
    map_window(&mut f, client, "b", (400, 200));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Left));
    let window = f.client(client).create_window();
    let surface = window.surface.clone();
    window.set_app_id("delayed");
    window.commit();
    f.double_roundtrip(client);
    f.state()
        .execute_action(&Action::FocusColumn(Direction::Right));
    adopt_last_configure(&mut f, client, &surface);
    let wd = window_by_app_id(&mut f, "delayed").unwrap();
    let order = [&wa, &wb, &wd].map(|w| f.state().stage.id_of(w).unwrap());
    assert_eq!(
        f.state()
            .stage
            .tiled_columns()
            .iter()
            .map(|c| c.selected)
            .collect::<Vec<_>>(),
        order
    );
}

#[test]
fn closing_an_unbuffered_window_does_not_change_tiling_focus_or_camera() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    adopt_last_configure(&mut f, client, &a);
    let active = f.state().stage.active_tile();
    let camera = f.state().camera();
    let target = f.state().camera_target();
    let window = f.client(client).create_window();
    let surface = window.surface.clone();
    window.commit();
    f.double_roundtrip(client);
    f.client(client).window(&surface).destroy();
    f.double_roundtrip(client);
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    assert_eq!(f.state().stage.active_tile(), active);
    assert_eq!(f.state().camera(), camera);
    assert_eq!(f.state().camera_target(), target);
    assert_eq!(f.state().tiling_pending_configures(), 0);
}

#[test]
fn tiling_buffer_commits_wait_for_other_clients_in_the_same_group() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "a", (400, 300));
    let b = map_window(&mut f, second, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let positions = [window_position(&mut f, &wa), window_position(&mut f, &wb)];
    let camera = f.state().camera();
    f.state().execute_action(&Action::ToggleTiling);
    let target = f.state().camera_target();
    let target_a = last_configured(&mut f, first, &a);
    let target_b = last_configured(&mut f, second, &b);
    assert_eq!(window_position(&mut f, &wa), positions[0]);
    assert_eq!(window_position(&mut f, &wb), positions[1]);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(window_position(&mut f, &wa), positions[0]);
    assert_eq!(window_position(&mut f, &wb), positions[1]);
    assert_eq!(f.state().camera(), camera);
    assert_eq!(f.state().camera_target(), target);
    assert_eq!(wa.geometry().size, (400, 300).into());
    assert_eq!(wb.geometry().size, (350, 250).into());
    adopt_last_configure(&mut f, second, &b);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target_a.into());
    assert_eq!(wb.geometry().size, target_b.into());
    assert!(window_position(&mut f, &wb).x > window_position(&mut f, &wa).x);
    assert_eq!(
        window_position(&mut f, &wb).y,
        window_position(&mut f, &wa).y
    );
}

#[test]
fn exiting_tiling_releases_a_buffer_waiting_for_a_stalled_peer() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "a", (400, 300));
    let b = map_window(&mut f, second, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(wa.geometry().size, (400, 300).into());
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(first);
    assert!(f.state().tiling_view.is_none());
    assert_eq!(wa.geometry().size, target.into());
}

#[test]
fn disconnecting_a_group_participant_releases_the_ready_peer() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "a", (400, 300));
    let b = map_window(&mut f, second, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(wa.geometry().size, (400, 300).into());
    f.kill_client(second);
    f.double_roundtrip(first);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target.into());
}

#[test]
fn a_group_deadline_releases_ready_buffers_without_a_peer_commit() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "a", (400, 300));
    let b = map_window(&mut f, second, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let before = window_position(&mut f, &wa);
    f.state().execute_action(&Action::ToggleTiling);
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(wa.geometry().size, (400, 300).into());
    assert_eq!(window_position(&mut f, &wa), before);
    std::thread::sleep(std::time::Duration::from_millis(310));
    f.pump(4);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target.into());
    assert_ne!(window_position(&mut f, &wa), before);
}

#[test]
fn a_client_chosen_buffer_size_does_not_stall_its_group() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "a", (400, 300));
    let b = map_window(&mut f, second, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(wa.geometry().size, (400, 300).into());
    ack_but_keep_size(&mut f, second, &b);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target.into());
    assert_eq!(wb.geometry().size, (350, 250).into());
}

#[test]
fn superseding_a_group_releases_its_queued_buffer_and_keeps_pending_peers() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "a", (400, 300));
    let b = map_window(&mut f, second, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let before = window_position(&mut f, &wa);
    f.state().execute_action(&Action::ToggleTiling);
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(wa.geometry().size, (400, 300).into());
    f.state().execute_action(&Action::CycleColumnWidth);
    let next_b = last_configured(&mut f, second, &b);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target.into());
    assert_eq!(window_position(&mut f, &wa), before);
    adopt_last_configure(&mut f, second, &b);
    assert_ne!(window_position(&mut f, &wa), before);
    assert_eq!(
        window_by_app_id(&mut f, "b").unwrap().geometry().size,
        next_b.into()
    );
}

#[test]
fn a_pending_merge_preserves_positions_until_both_buffers_arrive() {
    let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let b = map_window(&mut f, client, "b", (400, 200));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    adopt_last_configure(&mut f, client, &a);
    adopt_last_configure(&mut f, client, &b);
    let before = [window_position(&mut f, &wa), window_position(&mut f, &wb)];
    let target = f.state().camera_target();
    f.state()
        .execute_action(&Action::MoveIntoColumn(Direction::Left));
    assert_eq!(f.state().stage.tiled_columns().len(), 1);
    last_configured(&mut f, client, &a);
    adopt_last_configure(&mut f, client, &a);
    assert_eq!(window_position(&mut f, &wa), before[0]);
    assert_eq!(window_position(&mut f, &wb), before[1]);
    assert_eq!(f.state().camera_target(), target);
    adopt_last_configure(&mut f, client, &b);
    let after_a = window_position(&mut f, &wa);
    let after_b = window_position(&mut f, &wb);
    assert_eq!(after_b.x, after_a.x);
    assert_eq!(after_b.y, after_a.y + 400);
    assert_eq!(wa.geometry().size, (600, 400).into());
    assert_eq!(wb.geometry().size, (600, 400).into());
}

#[test]
fn tiling_anchor_translation_waits_for_other_surface_blockers_too() {
    use smithay::reexports::wayland_server::Resource;
    use smithay::wayland::compositor::{Barrier, add_blocker, add_pre_commit_hook};
    use smithay::wayland::seat::WaylandFocus;

    let config = driftline::config::Config::from_toml("[snap]\ngap = 0\n").unwrap();
    let mut f = Fixture::with_config(config);
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    let b = map_window(&mut f, client, "b", (400, 200));
    let c = map_window(&mut f, client, "c", (400, 200));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wc = window_by_app_id(&mut f, "c").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    for surface in [&a, &b, &c] {
        adopt_last_configure(&mut f, client, surface);
    }
    let before = window_position(&mut f, &wc);
    f.state().cancel_animations_everywhere();
    let camera = (f64::from(before.x) - 100.0, 0.0).into();
    f.state().set_camera(camera);
    let target = camera + smithay::utils::Point::from((40.0, 0.0));
    f.state().set_camera_target(Some(target));
    let pointer = f.state().seat.get_pointer().unwrap().current_location();
    f.client(client).window(&a).set_min_size(900, 0);
    f.client(client).window(&a).commit();
    assert_eq!(last_configured(&mut f, client, &a), (900, 800));
    f.client(client).window(&a).ack_last_and_commit();
    f.double_roundtrip(client);
    let surface = wa.wl_surface().unwrap().into_owned();
    let barrier = Barrier::new(false);
    let blocker = barrier.clone();
    let hook = add_pre_commit_hook::<crate::state::Driftline, _>(&surface, move |_, _, s| {
        add_blocker(s, blocker.clone());
    });
    let window = f.client(client).window(&a);
    window.set_size(900, 800);
    window.attach_new_buffer();
    window.commit();
    f.double_roundtrip(client);
    assert_eq!(wa.geometry().size, (600, 800).into());
    assert_eq!(window_position(&mut f, &wc), before);
    assert_eq!(f.state().camera(), camera);
    assert_eq!(f.state().camera_target(), Some(target));
    assert_eq!(
        f.state().seat.get_pointer().unwrap().current_location(),
        pointer
    );
    f.client(client).window(&b).set_min_size(700, 0);
    f.client(client).window(&b).commit();
    assert_eq!(last_configured(&mut f, client, &b), (700, 800));
    adopt_last_configure(&mut f, client, &b);
    assert_eq!(window_position(&mut f, &wc), before);
    assert_eq!(f.state().camera(), camera);
    assert_eq!(f.state().camera_target(), Some(target));
    barrier.signal();
    let owner = surface.client().unwrap();
    let client_state = owner.get_data::<crate::state::ClientState>().unwrap();
    let dh = f.state().display_handle.clone();
    client_state
        .compositor_state
        .blocker_cleared(f.state(), &dh);
    f.double_roundtrip(client);
    let after = window_position(&mut f, &wc);
    assert_eq!(after.x - before.x, 400);
    assert_eq!(wa.geometry().size, (900, 800).into());
    assert_eq!(f.state().camera().x, camera.x + 400.0);
    assert_eq!(f.state().camera_target().unwrap().x, target.x + 400.0);
    assert_eq!(
        f.state().seat.get_pointer().unwrap().current_location(),
        pointer + smithay::utils::Point::from((400.0, 0.0))
    );
    smithay::wayland::compositor::remove_pre_commit_hook(&surface, &hook);
}

#[test]
fn protocol_unmap_releases_a_ready_peer_and_remap_waits_for_a_buffer() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "ready", (400, 300));
    let b = map_window(&mut f, second, "hidden", (350, 250));
    let wa = window_by_app_id(&mut f, "ready").unwrap();
    let wb = window_by_app_id(&mut f, "hidden").unwrap();
    let aid = f.state().stage.id_of(&wa).unwrap();
    let bid = f.state().stage.id_of(&wb).unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(wa.geometry().size, (400, 300).into());
    let w = f.client(second).window(&b);
    w.attach_null();
    w.commit();
    f.double_roundtrip(second);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target.into());
    assert!(f.state().tiling_view.is_some());
    assert!(!f.state().stage.is_tiled(bid));
    assert_eq!(f.state().stage.active_tile(), Some(aid));
    assert_eq!(f.state().tiling_pending_configures(), 0);
    f.client(second).window(&b).commit();
    f.double_roundtrip(second);
    assert!(!f.state().stage.is_tiled(bid));
    let size = last_configured(&mut f, second, &b);
    assert!(size.0 > 350 && size.1 > 250);
    adopt_last_configure(&mut f, second, &b);
    assert_eq!(f.state().stage.id_of(&wb), Some(bid));
    assert!(f.state().stage.is_tiled(bid));
    assert_eq!(f.state().stage.tiled_columns().len(), 2);
}

#[test]
fn removing_the_last_output_releases_ready_buffers_and_rejects_placeholder_tiling() {
    let mut f = Fixture::new();
    let output = f.add_output(1, (1200, 800));
    let first = f.add_client();
    let second = f.add_client();
    let a = map_window(&mut f, first, "a", (400, 300));
    let b = map_window(&mut f, second, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    let ids = [f.state().stage.id_of(&wa), f.state().stage.id_of(&wb)];
    f.state().execute_action(&Action::ToggleTiling);
    let target = last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    assert_eq!(wa.geometry().size, (400, 300).into());
    f.remove_output(&output);
    f.double_roundtrip(first);
    assert_eq!(wa.geometry().size, target.into());
    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.tiled_columns().is_empty());
    assert_eq!(f.state().tiling_pending_configures(), 0);
    f.state().execute_action(&Action::ToggleTiling);
    assert!(f.state().tiling_view.is_none());
    assert_eq!(last_configured(&mut f, first, &a), (400, 300));
    assert_eq!(last_configured(&mut f, second, &b), (350, 250));
    adopt_last_configure(&mut f, first, &a);
    adopt_last_configure(&mut f, second, &b);
    let reconnected = f.add_output(1, (1200, 800));
    assert_ne!(output, reconnected);
    assert!(f.state().disconnected_outputs.is_empty());
    f.state().execute_action(&Action::ToggleTiling);
    assert!(f.state().tiling_view.is_some());
    last_configured(&mut f, first, &a);
    last_configured(&mut f, second, &b);
    adopt_last_configure(&mut f, first, &a);
    adopt_last_configure(&mut f, second, &b);
    f.double_roundtrip(first);
    assert_eq!(f.state().tiling_pending_configures(), 0);
    assert_eq!(
        [f.state().stage.id_of(&wa), f.state().stage.id_of(&wb)],
        ids
    );
}

#[test]
fn fullscreen_on_another_output_prevents_entering_tiling() {
    let mut f = Fixture::new();
    let active = f.add_output(1, (1200, 800));
    let fullscreen = f.add_output(2, (1280, 720));
    let client = f.add_client();
    let _surface = map_window(&mut f, client, "fullscreen", (400, 300));
    let window = window_by_app_id(&mut f, "fullscreen").unwrap();
    assert_eq!(f.state().active_output().as_ref(), Some(&active));

    f.state()
        .stage
        .set_fullscreen(&fullscreen.name(), window, (0, 0).into(), (400, 300).into());
    f.state().execute_action(&Action::ToggleTiling);

    assert!(f.state().tiling_view.is_none());
    assert!(f.state().stage.tiled_columns().is_empty());
    assert!(f.state().stage.has_fullscreen());
}
