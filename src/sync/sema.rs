use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::cell::{Cell, RefCell};

use crate::sbi;
use crate::thread::{self, Thread};

/// Atomic counting semaphore
///
/// # Examples
/// ```
/// let sema = Semaphore::new(0);
/// sema.down();
/// sema.up();
/// ```
#[derive(Clone)]
pub struct Semaphore {
    value: Cell<usize>,
    waiters: RefCell<VecDeque<Arc<Thread>>>,
}

unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}

impl Semaphore {
    /// Creates a new semaphore of initial value n.
    pub const fn new(n: usize) -> Self {
        Semaphore {
            value: Cell::new(n),
            waiters: RefCell::new(VecDeque::new()),
        }
    }

    /// P operation
    pub fn down(&self) {
        let old = sbi::interrupt::set(false);

        // Is semaphore available?
        while self.value() == 0 {
            // `push_front` ensures to wake up threads in a fifo manner
            self.waiters.borrow_mut().push_front(thread::current());

            // Block the current thread until it's awakened by an `up` operation
            thread::block();
        }
        self.value.set(self.value() - 1);

        sbi::interrupt::set(old);
    }

    /// V operation
    pub fn up(&self) {
        let old = sbi::interrupt::set(false);
        let count = self.value.replace(self.value() + 1);

        // Check if we need to wake up a sleeping waiter
        if let Some(thread) = self.waiters.borrow_mut().pop_back() {
            assert_eq!(count, 0);

            thread::wake_up(thread.clone());
        }

        sbi::interrupt::set(old);
    }

    /// Get the current value of a semaphore
    pub fn value(&self) -> usize {
        self.value.get()
    }
}
