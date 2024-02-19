#![allow(dead_code)]

use core::arch::{asm, global_asm};

use crate::error::OsError;
use crate::mem::in_kernel_space;
use crate::Result;

/// Read a single byte from user space.
///
/// ## Return
/// - `Ok(byte)`
/// - `Err`: A page fault happened.
fn read_user_byte(user_src: *const u8) -> Result<u8> {
    if in_kernel_space(user_src as usize) {
        return Err(OsError::BadPtr);
    }

    let mut ret_status: isize = 0;
    let mut byte: u8;
    unsafe {
        asm! {
            "call __knrl_read_usr_byte",
            in("a0") user_src,  lateout("a0") byte,
            inout("a1") ret_status
        }
    };

    if ret_status == 0 {
        Ok(byte)
    } else {
        Err(OsError::BadPtr)
    }
}

/// Write a single byte to user space.
///
/// ## Return
/// - `Ok(())`
/// - `Err`: A page fault happened.
fn write_user_byte(user_src: *const u8, value: u8) -> Result<()> {
    if in_kernel_space(user_src as usize) {
        return Err(OsError::BadPtr);
    }

    let mut ret_status: isize = 0;
    unsafe {
        asm! {
            "call __knrl_write_usr_byte",
            in("a0") user_src,
            inout("a1") ret_status,
            in("a2") value
        }
    };

    if ret_status == 0 {
        Ok(())
    } else {
        Err(OsError::BadPtr)
    }
}

extern "C" {
    pub fn __knrl_read_usr_byte(user_src: *const u8);
    pub fn __knrl_read_usr_exit();
    pub fn __knrl_write_usr_byte(user_src: *const u8);
    pub fn __knrl_write_usr_exit();
}

global_asm! {r#"
        .section .text
        .globl __knrl_read_usr_byte
        .globl __knrl_read_usr_exit

    __knrl_read_usr_byte:
        lb t0, (a0)
    __knrl_read_usr_exit:
        # pagefault handler will set a1 if any error occurs
        mv a0, t0
        ret

        .globl __knrl_write_usr_byte
        .globl __knrl_write_usr_exit

    __knrl_write_usr_byte:
        sb a2, (a0)
    __knrl_write_usr_exit:
        # pagefault handler will set a1 if any error occurs
        ret
"#}
