pub mod alarm;
pub mod change;
pub mod condvar;
pub mod fifo;
pub mod preempt;
pub mod sema;

use alloc::sync::Arc;

use crate::sbi::timer::TICKS_PER_SEC;
use crate::thread::*;

use super::pass;
