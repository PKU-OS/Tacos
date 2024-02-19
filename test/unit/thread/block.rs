use alloc::sync::Arc;

use crate::sync::{Mutex, Semaphore, Sleep};
use crate::thread::{self, Status};

static mut MUTEX: Option<Mutex<i32, Sleep>> = None;

const SCHEDULE_NUM: i32 = 10;

struct S {
    sema: Semaphore,
    mutex: Mutex<i32, Sleep>,
}

pub fn main() {
    let s = Arc::new(S {
        sema: Semaphore::new(0),
        mutex: Mutex::new(0),
    });
    let s1 = s.clone();

    let waiter = thread::spawn("waiter", move || waiter_mutex(s1));
    let guard = s.mutex.lock();

    s.sema.up();
    for _ in 0..SCHEDULE_NUM {
        thread::schedule();
    }

    assert_eq!(waiter.status(), Status::Blocked);
    kprintln!("Dropping mutex guard");
    drop(guard);

    for _ in 0..SCHEDULE_NUM {
        thread::schedule();
    }

    kprintln!("{:?}", waiter.status());
    assert_eq!(waiter.status(), Status::Ready);
}

fn waiter_mutex(s: Arc<S>) {
    s.sema.down();
    s.mutex.lock();
    loop {}
}
