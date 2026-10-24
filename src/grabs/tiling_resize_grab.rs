use smithay::{
    input::{
        SeatHandler,
        pointer::{ButtonEvent, GrabStartData, MotionEvent, PointerGrab, PointerInnerHandle},
    },
    output::Output,
    utils::{Logical, Point},
};

use crate::state::Driftline;
use driftline::layout::scrolling::ColumnWidth;
use driftline::stage::ColumnId;

/// Resizes one scrolling column horizontally while Alt+right-drag is held.
pub struct TilingResizeGrab {
    pub start_data: GrabStartData<Driftline>,
    column: ColumnId,
    output: Output,
    initial_width: i32,
    max_width: i32,
    changed: bool,
}

impl TilingResizeGrab {
    pub fn new(
        start_data: GrabStartData<Driftline>,
        column: ColumnId,
        output: Output,
        initial_width: i32,
        max_width: i32,
    ) -> Self {
        Self {
            start_data,
            column,
            output,
            initial_width: initial_width.max(1),
            max_width: max_width.max(1),
            changed: false,
        }
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
        let width =
            (f64::from(self.initial_width) + delta).clamp(1.0, f64::from(self.max_width)) as i32;
        data.tiling_width_indicator = Some((self.column, self.output.name(), width));
        if data
            .stage
            .set_column_width(self.column, ColumnWidth::Fixed(width))
        {
            data.refresh_tiling();
            data.mark_all_dirty();
            self.changed = true;
        }
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
            if self.changed {
                data.session_store_mark_dirty();
            }
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn unset(&mut self, data: &mut Driftline) {
        data.tiling_width_indicator = None;
        data.mark_all_dirty();
    }

    crate::grabs::forward_pointer_grab_methods!();
}
