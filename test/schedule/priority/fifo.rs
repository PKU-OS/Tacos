use alloc::sync::Arc;
use alloc::vec::Vec;

use super::pass;
use crate::sbi::interrupt;
use crate::thread::*;

const THREAD_CNT: usize = 16;
const ITERS: usize = 16;

fn thread_func(tid: usize, v: Arc<Mutex<Vec<usize>>>) {
    interrupt::set(false);

    for _ in 0..ITERS {
        v.lock().push(tid);
        schedule();
    }
}

pub fn main() {
    interrupt::set(false);

    let log = Arc::new(Mutex::new(Vec::new()));
    set_priority(PRI_DEFAULT + 2);

    for tid in 0..THREAD_CNT {
        let v = Arc::clone(&log);
        Builder::new(move || thread_func(tid, v))
            .name("simple thread")
            .priority(PRI_DEFAULT + 1)
            .spawn();
    }

    set_priority(PRI_DEFAULT);

    let v = log.lock();

    assert_eq!(THREAD_CNT * ITERS, v.len(), "Missing logs!");

    let slice = &v[0..THREAD_CNT];
    for s in v.chunks(THREAD_CNT) {
        assert_eq!(s, slice, "Not fifo schedule!");
    }

    pass();
}
