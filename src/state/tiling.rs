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
        if let Some(id) = self.stage.adjacent_column(direction == Direction::Left)
            && let Some(element) = self.stage.window_by_id(id).cloned()
        {
            self.stage.select_column(id);
            self.raise_and_focus_element(&element, SERIAL_COUNTER.next_serial());
            self.refresh_tiling();
        }
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
        if self.stage.tiled_columns().iter().any(|column| {
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
            self.stage.select_column(id);
        }
        let columns: Vec<_> = self
            .stage
            .tiled_columns()
            .iter()
            .filter_map(|column| {
                let element = self.stage.window_by_id(column.id)?;
                let window = element.client()?;
                let constraints = SizeConstraints::for_window(window);
                Some(Column {
                    id: column.id,
                    width: column.width,
                    constraints: Constraints {
                        min: constraints.min,
                        max: constraints.max,
                    },
                    chrome: self.element_chrome(element),
                })
            })
            .collect();
        let usable = self.get_usable_area();
        let gap = self.config.snap_gap.max(0.0).round() as i32;
        let mut placements = match scrolling::arrange(&columns, origin, usable.size, gap) {
            Ok(placements) => placements,
            Err(error) => {
                tracing::warn!(?error, "Leaving scrolling tiling after invalid layout");
                self.exit_tiling();
                return;
            }
        };
        // A client can delay or refuse a size. Reserve its current frame as
        // well as the requested one without feeding that size back into configures.
        let mut extra = 0i64;
        for placement in &mut placements {
            let Some(element) = self.stage.window_by_id(placement.id) else {
                continue;
            };
            let actual_width = i64::from(element.geometry().size.w)
                + 2 * i64::from(self.element_chrome(element).border);
            let width = i64::from(placement.frame.size.w).max(actual_width);
            let x = i64::from(placement.frame.loc.x) + extra;
            let content_x = i64::from(placement.content.loc.x) + extra;
            if i32::try_from(x + width).is_err()
                || i32::try_from(width).is_err()
                || i32::try_from(content_x).is_err()
            {
                self.exit_tiling();
                return;
            }
            placement.frame.loc.x = x as i32;
            placement.content.loc.x = content_x as i32;
            extra += width - i64::from(placement.frame.size.w);
            placement.frame.size.w = width as i32;
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
            .find(|p| Some(p.id) == self.stage.active_column())
            .map(|p| (p.id, p.frame));
        if let Some((id, frame)) = selected
            && self
                .tiling_view
                .as_ref()
                .is_some_and(|v| v.revealed != Some((id, frame)) || v.last_usable != Some(usable))
            && let Ok(target) = scrolling::reveal(frame, usable, self.camera().to_i32_round())
        {
            self.set_camera_target(Some(target.to_f64()));
        }
        if let Some(view) = &mut self.tiling_view {
            view.revealed = selected;
            view.last_usable = Some(usable);
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
