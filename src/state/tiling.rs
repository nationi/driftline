use smithay::output::Output;
use smithay::reexports::wayland_server::Resource;
use smithay::utils::{Logical, Point, SERIAL_COUNTER};
use smithay::wayland::seat::WaylandFocus;

use driftwm::config::Direction;
use driftwm::layout::scrolling::{self, Column, Constraints};
use driftwm::window_ext::WindowExt;

use super::{DriftWm, StageWindow, configured_window_size, output_state, visual_frame_center};
use crate::grabs::SizeConstraints;

pub(crate) struct TilingView {
    output: Output,
    camera: Point<f64, Logical>,
    zoom: f64,
    origin: Point<i32, Logical>,
    last_usable: Option<smithay::utils::Rectangle<i32, Logical>>,
    anchor: Option<(driftwm::stage::ColumnId, i32)>,
    revealed: Option<(
        driftwm::stage::ElementId,
        smithay::utils::Rectangle<i32, Logical>,
    )>,
}

impl DriftWm {
    pub(crate) fn toggle_tiling(&mut self) {
        if self.tiling_view.is_some() {
            self.exit_tiling();
            return;
        }
        let Some(output) = self.active_output() else {
            return;
        };
        if self.space.outputs().count() != 1
            || self.is_fullscreen()
            || !self.interactive_move.is_empty()
        {
            tracing::warn!(
                "Scrolling tiling currently requires one output and no active move or fullscreen"
            );
            return;
        }
        self.cancel_animations_everywhere();
        let camera = self.camera();
        let zoom = self.zoom();
        let usable = self.get_usable_area();
        let camera_origin: Point<i32, Logical> = camera.to_i32_round();
        let (Ok(x), Ok(y)) = (
            i32::try_from(i64::from(camera_origin.x) + i64::from(usable.loc.x)),
            i32::try_from(i64::from(camera_origin.y) + i64::from(usable.loc.y)),
        ) else {
            return;
        };
        let origin = Point::from((x, y));
        self.tiling_view = Some(TilingView {
            output,
            camera,
            zoom,
            origin,
            last_usable: None,
            anchor: None,
            revealed: None,
        });
        self.set_zoom(1.0);
        self.refresh_tiling();
    }

    pub(crate) fn exit_tiling(&mut self) {
        let Some(view) = self.tiling_view.take() else {
            return;
        };
        self.cancel_animations_everywhere();
        for column in self.stage.clear_tiling() {
            let Some(element) = self.stage.window_by_id(column.id).cloned() else {
                continue;
            };
            self.drop_owed_recenter(&element);
            if let Some(window) = element.client() {
                self.send_size_configure(window, column.restore.size);
            }
            let center = visual_frame_center(
                column.restore.loc,
                column.restore.size,
                f64::from(self.window_ssd_bar(&element)),
            );
            self.establish_exit_placement(
                &element,
                column.restore.loc,
                column.restore.size,
                center,
                true,
            );
        }
        {
            let mut os = output_state(&view.output);
            os.camera = view.camera;
            os.zoom = view.zoom;
        }
        self.update_output_from_camera();
        self.render.blur_geometry_generation += 1;
        self.mark_all_dirty();
        self.session_store_mark_dirty();
    }

    pub(crate) fn focus_tiling_column(&mut self, direction: Direction) {
        self.select_focused_tile();
        let target = match direction {
            Direction::Left => self.stage.adjacent_column(true),
            Direction::Right => self.stage.adjacent_column(false),
            Direction::Up => self.stage.adjacent_tile(true),
            Direction::Down => self.stage.adjacent_tile(false),
            _ => None,
        };
        if let Some(id) = target
            && let Some(element) = self.stage.window_by_id(id).cloned()
        {
            self.stage.select_tile(id);
            self.raise_and_focus_element(&element, SERIAL_COUNTER.next_serial());
            self.refresh_tiling();
        }
    }

    pub(crate) fn move_into_tiling_column(&mut self, direction: Direction) {
        if self.select_focused_tile() && self.stage.move_into_column(direction == Direction::Left) {
            self.refresh_tiling();
        }
    }

