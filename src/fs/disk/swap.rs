//! Swap file.
//!
// Swap may not be used.
#![allow(dead_code)]
use super::DISKFS;
use crate::fs::{File, FileSys};
use crate::io::Seek;
use crate::mem::PG_SIZE;
use crate::sync::{Lazy, Mutex, MutexGuard, Primitive};

pub struct Swap;

static SWAPFILE: Lazy<Mutex<File>> = Lazy::new(|| {
    Mutex::new(
        DISKFS
            .open(".glbswap".into())
            .expect("swap file \".glbswap\" should exist"),
    )
});

impl Swap {
    pub fn len() -> usize {
        SWAPFILE.lock().len().unwrap()
    }

    pub fn page_num() -> usize {
        // Round down.
        Self::len() / PG_SIZE
    }

    /// TODO: Design high-level interfaces, or do in lab3?
    pub fn lock() -> MutexGuard<'static, File, Primitive> {
        SWAPFILE.lock()
    }
}
