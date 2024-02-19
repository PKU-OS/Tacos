//! Memory Management
//!
//! Kernel is running on virtual memory, which begins from [mem::VM_BASE].
//! Physical memory begins from [mem::PM_BASE].
//!
//! There exist an one-to-one map from Kernel virtual memory(kvm) to physical
//! memory(pm): kvm = pm + [mem::OFFSET].
//!

pub mod layout;
pub mod malloc;
pub mod pagetable;
pub mod palloc;
pub mod userbuf;
mod utils;

pub use self::layout::*;
pub use self::malloc::{kalloc, kfree};
pub use self::pagetable::*;
pub use self::palloc::Palloc;
pub use self::utils::*;
