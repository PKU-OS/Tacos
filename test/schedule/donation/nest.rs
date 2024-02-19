use super::*;

static mut EXIT_STATUS: [i8; 3] = [0; 3];

fn medium(pair: Arc<(Sleep, Sleep)>) {
    let (m_lock, h_lock) = &*pair;

    h_lock.acquire();
    m_lock.acquire();

    let expected = PRI_DEFAULT + 2;
    let priority = get_priority();
    assert_eq!(
        expected, priority,
        "Medium thread should have priority {}. Actual priority {}.",
        expected, priority
    );

    m_lock.release();
    schedule();

    h_lock.release();
    schedule();

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 0, 1], "Medium should exit after high.");
        EXIT_STATUS[1] = 1;
    }
}

fn high(pair: Arc<(Sleep, Sleep)>) {
    let (_, h_lock) = &*pair;

    h_lock.acquire();

    h_lock.release();

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 0, 0], "Hish should exit first.");
        EXIT_STATUS[2] = 1;
    }
}

pub fn main() {
    let pair = Arc::new((Sleep::default(), Sleep::default()));

    let (m_lock, _) = &*pair;

    m_lock.acquire();

    let create_and_check = |f: fn(Arc<(Sleep, Sleep)>), expected| {
        let p = Arc::clone(&pair);
        Builder::new(move || f(p))
            .name("child")
            .priority(expected)
            .spawn();

        schedule();

        let priority = get_priority();
        assert_eq!(
            expected, priority,
            "Low thread should have priority {}. Actual priority {}.",
            expected, priority
        );
    };

    // Create two child thread with higher priority. They are donating priority to main.
    let m_priority = PRI_DEFAULT + 1;
    let h_priority = PRI_DEFAULT + 2;
    create_and_check(medium, m_priority);
    create_and_check(high, h_priority);

    m_lock.release();

    let priority = get_priority();
    assert_eq!(
        PRI_DEFAULT, priority,
        "Low thread should have priority {}. Actual priority {}.",
        PRI_DEFAULT, priority
    );

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 1, 1], "Medium & high should have exited.");
    };

    pass();
}
