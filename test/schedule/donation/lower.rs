use super::*;

fn child(lock: Arc<Sleep>) {
    lock.acquire();

    kprintln!("Child got lock.");

    lock.release();

    kprintln!("Child done.");
}

pub fn main() {
    let lock = Arc::new(Sleep::default());

    lock.acquire();

    let l = Arc::clone(&lock);
    let child_priority = PRI_DEFAULT + 10;
    let child = Builder::new(move || child(l))
        .name("child")
        .priority(child_priority)
        .spawn();

    let self_priority = get_priority();
    assert_eq!(
        child_priority, self_priority,
        "Main thread should have priority {}. Actual priority {}.",
        child_priority, self_priority
    );

    kprintln!("Lowering priority...");
    let lower_priority = PRI_DEFAULT - 10;
    set_priority(lower_priority);

    assert_eq!(
        child_priority, self_priority,
        "Main thread should have priority {}. Actual priority {}.",
        child_priority, self_priority
    );

    lock.release();

    assert_eq!(
        child.status(),
        Status::Dying,
        "Child thread must have finished."
    );

    let self_priority = get_priority();
    assert_eq!(
        lower_priority, self_priority,
        "Main thread should have priority {}. Actual priority {}.",
        lower_priority, self_priority
    );

    pass();
}
