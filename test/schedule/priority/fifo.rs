use alloc::sync::Arc;
use alloc::vec::Vec;

use super::pass;
use crate::sbi::interrupt;
use crate::sync::Condvar;
use crate::thread::*;

const THREAD_CNT: usize = 16;
const ITERS: usize = 16;

fn thread_func(
    tid: usize,
    v: Arc<Mutex<Vec<usize>>>,
    cvar: Arc<Condvar>,
    mutex: Arc<Mutex<usize>>,
) {
    interrupt::set(false);

    let mut guard = mutex.lock();
    *guard += 1;
    while *guard != THREAD_CNT {
        cvar.wait(&mut guard);
    }
    cvar.notify_all();
    drop(guard);

    for _ in 0..ITERS {
        v.lock().push(tid);
        schedule();
    }
}

pub fn main() {
    interrupt::set(false);

    let log = Arc::new(Mutex::new(Vec::new()));
    let cvar = Arc::new(Condvar::new());
    let mutex = Arc::new(Mutex::new(0));
    set_priority(PRI_DEFAULT + 2);

    for tid in 0..THREAD_CNT {
        let v = Arc::clone(&log);
        let c = Arc::clone(&cvar);
        let m = Arc::clone(&mutex);
        Builder::new(move || thread_func(tid, v, c, m))
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
