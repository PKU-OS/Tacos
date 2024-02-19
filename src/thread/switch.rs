//! Context Switch
//!
//! [`switch`] stores ra(return address), sp(stack pointer) and all
//! callee-saved general purpose registers on the current stack.
//!
//! According to the RISC-V calling convention, all caller-saved registers
//! should've already been preserved in the stack frame of the caller
//! (i.e. [`Manager::schedule`]).

use alloc::sync::Arc;
use core::arch::global_asm;

use crate::thread::{Context, Manager, Thread};

#[allow(improper_ctypes)]
extern "C" {
    /// Save current registers in the "old" context, and load from the "new" context.
    ///
    /// The first argument is not used in this function, but it
    /// will be forwarded to [`schedule_tail_wrapper`].
    pub fn switch(previous: *const Thread, old: *mut Context, new: *mut Context);
}

global_asm! {r#"
    .section .text
        .globl switch
    switch:
        sd ra, 0x0(a1)
        ld ra, 0x0(a2)
        sd sp, 0x8(a1)
        ld sp, 0x8(a2)
        sd  s0, 0x10(a1)
        ld  s0, 0x10(a2)
        sd  s1, 0x18(a1)
        ld  s1, 0x18(a2)
        sd  s2, 0x20(a1)
        ld  s2, 0x20(a2)
        sd  s3, 0x28(a1)
        ld  s3, 0x28(a2)
        sd  s4, 0x30(a1)
        ld  s4, 0x30(a2)
        sd  s5, 0x38(a1)
        ld  s5, 0x38(a2)
        sd  s6, 0x40(a1)
        ld  s6, 0x40(a2)
        sd  s7, 0x48(a1)
        ld  s7, 0x48(a2)
        sd  s8, 0x50(a1)
        ld  s8, 0x50(a2)
        sd  s9, 0x58(a1)
        ld  s9, 0x58(a2)
        sd s10, 0x60(a1)
        ld s10, 0x60(a2)
        sd s11, 0x68(a1)
        ld s11, 0x68(a2)

        j schedule_tail_wrapper
"#}

/// A thin wrapper over [`Manager::schedule_tail`]
///
/// Note: Stack is not properly built in [`switch`]. Therefore,
/// this function should never be inlined.
#[no_mangle]
#[inline(never)]
extern "C" fn schedule_tail_wrapper(previous: *const Thread) {
    Manager::get().schedule_tail(unsafe { Arc::from_raw(previous) });
}
