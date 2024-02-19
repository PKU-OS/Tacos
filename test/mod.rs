#![allow(dead_code)]

use alloc::sync::Arc;

use crate::sync::Semaphore;

mod schedule;
mod unit;
pub mod user;

pub fn main(sema: Arc<Semaphore>, _bootargs: &str) {
    #[cfg(feature = "test-unit")]
    unit::main();

    #[cfg(feature = "test-user")]
    user::main(_bootargs);

    #[cfg(feature = "test-schedule")]
    schedule::main(_bootargs);

    kprintln!("Leaving test...");
    sema.up();
}
