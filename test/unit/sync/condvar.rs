use alloc::sync::Arc;

use crate::sync::{Condvar, Mutex};
use crate::thread::{self, Status};

pub fn main() {
    let s = Arc::new((Condvar::new(), Mutex::new(0)));
    let s1 = s.clone();
    let p = thread::spawn("producer_six", move || producer_six(s1));

    let (cvar, lock) = &*s;
    let mut guard = lock.lock();
    while *guard < 6 {
        // Waiting for producer_one, producer_all and consumer.
        cvar.wait(&mut guard);
    }
    thread::schedule();

    assert_eq!(p.status(), Status::Dying);
    kprintln!("Main continue.");
}

fn producer_six(s: Arc<(Condvar, Mutex<i32>)>) {
    let (cvar, lock) = &*s;
    kprintln!("Producer_six begins to work.");
    for i in 0..6 {
        let mut guard = lock.lock();
        *guard += 1;
        cvar.notify_all();
        kprintln!("Producer_six iteration {}.", i);
        thread::schedule();
    }
}
