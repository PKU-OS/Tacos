//! Manages kernel and user page tables.
//!
//! ## Risc-v Pagetable Scheme
//! The risc-v Sv39 scheme has three levels of page-table pages.
//! A page-table page contains 512 64-bit PTEs.
//! A 64-bit virtual address is split into five fields:
//!   39..63 -- each be the same to bit 38.
//!   30..38 -- 9 bits of level-2 index.
//!   21..29 -- 9 bits of level-1 index.
//!   12..20 -- 9 bits of level-0 index.
//!    0..11 -- 12 bits of byte offset within the page.
//! ref: <https://five-embeddev.com/riscv-isa-manual/latest/supervisor.html#sec:sv39>
//!
//! ## Design
//! The struct [`PageTable`] holds reference to a in-momory Sv39 page table. And
//! [`KernelPgTable`], which has only one instance, manages kernel memory. To build
//! a user page table, you should always start from calling [`KernelPgTable::clone`].
//! This method replicates the kernel page table as a template for all user page tables.
//! Having kernel pages existing in all user memory spaces, there will be no need to
//! switch page table when doing a system call.

mod entry;

use core::ptr;
use core::{arch::asm, mem::transmute};

use crate::mem::{
    layout::{MMIO_BASE, PLIC_BASE, VM_BASE},
    malloc::{kalloc, kfree},
    palloc::UserPool,
    utils::{PageAlign, PhysAddr, PG_SIZE},
};
use crate::mem::{KERN_BASE, VM_OFFSET};
use crate::sync::OnceCell;

pub use self::entry::*;

/// Reference to a in-memory page table
pub struct PageTable {
    /// Each page table has 512 entries.
    entries: &'static mut [Entry; Self::NENTRY],
}

impl PageTable {
    const NENTRY: usize = 512;
    const PX_MASK: usize = Self::NENTRY - 1;
    const SV39_MODE: usize = 0x8 << 60;

    /// Activates `self` as the effective page table.
    pub fn activate(&self) {
        // SATP layout: MODE(WARL) 4 bit | ASID(WARL) 16 bits | PPN(WARL) 44 bits
        let satp: usize = PhysAddr::from(self.entries.as_ptr()).ppn() | Self::SV39_MODE;
        unsafe {
            asm!(
                "sfence.vma zero, zero",
                "csrw satp, {satp}",
                "sfence.vma zero, zero",
                satp = in(reg) satp
            );
        }
    }

    /// Maps `pa` to `va` and allocates page table when necessary.
    pub fn map(&mut self, pa: PhysAddr, va: usize, size: usize, flag: PTEFlags) {
        assert!(pa.is_aligned() && va.is_aligned(), "address misaligns");

        let pa_end = pa.value() + size;
        let (mut pa, mut va) = (pa.value(), va);

        while pa < pa_end {
            let mut l1_table = self.walk_or_create(Self::px(2, va), flag.contains(PTEFlags::G));
            let l0_table = l1_table.walk_or_create(Self::px(1, va), flag.contains(PTEFlags::G));
            l0_table.entries[Self::px(0, va)] = Entry::new(PhysAddr::from_pa(pa), flag);
            pa += PG_SIZE;
            va += PG_SIZE;
        }
    }

    /// Finds the corresponding entry by the given virtual address
    pub fn get_pte(&self, va: usize) -> Option<&Entry> {
        self.walk(Self::px(2, va)).and_then(|l1_table| {
            l1_table
                .walk(Self::px(1, va))
                .map(|l0_table| l0_table.entries.get(Self::px(0, va)).unwrap())
        })
    }

    /// Free all memory used by this pagetable back to where they were allocated.
    pub unsafe fn destroy(&mut self) {
        unsafe fn destroy_imp(pgt: &mut PageTable, level: usize) {
            assert!((0..=2).contains(&level));

            pgt.entries
                .iter()
                .filter(|entry| entry.is_valid() && !entry.is_global())
                .for_each(|entry| {
                    let va = entry.pa().into_va();
                    if entry.is_leaf() {
                        UserPool::dealloc_pages(va as *mut _, 1 << (9 * level));
                    } else {
                        destroy_imp(&mut PageTable::from_raw(va as *mut _), level - 1);
                    }
                });
            kfree(pgt.entries.as_mut_ptr().cast(), PG_SIZE, PG_SIZE);
        }
        destroy_imp(self, 2);
    }

