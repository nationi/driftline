use std::collections::HashMap;
use std::time::{Duration, Instant};

use smithay::utils::{Logical, Serial, Size};

use super::Placement;
use crate::stage::{ElementId, TilingLayoutId};

pub const CONFIGURE_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutGeneration {
    layout: TilingLayoutId,
    revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigureProgress {
    Pending,
    Applied,
    TimedOut,
}

struct Request {
    size: Size<i32, Logical>,
    serial: Serial,
    deadline: Instant,
    progress: ConfigureProgress,
    late_resize_taken: bool,
}

/// Tracks the latest request per live tile; it never blocks or retains history.
pub struct ConfigureBatch {
    generation: LayoutGeneration,
    layout: Vec<Placement>,
    requests: HashMap<ElementId, Request>,
}

impl ConfigureBatch {
    pub fn new(layout: TilingLayoutId) -> Self {
        Self {
            generation: LayoutGeneration {
                layout,
                revision: 0,
            },
            layout: Vec::new(),
            requests: HashMap::new(),
        }
    }

    pub fn update_layout(&mut self, layout: &[Placement]) -> LayoutGeneration {
        if self.layout != layout {
            self.generation.revision = self
                .generation
                .revision
                .checked_add(1)
                .expect("layout generation exhausted");
            self.layout = layout.to_vec();
            self.requests.retain(|id, request| {
                layout
                    .iter()
                    .any(|p| p.id == *id && p.content.size == request.size)
            });
        }
        self.generation
    }

    pub fn record(&mut self, id: ElementId, serial: Serial, now: Instant) {
        let Some(placement) = self.layout.iter().find(|p| p.id == id) else {
            return;
        };
        self.requests.insert(
            id,
            Request {
                size: placement.content.size,
                serial,
                deadline: now + CONFIGURE_TIMEOUT,
                progress: ConfigureProgress::Pending,
                late_resize_taken: false,
            },
        );
    }

    /// A cached committed serial is required; an ack alone cannot satisfy this.
    pub fn observe_commit(
        &mut self,
        generation: LayoutGeneration,
        id: ElementId,
        serial: Serial,
        size: Size<i32, Logical>,
    ) {
        if generation != self.generation {
            return;
        }
        if let Some(request) = self.requests.get_mut(&id)
            && request.progress == ConfigureProgress::Pending
            && serial >= request.serial
            && size == request.size
        {
            request.progress = ConfigureProgress::Applied;
        }
    }

    pub fn advance(&mut self, now: Instant) {
        for request in self.requests.values_mut() {
            if request.progress == ConfigureProgress::Pending && now >= request.deadline {
                request.progress = ConfigureProgress::TimedOut;
            }
        }
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.requests
            .values()
            .filter(|r| r.progress == ConfigureProgress::Pending)
            .map(|r| r.deadline)
            .min()
    }

    pub fn pending_count(&self) -> usize {
        self.requests
            .values()
            .filter(|r| r.progress == ConfigureProgress::Pending)
            .count()
    }

    /// Consume the first buffer response after a timed-out request, without
    /// reopening its group or arming an animation on every subsequent repaint.
    pub fn take_late_resize(
        &mut self,
        id: ElementId,
        serial: Serial,
    ) -> Option<Size<i32, Logical>> {
        let request = self.requests.get_mut(&id)?;
        if request.progress != ConfigureProgress::TimedOut
            || request.late_resize_taken
            || serial < request.serial
        {
            return None;
        }
        request.late_resize_taken = true;
        Some(request.size)
    }

    pub fn progress(&self, id: ElementId) -> Option<ConfigureProgress> {
        self.requests.get(&id).map(|r| r.progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::utils::Rectangle;

    fn batch() -> ConfigureBatch {
        let mut stage = crate::stage::Stage::<crate::stage::mock::TestWindow>::new();
        ConfigureBatch::new(stage.begin_tiling())
    }

    #[test]
    fn a_generation_from_a_previous_layout_is_never_accepted() {
        let mut stage = crate::stage::Stage::<crate::stage::mock::TestWindow>::new();
        let mut first = ConfigureBatch::new(stage.begin_tiling());
        let old = first.update_layout(&[placement(1, 300)]);
        stage.clear_tiling();
        let mut next = ConfigureBatch::new(stage.begin_tiling());
        let generation = next.update_layout(&[placement(1, 300)]);
        next.record(ElementId(1), 12.into(), Instant::now());
        next.observe_commit(old, ElementId(1), 12.into(), (300, 600).into());
        assert_eq!(next.pending_count(), 1);
        next.observe_commit(generation, ElementId(1), 12.into(), (300, 600).into());
        assert_eq!(next.pending_count(), 0);
    }

    fn placement(id: u64, width: i32) -> Placement {
        let rect = Rectangle::from_size((width, 600).into());
        Placement {
            id: ElementId(id),
            frame: rect,
            content: rect,
        }
    }

    #[test]
    fn an_old_generation_or_serial_cannot_settle_a_new_request() {
        let mut batch = batch();
        let now = Instant::now();
        let first = batch.update_layout(&[placement(1, 300)]);
        batch.record(ElementId(1), 10.into(), now);
        let next = batch.update_layout(&[placement(1, 400)]);
        assert_ne!(first, next);
        batch.record(ElementId(1), 11.into(), now);
        batch.observe_commit(first, ElementId(1), 11.into(), (400, 600).into());
        batch.observe_commit(next, ElementId(1), 10.into(), (400, 600).into());
        batch.observe_commit(next, ElementId(1), 11.into(), (300, 600).into());
        assert_eq!(batch.pending_count(), 1);
        batch.observe_commit(next, ElementId(1), 12.into(), (400, 600).into());
        assert_eq!(
            batch.progress(ElementId(1)),
            Some(ConfigureProgress::Applied)
        );
    }

    #[test]
    fn stalled_tiles_expire_independently_and_late_commits_do_not_reopen_them() {
        let mut batch = batch();
        let now = Instant::now();
        let generation = batch.update_layout(&[placement(1, 300), placement(2, 300)]);
        batch.record(ElementId(1), 10.into(), now);
        batch.record(ElementId(2), 11.into(), now);
        batch.observe_commit(generation, ElementId(2), 11.into(), (300, 600).into());
        batch.advance(now + CONFIGURE_TIMEOUT - Duration::from_nanos(1));
        assert_eq!(batch.pending_count(), 1);
        batch.advance(now + CONFIGURE_TIMEOUT);
        batch.observe_commit(generation, ElementId(1), 10.into(), (300, 600).into());
        assert_eq!(batch.pending_count(), 0);
        assert_eq!(
            batch.progress(ElementId(1)),
            Some(ConfigureProgress::TimedOut)
        );
        assert_eq!(
            batch.progress(ElementId(2)),
            Some(ConfigureProgress::Applied)
        );
    }

    #[test]
    fn unchanged_targets_keep_their_deadlines_and_dead_tiles_are_pruned() {
        let mut batch = batch();
        let now = Instant::now();
        let generation = batch.update_layout(&[placement(1, 300)]);
        batch.record(ElementId(1), 10.into(), now);
        assert_eq!(batch.update_layout(&[placement(1, 300)]), generation);
        let mut moved = placement(1, 300);
        moved.frame.loc.x = 100;
        moved.content.loc.x = 100;
        batch.update_layout(&[moved]);
        batch.advance(now + CONFIGURE_TIMEOUT);
        assert_eq!(
            batch.progress(ElementId(1)),
            Some(ConfigureProgress::TimedOut)
        );
        batch.update_layout(&[]);
        assert_eq!(batch.progress(ElementId(1)), None);
        assert_eq!(batch.pending_count(), 0);
    }

    #[test]
    fn serial_comparison_handles_protocol_wraparound() {
        let mut batch = batch();
        let generation = batch.update_layout(&[placement(1, 300)]);
        batch.record(ElementId(1), (u32::MAX - 1).into(), Instant::now());
        batch.observe_commit(generation, ElementId(1), 1.into(), (300, 600).into());
        assert_eq!(batch.pending_count(), 0);
    }
    #[test]
    fn late_resize_is_serial_checked_and_consumed_once() {
        let now = Instant::now();
        let mut batch = batch();
        batch.update_layout(&[placement(1, 300)]);
        batch.record(ElementId(1), 10.into(), now);
        assert_eq!(batch.take_late_resize(ElementId(1), 10.into()), None);
        batch.advance(now + CONFIGURE_TIMEOUT);
        assert_eq!(batch.take_late_resize(ElementId(1), 9.into()), None);
        assert_eq!(
            batch.take_late_resize(ElementId(1), 10.into()),
            Some((300, 600).into())
        );
        assert_eq!(batch.take_late_resize(ElementId(1), 11.into()), None);
        assert_eq!(
            batch.progress(ElementId(1)),
            Some(ConfigureProgress::TimedOut)
        );
        assert_eq!(batch.next_deadline(), None);
    }

    #[test]
    fn a_new_request_replaces_the_consumed_late_resize() {
        let now = Instant::now();
        let mut batch = batch();
        batch.update_layout(&[placement(1, 300)]);
        batch.record(ElementId(1), 10.into(), now);
        batch.advance(now + CONFIGURE_TIMEOUT);
        assert!(batch.take_late_resize(ElementId(1), 10.into()).is_some());
        batch.update_layout(&[placement(1, 400)]);
        batch.record(ElementId(1), 20.into(), now);
        batch.advance(now + CONFIGURE_TIMEOUT);
        assert_eq!(batch.take_late_resize(ElementId(1), 10.into()), None);
        assert_eq!(
            batch.take_late_resize(ElementId(1), 20.into()),
            Some((400, 600).into())
        );
    }
}
