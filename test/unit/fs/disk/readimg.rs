use crate::fs::disk::{Swap, DISKFS};
use crate::fs::FileSys;
use crate::io::prelude::*;
use crate::Result;

pub fn main() -> Result<()> {
    let file = DISKFS.open("exit".into())?;
    kprintln!("[DISKFS.READIMG] Length of exit: {}", file.len()?);

    kprintln!(
        "[DISKFS.READIMG] Swap file length: {}, pages: {}",
        Swap::len(),
        Swap::page_num()
    );
    let mut swap = Swap::lock();
    swap.write_from(0xfabcdeusize)?;
    swap.rewind()?;
    assert_eq!(swap.read_into::<usize>()?, 0xfabcde);
    kprintln!("[DISKFS.READIMG] Swap read/write works.");

    Ok(())
}
