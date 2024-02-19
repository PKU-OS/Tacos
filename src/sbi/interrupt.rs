//! RISC-V timer & external interrupt

use riscv::register;

#[inline]
fn on() {
    unsafe {
        register::sie::set_stimer();
        register::sie::set_sext();
    };
}

#[inline]
fn off() {
    unsafe {
        register::sie::clear_sext();
        register::sie::clear_stimer();
    };
}

/// Get timer & external interrupt level. `true` means interruptible.
#[inline]
pub fn get() -> bool {
    register::sie::read().stimer()
}

/// Set timer & external interrupt level.
pub fn set(level: bool) -> bool {
    let old = get();

    // To avoid unnecessary overhead, we only (un)set timer when its status need
    // to be changed. Synchronization is not required here as any changes to
    // timer status will be restored by other threads.
    if old != level {
        if level {
            on();
        } else {
            off();
        }
    }

    old
}

pub fn init() {
    crate::sbi::timer::next();

    set(true);

    unsafe { register::sstatus::set_sie() };
}
