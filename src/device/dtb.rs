//! DeviceTree Blob
//!
//! ref: <https://devicetree-specification.readthedocs.io>
//! > Boot is a complex process, to initialize and boot a computer system,
//! > various software components interact...
//! >
//! > Boot program is used to generically refer to a software component that
//! > initializes the system state and executes another software component
//! > referred to as a client program.
//! >
//! > The devicetree provides a complete boot program to client program
//! > interface definition, combined with minimum system requirements that
//! > facilitate the development of a wide variety of systems.
//!
//! DTB(Device Tree Blob) is the compact binary representation of the devicetree.
//! This module contains data structures that are used to read the devicetree from
//! DTB.
//!

#[macro_use]
mod macros;
mod header;
mod property;

use core::ffi;

use self::header::Header;
use self::property::Property;

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

pub struct DeviceTree {
    base: *const u8,
    header: Header,
}

impl DeviceTree {
    pub fn new(base: usize) -> Self {
        Self {
            base: base as *const _,
            header: Header::from_raw(base),
        }
    }

    /// # Safety
    ///
    /// Depends on the structure of device tree blob.
    ///
    /// # Return
    /// `(usize, usize, &str)`: (pm_start, pm_len, bootargs).
    ///
    /// TODO: rewrite this fn.
    pub unsafe fn traverse(&self) -> (usize, usize, *const i8) {
        #[cfg(feature = "debug")]
        kprintln!("[DTB] Start device enum...");
        let dt_struct_base = self.base.add(self.header.off_dt_struct as usize) as *const u32;

        let mut off = 0;
        let mut ret = (0, 0, 0 as *const i8);
        let mut _debug_prefix_num = 0;
        loop {
            let ptr = dt_struct_base.add(off);
            let token = u32::from_be(*ptr);

            match token {
                FDT_BEGIN_NODE => {
                    let name = ptr.add(1) as *const ffi::c_char;
                    let name = ffi::CStr::from_ptr(name).to_str().expect("Bad dtb str");

                    #[cfg(feature = "debug")]
                    {
                        _debug_prefix(_debug_prefix_num);
                        // To avoid timestamp printing we use kprint!() and '\n' here,
                        // so as below.
                        kprint!("Node={}\n", name);
                        _debug_prefix(_debug_prefix_num);
                        kprint!("{{\n");
                    }

                    // The string is padded to be i32 align, saying the Spec.
                    // So we skip the node name in `i32` length by (namelen + 3) / 4.
                    // Also an additional byte '\0' (i.e., namelen + 1 then + 3).
                    let namelen = name.len();
                    off += (namelen + 4) / 4;

                    if *name == *"memory@80000000" {
                        let info = self.extract_memory_info(ptr);
                        ret.0 = info.0;
                        ret.1 = info.1;

                        #[cfg(feature = "debug")]
                        {
                            _debug_prefix(_debug_prefix_num);
                            kprint!("Extract memory info {:#x} {:#x}\n", info.0, info.1);
                        }
                    }
                    _debug_prefix_num += 1;
                }
                FDT_END_NODE => {
                    _debug_prefix_num -= 1;

                    #[cfg(feature = "debug")]
                    {
                        _debug_prefix(_debug_prefix_num);
                        kprint!("}}\n");
                        _debug_prefix(_debug_prefix_num);
                        kprint!("Node end\n");
                    }
                }
                FDT_PROP => {
                    let prop = Property::from_raw(ptr.add(1) as usize);
                    let name = self.extract_str_from_offset(prop.nameoff);

                    #[cfg(feature = "debug")]
                    {
                        _debug_prefix(_debug_prefix_num);
                        kprint!(" |_ Property=(len={}, name=\'{}\')\n", prop.len, name);
                    }

                    if *name == *"bootargs" {
                        let bootargs = ptr.add(3) as *const ffi::c_char;
                        ret.2 = bootargs;
                        #[cfg(feature = "debug")]
                        {
                            _debug_prefix(_debug_prefix_num);
                            kprint!(" bootargs: {:?}\n", bootargs);
                        }
                    }

                    // Skip the property header.
                    off += 2;
                    // Skip the property data.
                    off += (prop.len as usize + 3) / 4;
                }
                FDT_END => break,
                FDT_NOP => { /* NOP token must be ignored. */ }
                _ => { /* Occasionally we meet unexpected token, skip them but don't panic. */ }
            }
            // Skip one token.
            off += 1;
        }
        ret
    }

    unsafe fn extract_str_from_offset(&self, nameoff: u32) -> &str {
        let name = self
            .base
            .add((self.header.off_dt_strings + nameoff) as usize);
        ffi::CStr::from_ptr(name as *const ffi::c_char)
            .to_str()
            .expect("Bad dtb str")
    }

    unsafe fn extract_memory_info(&self, addr: *const u32) -> (usize, usize) {
        let mut off = 0;
        loop {
            let ptr: *const u32 = addr.add(off);
            let token = u32::from_be(*ptr);

            if token == FDT_PROP {
                let Property { nameoff, .. } = Property::from_raw(ptr.add(1) as usize);
                if *self.extract_str_from_offset(nameoff) == *"reg" {
                    return (
                        usize::from_be(*ptr.add(3).cast()),
                        usize::from_be(*ptr.add(5).cast()),
                    );
                }
            }

            off += 1;
        }
    }
}

fn _debug_prefix(num: usize) {
    for _ in 0..num {
        kprint!("  ");
    }
}
