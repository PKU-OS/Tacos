mod list;

pub use self::list::{InMemList, IterMut};

use crate::mem::layout::VM_OFFSET;

pub const PG_SHIFT: usize = 12;
pub const PG_MASK: usize = (1 << PG_SHIFT) - 1;
pub const PG_SIZE: usize = 1 << PG_SHIFT;

/// Physical Address
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub fn value(&self) -> usize {
        self.0
    }

    /// Physical page number
    pub fn ppn(&self) -> usize {
        self.0 >> PG_SHIFT
    }

    /// Translates physical address to virtual address.
    pub fn into_va(&self) -> usize {
        self.0 + VM_OFFSET
    }

    pub fn from_pa(pa: usize) -> Self {
        Self(pa)
    }
}

// Convert a virtual address(stored in usize) to a physical address.
impl From<usize> for PhysAddr {
    fn from(va: usize) -> Self {
        assert!(in_kernel_space(va));
        Self(va - VM_OFFSET)
    }
}

// Convert a pointer(in virtual address) to a physical address.
impl<T> From<*const T> for PhysAddr {
    fn from(pa: *const T) -> Self {
        PhysAddr::from(pa as usize)
    }
}

// Convert a pointer(in virtual address) to a physical address.
impl<T> From<*mut T> for PhysAddr {
    fn from(pa: *mut T) -> Self {
        PhysAddr::from(pa as usize)
    }
}

/// Checks if a virtual memory address is valid (lies in the kernel space)
/// Contains kernel, sbi, mmio and plic memory.
pub fn in_kernel_space(va: usize) -> bool {
    va & VM_OFFSET == VM_OFFSET
}

pub const fn div_round_up(n: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    round_up(n, align) / align
}

pub const fn round_up(n: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    (n + align - 1) & !(align - 1)
}

pub const fn round_down(n: usize, align: usize) -> usize {
    assert!(align.is_power_of_two());
    n & !(align - 1)
}

pub const fn prev_power_of_two(num: usize) -> usize {
    1 << (64 - num.leading_zeros() as usize - 1)
}

/// Aligned to page boundary.
pub trait PageAlign: Copy + Eq + Sized {
    fn floor(self) -> Self;

    fn ceil(self) -> Self;

    fn is_aligned(self) -> bool {
        self.floor() == self
    }
}

impl PageAlign for usize {
    fn floor(self) -> Self {
        (self >> PG_SHIFT) << PG_SHIFT
    }

    fn ceil(self) -> Self {
        ((self + PG_SIZE - 1) >> PG_SHIFT) << PG_SHIFT
    }
}

impl PageAlign for PhysAddr {
    fn floor(self) -> Self {
        PhysAddr(self.0.floor())
    }

    fn ceil(self) -> Self {
        PhysAddr(self.0.ceil())
    }
}
