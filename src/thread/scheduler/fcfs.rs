use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::thread::{Schedule, Thread};

/// FIFO scheduler.
#[derive(Default)]
pub struct Fcfs(VecDeque<Arc<Thread>>);

impl Schedule for Fcfs {
    fn register(&mut self, thread: Arc<Thread>) {
        self.0.push_front(thread)
    }

    fn schedule(&mut self) -> Option<Arc<Thread>> {
        self.0.pop_back()
    }
}
