use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use crate::sync::{self, Lock};

/// A mutual exclusion primitive useful for protecting shared data
///
/// # Examples
/// ```
/// let foo = Mutex::new(7);
/// {
///     let mut foo = foo.lock();
///     foo += 3;
/// }
/// assert_eq!(foo.lock(), 10);
/// ```
#[derive(Debug, Default)]
pub struct Mutex<T, L: Lock = sync::Primitive> {
    value: UnsafeCell<T>,
    lock: L,
}

// The only access to a Mutex's value is MutexGuard, so safety is guaranteed here.
// Requiring T to be Send-able prohibits implementing Sync for types like Mutex<*mut T>.
unsafe impl<T: Send, L: Lock> Sync for Mutex<T, L> {}
unsafe impl<T: Send, L: Lock> Send for Mutex<T, L> {}

impl<T, L: Lock> Mutex<T, L> {
    /// Creates a mutex in an unlocked state ready for use.
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            lock: L::default(),
        }
    }

    /// Acquires a mutex, blocking the current thread until it is able to do so.
    pub fn lock(&self) -> MutexGuard<'_, T, L> {
        self.lock.acquire();
        MutexGuard(self)
    }
}

/// An RAII implementation of a “scoped lock” of a mutex.
/// When this structure is dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through
/// this guard via its Deref and DerefMut implementations.
pub struct MutexGuard<'a, T, L: Lock>(&'a Mutex<T, L>);

unsafe impl<T: Sync, L: Lock> Sync for MutexGuard<'_, T, L> {}

impl<T, L: Lock> Deref for MutexGuard<'_, T, L> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.value.get() }
    }
}

impl<T, L: Lock> DerefMut for MutexGuard<'_, T, L> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.value.get() }
    }
}

impl<T, L: Lock> Drop for MutexGuard<'_, T, L> {
    fn drop(&mut self) {
        self.0.lock.release();
    }
}

// Useful in Condvar
impl<T, L: Lock> MutexGuard<'_, T, L> {
    pub(super) fn release(&self) {
        self.0.lock.release();
    }

    pub(super) fn acquire(&self) {
        self.0.lock.acquire();
    }
}
