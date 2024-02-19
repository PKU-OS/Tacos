use crate::sync::{Condvar, Mutex};

use super::*;

const THREAD_CNT: usize = 10;
const WAKE_TIME: i64 = 5 * TICKS_PER_SEC as i64;
static mut EXIT_STATUS: [i8; THREAD_CNT + 1] = [0; THREAD_CNT + 1];

const EXPECTED_STATUS: [[i8; THREAD_CNT + 1]; THREAD_CNT + 1] = [
    // Similar with ./alarm.rs
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1],
    [0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1],
    [0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0],
    [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0],
];

fn cvar_priority_thread(tid: usize, pair: Arc<(Mutex<bool>, Condvar)>) {
    // Wait for signal.
    let (lock, cvar) = &*pair;
    let mut guard = lock.lock();
    cvar.wait(&mut guard);

    // We're notified by the Main thread and are going to exit.
    unsafe {
        // Check other thread's status before exit.
        assert_eq!(
            EXIT_STATUS, EXPECTED_STATUS[tid],
            "When thread {} exit, expected status is {:?}, but real status is {:?}.",
            tid, EXPECTED_STATUS[tid], EXIT_STATUS
        );

        // Mark self as exited.
        EXIT_STATUS[tid] = 1;
    }
}

pub fn main() {
    // Main thread has tid 0.
    let pair = Arc::new((Mutex::new(false), Condvar::new()));

    set_priority(PRI_MIN);

    for tid in 1..=THREAD_CNT {
        let priority = PRI_DEFAULT - ((tid as u32 + 6) % 10) - 1;
        let p = Arc::clone(&pair);
        Builder::new(move || cvar_priority_thread(tid, p))
            .name("child")
            .priority(priority)
            .spawn();
    }

    let (lock, cvar) = &*pair;

    for _ in 1..=THREAD_CNT {
        let _g = lock.lock();
        cvar.notify_one();
    }

    unsafe {
        assert_eq!(
            EXIT_STATUS, EXPECTED_STATUS[0],
            "When main thread {} exit, expected status is {:?}, but real status is {:?}.",
            0, EXPECTED_STATUS[0], EXIT_STATUS
        );
    }

    pass();
}
