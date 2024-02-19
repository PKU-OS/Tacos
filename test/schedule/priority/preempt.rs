use super::*;
use crate::thread;

const YIELD_TIMES: usize = 5;

fn yielding_thread(main_thread: Arc<Thread>) {
    assert_eq!(main_thread.status(), Status::Ready);

    for i in 0..YIELD_TIMES {
        kprintln!("Child thread yield {} times.", i);
        schedule();
    }

    assert_eq!(
        main_thread.status(),
        Status::Ready,
        "Main thread should exit after child thread!"
    );
    kprintln!("Thread 2 exiting...");
}

pub fn main() {
    kprintln!("Creating a high-priority thread 2");

    let main_thread = thread::current();

    let child = Builder::new(move || yielding_thread(main_thread))
        .name("thread 2")
        .priority(PRI_DEFAULT + 1)
        .spawn();

    assert_eq!(
        child.status(),
        Status::Dying,
        "Thread 2 should have just exited."
    );

    pass();
}