    pub(crate) fn expel_tiling_window(&mut self) {
        if self.select_focused_tile() && self.stage.expel_from_column() {
            self.refresh_tiling();
        }
    }

    pub(crate) fn move_tiling_column(&mut self, direction: Direction) {
        if self.select_focused_tile() && self.stage.move_column(direction == Direction::Left) {
            self.refresh_tiling();
        }
    }

    pub(crate) fn move_tiling_window(&mut self, direction: Direction) {
        if self.select_focused_tile() && self.stage.move_tile(direction == Direction::Up) {
            self.refresh_tiling();
        }
    }

    fn select_focused_tile(&mut self) -> bool {
        let Some(element) = self.focused_element() else {
            return false;
        };
        self.stage
            .id_of(&element)
            .is_some_and(|id| self.stage.select_tile(id))
    }

    fn tiling_eligible(&self, element: &StageWindow) -> bool {
        let Some(window) = element.client() else {
            return false;
        };
        let Some(surface) = window.wl_surface() else {
            return false;
        };
        !window.is_widget()
            && window.parent_surface().is_none()
            && self.is_canvas_window(element)
            && !self.stage.is_fit(element)
            && !self.stage.is_fill(element)
            && !self.pending_center.contains(&*surface)
            && !self.pending_recenter.contains_key(&surface.id())
            && !self.element_under_interactive_grab(element)
            && !self.hidden_by_deferred_adopt(element)
    }

