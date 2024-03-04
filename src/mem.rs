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

use core::mem::size_of;

pub use self::layout::*;
pub use self::malloc::{kalloc, kfree};
pub use self::pagetable::*;
pub use self::palloc::Palloc;
pub use self::utils::*;

pub fn get_pte(va: usize) -> Option<Entry> {
    match crate::thread::Manager::get().current.lock().pagetable {
        Some(ref pt) => pt.lock().get_pte(va).copied(),
        None => KernelPgTable::get().get_pte(va).copied(),
    }
}

/// Translate a virtual address (pointer, slice) to a kernel virtual address
/// if it's in user space. The translated user object is supposed to be in a page.
pub trait Translate: Sized {
    fn translate(self) -> Option<Self>;
}

fn in_same_page(va1: usize, va2: usize) -> bool {
    va1 / PG_SIZE == va2 / PG_SIZE
}

fn translate(va: usize, len: usize) -> Option<usize> {
    if in_kernel_space(va) {
        return Some(va);
    }

    if !in_same_page(va, va + len - 1) {
        return None;
    }

    let pageoff = va & 0xFFF;
    let pte = get_pte(va)?;
    Some(pte.pa().into_va() | pageoff)
}

impl<T> Translate for *const T {
    fn translate(self) -> Option<Self> {
        translate(self as usize, size_of::<T>()).map(|va| va as *const T)
    }
}

impl<T> Translate for *mut T {
    fn translate(self) -> Option<Self> {
        translate(self as usize, size_of::<T>()).map(|va| va as *mut T)
    }
}

impl<T> Translate for &[T] {
    fn translate(self) -> Option<Self> {
        let ptr = self.as_ptr();
        let len = self.len();
        translate(ptr as usize, len * size_of::<T>())
            .map(|va| va as *const T)
            .map(|ptr| unsafe { core::slice::from_raw_parts(ptr, len) })
    }
}

impl<T> Translate for &mut [T] {
    fn translate(self) -> Option<Self> {
        let ptr = self.as_mut_ptr();
        let len = self.len();
        translate(ptr as usize, len * size_of::<T>())
            .map(|va| va as *mut T)
            .map(|ptr| unsafe { core::slice::from_raw_parts_mut(ptr, len) })
    }
}
