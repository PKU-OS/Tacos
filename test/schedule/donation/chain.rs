use super::*;

const DEPTH: usize = 8;
static mut EXIT_STATUS: [i8; 2 * DEPTH + 1] = [0; 2 * DEPTH + 1];

fn interloop(tid: usize) {
    unsafe {
        assert_eq!(
            EXIT_STATUS[tid + 1],
            1,
            "Thread {} should exit later than thread {}",
            tid,
            tid + 1
        );
        EXIT_STATUS[tid] = 1;
    }
}

fn donor(tid: usize, first: Arc<Sleep>, second: Arc<Sleep>) {
    if tid < 2 * (DEPTH - 1) {
        first.acquire();
    }

    second.acquire();
    second.release();

    let expected = ((DEPTH - 1) * 2) as u32;
    let priority = get_priority();
    assert_eq!(
        expected, priority,
        "Thread {} should have priority {}. Actual priority {}.",
        tid, expected, priority
    );

    if tid < 2 * (DEPTH - 1) {
        first.release();

        unsafe {
            assert_eq!(
                EXIT_STATUS[tid + 1],
                1,
                "Thread {} should exit later than thread {}",
                tid,
                tid + 1
            );
        }
    }

    unsafe {
        EXIT_STATUS[tid] = 1;
    }

    let expected = tid as u32;
    let priority = get_priority();
    assert_eq!(
        expected, priority,
        "Thread {} should finish with priority {}. Actual priority {}.",
        tid, expected, priority
    );
}

pub fn main() {
    let locks: [_; DEPTH] = core::array::from_fn(|_| Arc::new(Sleep::default()));

    set_priority(PRI_MIN);

    locks[0].acquire();

    for p in 1..DEPTH {
        let expected = 2 * p as u32;
        let first = Arc::clone(&locks[p]);
        let second = Arc::clone(&locks[p - 1]);

        Builder::new(move || donor(expected as usize, first, second))
            .name("Donor")
            .priority(expected)
            .spawn();

        let priority = get_priority();
        assert_eq!(
            expected, priority,
            "Main should have priority {}. Actual priority {}.",
            expected, priority
        );

        Builder::new(move || interloop(expected as usize - 1))
            .name("interloop")
            .priority(expected - 1)
            .spawn();
    }

    locks[0].release();

    let expected = PRI_MIN;
    let priority = get_priority();
    assert_eq!(
        expected, priority,
        "Main thread should finish with priority {}. Actual priority {}.",
        expected, priority
    );

    pass();
}
