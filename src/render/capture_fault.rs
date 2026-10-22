use std::cell::{Cell, RefCell};
use std::rc::Rc;

type ArmedFailure = (usize, Rc<Cell<bool>>);

thread_local! {
    static FAILURE: RefCell<Option<ArmedFailure>> = const { RefCell::new(None) };
}

pub(crate) struct CaptureAllocationFailure {
    previous: Option<ArmedFailure>,
    triggered: Rc<Cell<bool>>,
}

impl CaptureAllocationFailure {
    pub(crate) fn after(successes: usize) -> Self {
        let triggered = Rc::new(Cell::new(false));
        let previous = FAILURE.with_borrow_mut(|slot| slot.replace((successes, triggered.clone())));
        Self {
            previous,
            triggered,
        }
    }

    pub(crate) fn triggered(&self) -> bool {
        self.triggered.get()
    }
}

impl Drop for CaptureAllocationFailure {
    fn drop(&mut self) {
        FAILURE.with_borrow_mut(|slot| *slot = self.previous.take());
    }
}

pub(super) fn should_fail() -> bool {
    FAILURE.with_borrow_mut(|slot| {
        let Some((remaining, triggered)) = slot.as_mut() else {
            return false;
        };
        if *remaining == 0 {
            triggered.set(true);
            *slot = None;
            true
        } else {
            *remaining -= 1;
            false
        }
    })
}
