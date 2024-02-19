//! Kernel memory allocator

use core::alloc::{GlobalAlloc, Layout};
use core::cmp::max;
use core::mem::size_of;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use crate::mem::palloc::Palloc;
use crate::mem::utils::*;
use crate::sync::{Intr, Lazy, Mutex};

const ARENA_MAGIC: u32 = 0x9a548eed;
const MAX_BLKSIZE: usize = PG_SIZE / 4;

/// Metadata for a page of memory
///
/// An arena can hold memory blocks of sizes that are up to [`MAX_BLKSIZE`]
#[repr(C)]
struct Arena {
    /// Always set to [`ARENA_MAGIC`]
    magic: u32,
    /// The number of free blocks in this page; or number of pages for big blocks.
    free_cnt: u32,
    /// Points to an arena's corresponding descriptor
    desc: *const Desc,
}

impl Arena {
    /// Retrieves an arena from any block lying in this arena
    unsafe fn from_block(blk: *const u8) -> &'static mut Self {
        let arena = ((blk as usize & !PG_MASK) as *mut Self).as_mut().unwrap();
        assert_eq!(arena.magic, ARENA_MAGIC);
        assert!(!arena.desc.is_null());
        // Make sure the block is aligned to its size
        assert!(blk as usize % arena.desc.as_ref().unwrap().block_size == 0);
        arena
    }

    /// Gets a block from an arena by the given index
    unsafe fn get_block(&self, idx: usize) -> *const u8 {
        assert_eq!(self.magic, ARENA_MAGIC);
        assert!(idx < self.desc.as_ref().unwrap().blocks_per_arena);

        let addr = self as *const _ as usize;
        assert!(addr % PG_SIZE == 0);

        let block_size = self.desc.as_ref().unwrap().block_size;
        let offset = round_up(size_of::<Self>(), block_size);

        (addr + offset + idx * block_size) as *const u8
    }
}

/// Heap Memory Descriptor
///
/// A descriptor represents a pool of heap memory where
/// every memory block is of the same size. It's guaranteed
/// that any blocks in a descriptor is aligned to its size.
struct Desc {
    block_size: usize,
    blocks_per_arena: usize,
    free_list: InMemList,
    allocated: usize,
    free: usize,
    total: usize,
}

impl Desc {
    /// Creates a new descriptor
    const fn new(block_size: usize) -> Self {
        Self {
            block_size,
            blocks_per_arena: (PG_SIZE - round_up(size_of::<Self>(), block_size)) / block_size,
            free_list: InMemList::new(),
            allocated: 0,
            free: 0,
            total: 0,
        }
    }

    /// Allocates a free memory block from this descriptor
    unsafe fn alloc(&mut self) -> *mut u8 {
        if self.free_list.is_empty() {
            let mut arena: NonNull<Arena> = NonNull::new_unchecked(Palloc::alloc(1)).cast();
            arena.as_mut().magic = ARENA_MAGIC;
            arena.as_mut().desc = self as *const Self;
            arena.as_mut().free_cnt = self.blocks_per_arena as u32;

            for i in 0..self.blocks_per_arena {
                let block = arena.as_ref().get_block(i);
                self.free_list.push(block as *mut _);
            }

            self.total += PG_SIZE;
            self.free += self.block_size * self.blocks_per_arena;
        }

        self.allocated += self.block_size;
        self.free -= self.block_size;

        let block = self.free_list.pop().unwrap() as *mut u8;
        let arena = Arena::from_block(block);
        arena.free_cnt -= 1;

        block
    }

    /// Returns a memory block back to this descriptor
    ///
    /// Pages in the arena will not be returned to the page allocator.
    unsafe fn dealloc(&mut self, ptr: *mut u8) {
        self.free_list.push(ptr.cast());
        self.allocated -= self.block_size;
        self.free += self.block_size;

        let arena = Arena::from_block(ptr);
        arena.free_cnt += 1;
    }
}