    pub(crate) fn refresh_tiling(&mut self) {
        let Some(view) = self.tiling_view.as_ref() else {
            return;
        };
        if self.space.outputs().count() != 1
            || self.active_output().as_ref() != Some(&view.output)
            || self.zoom() != 1.0
        {
            self.exit_tiling();
            return;
        }
        let origin = view.origin;
        if self.stage.tiled_windows().any(|column| {
            self.stage
                .window_by_id(column.id)
                .is_some_and(|w| !self.tiling_eligible(w))
        }) {
            self.exit_tiling();
            return;
        }
        let candidates: Vec<_> = self
            .stage
            .windows()
            .filter(|w| self.tiling_eligible(w))
            .filter_map(|w| Some((self.stage.id_of(w)?, configured_window_size(w.client()?))))
            .collect();
        let mut added = false;
        for (id, restore_size) in candidates {
            added |= self.stage.tile(id, restore_size);
        }
        if let Some(element) = self.focused_element()
            && let Some(id) = self.stage.id_of(&element)
        {
            self.stage.select_tile(id);
        }
        let columns: Vec<Vec<Column>> = self
            .stage
            .tiled_columns()
            .iter()
            .map(|column| {
                column
                    .windows
                    .iter()
                    .filter_map(|tile| {
                        let element = self.stage.window_by_id(tile.id)?;
                        let window = element.client()?;
                        let constraints = SizeConstraints::for_window(window);
                        Some(Column {
                            id: tile.id,
                            width: column.width,
                            constraints: Constraints {
                                min: constraints.min,
                                max: constraints.max,
                            },
                            chrome: self.element_chrome(element),
                        })
                    })
                    .collect()
            })
            .collect();
        let usable = self.get_usable_area();
        let gap = self.config.snap_gap.max(0.0).round() as i32;
        let mut placements = match scrolling::arrange_columns(&columns, origin, usable.size, gap) {
            Ok(placements) => placements,
            Err(error) => {
                tracing::warn!(?error, "Leaving scrolling tiling after invalid layout");
                self.exit_tiling();
                return;
            }
        };
        // A client can delay or refuse a size. Reserve its current frame as
        // well as the requested one without feeding that size back into configures.
        let mut extra_x = 0i64;
        let mut start = 0;
        for column in &columns {
            let group = &mut placements[start..start + column.len()];
            start += column.len();
            let desired_width = group
                .iter()
                .map(|p| i64::from(p.frame.size.w))
                .max()
                .unwrap_or(1);
            let mut reserved_width = desired_width;
            let mut extra_y = 0i64;
            for placement in group {
                let Some(element) = self.stage.window_by_id(placement.id) else {
                    continue;
                };
                let chrome = self.element_chrome(element);
                let actual = element.geometry().size;
                let width = i64::from(placement.frame.size.w)
                    .max(i64::from(actual.w) + 2 * i64::from(chrome.border));
                let height = i64::from(placement.frame.size.h).max(
                    i64::from(actual.h) + i64::from(chrome.bar) + 2 * i64::from(chrome.border),
                );
                let x = i64::from(placement.frame.loc.x) + extra_x;
                let y = i64::from(placement.frame.loc.y) + extra_y;
                let content_x = i64::from(placement.content.loc.x) + extra_x;
                let content_y = i64::from(placement.content.loc.y) + extra_y;
                if [x + width, y + height, width, height, content_x, content_y]
                    .iter()
                    .any(|v| i32::try_from(*v).is_err())
                {
                    self.exit_tiling();
                    return;
                }
                placement.frame.loc = (x as i32, y as i32).into();
                placement.content.loc = (content_x as i32, content_y as i32).into();
                extra_y += height - i64::from(placement.frame.size.h);
                placement.frame.size = (width as i32, height as i32).into();
                reserved_width = reserved_width.max(width);
            }
            extra_x += reserved_width - desired_width;
        }
        let mut changed = false;
        for placement in &placements {
            let Some(element) = self.stage.window_by_id(placement.id).cloned() else {
                continue;
            };
            if self.stage.position_of(&element) != Some(placement.content.loc) {
                self.stage.set_position(&element, placement.content.loc);
                changed = true;
            }
            if let Some(window) = element.client()
                && configured_window_size(window) != placement.content.size
            {
                self.send_size_configure(window, placement.content.size);
                changed = true;
            }
        }
        if changed {
            self.render.blur_geometry_generation += 1;
            self.mark_all_dirty();
        }
        let selected = placements
            .iter()
            .find(|p| Some(p.id) == self.stage.active_tile())
            .map(|p| (p.id, p.frame));
        let anchor = self
            .stage
            .active_column()
            .zip(selected.map(|(_, frame)| frame.loc.x));
        let translated = if let Some((column, x)) = anchor
            && let Some((previous_column, previous_x)) =
                self.tiling_view.as_ref().and_then(|v| v.anchor)
            && column == previous_column
            && x != previous_x
        {
            // Structural changes before the selected column must not scroll
            // its content under a stationary viewport, including mid-flight.
            let delta = f64::from(x) - f64::from(previous_x);
            self.with_output_state(|os| {
                os.camera.x += delta;
                if let Some(target) = &mut os.camera_target {
                    target.x += delta;
                }
            });
            self.update_output_from_camera();
            if let Some(pointer) = self.seat.get_pointer() {
                self.warp_pointer(pointer.current_location() + Point::from((delta, 0.0)));
            }
            true
        } else {
            false
        };
        if let Some((id, frame)) = selected
            && self.tiling_view.as_ref().is_some_and(|v| {
                let translation_only = translated
                    && v.last_usable == Some(usable)
                    && v.revealed.is_some_and(|(previous_id, previous_frame)| {
                        previous_id == id
                            && previous_frame.size == frame.size
                            && previous_frame.loc.y == frame.loc.y
                    });
                !translation_only
                    && (v.revealed != Some((id, frame)) || v.last_usable != Some(usable))
            })
            && let Ok(target) = scrolling::reveal(frame, usable, self.camera().to_i32_round())
        {
            self.set_camera_target(Some(target.to_f64()));
        }
        if let Some(view) = &mut self.tiling_view {
            view.revealed = selected;
            view.last_usable = Some(usable);
            view.anchor = anchor;
        }
        if added {
            self.session_store_mark_dirty();
        }
    }

    pub(super) fn tiling_canvas_view(&self, output: &Output) -> Option<(Point<f64, Logical>, f64)> {
        self.tiling_view
            .as_ref()
            .filter(|view| &view.output == output)
            .map(|view| (view.camera, view.zoom))
    }
}
