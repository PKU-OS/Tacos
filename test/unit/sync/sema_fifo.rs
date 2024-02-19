use crate::thread;
use crate::sync::Semaphore;

use alloc::sync::Arc;

fn multi_thread_sema(sema: Arc<Semaphore>) {
    let cnt = sema.value();
    kprintln!("multi_thread_sema(): Thread {} downing sema.", cnt);
    sema.down();
    kprintln!("multi_thread_sema(): Thread {} down sema success.", cnt);
    for i in 0..2 {
        kprintln!("Thread {} yield {} times: ", cnt, i + 1);
        thread::schedule();
    }
    kprintln!("multi_thread_sema(): Thread {} up sema.", cnt);
    sema.up();
    thread::schedule();
}

pub fn main() {
    kprintln!("############################## MUTEX SEMA TEST ##############################");
    kprintln!("test::mutex::main(): Test semaphore.");
    kprintln!("test::mutex::main(): Initialize global semaphore size to 5.");
    let s1 = Arc::new(Semaphore::new(5));
    let s2 = s1.clone();
    let s3 = s1.clone();
    let s4 = s1.clone();
    let s5 = s1.clone();
    let s6 = s1.clone();
    thread::spawn("sema5-0", move || multi_thread_sema(s1));
    thread::spawn("sema5-1", move || multi_thread_sema(s2));
    thread::spawn("sema5-2", move || multi_thread_sema(s3));
    thread::spawn("sema5-3", move || multi_thread_sema(s4));
    thread::spawn("sema5-4", move || multi_thread_sema(s5));
    thread::spawn("sema5-5", move || multi_thread_sema(s6));
    for i in 0..10 {
        kprintln!("Test thread yield {} times: ", i + 1);
        thread::schedule();
    }
}
