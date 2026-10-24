use std::time::{Duration, Instant};

use smithay::{
    input::{
        SeatHandler,
        pointer::{ButtonEvent, GrabStartData, MotionEvent, PointerGrab, PointerInnerHandle},
    },
    output::Output,
    utils::{Logical, Point},
};

use crate::state::Driftline;
use driftline::canvas::{CanvasPos, canvas_to_screen};
use driftline::layout::scrolling::ColumnWidth;
use driftline::stage::ColumnId;

const RESIZE_REFRESH_INTERVAL: Duration = Duration::from_millis(16);
const EDGE_PAN_ZONE: f64 = 96.0;
const EDGE_PAN_MIN: f64 = 0.35;
const EDGE_PAN_MAX: f64 = 1.5;

/// Resizes one scrolling column horizontally while Alt+right-drag is held.
pub struct TilingResizeGrab {
    pub start_data: GrabStartData<Driftline>,
    column: ColumnId,
    output: Output,
    initial_width: i32,
    min_width: i32,
    max_width: i32,
    last_refresh: Option<Instant>,
    changed: bool,
}

impl TilingResizeGrab {
    pub fn new(
        start_data: GrabStartData<Driftline>,
        column: ColumnId,
        output: Output,
        initial_width: i32,
        min_width: i32,
        max_width: i32,
    ) -> Self {
        Self {
            start_data,
            column,
            output,
            initial_width: initial_width.max(1),
            min_width: min_width.max(1),
            max_width: max_width.max(min_width).max(1),
            last_refresh: None,
            changed: false,
        }
    }

    fn update_edge_pan(&self, data: &Driftline, location: Point<f64, Logical>) {
        let (camera, zoom) = {
            let os = crate::state::output_state(&self.output);
            (os.camera, os.zoom)
        };
        let screen = canvas_to_screen(CanvasPos(location), camera, zoom).0;
        let viewport = crate::state::output_logical_size(&self.output);
        let (distance, direction) = if screen.x < EDGE_PAN_ZONE {
            (screen.x, -1.0)
        } else if viewport.w as f64 - screen.x < EDGE_PAN_ZONE {
            (viewport.w as f64 - screen.x, 1.0)
        } else {
            data.clear_edge_pan(&self.output);
            return;
        };

        let depth = ((EDGE_PAN_ZONE - distance) / EDGE_PAN_ZONE).clamp(0.0, 1.0);
        let speed = EDGE_PAN_MIN + (EDGE_PAN_MAX - EDGE_PAN_MIN) * depth * depth;
        data.update_edge_pan_request(
            &self.output,
            Some(Point::from((direction * speed, 0.0))),
            screen,
        );
    }
}

impl PointerGrab<Driftline> for TilingResizeGrab {
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
        if data.focused_output.as_ref() != Some(&self.output) {
            data.focused_output = Some(self.output.clone());
        }
        data.stage.activate_tiling_output(&self.output.name());
        let delta = (event.location.x - self.start_data.location.x).round();
        let width = (f64::from(self.initial_width) + delta)
            .clamp(f64::from(self.min_width), f64::from(self.max_width)) as i32;
        data.tiling_width_indicator = Some((self.column, self.output.name(), width));
        if data
            .stage
            .set_column_width(self.column, ColumnWidth::Fixed(width))
        {
            let now = Instant::now();
            // Pointer devices can report much faster than the display refreshes.
            // Coalesce the expensive layout/configure work; release flushes the
            // final width below.
            if self
                .last_refresh
                .is_none_or(|last| now.duration_since(last) >= RESIZE_REFRESH_INTERVAL)
            {
                data.refresh_tiling();
                self.last_refresh = Some(now);
            }
            data.mark_all_dirty();
            self.changed = true;
        }
        self.update_edge_pan(data, event.location);
        handle.motion(data, None, event);
    }

    fn button(
        &mut self,
        data: &mut Driftline,
        handle: &mut PointerInnerHandle<'_, Driftline>,
        event: &ButtonEvent,
    ) {
        handle.button(data, event);
        if handle.current_pressed().is_empty() {
            data.clear_edge_pan(&self.output);
            if self.changed {
                data.refresh_tiling();
                data.session_store_mark_dirty();
            }
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn unset(&mut self, data: &mut Driftline) {
        data.clear_edge_pan(&self.output);
        data.tiling_width_indicator = None;
        data.mark_all_dirty();
    }

    crate::grabs::forward_pointer_grab_methods!();
}
