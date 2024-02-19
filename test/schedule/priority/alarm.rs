use crate::sbi::timer::{timer_elapsed, timer_ticks};
use crate::sync::Semaphore;

use super::*;

const THREAD_CNT: usize = 10;
const WAKE_TIME: i64 = 5 * TICKS_PER_SEC as i64;
static mut EXIT_STATUS: [i8; THREAD_CNT + 1] = [0; THREAD_CNT + 1];

const EXPECTED_STATUS: [[i8; THREAD_CNT + 1]; THREAD_CNT + 1] = [
    // Thread 0 is the main thread. Every other thread should have exited.
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    // When thread 1 exit, thread 0 is running.
    [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1],
    // When thread 2 exit, thread 1 should have exited.
    [0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1],
    [0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1],
    [0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1],
    [0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0],
];

fn alarm_priority_thread(tid: usize, sema: Arc<Semaphore>) {
    // Busy-wait until the current time changes.
    let start = timer_ticks();
    while timer_elapsed(start) == 0 {}

    sleep(WAKE_TIME - start);

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

    sema.up();
}

pub fn main() {
    let s = Arc::new(Semaphore::new(0));

    // Main thread has tid 0.
    for tid in 1..=THREAD_CNT {
        let priority = PRI_DEFAULT - ((tid as u32 + 4) % 10) - 1;
        let s = s.clone();
        Builder::new(move || alarm_priority_thread(tid, s))
            .name("child")
            .priority(priority)
            .spawn();
    }

    set_priority(PRI_MIN);

    for _ in 1..=THREAD_CNT {
        s.down();
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
