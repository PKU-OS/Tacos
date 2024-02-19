use crate::thread;

#[allow(unused)]
pub fn main() {
    thread::spawn("t1", spin);
    thread::spawn("t2", spin);

    for iter in 0..20 {
        kprintln!("Yield {} from initial thread", iter);
        thread::schedule();
    }
}

fn spin() {
    for iter in 0..15 {
        kprintln!("Schedule {} thread {}", iter, thread::current().name());
        unsafe { core::arch::asm!("wfi") };
    }
}
