#![no_std]
#![no_main]

extern crate alloc;
extern crate bitflags;
extern crate elf_rs;
extern crate fdt;
extern crate riscv;

#[macro_use]
pub mod sbi;
pub mod boot;
pub mod device;
pub mod error;
pub mod fs;
pub mod io;
pub mod mem;
pub mod sync;
pub mod thread;
pub mod trap;
pub mod userproc;

#[cfg(feature = "test")]
#[path = "../test/mod.rs"]
mod test;

pub use error::OsError;

use core::{ptr, slice, str};
use fdt::{standard_nodes::MemoryRegion, Fdt};
use riscv::register;

use fs::{disk::DISKFS, FileSys};
use mem::PhysAddr;

extern "C" {
    fn sbss();
    fn ebss();
    fn ekernel();
    fn bootstack();
}

pub type Result<T> = core::result::Result<T, OsError>;

/// Initializes major components of our kernel
///
/// Note: `extern "C"` ensures this function adhere to the C calling convention.
/// (ref: https://doc.rust-lang.org/nomicon/ffi.html?highlight=calling%20convention#rust-side)
#[no_mangle]
pub extern "C" fn main(hart_id: usize, dtb: usize) -> ! {
    kprintln!("Hello, World!");

    // Flush BSS since they are not loaded and the corresponding memory may be random
    unsafe { ptr::write_bytes(sbss as *mut u8, 0, ebss as usize - sbss as usize) };

    // Parse the device tree.
    let devtree = unsafe { Fdt::from_ptr(dtb as *const u8).unwrap() };
    // Get the start point and length of physical memory
    let (pm_base, pm_len) = {
        let memory = devtree.memory();
        let mut regions = memory.regions();
        let MemoryRegion {
            starting_address,
            size,
        } = regions.next().expect("No memory info.");
        assert_eq!(regions.next(), None, "Unknown memory region");
        (
            starting_address as usize,
            size.expect("Unknown physical memory length"),
        )
    };
    assert_eq!(pm_base, mem::PM_BASE, "Error constant mem::PM_BASE.");
    // Get the boot arguments.
    let _bootargs: &'static str = unsafe {
        let (vm, len) = {
            let bootargs = devtree.chosen().bootargs().unwrap();
            let len = bootargs.len();
            (PhysAddr::from_pa(bootargs.as_ptr() as usize).into_va(), len)
        };
        str::from_utf8(slice::from_raw_parts(vm as *const u8, len)).unwrap()
    };

    // Initialize memory management.
    let ram_base = ekernel as usize;
    let ram_tail = dtb + mem::VM_OFFSET; // Current we do not reuse dtb area.
    mem::init(ram_base, ram_tail, pm_len);

    #[cfg(feature = "debug")]
    {
        kprintln!("RAM: 0x{:x} - 0x{:x}", ram_base, ram_tail);
        kprintln!("BOOTARGS: {:?}", _bootargs);
    }

    trap::set_strap_entry();

    unsafe {
        register::sstatus::set_sie();
        register::sstatus::set_sum();
    };

    device::plic::init(hart_id);
    #[cfg(feature = "debug")]
    kprintln!("Virtio inited.");

    // Init timer & external interrupt
    sbi::interrupt::init();

    #[cfg(feature = "test")]
    {
        use alloc::sync::Arc;
        let sema = Arc::new(sync::Semaphore::new(0));
        let sema2 = sema.clone();
        thread::spawn("test", move || crate::test::main(sema2, _bootargs));
        sema.down();
    }

    #[cfg(feature = "shell")]
    {
        // TODO: Lab 0
    }

    DISKFS.unmount();

    kprintln!("Goodbye, World!");

    sbi::reset(
        sbi::system_reset::Type::Shutdown,
        sbi::system_reset::Reason::NoReason,
    )
}

/* ---------------------------------- PANIC --------------------------------- */
#[panic_handler]
unsafe fn panic(info: &core::panic::PanicInfo) -> ! {
    // Disable interrupts until shutting down the whole system
    sbi::interrupt::set(false);

    // Report the reason for invoking `panic`
    kprintln!("{}", info);

    sbi::reset(
        sbi::system_reset::Type::Shutdown,
        sbi::system_reset::Reason::SystemFailure,
    )
}
