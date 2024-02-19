//! Disk sector free bitmap.
//!
use alloc::boxed::Box;
use alloc::vec;

use super::inode::Inode;
use super::{Inum, FREE_MAP_SECTOR, ROOT_DIR_SECTOR};
use crate::fs::Vnode;
use crate::{OsError, Result};

/// Disk sector free bitmap.
pub(super) struct FreeMap {
    size: u32,
    bits: Box<[u8]>,
}

impl FreeMap {
    /// Format the disk and return a free map.
    pub(super) fn new_format(size: u32) -> Result<Self> {
        let bitmap_len_in_byte = (size as usize + 7) / 8;
        let mut free_map = FreeMap {
            size,
            bits: vec![0; bitmap_len_in_byte].into(),
        };
        free_map.set(FREE_MAP_SECTOR);
        free_map.set(ROOT_DIR_SECTOR);
        let start = free_map.alloc(super::bytes_to_sectors(bitmap_len_in_byte))?;

        #[cfg(feature = "debug")]
        kprintln!(
            "Freemap format at sector {}, len={}",
            start,
            super::bytes_to_sectors(bitmap_len_in_byte)
        );

        Inode::create(FREE_MAP_SECTOR, start, bitmap_len_in_byte)?;
        Ok(free_map)
    }

    // Load from the disk.
    pub(super) fn load(size: u32) -> Result<Self> {
        let inode = Inode::open(FREE_MAP_SECTOR)?;
        let len = inode.len();
        assert!(len == (size as usize + 7) / 8);
        let mut free_map = FreeMap {
            size,
            bits: vec![0; len].into(),
        };
        inode.read_at(&mut free_map.bits, 0)?;
        Ok(free_map)
    }

    // Flush to the disk.
    pub(super) fn flush(&self) -> Result<()> {
        let inode = Inode::open(FREE_MAP_SECTOR)?;
        inode.write_at(&self.bits, 0)?;
        Ok(())
    }

    fn get(&self, sector: Inum) -> bool {
        assert!(sector < self.size);
        self.bits[sector as usize / 8] & (1 << sector % 8) != 0
    }

    fn set(&mut self, sector: Inum) {
        assert!(sector < self.size);
        self.bits[sector as usize / 8] |= 1 << sector % 8;
    }

    fn reset(&mut self, sector: Inum) {
        assert!(sector < self.size);
        self.bits[sector as usize / 8] &= !(1 << sector % 8);
    }

    /// Allocate a contiguous array of sectors with `cnt` length.
    pub(super) fn alloc(&mut self, cnt: u32) -> Result<Inum> {
        if cnt == 0 {
            return Ok(0);
        }
        if cnt >= self.size {
            return Err(OsError::DiskSectorAllocFail);
        }
        'outer: for mut i in 0..self.size - cnt {
            for _ in 0..cnt {
                if self.get(i) {
                    // `for` will skip this bit for us.
                    // Donot need `i += 1`.
                    continue 'outer;
                }
                // Skip the tested bits. If this '0' chunk is
                // unusable (not long enough), any of these sectors
                // will be unusable and need not to be tested again.
                i += 1;
            }

            for j in i - cnt..i {
                self.set(j);
            }
            return Ok(i - cnt);
        }

        Err(OsError::DiskSectorAllocFail)
    }

    /// Allocate a contiguous array but specify its start sector.
    pub(super) fn in_place_alloc(&mut self, start: Inum, cnt: u32) -> Result<()> {
        if cnt == 0 {
            return Ok(());
        }
        // Check.
        for i in start..start + cnt {
            if self.get(i) {
                return Err(OsError::DiskSectorAllocFail);
            }
        }
        // Check passed.
        for i in start..start + cnt {
            self.set(i);
        }
        Ok(())
    }

    /// Deallocate a contiguous array of sectors with ***length <= `cnt`***.
    pub(super) fn dealloc(&mut self, sector: Inum, cnt: u32) {
        for i in sector..sector + cnt {
            if !self.get(i) {
                break;
            }
            self.reset(i);
        }
    }
}
