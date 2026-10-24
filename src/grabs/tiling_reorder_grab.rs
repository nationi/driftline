use smithay::{
    input::{
        SeatHandler,
        pointer::{ButtonEvent, GrabStartData, MotionEvent, PointerGrab, PointerInnerHandle},
    },
    utils::{Logical, Point},
};

use crate::grabs::MoveGrab;
use crate::state::{Driftline, output_logical_size, output_state};
use driftline::canvas::{CanvasPos, canvas_to_screen};
use driftline::stage::ElementId;

/// Holds an Alt+drag on a tiled window and reorders it on release.
pub struct TilingReorderGrab {
    pub start_data: GrabStartData<Driftline>,
    source: ElementId,
    output_name: String,
    last_output_name: String,
    last_position: Point<f64, Logical>,
}

impl TilingReorderGrab {
    pub fn new(
        start_data: GrabStartData<Driftline>,
        source: ElementId,
        output_name: String,
    ) -> Self {
        let last_position = start_data.location;
        Self {
            start_data,
            source,
            last_output_name: output_name.clone(),
            output_name,
            last_position,
        }
    }

    fn clear_edge_pan(&self, data: &Driftline) {
        if let Some(output) = data.output_by_name(&self.output_name) {
            data.clear_edge_pan(&output);
        }
    }

    fn update_edge_pan(&self, data: &Driftline, location: Point<f64, Logical>) {
        let Some(output) = data.output_by_name(&self.output_name) else {
            return;
        };
        if data
            .active_output()
            .is_none_or(|active| active.name() != self.output_name)
        {
            data.clear_edge_pan(&output);
            return;
        }
        let Some(columns) = data.stage.tiled_columns_for_output(&self.output_name) else {
            data.clear_edge_pan(&output);
            return;
        };
        let Some(column_index) = columns
            .iter()
            .position(|column| column.windows.iter().any(|tile| tile.id == self.source))
        else {
            data.clear_edge_pan(&output);
            return;
        };
        let (camera, zoom) = {
            let state = output_state(&output);
            (state.camera, state.zoom)
        };
        let screen_pos = canvas_to_screen(CanvasPos(location), camera, zoom).0;
        let output_size = output_logical_size(&output);
        let width = f64::from(output_size.w);
        let zone = data.config.edge_zone;
        let horizontal = MoveGrab::edge_pan_velocity(
            screen_pos,
            width,
            f64::from(output_size.h),
            zone,
            data.config.edge_pan_min,
            data.config.edge_pan_max,
        )
        .map(|velocity| Point::from((velocity.x, 0.0)));
        let has_right_neighbor = column_index + 1 < columns.len();
        let velocity = if zone > 0.0 && screen_pos.x < zone && column_index > 0 {
            horizontal.filter(|velocity| velocity.x < 0.0)
        } else if zone > 0.0 && width - screen_pos.x < zone && has_right_neighbor {
            horizontal.filter(|velocity| velocity.x > 0.0)
        } else {
            None
        };
        data.update_edge_pan_request(&output, velocity, screen_pos);
    }
}

impl PointerGrab<Driftline> for TilingReorderGrab {
    fn motion(
        &mut self,
        data: &mut Driftline,
        handle: &mut PointerInnerHandle<'_, Driftline>,
        _focus: Option<(
            <Driftline as SeatHandler>::PointerFocus,
            Point<f64, Logical>,
        )>,
        event: &MotionEvent,
    ) {
        self.last_position = event.location;
        self.last_output_name = data
            .active_output()
            .map(|output| output.name())
            .unwrap_or_else(|| self.output_name.clone());
        let destination = self.last_output_name.as_str();
        let transfer = destination != self.output_name;
        let target = if transfer {
            data.stage
                .active_column_for_output(destination)
                .and_then(|column| data.stage.column(column))
                .map(|column| column.selected)
        } else {
            data.tiling_drop_feedback_target(self.source, &self.output_name, event.location)
        };
        data.tiling_drop_target = target.map(|id| (id, destination.to_owned()));
        data.tiling_drag_visual = Some((
            self.source,
            destination.to_owned(),
            event.location - self.start_data.location,
        ));
        data.mark_all_dirty();
        handle.motion(data, None, event);
        if transfer {
            self.clear_edge_pan(data);
        } else {
            self.update_edge_pan(data, event.location);
        }
    }

    fn button(
        &mut self,
        data: &mut Driftline,
        handle: &mut PointerInnerHandle<'_, Driftline>,
        event: &ButtonEvent,
    ) {
        handle.button(data, event);
        if handle.current_pressed().is_empty() {
            self.clear_edge_pan(data);
            let source = self.source;
            let output_name = self.output_name.clone();
            let destination_name = self.last_output_name.clone();
            let position = self.last_position;
            let drag_delta = self.last_position - self.start_data.location;
            data.loop_handle.insert_idle(move |data| {
                if destination_name != output_name {
                    if let Some(destination) = data.output_by_name(&destination_name) {
                        data.move_tiled_window_to_output(source, &destination);
                    }
                } else {
                    data.complete_tiling_reorder(source, &output_name, position, drag_delta);
                }
            });
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn unset(&mut self, data: &mut Driftline) {
        self.clear_edge_pan(data);
        data.tiling_drag_visual = None;
        data.tiling_drop_target = None;
        data.mark_all_dirty();
    }

    crate::grabs::forward_pointer_grab_methods!();
}
