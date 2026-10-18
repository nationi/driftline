use driftwm::config::{Action, Direction};
use smithay::utils::SERIAL_COUNTER;

use super::{
    Fixture, ack_but_keep_size, adopt_last_configure, configure_count, last_configured, map_window,
    window_by_app_id, window_position,
};

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
    map_window(&mut f, client, "b", (350, 250));
    let wa = window_by_app_id(&mut f, "a").unwrap();
    let wb = window_by_app_id(&mut f, "b").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
    f.double_roundtrip(client);
    ack_but_keep_size(&mut f, client, &a);
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
    f.state().execute_action(&Action::CycleColumnWidth);
    assert!(last_configured(&mut f, client, &a).0 > half);
    f.state()
        .execute_action(&Action::CenterNearest(Direction::Right));
    assert_ne!(f.state().stage.active_tile(), Some(aid));
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
fn a_second_output_leaves_tiling_and_multiple_outputs_cannot_enter_it() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let surface = map_window(&mut f, client, "a", (400, 300));
    f.state().execute_action(&Action::ToggleTiling);
    f.add_output(2, (1920, 1080));
    f.state().refresh_and_flush_clients();
    assert!(f.state().tiling_view.is_none());
    assert_eq!(last_configured(&mut f, client, &surface), (400, 300));
    f.state().execute_action(&Action::ToggleTiling);
    assert!(f.state().tiling_view.is_none());
}

#[test]
fn incompatible_actions_leave_tiling_before_applying_canvas_policy() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    map_window(&mut f, client, "a", (400, 300));
    f.state().execute_action(&Action::ToggleTiling);
    f.state().execute_action(&Action::FitWindow);
    assert!(f.state().tiling_view.is_none());
    let window = window_by_app_id(&mut f, "a").unwrap();
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
        .execute_action(&Action::CenterNearest(Direction::Down));
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
    adopt_last_configure(&mut f, client, &b);
    assert!(window_position(&mut f, &wb).y < window_position(&mut f, &wa).y + 700);
}

#[test]
fn removing_a_left_column_preserves_the_active_column_screen_position_and_flight() {
    let mut f = Fixture::new();
    f.add_output(1, (1200, 800));
    let client = f.add_client();
    let a = map_window(&mut f, client, "a", (400, 200));
    map_window(&mut f, client, "b", (400, 200));
    map_window(&mut f, client, "c", (400, 200));
    let wc = window_by_app_id(&mut f, "c").unwrap();
    f.state().execute_action(&Action::ToggleTiling);
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
