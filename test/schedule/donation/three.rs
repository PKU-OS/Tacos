use super::*;

static mut EXIT_STATUS: [i8; 4] = [0; 4];

fn low() {
    unsafe {
        assert_eq!(
            EXIT_STATUS,
            [0, 0, 1, 1],
            "Low should exit after high and medium."
        );
        EXIT_STATUS[1] = 1;
    }
}

fn medium(m_lock: Arc<Sleep>) {
    m_lock.acquire();

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 0, 0, 1], "Medium should exit after high.");
        EXIT_STATUS[2] = 1;
    }

    m_lock.release();
}

fn high(h_lock: Arc<Sleep>) {
    h_lock.acquire();

    unsafe {
        assert_eq!(EXIT_STATUS, [0, 0, 0, 0], "Hish should exit first.");
        EXIT_STATUS[3] = 1;
    }

    h_lock.release();
}

pub fn main() {
    let m_lock = Arc::new(Sleep::default());
    let h_lock = Arc::new(Sleep::default());
    m_lock.acquire();
    h_lock.acquire();

    let create_and_check = |f: fn(Arc<Sleep>), expected, lock| {
        let l = Arc::clone(lock);
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

    // Create three child thread with higher priority. They are donating priority to main.
    let l_priority = PRI_DEFAULT + 1;
    let m_priority = PRI_DEFAULT + 3;
    let h_priority = PRI_DEFAULT + 5;
    create_and_check(medium, m_priority, &m_lock);
    Builder::new(low).name("child").priority(l_priority).spawn();
    create_and_check(high, h_priority, &h_lock);

    let release_and_check = |expected, lock: &Arc<Sleep>| {
        lock.release();

        let priority = get_priority();
        assert_eq!(
            expected, priority,
            "This thread should have priority {}. Actual priority {}.",
            expected, priority
        );
    };

    release_and_check(h_priority, &m_lock);
    release_and_check(PRI_DEFAULT, &h_lock);

    unsafe {
        assert_eq!(
            EXIT_STATUS,
            [0, 1, 1, 1],
            "Low, medium & high should have exited."
        );
    };

    pass();
}
