//! Root dir.
//!
use super::{Inum, Path};
use crate::fs::File;
use crate::io::prelude::*;
use crate::{OsError, Result};

const FILE_NAME_LEN_MAX: usize = 28;

/// 32-byte entry.
#[repr(C)]
pub struct DirEntry {
    name: [u8; FILE_NAME_LEN_MAX],
    inum: Inum,
}

impl DirEntry {
    pub fn is_valid(&self) -> bool {
        self.name[0] != '#' as u8 && self.name[0] != 0
    }

    pub fn invalidate(&mut self) {
        self.name[0] = '#' as u8
    }
}

/// Currently only support root dir. Other files should not be dir.
pub struct RootDir(pub(super) File);

impl RootDir {
    /// Convert a path to inumber. This will iteratively search through the
    /// root dir entries, return the first entry that with the same name of given one.
    pub fn path2inum(&mut self, path: &Path) -> Result<Inum> {
        self.0.rewind()?;
        while let Ok(entry) = self.0.read_into::<DirEntry>() {
            if !entry.is_valid() {
                continue;
            }
            let name = unsafe {
                core::ffi::CStr::from_ptr(&entry.name as *const u8 as *const i8)
                    .to_str()
                    .or(Err(OsError::CstrFormatErr))?
            };
            // Deref `Path` to `String`.
            if path.eq(name) {
                return Ok(entry.inum);
            }
        }
        Err(OsError::NoSuchFile)
    }

    /// Check if there is a file with the given name.
    ///
    /// # See
    /// [`path2inum()`].
    pub fn exists(&mut self, path: &Path) -> bool {
        self.path2inum(path).is_ok()
    }

    /// Insert an entry with given name and inumber.
    pub fn insert(&mut self, path: &Path, inum: Inum) -> Result<()> {
        if !path.is_ascii() {
            return Err(OsError::CstrFormatErr);
        }
        let pos = self.first_invalid()?;
        let mut entry = DirEntry {
            name: [0; FILE_NAME_LEN_MAX],
            inum,
        };
        entry.name[0..core::cmp::min(path.len(), FILE_NAME_LEN_MAX - 1)]
            .copy_from_slice(path.as_bytes());
        self.0.seek(SeekFrom::Start(pos))?;
        self.0.write_from(entry)?;
        Ok(())
    }

    /// Remove an entry from root dir by given inumber.
    pub fn remove(&mut self, inum: Inum) -> Result<()> {
        self.0.rewind()?;
        while let Ok(mut entry) = self.0.read_into::<DirEntry>() {
            if entry.inum == inum {
                entry.invalidate();
                self.0.seek(SeekFrom::Current(-32))?;
                self.0.write_from(entry)?;
            }
        }
        // Ignore unexisting file.
        Ok(())
    }

    /// Find the first invalid place of entry. We may use it to insert a new one later.
    fn first_invalid(&mut self) -> Result<usize> {
        self.0.rewind()?;
        while let Ok(entry) = self.0.read_into::<DirEntry>() {
            if !entry.is_valid() {
                return Ok(self.0.seek(SeekFrom::Current(-32)).unwrap());
            }
        }
        Err(OsError::RootDirFull)
    }
}
