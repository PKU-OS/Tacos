use core::cell::Cell;

use super::{Intr, Lock};

#[derive(Clone, Copy)]
pub enum OnceState {
    InComplete,
    Complete,
}

/// A synchronization primitive which can be
/// used to run a one-time global initialization.
pub struct Once {
    inner: Cell<OnceState>,
    lock: Intr,
}

unsafe impl Sync for Once {}

impl Once {
    pub const fn new() -> Self {
        Self {
            inner: Cell::new(OnceState::InComplete),
            lock: Intr::new(),
        }
    }

    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.is_completed() {
            return;
        }

        self.lock.acquire();
        if matches!(self.inner.get(), OnceState::InComplete) {
            f();
            self.inner.set(OnceState::Complete);
        }
        self.lock.release();
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.inner.get(), OnceState::Complete)
    }
}

/// A cell which can only be initialized once
pub struct OnceCell<T> {
    inner: Cell<Option<T>>,
    once: Once,
}

unsafe impl<T: Sync + Send> Sync for OnceCell<T> {}

impl<T> OnceCell<T> {
    /// Creates a lazy-initialized cell.
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            inner: Cell::new(None),
        }
    }

    pub fn init<F>(&self, f: F)
    where
        F: FnOnce() -> T,
    {
        self.once.call_once(|| self.inner.set(Some(f())));
    }

    /// Initialize or get the value from a cell. A cell will only
    /// be initialized **once**.
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if self.once.is_completed() {
            return self.get();
        }

        self.init(f);
        self.get()
    }

    /// Gets the reference to the underlying value.
    /// Returns None if the cell is empty, or being initialized.
    pub fn get(&self) -> &T {
        unsafe {
            match *self.inner.as_ptr() {
                Some(ref x) => x,
                None => unreachable!("attempted to derefence an uninitialized once_cell"),
            }
        }
    }
}
