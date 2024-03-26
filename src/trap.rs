//! Trap handler
//!

mod pagefault;
mod syscall;

use crate::device::{plic, virtio};
use crate::sbi;
use crate::thread;
use core::arch;

use riscv::register::scause::{Exception::*, Interrupt::*, Trap::*};
use riscv::register::sstatus::*;
use riscv::register::*;

/* -------------------------------------------------------------------------- */
/*                                    FRAME                                   */
/* -------------------------------------------------------------------------- */

#[repr(C)]

/// Trap context
pub struct Frame {
    /// General regs[0..31].
    pub x: [usize; 32],
    /// CSR sstatus.
    pub sstatus: Sstatus,
    /// CSR sepc.
    pub sepc: usize,
}

pub fn set_strap_entry() {
    unsafe { stvec::write(trap_entry_k as usize, stvec::TrapMode::Direct) }
}

pub fn stvec() -> usize {
    stvec::read().address()
}

#[no_mangle]
pub extern "C" fn trap_handler(frame: &mut Frame) {
    #[cfg(feature = "debug")]
    kprintln!("[TRAP] enter trap handler");

    // Force to use kernel handler. Rely on trap_exit_k to restore the proper one.
    set_strap_entry();

    let scause = scause::read().cause();
    let stval = stval::read();

    #[cfg(feature = "debug")]
    kprintln!(
        "[TRAP] {:?}, tval={:#x}, sepc={:#x}",
        scause,
        stval,
        frame.sepc
    );

    match scause {
        Exception(UserEnvCall) => {
            let id = frame.x[17];
            let args = [frame.x[10], frame.x[11], frame.x[12]];
            #[cfg(feature = "debug")]
            kprintln!("[TRAP] User ECall, ID={}, args={:?}", id, args);
            unsafe { riscv::register::sstatus::set_sie() };
            // Increase sepc by 1 to skip ecall.
            frame.sepc += 4;
            frame.x[10] = syscall::syscall_handler(id, args) as usize;
        }

        Interrupt(SupervisorTimer) => {
            sbi::timer::tick();
            unsafe { riscv::register::sstatus::set_sie() };
            thread::schedule();
        }

        Interrupt(SupervisorExternal) => unsafe {
            // Get the interrupt source.
            let id = plic::read_claim();

            // Handle the interrupt.
            match id as _ {
                0 => panic!("There should be an interrupt"),
                plic::VIRTIO0_ID => virtio::handle_interrupt(),
                _ => panic!("Unknown Interrupt ID: {}", id),
            }

            // Tell PLIC we've done with the interrupt.
            plic::write_completion(id);
        },

        Exception(InstructionFault) | Exception(IllegalInstruction) => {
            // TODO: kill user process but not panic kernel
            panic!("Instruction failure");
        }

        Exception(f @ LoadPageFault)
        | Exception(f @ StorePageFault)
        | Exception(f @ InstructionPageFault) => {
            pagefault::handler(frame, f, stval);
        }

        _ => {
            unimplemented!(
                "Unsupported trap {:?}, stval={:#x}, sepc={:#x}",
                scause,
                stval,
                frame.sepc,
            )
        }
    }

    #[cfg(feature = "debug")]
    kprintln!("[TRAP] exit");
}

extern "C" {
    pub fn trap_entry_u();
    pub fn trap_exit_u();
    pub fn trap_entry_k();
    pub fn trap_exit_k();
}

/* ----------------------------- USER INTR ENTRY ---------------------------- */

