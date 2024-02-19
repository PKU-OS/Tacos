//!
//! Creates `THREAD_CNT` threads, each of which sleeps a different, fixed
//! duration, `iters` times.  Records the wake-up order and verifies
//! that it is valid.
//!

use super::*;

const THREAD_CNT: usize = 5;
const ITERS_MAX: usize = 7;
const TEST_START: i64 = 10;
static mut WAKE_TICKS: [[i64; ITERS_MAX]; THREAD_CNT] = [[0; ITERS_MAX]; THREAD_CNT];

fn sleeper(tid: usize, iters: usize) {
    // Make sure we're at the beginning of a tiemr tick.
    thread::sleep(1);

    // Sleeper with different id has different sleep durations.
    let duration = tid + 1;

    // Sleep & record wake up time.
    for i in 0..iters {
        let until: i64 = TEST_START + (duration * (i + 1)) as i64;
        thread::sleep(until - timer::timer_ticks());
        unsafe { WAKE_TICKS[tid][i] = timer::timer_ticks() };
        thread::schedule();
    }
}

fn test_sleeper(iters: usize) {
    for tid in 0..THREAD_CNT {
        // TODO: If thread.name use String, the name could be more precise.
        thread::spawn("Sleeper", move || sleeper(tid, iters));
    }

    // Wait long enough for all the threads to finish.
    thread::sleep((20 + THREAD_CNT * iters) as i64);

    for i in 0..iters {
        for tid in 0..THREAD_CNT {
            let duration = tid + 1;
            let wake_time = TEST_START + (duration * (i + 1)) as i64;
            let real_time = unsafe { WAKE_TICKS[tid][i] };

            assert!(
                wake_time > 0,
                "Sleeper {} is supposed to wake up at tick {}, but it doesn't.",
                tid,
                wake_time,
            );

            assert_eq!(
                real_time, wake_time,
                "Sleeper {} is supposed to wake up at tick {}, but wakes up at tick {}.",
                tid, wake_time, real_time
            );
        }
    }

    pass();
}

pub fn main() {
    test_sleeper(7);
}

pub mod single {
    use super::test_sleeper;

    pub fn main() {
        test_sleeper(1);
    }
}
