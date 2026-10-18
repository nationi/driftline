use std::collections::HashMap;
use std::time::Instant;

use smithay::utils::Serial;
use smithay::wayland::compositor::Barrier;

use super::transaction::{CONFIGURE_TIMEOUT, LayoutGeneration};
use crate::stage::ElementId;

/// A pre-commit barrier for one immutable set of configure requests.
///
/// The owner must wake affected clients after a method returns `true`, and after
/// dropping a pending group. A timer must call `expire` at `deadline` even when
/// no client commits. This type does not install hooks or event-loop sources.
pub struct CommitGroup {
    generation: LayoutGeneration,
    waiting: HashMap<ElementId, Serial>,
    deadline: Instant,
    barrier: Barrier,
    released: bool,
}

impl CommitGroup {
    pub fn new(
        generation: LayoutGeneration,
        requests: HashMap<ElementId, Serial>,
        now: Instant,
    ) -> Self {
        let released = requests.is_empty();
        Self {
            generation,
            waiting: requests,
            deadline: now + CONFIGURE_TIMEOUT,
            barrier: Barrier::new(released),
            released,
        }
    }

    pub fn pending_requests(&self) -> &HashMap<ElementId, Serial> {
        &self.waiting
    }

    pub fn accepts(&self, id: ElementId, serial: Serial) -> bool {
        !self.released
            && self
                .waiting
                .get(&id)
                .is_some_and(|expected| serial >= *expected)
    }

    pub fn blocker(&self) -> Barrier {
        self.barrier.clone()
    }

    pub fn deadline(&self) -> Instant {
        self.deadline
    }

    /// Call only for a new-buffer pre-commit, using its acknowledged serial.
    /// Client-chosen geometry is accepted, as in niri; an ack alone is not ready.
    pub fn arrive(
        &mut self,
        generation: LayoutGeneration,
        id: ElementId,
        serial: Serial,
        now: Instant,
    ) -> bool {
        if self.expire(now) {
            return true;
        }
        if self.released || generation != self.generation {
            return false;
        }
        if self
            .waiting
            .get(&id)
            .is_some_and(|expected| serial >= *expected)
        {
            self.waiting.remove(&id);
        }
        self.release_if_empty()
    }

    /// Removes a participant that has unmapped or disconnected.
    pub fn remove(&mut self, id: ElementId) -> bool {
        self.waiting.remove(&id);
        self.release_if_empty()
    }

    pub fn expire(&mut self, now: Instant) -> bool {
        if now >= self.deadline {
            self.release()
        } else {
            false
        }
    }

    /// Releases queued commits on supersession or mode exit; never discards them.
    pub fn cancel(&mut self) -> bool {
        self.release()
    }

    fn release_if_empty(&mut self) -> bool {
        self.waiting.is_empty() && self.release()
    }

    fn release(&mut self) -> bool {
        if self.released {
            return false;
        }
        self.released = true;
        self.waiting.clear();
        self.barrier.signal();
        true
    }
}

impl Drop for CommitGroup {
    fn drop(&mut self) {
        self.release();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::scrolling::transaction::ConfigureBatch;
    use smithay::wayland::compositor::{Blocker, BlockerState};
    use std::time::Duration;

    fn generation() -> LayoutGeneration {
        let mut stage = crate::stage::Stage::<crate::stage::mock::TestWindow>::new();
        ConfigureBatch::new(stage.begin_tiling()).update_layout(&[])
    }

    fn group(now: Instant) -> CommitGroup {
        CommitGroup::new(
            generation(),
            HashMap::from([(ElementId(1), 10.into()), (ElementId(2), 11.into())]),
            now,
        )
    }

    #[test]
    fn early_peer_waits_until_the_entire_group_arrives() {
        let now = Instant::now();
        let mut group = group(now);
        let blocker = group.blocker();
        assert!(!group.arrive(group.generation, ElementId(2), 11.into(), now));
        assert_eq!(blocker.state(), BlockerState::Pending);
        assert!(!group.arrive(group.generation, ElementId(2), 12.into(), now));
        assert!(!group.arrive(group.generation, ElementId(1), 9.into(), now));
        assert!(group.arrive(group.generation, ElementId(1), 10.into(), now));
        assert_eq!(blocker.state(), BlockerState::Released);
        assert!(!group.cancel());
    }

    #[test]
    fn deadline_is_shared_exact_and_terminal() {
        let now = Instant::now();
        let mut group = group(now);
        let blocker = group.blocker();
        assert!(!group.arrive(group.generation, ElementId(1), 10.into(), now));
        assert!(!group.expire(group.deadline() - Duration::from_nanos(1)));
        assert!(group.expire(now + CONFIGURE_TIMEOUT));
        assert_eq!(blocker.state(), BlockerState::Released);
        assert!(!group.arrive(group.generation, ElementId(2), 11.into(), now));
        assert!(!group.expire(now + CONFIGURE_TIMEOUT));
    }

    #[test]
    fn removal_cancel_and_drop_release_existing_blockers() {
        let now = Instant::now();
        let mut group = group(now);
        let blocker = group.blocker();
        assert!(!group.remove(ElementId(99)));
        assert!(!group.remove(ElementId(1)));
        assert!(group.remove(ElementId(2)));
        assert_eq!(blocker.state(), BlockerState::Released);
        let mut next = self::group(now);
        let blocker = next.blocker();
        assert!(next.cancel());
        assert_eq!(blocker.state(), BlockerState::Released);
        let pending = self::group(now);
        let blocker = pending.blocker();
        drop(pending);
        assert_eq!(blocker.state(), BlockerState::Released);
    }

    #[test]
    fn old_generation_cannot_release_a_new_group_and_serials_wrap() {
        let now = Instant::now();
        let mut stage = crate::stage::Stage::<crate::stage::mock::TestWindow>::new();
        let old = ConfigureBatch::new(stage.begin_tiling()).update_layout(&[]);
        stage.clear_tiling();
        let generation = ConfigureBatch::new(stage.begin_tiling()).update_layout(&[]);
        let mut group = CommitGroup::new(
            generation,
            HashMap::from([(ElementId(1), (u32::MAX - 1).into())]),
            now,
        );
        assert!(!group.arrive(old, ElementId(1), 1.into(), now));
        assert_eq!(group.blocker().state(), BlockerState::Pending);
        assert!(group.arrive(generation, ElementId(1), 1.into(), now));
    }

    #[test]
    fn empty_groups_are_released_and_unrelated_groups_are_independent() {
        let now = Instant::now();
        let empty = CommitGroup::new(generation(), HashMap::new(), now);
        assert_eq!(empty.blocker().state(), BlockerState::Released);
        let mut first = group(now);
        let second = group(now);
        first.cancel();
        assert_eq!(second.blocker().state(), BlockerState::Pending);
    }
}