arch::global_asm! {r#"
    .section .text
        .globl trap_entry_u
        .globl trap_exit_u

    .align 2  # 4-aligned. See trap_entry_k for details.
    trap_entry_u:

    # (1) Immediately switch to kernel stack.
    # We assume that `sscratch` stores the kernel stack.
        csrrw sp, sscratch, sp

    # Now `sscratch` stores user stack.
    # See the comments in `trap_exit_u` for details of `sscratch` management.

    # (2) Stores Frame on kernel stack.
        addi sp, sp, -34*8

    # (2.1) Save general purpose regs except sp and x0.
        # sd x0, 0*8(sp)
        sd x1,   1*8(sp)
        # sd sp, 2*8(sp)
        sd x3,   3*8(sp)
        sd x4,   4*8(sp)
        sd x5,   5*8(sp)
        sd x6,   6*8(sp)
        sd x7,   7*8(sp)
        sd x8,   8*8(sp)
        sd x9,   9*8(sp)
        sd x10, 10*8(sp)
        sd x11, 11*8(sp)
        sd x12, 12*8(sp)
        sd x13, 13*8(sp)
        sd x14, 14*8(sp)
        sd x15, 15*8(sp)
        sd x16, 16*8(sp)
        sd x17, 17*8(sp)
        sd x18, 18*8(sp)
        sd x19, 19*8(sp)
        sd x20, 20*8(sp)
        sd x21, 21*8(sp)
        sd x22, 22*8(sp)
        sd x23, 23*8(sp)
        sd x24, 24*8(sp)
        sd x25, 25*8(sp)
        sd x26, 26*8(sp)
        sd x27, 27*8(sp)
        sd x28, 28*8(sp)
        sd x29, 29*8(sp)
        sd x30, 30*8(sp)
        sd x31, 31*8(sp)

    # Now we can use registers.
    # (2.2) Save CSRs.
        csrr t0, sstatus
        csrr t1, sepc

    # (2.3) Save user stack.
    # From now, `sscratch` is useless.
        csrr t2, sscratch

    # Save them.
        sd t0, 32*8(sp)
        sd t1, 33*8(sp)
        sd t2,  2*8(sp)  # save to `x2`

    # (3) Call trap handler.
    # Must use `call`.
        mv   a0, sp   # pass frame
        call trap_handler

    # We must `call` trap handler, because we rely on `ret` to return to
    # trap_exit_u. We arrange the entry and exit to be together, so we donot
    # need to explicitly jump to exit in the handler. Thus, `call` is required,
    # and `j` would be incorrect.
    #
    # Also, you can regard the `trap_exit_u` label as pseudo as it doesn't point
    # to a function entry. However we still need it because user process creation
    # relies on it.

    trap_exit_u:

    # (1) Restore CSR.
    # TODO: should we restore `stvec` here?
        ld   t0, 32*8(sp)
        ld   t1, 33*8(sp)
        ld   t2,  2*8(sp)
        csrw sstatus,  t0
        csrw sepc,     t1
        csrw sscratch, t2

    # (2) Restore stvec.
    # In exit U we can be sure stvec is entry_u.
        la t0, trap_entry_u
        csrw stvec, t0

    # (3) Restore general-purpose regs.
        # ld x0, 0*8(sp)
        ld x1,   1*8(sp)
        # ld sp, 2*8(sp)
        ld x3,   3*8(sp)
        ld x4,   4*8(sp)
        ld x5,   5*8(sp)
        ld x6,   6*8(sp)
        ld x7,   7*8(sp)
        ld x8,   8*8(sp)
        ld x9,   9*8(sp)
        ld x10, 10*8(sp)
        ld x11, 11*8(sp)
        ld x12, 12*8(sp)
        ld x13, 13*8(sp)
        ld x14, 14*8(sp)
        ld x15, 15*8(sp)
        ld x16, 16*8(sp)
        ld x17, 17*8(sp)
        ld x18, 18*8(sp)
        ld x19, 19*8(sp)
        ld x20, 20*8(sp)
        ld x21, 21*8(sp)
        ld x22, 22*8(sp)
        ld x23, 23*8(sp)
        ld x24, 24*8(sp)
        ld x25, 25*8(sp)
        ld x26, 26*8(sp)
        ld x27, 27*8(sp)
        ld x28, 28*8(sp)
        ld x29, 29*8(sp)
        ld x30, 30*8(sp)
        ld x31, 31*8(sp)

    # (4) Restore kernel stack (pop frame).
        addi sp, sp, 34*8

    # (5) Restore `sscratch` and user stack.
        csrrw sp, sscratch, sp

    # Now `sp` is user stack, `sscratch` is kernel stack. In `trap_entry_u`,
    # we assume `sscratch` store the kernel stack, here is the save procedure.
    # It might be wondered that whether `sscratch` can remain correct during
    # context switch. We rely on four principles:
    #  - In U-mode, `sscratch` can not be touched;
    #  - In K-mode, `sscratch` is not used;
    #  - K-mode doesn't rely on `sscratch` to switch `sp` (saved by Context);
    #  - When switching from a U-proc to another U-proc, it first enters trap_entry_u.
    # Based on these principles, it is fine that we just restore `sscratch` at
    # the exit, and do not save it anywhere else.

    # (6) Return.
        sret
"#
}

