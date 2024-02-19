use crate::device::virtio::SECTOR_SIZE;
use crate::fs::disk::DISKFS;
use crate::fs::{File, FileSys};
use crate::io::prelude::*;
use crate::{OsError, Result};

const INPLACE: u32 = 4;
const OUTOF: u32 = 8;

pub fn main() -> Result<()> {
    // Create a file with initially 1 sector.
    let mut f = DISKFS.create("txt".into())?;
    // Tag as remove for more than once test.
    DISKFS.remove("txt".into())?;

    in_place_extend(&mut f)?;
    out_of_place_extend(&mut f)?;
    shrink(&mut f)?;

    kprintln!("[DISKFS.CHLEN] Done.");
    Ok(())
}

fn shrink(f: &mut File) -> Result<()> {
    f.set_len(SECTOR_SIZE)?;
    let pos = f.seek(SeekFrom::End(0))?;
    // TODO: Error type.
    Some(pos - SECTOR_SIZE)
        .filter(|num| *num == 0)
        .ok_or(OsError::UserError)?;
    kprintln!("[DISKFS.CHLEN] Shrinking succeeds!");
    Ok(())
}

fn out_of_place_extend(f: &mut File) -> Result<()> {
    // Use a blocker to prevent in-place extend.
    let _b = DISKFS.create("blker".into())?;
    DISKFS.remove("blker".into())?;

    // Write to extend the len after in-place test.
    linear_write_usize(f, OUTOF)?;
    linear_check_usize(f, OUTOF)?;
    kprintln!("[DISKFS.CHLEN] Out-of-place extending succeeds!");
    Ok(())
}

fn in_place_extend(f: &mut File) -> Result<()> {
    f.set_len(SECTOR_SIZE)?;
    // Write to extend the initialize len.
    linear_write_usize(f, INPLACE)?;
    linear_check_usize(f, INPLACE)?;
    kprintln!("[DISKFS.CHLEN] In-place extending succeeds!");
    Ok(())
}

fn linear_check_usize(f: &mut File, sectors: u32) -> Result<()> {
    let cnt = s2u(sectors);
    f.rewind()?;
    for value in 0..cnt {
        let read: usize = f.read_into()?;
        if read != value {
            return Err(OsError::UserError);
        }
    }
    Ok(())
}

fn linear_write_usize(f: &mut File, sectors: u32) -> Result<()> {
    let cnt = s2u(sectors);
    f.rewind()?;
    for value in 0..cnt {
        f.write_from(value)?;
    }
    Ok(())
}

fn s2u(sectors: u32) -> usize {
    sectors as usize * SECTOR_SIZE / core::mem::size_of::<usize>()
}
