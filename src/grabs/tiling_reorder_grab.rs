use smithay::{
    input::{
        SeatHandler,
        pointer::{ButtonEvent, GrabStartData, MotionEvent, PointerGrab, PointerInnerHandle},
    },
    utils::{Logical, Point},
};

use crate::state::Driftline;
use driftline::stage::ElementId;

/// Holds an Alt+drag on a tiled window and swaps it with the tile under the
/// pointer when released.
pub struct TilingReorderGrab {
    pub start_data: GrabStartData<Driftline>,
    source: ElementId,
    output_name: String,
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
            output_name,
            last_position,
        }
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
        data.tiling_drop_target = data
            .tiling_drop_feedback_target(self.source, &self.output_name, event.location)
            .map(|id| (id, self.output_name.clone()));
        data.tiling_drag_visual = Some((
            self.source,
            self.output_name.clone(),
            event.location - self.start_data.location,
        ));
        data.mark_all_dirty();
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
            let source = self.source;
            let output_name = self.output_name.clone();
            let position = self.last_position;
            let drag_delta = self.last_position - self.start_data.location;
            data.loop_handle.insert_idle(move |data| {
                data.complete_tiling_reorder(source, &output_name, position, drag_delta);
            });
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn unset(&mut self, data: &mut Driftline) {
        data.tiling_drag_visual = None;
        data.tiling_drop_target = None;
        data.mark_all_dirty();
    }

    crate::grabs::forward_pointer_grab_methods!();
}
