//! File System Interface
//!

pub mod disk;
pub mod inmem;

use alloc::sync::Arc;

use crate::io::{Read, Seek, Write};
use crate::sync::Mutex;
use crate::Result;

/* -------------------------------------------------------------------------- */
/*                                 File System                                */
/* -------------------------------------------------------------------------- */

/// File system interface.
///
/// A file system receives an `Identifier` type, which helps
/// the FS to locate specific files.
///
/// Typically a FS has only 1 instance during kernel running,
/// thus, this trait is designed to be [`Send`] and [`Sync`].
///
/// ## Examples
/// See [`inmem::MemFs`].
pub trait FileSys: Sync + Send + Sized {
    type Path;
    type Device;

    fn mount(device: Self::Device) -> Result<Self>;
    fn unmount(&self);

    fn open(&self, id: Self::Path) -> Result<File>;
    fn close(&self, file: File);
    fn create(&self, id: Self::Path) -> Result<File>;
    fn remove(&self, id: Self::Path) -> Result<()>;
}

/* -------------------------------------------------------------------------- */
/*                                Virtual Inode                               */
/* -------------------------------------------------------------------------- */

/// Virtual inode interface.
///
/// An inode is typically held by one or multiple [`File`]
/// and provides methods to allow [`File`]s access the data.
///
/// Typically an inode can be referenced by multiple [`File`]s,
/// thus, this trait is designed to be [`Send`] and [`Sync`].
pub trait Vnode: Sync + Send {
    fn read_at(&self, buf: &mut [u8], off: usize) -> Result<usize>;
    fn write_at(&self, buf: &[u8], off: usize) -> Result<usize>;
    fn deny_write(&self);
    fn allow_write(&self);

    fn len(&self) -> usize;
    fn resize(&self, size: usize) -> Result<()>;
    fn close(&self);
}

/* -------------------------------------------------------------------------- */
/*                                    File                                    */
/* -------------------------------------------------------------------------- */

/// A file descriptor, binding with a [`Vnode`], that has
/// independent position and permissions. It provides basic
/// file I/O interface.
#[derive(Clone)]
pub struct File {
    vnode: Arc<dyn Vnode>,
    pos: usize,
    deny_write: bool,
}

impl File {
    pub fn set_len(&mut self, size: usize) -> Result<()> {
        self.vnode.resize(size)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let cnt = self.vnode.read_at(buf, self.pos)?;
        self.pos += cnt;
        Ok(cnt)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let cnt = self.vnode.write_at(buf, self.pos)?;
        self.pos += cnt;
        Ok(cnt)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Seek for File {
    fn len(&self) -> Result<usize> {
        Ok(self.vnode.len())
    }

    fn pos(&mut self) -> Result<&mut usize> {
        Ok(&mut self.pos)
    }
}

impl File {
    pub fn new(vnode: Arc<dyn Vnode>) -> Self {
        Self {
            vnode,
            pos: 0,
            deny_write: false,
        }
    }

    pub fn deny_write(&mut self) {
        self.deny_write = true;
        self.vnode.deny_write();
    }
}

impl Drop for File {
    fn drop(&mut self) {
        if self.deny_write {
            self.vnode.allow_write();
        }
    }
}
