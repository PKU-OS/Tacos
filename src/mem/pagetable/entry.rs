//! Page Table Entry

use crate::mem::utils::{PhysAddr, PG_SHIFT};

/// The format of Sv39 page table entry:
/// |  63-54 |  53-28 |  27-19 |  18-10 | 9-8 |7|6|5|4|3|2|1|0|
/// | Unused | PPN[2] | PPN[1] | PPN[0] | RSW |D|A|G|U|X|W|R|V|
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct Entry(usize);

bitflags::bitflags! {
    pub struct PTEFlags: usize {
        /// Valid
        const V = 0b0000_0001;
        /// Readable
        const R = 0b0000_0010;
        /// Writable
        const W = 0b0000_0100;
        /// Executable
        const X = 0b0000_1000;
        /// It this page accessible to user mode?
        const U = 0b0001_0000;
        /// Global mappings (are those that exist in all address spaces)
        const G = 0b0010_0000;
        /// Accessed
        const A = 0b0100_0000;
        /// Dirty
        const D = 0b1000_0000;
    }
}

impl Entry {
    const FLAG_SHIFT: usize = 10;
    const PPN_MASK: usize = (1 << 44) - 1;

    pub fn new(pa: PhysAddr, flags: PTEFlags) -> Entry {
        Entry((((pa.value() >> PG_SHIFT) & Self::PPN_MASK) << Self::FLAG_SHIFT) | flags.bits())
    }

    fn flag(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    fn ppn(&self) -> usize {
        self.0 >> Self::FLAG_SHIFT & Self::PPN_MASK
    }

    /// Physical address where the entry maps to
    pub fn pa(&self) -> PhysAddr {
        PhysAddr::from_pa(self.ppn() << PG_SHIFT)
    }

    pub fn is_valid(&self) -> bool {
        self.flag().contains(PTEFlags::V)
    }

    pub fn is_global(&self) -> bool {
        self.flag().contains(PTEFlags::G)
    }

    pub fn is_rwable(&self) -> bool {
        self.flag().contains(PTEFlags::R | PTEFlags::W)
    }

    pub fn is_user(&self) -> bool {
        self.flag().contains(PTEFlags::U)
    }

    /// A PTE is a leaf PTE when at least one bit in R, W and X
    /// is set; otherwise, it is a pointer to the next level of
    /// the page table.
    pub fn is_leaf(&self) -> bool {
        self.flag()
            .intersects(PTEFlags::R | PTEFlags::W | PTEFlags::X)
    }
}
