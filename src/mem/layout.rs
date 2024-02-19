//     +------------------+
//     |                  |
//     /\/\/\/\/\/\/\/\/\/\
//     /\/\/\/\/\/\/\/\/\/\
//     |                  |
//     |      Unused      |
//     |                  |
//     +------------------+  <- End of kernel
//     |      kernel      |
//     +------------------+  <- 0x80200000
//     |                  |
//     |      Unused      |
//     |                  |
//     +------------------+
//     |       MMIO       |
//     +------------------+  <- 0x10001000
//     |                  |
//     |      Unused      |
//     |                  |
//     +------------------+  <- 0x0c400000
//     |       PLIC       |
//     +------------------+  <- 0x0c000000
//     |                  |
//     |    Low Memory    |
//     |                  |
//     +------------------+  <- 0x00000000

pub const VM_BASE: usize = 0xFFFFFFC080000000;
pub const PM_BASE: usize = 0x0000000080000000;
pub const KERN_BASE: usize = 0x0000000080200000;
pub const VM_OFFSET: usize = VM_BASE - PM_BASE;
pub const PLIC_BASE: usize = 0xC000000 + VM_OFFSET;
pub const MMIO_BASE: usize = 0x10001000 + VM_OFFSET;
