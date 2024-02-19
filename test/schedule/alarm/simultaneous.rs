use super::*;

const THREAD_CNT: usize = 3;
const ITERS: usize = 10;
const TEST_START: i64 = 10;

static mut WAKE_TICKS: [[i64; ITERS]; THREAD_CNT] = [[0; ITERS]; THREAD_CNT];

pub fn main() {
    for tid in 0..THREAD_CNT {
        thread::spawn("Sleeper", move || sleeper(tid));
    }

    // Wait long enough for all the threads to finish.
    thread::sleep((20 + ITERS) as i64);

    for i in 0..ITERS {
        let wake_time = TEST_START + (i + 1) as i64;
        for tid in 0..THREAD_CNT {
            let real_time = unsafe { WAKE_TICKS[tid][i] };
            assert_eq!(
                real_time, wake_time,
                "Sleeper {} is supposed to wake up at tick {}, but wakes up at tick {}",
                tid, wake_time, real_time
            );
        }
    }

    pass();
}

fn sleeper(tid: usize) {
    // Make sure we're at the beginning of a tiemr tick.
    thread::sleep(1);

    for i in 0..ITERS {
        let until: i64 = TEST_START + (i + 1) as i64;
        thread::sleep(until - timer::timer_ticks());
        unsafe { WAKE_TICKS[tid][i] = timer::timer_ticks() };
        thread::schedule();
    }
}
