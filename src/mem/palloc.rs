//! Global Page Allocator

use core::cmp::min;

use crate::mem::utils::*;
use crate::sync::{Intr, Lazy, Mutex};

// BuddyAllocator allocates at most `1<<MAX_ORDER` pages at a time
const MAX_ORDER: usize = 8;
// How many pages are there in the user memory pool
const USER_POOL_LIMIT: usize = 256;

/// Buddy Allocator. It allocates and deallocates memory page-wise.
#[derive(Debug)]
struct BuddyAllocator {
    /// The i-th free list is in charge of memory chunks of 2^i pages
    free_lists: [InMemList; MAX_ORDER + 1],
    /// How many memory does the buddy allocator control
    total: usize,
    /// The number of pages allocated
    allocated: usize,
}

impl BuddyAllocator {
    /// This struct can not be moved due to self reference.
    /// So, construct it and then call `init`.
    const fn empty() -> Self {
        Self {
            free_lists: [InMemList::new(); MAX_ORDER + 1],
            total: 0,
            allocated: 0,
        }
    }

    /// Take the memory segmant from `start` to `end` into page allocator's record
    unsafe fn insert_range(&mut self, start: usize, end: usize) {
        let start = round_up(start, PG_SIZE);
        let end = round_down(end, PG_SIZE);
        self.total += end - start;

        let mut current_start: usize = start;
        while current_start < end {
            // find the biggest alignment of `current_start`
            let size = min(
                1 << current_start.trailing_zeros(),
                prev_power_of_two(end - current_start),
            );
            let order = size.trailing_zeros() as usize - PG_SHIFT;
            // The order we found cannot exceed the preset maximun order
            let order = min(order, MAX_ORDER);
            self.free_lists[order].push(current_start as *mut usize);
            current_start += (1 << order) * PG_SIZE;
        }
    }

    /// Allocate n pages and returns the virtual address.
    unsafe fn alloc(&mut self, n: usize) -> *mut u8 {
        assert!(n <= 1 << MAX_ORDER, "request is too large");

        let order = n.next_power_of_two().trailing_zeros() as usize;
        for i in order..self.free_lists.len() {
            // Find the first non-empty list
            if !self.free_lists[i].is_empty() {
                // Split buffers (from large to small groups)
                for j in (order..i).rev() {
                    // Try to find a large block of group j+1 and then
                    // split it into two blocks of group j
                    if let Some(block) = self.free_lists[j + 1].pop() {
                        let half = (block as usize + (1 << j) * PG_SIZE) as *mut usize;
                        self.free_lists[j].push(half);
                        self.free_lists[j].push(block);
                    }
                }
                self.allocated += 1 << order;
                return self.free_lists[order].pop().unwrap().cast();
            }
        }

        unreachable!("memory is exhausted");
    }

    /// Deallocate a chunk of pages
    unsafe fn dealloc(&mut self, ptr: *mut u8, n: usize) {
        let order = n.next_power_of_two().trailing_zeros() as usize;
        self.free_lists[order].push(ptr.cast());

        // Merge free lists
        let mut curr_ptr = ptr as usize;
        let mut curr_order = order;

        while curr_order < MAX_ORDER {
            // Find the buddy block of the current block
            let buddy = curr_ptr ^ (1 << (curr_order + PG_SHIFT));
            // Try to find and merge blocks
            if let Some(blk) = self.free_lists[curr_order]
                .iter_mut()
                .find(|blk| blk.value() as usize == buddy)
            {
                blk.pop();
                // Merge two blocks into a bigger one
                self.free_lists[curr_order].pop();
                curr_ptr = min(curr_ptr, buddy);
                self.free_lists[curr_order + 1].push(curr_ptr as *mut _);
                // Attempt to form a even bigger block in the next iteration
                curr_order += 1;
            } else {
                break;
            }
        }

        self.allocated -= 1 << order;
    }
}

/// Wraps the buddy allocator
pub struct Palloc(Lazy<Mutex<BuddyAllocator, Intr>>);

unsafe impl Sync for Palloc {}

impl Palloc {
    /// Initialize the page-based allocator
    pub unsafe fn init(start: usize, end: usize) {
        Self::instance().lock().insert_range(start, end);
    }

    /// Allocate n pages of a consecutive memory segment
    pub unsafe fn alloc(n: usize) -> *mut u8 {
        Self::instance().lock().alloc(n)
    }

    /// Free n pages of memory starting at `ptr`
    pub unsafe fn dealloc(ptr: *mut u8, n: usize) {
        Self::instance().lock().dealloc(ptr, n)
    }

    fn instance() -> &'static Mutex<BuddyAllocator, Intr> {
        static PALLOC: Palloc = Palloc(Lazy::new(|| Mutex::new(BuddyAllocator::empty())));

        &PALLOC.0
    }
}

pub struct UserPool(Lazy<Mutex<BuddyAllocator, Intr>>);

// UserPool has only one instance, so it's safe to claim it as `Sync`
unsafe impl Sync for UserPool {}

impl UserPool {
    /// Allocate n pages of consecutive space
    pub unsafe fn alloc_pages(n: usize) -> *mut u8 {
        Self::instance().lock().alloc(n)
    }

    /// Free n pages of memory starting at `ptr`
    pub unsafe fn dealloc_pages(ptr: *mut u8, n: usize) {
        Self::instance().lock().dealloc(ptr, n)
    }

    fn instance() -> &'static Mutex<BuddyAllocator, Intr> {
        static USERPOOL: UserPool = UserPool(Lazy::new(|| unsafe {
            let mut alloc = BuddyAllocator::empty();
            let chunk_size = 1 << MAX_ORDER;
            for _ in (0..USER_POOL_LIMIT).step_by(chunk_size) {
                let start = Palloc::alloc(chunk_size) as usize;
                alloc.insert_range(start, start + chunk_size * PG_SIZE);
            }
            Mutex::new(alloc)
        }));

        &USERPOOL.0
    }
}
