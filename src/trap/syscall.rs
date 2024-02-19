//! Syscall handlers
//!

#![allow(dead_code)]

/* -------------------------------------------------------------------------- */
/*                               SYSCALL NUMBER                               */
/* -------------------------------------------------------------------------- */

const SYS_HALT: usize = 1;
const SYS_EXIT: usize = 2;
const SYS_EXEC: usize = 3;
const SYS_WAIT: usize = 4;
const SYS_REMOVE: usize = 5;
const SYS_OPEN: usize = 6;
const SYS_READ: usize = 7;
const SYS_WRITE: usize = 8;
const SYS_SEEK: usize = 9;
const SYS_TELL: usize = 10;
const SYS_CLOSE: usize = 11;
const SYS_FSTAT: usize = 12;

pub fn syscall_handler(_id: usize, _args: [usize; 3]) -> isize {
    // TODO: LAB2 impl
    -1
}
