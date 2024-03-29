use crate::mem::userbuf::{
    __knrl_read_usr_byte, __knrl_read_usr_exit, __knrl_write_usr_byte, __knrl_write_usr_exit,
};
use crate::mem::PageTable;
use crate::thread::{self};
use crate::trap::Frame;
use crate::userproc;

use riscv::register::scause::Exception::{self, *};
use riscv::register::sstatus::{self, SPP};

pub fn handler(frame: &mut Frame, fault: Exception, addr: usize) {
    let privilege = frame.sstatus.spp();

    let present = {
        let table = unsafe { PageTable::effective_pagetable() };
        match table.get_pte(addr) {
            Some(entry) => entry.is_valid(),
            None => false,
        }
    };

    unsafe { sstatus::set_sie() };

    kprintln!(
        "Page fault at {:#x}: {} error {} page in {} context.",
        addr,
        if present { "rights" } else { "not present" },
        match fault {
            StorePageFault => "writing",
            LoadPageFault => "reading",
            InstructionPageFault => "fetching instruction",
            _ => panic!("Unknown Page Fault"),
        },
        match privilege {
            SPP::Supervisor => "kernel",
            SPP::User => "user",
        }
    );

    match privilege {
        SPP::Supervisor => {
            if frame.sepc == __knrl_read_usr_byte as _ {
                // Failed to read user byte from kernel space when trap in pagefault
                frame.x[11] = 1; // set a1 to non-zero
                frame.sepc = __knrl_read_usr_exit as _;
            } else if frame.sepc == __knrl_write_usr_byte as _ {
                // Failed to write user byte from kernel space when trap in pagefault
                frame.x[11] = 1; // set a1 to non-zero
                frame.sepc = __knrl_write_usr_exit as _;
            } else {
                panic!("Kernel page fault");
            }
        }
        SPP::User => {
            kprintln!(
                "User thread {} dying due to page fault.",
                thread::current().name()
            );
            userproc::exit(-1);
        }
    }
}
