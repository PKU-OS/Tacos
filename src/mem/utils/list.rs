use core::marker::PhantomData;

/// A single linked list designed for memory allocators
#[derive(Debug, Clone, Copy)]
pub struct InMemList {
    head: *mut usize,
}

impl InMemList {
    /// Creates a new linked list
    pub const fn new() -> Self {
        Self {
            head: core::ptr::null_mut(),
        }
    }

    /// Is this list empty or not
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    /// Pushes an item to the front of a list
    pub unsafe fn push(&mut self, item: *mut usize) {
        *item = self.head as usize;
        self.head = item;
    }

    /// Removes an item from the front of a list
    pub fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,
            false => {
                // Advance head pointer
                let popped_item = self.head;
                self.head = unsafe { *popped_item as *mut usize };
                Some(popped_item)
            }
        }
    }

    /// Return an mutable iterator over the items in the list
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            phantom: Default::default(),
        }
    }
}

pub struct IterMut<'a> {
    prev: *mut usize,
    curr: *mut usize,
    phantom: PhantomData<&'a mut InMemList>,
}

/// Represent a mutable node in `LinkedList`
pub struct ListNode {
    prev: *mut usize,
    curr: *mut usize,
}

impl ListNode {
    /// Remove the node from the list
    pub fn pop(self) -> *mut usize {
        // Skip the current one
        unsafe {
            *(self.prev) = *(self.curr);
        }
        self.curr
    }

    /// Returns the pointed address
    pub fn value(&self) -> *mut usize {
        self.curr
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = ListNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let res = ListNode {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            self.curr = unsafe { *self.curr as *mut usize };
            Some(res)
        }
    }
}
