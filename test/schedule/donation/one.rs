use super::*;

static mut EXIT_STATUS: [i8; 3] = [0; 3];

fn medium(lock: Arc<Sleep>) {
    lock.acquire();

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 0, 1], "Medium should exit after high.");
        EXIT_STATUS[1] = 1;
    };

    lock.release();
}

fn high(lock: Arc<Sleep>) {
    lock.acquire();

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 0, 0], "Hish should exit first.");
        EXIT_STATUS[2] = 1;
    };

    lock.release();
}

pub fn main() {
    let lock = Arc::new(Sleep::default());

    lock.acquire();

    let create_and_check = |f: fn(Arc<Sleep>), tid| {
        let l = Arc::clone(&lock);
        let expected = PRI_DEFAULT + tid;
        Builder::new(move || f(l))
            .name("child")
            .priority(expected)
            .spawn();
        let priority = get_priority();
        assert_eq!(
            expected, priority,
            "This thread should have priority {}. Actual priority {}.",
            expected, priority
        );
    };
    create_and_check(medium, 1);
    create_and_check(high, 2);

    lock.release();

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 1, 1], "Medium & high should have exited.");
    };

    pass();
}
