//! Synchronization and Interior Mutability
//!

pub mod condvar;
pub mod intr;
pub mod lazy;
pub mod mutex;
pub mod once;
pub mod sema;
pub mod sleep;
pub mod spin;

pub use self::condvar::Condvar;
pub use self::intr::Intr;
pub use self::lazy::Lazy;
pub use self::mutex::{Mutex, MutexGuard};
pub use self::once::{Once, OnceCell};
pub use self::sema::Semaphore;
pub use self::sleep::Sleep;
pub use self::spin::Spin;
pub type Primitive = sleep::Sleep;

/// Lock trait is used to synchronize between different threads, the implementation
/// of Mutex relies on this trait. Of cource we want every struct that implements
/// to be [Sync], thus we can send its reference through different threads and
/// use the reference to synchronize.
///
/// Check out comments in [`Mutex`] for more details.
pub trait Lock: Default + Sync + 'static {
    fn acquire(&self);
    fn release(&self);
}