/* ---------------------------- KERNEL INTR ENTRY --------------------------- */

arch::global_asm! {r#"
    .section .text
        .globl trap_entry_k
        .globl trap_exit_k

    .align 2 # Address of trap handlers must be 4-byte aligned.
    # Refer to The RISC-V Instruction Set Manual for more details.
    # https://five-embeddev.com/riscv-isa-manual/latest/supervisor.html#supervisor-trap-vector-base-address-register-stvec

    trap_entry_k:
        addi sp, sp, -34*8

    # save general-purpose registers
        # sd x0, 0*8(sp)
        sd x1,   1*8(sp)
        # sd x2, 2*8(sp)
        sd x3,   3*8(sp)
        sd x4,   4*8(sp)
        sd x5,   5*8(sp)
        sd x6,   6*8(sp)
        sd x7,   7*8(sp)
        sd x8,   8*8(sp)
        sd x9,   9*8(sp)
        sd x10, 10*8(sp)
        sd x11, 11*8(sp)
        sd x12, 12*8(sp)
        sd x13, 13*8(sp)
        sd x14, 14*8(sp)
        sd x15, 15*8(sp)
        sd x16, 16*8(sp)
        sd x17, 17*8(sp)
        sd x18, 18*8(sp)
        sd x19, 19*8(sp)
        sd x20, 20*8(sp)
        sd x21, 21*8(sp)
        sd x22, 22*8(sp)
        sd x23, 23*8(sp)
        sd x24, 24*8(sp)
        sd x25, 25*8(sp)
        sd x26, 26*8(sp)
        sd x27, 27*8(sp)
        sd x28, 28*8(sp)
        sd x29, 29*8(sp)
        sd x30, 30*8(sp)
        sd x31, 31*8(sp)

    # save sstatus & sepc
        csrr t0, sstatus
        csrr t1, sepc
        sd   t0, 32*8(sp)
        sd   t1, 33*8(sp)

        mv   a0, sp
        call trap_handler


    trap_exit_k:

    # load sstatus & sepc
        ld   t0, 32*8(sp)
        ld   t1, 33*8(sp)
        csrw sstatus,  t0
        csrw sepc,     t1

    # restore stvec
        la t0, trap_entry_k
        csrw stvec, t0

    # load general-purpose registers
        # ld x0, 0*8(sp)
        ld x1,   1*8(sp)
        # ld x2, 2*8(sp)
        ld x3,   3*8(sp)
        ld x4,   4*8(sp)
        ld x5,   5*8(sp)
        ld x6,   6*8(sp)
        ld x7,   7*8(sp)
        ld x8,   8*8(sp)
        ld x9,   9*8(sp)
        ld x10, 10*8(sp)
        ld x11, 11*8(sp)
        ld x12, 12*8(sp)
        ld x13, 13*8(sp)
        ld x14, 14*8(sp)
        ld x15, 15*8(sp)
        ld x16, 16*8(sp)
        ld x17, 17*8(sp)
        ld x18, 18*8(sp)
        ld x19, 19*8(sp)
        ld x20, 20*8(sp)
        ld x21, 21*8(sp)
        ld x22, 22*8(sp)
        ld x23, 23*8(sp)
        ld x24, 24*8(sp)
        ld x25, 25*8(sp)
        ld x26, 26*8(sp)
        ld x27, 27*8(sp)
        ld x28, 28*8(sp)
        ld x29, 29*8(sp)
        ld x30, 30*8(sp)
        ld x31, 31*8(sp)

        addi sp, sp, 34*8
        sret
"#}