/// An elastic kernel heap. It's a memory allocator more fine-grained than [`Palloc`].
///
/// It's able to serve requests of any size. For requests no larger than 1024 bytes,
/// they are assigned to ["descriptors"](`Desc`) that manages blocks of that size.
/// Otherwise, the request will go directly to [`Palloc`].
pub struct Heap {
    descs: [Mutex<Desc, Intr>; 8],
    /// The sum of requests that can't fit in any descriptors,
    /// namely requests that are larger than [`MAX_BLKSIZE`])
    spilled_alloc: AtomicUsize,
    /// The total amount of pages used for spilled requests
    spilled_total: AtomicUsize,
}

impl Heap {
    /// Get the reference of the unique kernel heap
    pub fn get() -> &'static Heap {
        static HEAP: Lazy<Heap> = Lazy::new(|| Heap {
            // block_size ranges from 8 to 1024
            descs: core::array::from_fn::<_, 8, _>(|idx| Mutex::new(Desc::new(1 << (idx + 3)))),
            spilled_alloc: AtomicUsize::new(0),
            spilled_total: AtomicUsize::new(0),
        });

        &HEAP
    }

    /// Allocates a memory block that is in align with the layout.
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            // return an invalid but well-aligned pointer for zero-sized requests
            return NonNull::dangling().as_ptr();
        }

        assert!(layout.align().is_power_of_two());
        // not able to handle align requests that are larger than one page
        assert!(layout.align() <= PG_SIZE);

        let size = max(layout.size().next_power_of_two(), layout.align());
        if size <= MAX_BLKSIZE {
            return self.descs[size.trailing_zeros().saturating_sub(3) as usize]
                .lock()
                .alloc();
        }

        // delegate the allocation request to PALLOC
        let pages = (layout.size() + PG_SIZE - 1) / PG_SIZE;
        self.spilled_alloc.fetch_add(size, Relaxed);
        self.spilled_total.fetch_add(pages, Relaxed);

        Palloc::alloc(pages)
    }

    /// Deallocates a memory block
    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let size = max(layout.size().next_power_of_two(), layout.align());
        if size <= MAX_BLKSIZE {
            return self.descs[size.trailing_zeros().saturating_sub(3) as usize]
                .lock()
                .dealloc(ptr);
        }

        // delegate the deallocation request to PALLOC
        let pages = (layout.size() + PG_SIZE - 1) / PG_SIZE;
        self.spilled_alloc.fetch_sub(size, Relaxed);
        self.spilled_total.fetch_sub(pages, Relaxed);

        assert!(ptr as usize % PG_SIZE == 0);
        Palloc::dealloc(ptr, pages);
    }

    /// The amount of free memory in heap
    pub fn free(&self) -> usize {
        self.descs.iter().fold(0, |free, desc| {
            let desc = desc.lock();
            free + desc.free
        })
    }

    /// How much memory was allocated from the "heap"
    pub fn allocated(&self) -> usize {
        self.descs.iter().fold(0, |allocated, desc| {
            let desc = desc.lock();
            allocated + desc.allocated
        }) + self.spilled_alloc.load(Relaxed)
    }

    /// The size of heap, including all metadata
    pub fn total(&self) -> usize {
        self.descs.iter().fold(0, |total, desc| {
            let desc = desc.lock();
            total + desc.total
        }) + self.spilled_total.load(Relaxed) * PG_SIZE
    }
}

unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}

pub struct Malloc;

unsafe impl GlobalAlloc for Malloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        Heap::get().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        Heap::get().dealloc(ptr, layout)
    }
}

pub fn kalloc(size: usize, align: usize) -> *mut u8 {
    unsafe { MALLOC.alloc(Layout::from_size_align(size, align).unwrap()) }
}

pub fn kfree(ptr: *mut u8, size: usize, align: usize) {
    unsafe { MALLOC.dealloc(ptr, Layout::from_size_align(size, align).unwrap()) }
}

#[global_allocator]
static MALLOC: Malloc = Malloc;
