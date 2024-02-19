use crate::thread;

pub fn main() {
    thread::spawn("t1", spin);
    thread::spawn("t2", spin);

    for iter in 0..20 {
        kprintln!("Yield {} from test thread", iter);
        thread::schedule();
        kprintln!("Yield {} again from test thread", iter);
    }
}

fn spin() {
    let name = thread::current().name();
    for iter in 0..15 {
        kprintln!("Yield {} from thread {}", iter, name);
        thread::schedule();
    }
}
