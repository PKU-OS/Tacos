#![allow(dead_code)]

mod alarm;
mod donation;
mod priority;

fn pass() {
    kprintln!("[PASS]");
}

static NAME2CASE: [(&str, fn()); 18] = [
    ("alarm-zero", alarm::boundary::zero::main),
    ("alarm-negative", alarm::boundary::negative::main),
    ("alarm-simultaneous", alarm::simultaneous::main),
    ("alarm-single", alarm::multiple::single::main),
    ("alarm-multiple", alarm::multiple::main),
    ("priority-alarm", priority::alarm::main),
    ("priority-condvar", priority::condvar::main),
    ("priority-sema", priority::sema::main),
    ("priority-change", priority::change::main),
    ("priority-preempt", priority::preempt::main),
    ("priority-fifo", priority::fifo::main),
    ("donation-chain", donation::chain::main),
    ("donation-lower", donation::lower::main),
    ("donation-nest", donation::nest::main),
    ("donation-one", donation::one::main),
    ("donation-sema", donation::sema::main),
    ("donation-two", donation::two::main),
    ("donation-three", donation::three::main),
];

pub fn main(case: &str) {
    for (name, f) in NAME2CASE.iter() {
        if case.eq(*name) {
            f();
            return;
        }
    }

    kprintln!("Invalid test case: {}", case);
}
