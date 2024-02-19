//! RISC-V Timer Interface

use core::sync::atomic::{AtomicI64, Ordering::SeqCst};

use crate::sbi::set_timer;

pub const TICKS_PER_SEC: usize = 10;
pub const CLOCK_PRE_SEC: usize = 12500000;

/// Get the clock's raw reading
pub fn clock() -> usize {
    riscv::register::time::read()
}

/// Get the current clock reading in milliseconds
#[inline]
pub fn time_ms() -> usize {
    clock() * 1_000 / CLOCK_PRE_SEC
}

/// Get the current clock reading in microseconds
#[inline]
pub fn time_us() -> usize {
    clock() * 1_000_000 / CLOCK_PRE_SEC
}

/// Set the next moment when timer interrupt should happen
#[inline]
pub fn next() {
    set_timer(clock() + CLOCK_PRE_SEC / TICKS_PER_SEC);
}

static TICKS: AtomicI64 = AtomicI64::new(0);

/// Returns the number of timer ticks since booted.
pub fn timer_ticks() -> i64 {
    TICKS.load(SeqCst)
}

/// Increments timer ticks by 1 and sets the next timer interrupt.
pub fn tick() {
    TICKS.fetch_add(1, SeqCst);
    next();
}

/// Returns how many timer ticks elapsed since "then", which should be a
/// value returned by [`timer_ticks()`].
pub fn timer_elapsed(then: i64) -> i64 {
    timer_ticks() - then
}
