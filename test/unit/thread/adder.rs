use alloc::sync::Arc;

use crate::sync::{self, Mutex, Semaphore};
use crate::thread;

static mut X: Option<Mutex<usize, sync::Sleep>> = None;
static mut Y: usize = 0;

static NUM: usize = 5000;

#[allow(unused)]
pub fn main() {
    let finish = Arc::new(Semaphore::new(0));
    let f1 = finish.clone();
    let f2 = finish.clone();
    let f3 = finish.clone();
    let f4 = finish.clone();
    unsafe {
        X.replace(Mutex::new(0));
    }

    thread::spawn("good_adder1", move || good_adder(f1));
    thread::spawn("good_adder2", move || good_adder(f2));
    finish.down();
    finish.down();
    kprintln!("Good adder done.");

    thread::spawn("bad_adder1", move || bad_adder(f3));
    thread::spawn("bad_adder2", move || bad_adder(f4));
    finish.down();
    finish.down();
    kprintln!("Bad adder done.");

    assert_eq!(unsafe { *(X.as_ref().unwrap().lock()) }, 2 * NUM);
    kprintln!("Bad adder results: {}:{}", unsafe { Y }, 2 * NUM);
}

pub fn good_adder(finish: Arc<Semaphore>) {
    let mut i = 0;
    while i < NUM {
        let mut x = unsafe { X.as_ref().unwrap().lock() };
        for _ in 0..(i % 100) {}
        *x += 1;
        i += 1;
    }

    finish.up();
}

pub fn bad_adder(finish: Arc<Semaphore>) {
    let mut i = 0;
    while i < NUM {
        let mut y = unsafe { Y };
        y += 1;
        for _ in 0..(i % 100) {}
        unsafe { Y = y };

        i += 1;
    }

    finish.up();
}
