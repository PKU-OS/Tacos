use crate::thread;

use super::*;

fn changing_thread(main_thread: Arc<Thread>) {
    assert_eq!(main_thread.status(), Status::Ready);

    kprintln!("Thread 2 lowering priority");
    set_priority(PRI_DEFAULT - 1);

    assert_eq!(main_thread.status(), Status::Ready);
    kprintln!("Thread 2 exiting...");
}

pub fn main() {
    // TODO: assert(!mlfqs)

    let main_thread = thread::current();

    kprintln!("Creating a high-priority thread 2");
    let child = Builder::new(move || changing_thread(main_thread))
        .name("thread 2")
        .spawn();

    assert_eq!(
        child.status(),
        Status::Ready,
        "Thread 2 should have just lowered its priority."
    );

    set_priority(PRI_DEFAULT - 2);

    assert_eq!(
        child.status(),
        Status::Dying,
        "Thread 2 shoud have just exited"
    );

    pass();
}
