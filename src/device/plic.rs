//! RISC-V Platform Level Interrupt Controller (PLIC) Support
//!
//! Accessing PLIC is fundamentally unsafe, because its state may be changed by
//! external interrupt or interrupt handler at any time when external interrupt is on.
//! Therefore, it is recommended to only call these functions in kernel initialization or interrupt handler.
//!
//! For more information, see <https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc>.
//!

use crate::mem::PLIC_BASE;
use crate::sync::OnceCell;

/// Hard-coded Virtio device 0 interrupt identifier (ID).
pub const VIRTIO0_ID: usize = 1;

// Hart ID.
static HART_ID: OnceCell<usize> = OnceCell::new();

/// Initialization.
pub fn init(hart_id: usize) {
    HART_ID.init(|| hart_id);
    unsafe {
        // Set this hart's S-mode priority threshold.
        write_threshold(0);

        // Set interrupt priority of Virtio device 0.
        // 0 means no interrupt. Any positive value is OK.
        write_priority(VIRTIO0_ID, 1);

        // Enable this hart to receive interrupts from Virtio device 0.
        set_enable(VIRTIO0_ID);
    }
}

/// Read interrupt source priority of an ID.
pub unsafe fn read_priority(id: usize) -> u32 {
    get_priority_ptr(id).read_volatile()
}

/// Write interrupt source priority of an ID.
pub unsafe fn write_priority(id: usize, val: u32) {
    get_priority_ptr(id).write_volatile(val)
}

/// Read interrupt pending bit of an ID. This bit is set by PLIC.
pub unsafe fn read_pending(id: usize) -> bool {
    get_pending_ptr(id).read_volatile() & (1 << id % 32) != 0
}

/// Read S mode interrupt enable bit of an ID for this hart.
pub unsafe fn read_enable(id: usize) -> bool {
    get_enable_ptr(hart_id(), id).read_volatile() & (1 << id % 32) != 0
}

/// Set S mode interrupt enable bit of an ID for this hart.
pub unsafe fn set_enable(id: usize) {
    let ptr = get_enable_ptr(hart_id(), id);
    let prev = ptr.read_volatile();
    ptr.write_volatile(prev | (1 << id % 32));
}

/// Clear S mode interrupt enable bit of an ID for this hart.
pub unsafe fn clear_enable(id: usize) {
    let ptr = get_enable_ptr(hart_id(), id);
    let prev = ptr.read_volatile();
    ptr.write_volatile(prev & !(1 << id % 32));
}

/// Read S mode priority threshold for this hart.
pub unsafe fn read_threshold() -> u32 {
    get_threshold_ptr(hart_id()).read_volatile()
}

/// Write S mode priority threshold for this hart.
pub unsafe fn write_threshold(val: u32) {
    get_threshold_ptr(hart_id()).write_volatile(val)
}

/// Read the S mode interrupt claim register for this hart. This also starts an interrupt claim process.
/// See "Interrupt Claim Process" section in the spec for more information.
pub unsafe fn read_claim() -> u32 {
    get_claim_ptr(hart_id()).read_volatile()
}

/// Write the S mode interrupt completion register for this hart.
pub unsafe fn write_completion(val: u32) {
    get_completion_ptr(hart_id()).write_volatile(val)
}

// Get hart ID.
fn hart_id() -> usize {
    *HART_ID.get()
}

// Address calculation helper functions.
// See "Memory Map" section in the spec for more information.
fn get_priority_ptr(id: usize) -> *mut u32 {
    (PLIC_BASE + 0x4 * id) as _
}
fn get_pending_ptr(id: usize) -> *const u32 {
    (PLIC_BASE + 0x1000 + 0x4 * (id / 32)) as _
}
fn get_enable_ptr(hart_id: usize, id: usize) -> *mut u32 {
    (PLIC_BASE + 0x2000 + 0x80 * (2 * hart_id + 1) + 0x4 * (id / 32)) as _
}
fn get_threshold_ptr(hart_id: usize) -> *mut u32 {
    (PLIC_BASE + 0x200000 + 0x1000 * (2 * hart_id + 1)) as _
}
fn get_claim_ptr(hart_id: usize) -> *const u32 {
    get_completion_ptr(hart_id)
}
fn get_completion_ptr(hart_id: usize) -> *mut u32 {
    (PLIC_BASE + 0x200000 + 0x1000 * (2 * hart_id + 1) + 0x4) as _
}
