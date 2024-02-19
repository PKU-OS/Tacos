use crate::device::virtio::SECTOR_SIZE;
use crate::fs::disk::DISKFS;
use crate::fs::FileSys;
use crate::io::prelude::*;
use crate::sync::{Mutex, Semaphore};
use crate::thread::*;
use crate::{OsError, Result};

const FNAME: &str = "/disk-sync";

pub fn main() {
    let barrier = alloc::sync::Arc::from(Mutex::<()>::new(()));
    let teller = alloc::sync::Arc::from(Semaphore::new(0));

    let b1 = barrier.clone();
    let t1 = teller.clone();
    let b2 = barrier.clone();
    let t2 = teller.clone();

    let child1 = move || -> Result<()> {
        // (1) Open the file.
        let mut f = DISKFS.open(FNAME.into())?;
        kprintln!("[DISKFS.SYNC] Child1 got file.");
        // (2) Write the thing asynchronously.
        f.rewind()?;
        for value in 0..SECTOR_SIZE / core::mem::size_of::<usize>() {
            f.write_from(value)?;
        }
        kprintln!("[DISKFS.SYNC] Child1 wrote things.");
        // (3) Barrier.
        t1.up();
        b1.lock();
        kprintln!("[DISKFS.SYNC] Child1 passed barrier.");
        // (5) Check.
        f.rewind()?;
        for value in 0..SECTOR_SIZE / core::mem::size_of::<usize>() {
            if f.read_into::<usize>()? != value {
                return Err(OsError::UserError);
            }
        }
        kprintln!("[DISKFS.SYNC] Child1 passed.");
        t1.up();
        Ok(())
    };
    let child2 = move || -> Result<()> {
        // (1) Open the file.
        let mut f = DISKFS.open(FNAME.into())?;
        kprintln!("[DISKFS.SYNC] Child2 got file.");
        // (2) Use a blocker to prevent file1 can be in-place extended.
        let _b = DISKFS.create("blker".into())?;
        // And asynchronously extend it, the file may be moved to other
        // part of the disk, but the write of child1 should be remain.
        f.set_len(2 * SECTOR_SIZE)?;
        // (3) Write sth.
        f.seek(SeekFrom::Start(SECTOR_SIZE))?;
        for value in 0..SECTOR_SIZE / core::mem::size_of::<usize>() {
            f.write_from(value)?;
        }
        kprintln!("[DISKFS.SYNC] Child2 wrote things.");
        // (4) Barrier.
        t2.up();
        b2.lock();
        kprintln!("[DISKFS.SYNC] Child2 passed barrier.");
        // (5) Check.
        f.seek(SeekFrom::Start(SECTOR_SIZE))?;
        for value in 0..SECTOR_SIZE / core::mem::size_of::<usize>() {
            if f.read_into::<usize>()? != value {
                return Err(OsError::UserError);
            }
        }
        kprintln!("[DISKFS.SYNC] Child2 passed.");
        t2.up();
        Ok(())
    };
    let mut f = DISKFS.create(FNAME.into()).unwrap();
    f.set_len(SECTOR_SIZE).unwrap();
    // Tag as removed for multi-time tests.
    DISKFS.remove(FNAME.into()).unwrap();
    kprintln!("[DISKFS.SYNC] Created file.");
    {
        spawn("child1", move || child1().unwrap());
        kprintln!("[DISKFS.SYNC] Spawned child1.");
        spawn("child2", move || child2().unwrap());
        kprintln!("[DISKFS.SYNC] Spawned child2.");
    }
    {
        // Barrier.
        let _guard = barrier.lock();
        teller.down();
        teller.down();
        kprintln!("[DISKFS.SYNC] Main passed barrier.");
    }
    teller.down();
    teller.down();
    kprintln!("[DISKFS.SYNC] Done.");
}