    /// Allocates a page to build a new page table
    fn new() -> Self {
        let page = kalloc(PG_SIZE, PG_SIZE);

        unsafe {
            // Clear the pagetable. A page table is exactly the size of
            // a page and must always be aligned to a page boundary.
            ptr::write_bytes(page, 0, PG_SIZE);

            Self::from_raw(page.cast())
        }
    }

    /// Interprets a page of raw memory as a page table
    unsafe fn from_raw(entries: *mut Entry) -> Self {
        assert!((entries as usize).is_aligned());
        Self {
            entries: transmute(entries),
        }
    }

    fn walk(&self, index: usize) -> Option<PageTable> {
        self.entries
            .get(index)
            .filter(|e| e.is_valid())
            .map(|e| unsafe { Self::from_raw(e.pa().into_va() as *mut _) })
    }

    fn walk_or_create(&mut self, index: usize, is_global: bool) -> PageTable {
        let mut flag = PTEFlags::V;
        flag.set(PTEFlags::G, is_global);

        self.walk(index).unwrap_or_else(|| {
            let table = PageTable::new();
            let pa = PhysAddr::from(table.entries.as_ptr());
            self.entries[index] = Entry::new(pa, flag);
            table
        })
    }

    fn px(level: u32, va: usize) -> usize {
        fn px_shift(level: u32) -> usize {
            use crate::mem::utils::PG_SHIFT;

            PG_SHIFT + 9 * level as usize
        }

        (va >> px_shift(level)) & Self::PX_MASK
    }
}

pub struct KernelPgTable(OnceCell<PageTable>);

impl KernelPgTable {
    pub fn get() -> &'static PageTable {
        Self::instance().get()
    }

    /// Clones entries in the kernel page table. Use them as a template for user page tables.
    /// This method ensures all kernel memory mappings exist in user memory space.
    pub fn clone() -> PageTable {
        let other = PageTable::new();
        other.entries.copy_from_slice(Self::get().entries);
        other
    }

    /// Initializes the kernel page table which manages `ram_size` bytes of memory
    pub fn init(ram_size: usize) {
        Self::instance().init(|| Self::init_inner(ram_size))
    }

    /// Set up all kernel page table entries.
    ///
    /// At the entrance of kernel, a crude page table was set up to support basic
    /// paging capability. To strengthen memory protection, it's necessary to set up
    /// a fine-grained page table.
    pub fn init_inner(ram_size: usize) -> PageTable {
        let mut root = PageTable::new();

        // Kernel's code and data exist in all memory spaces, therefore the global bit is set.
        let rx = PTEFlags::R | PTEFlags::X | PTEFlags::G | PTEFlags::V;
        let rw = PTEFlags::R | PTEFlags::W | PTEFlags::G | PTEFlags::V;

        extern "C" {
            fn etext();
        }

        let etext = etext as usize;
        let kr_base = KERN_BASE + VM_OFFSET;
        let kr_end = VM_BASE + ram_size;

        // map kernel text executable and read-only.
        root.map(PhysAddr::from_pa(KERN_BASE), kr_base, etext - kr_base, rx);

        // map kernel data and the physical RAM we'll make use of.
        root.map(PhysAddr::from(etext), etext, kr_end - etext, rw);

        // PLIC
        root.map(PhysAddr::from(PLIC_BASE), PLIC_BASE, 0x400000, rw);

        // virtio mmio disk interface
        root.map(PhysAddr::from(MMIO_BASE), MMIO_BASE, PG_SIZE, rw);

        root.activate();
        root
    }

    fn instance() -> &'static OnceCell<PageTable> {
        static PAGETABLE: KernelPgTable = KernelPgTable(OnceCell::new());

        &PAGETABLE.0
    }
}
