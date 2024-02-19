use core::sync::atomic::{AtomicBool, Ordering::SeqCst};

use crate::sbi::interrupt;
use crate::sync::Lock;

/// Spin lock.
#[derive(Debug, Default)]
pub struct Spin(AtomicBool);

impl Spin {
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }
}

impl Lock for Spin {
    fn acquire(&self) {
        while self.0.fetch_or(true, SeqCst) {
            assert!(interrupt::get(), "may block");
        }
    }

    fn release(&self) {
        self.0.store(false, SeqCst);
    }
}
