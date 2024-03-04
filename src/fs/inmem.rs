use alloc::boxed::Box;
use alloc::sync::Weak;
use alloc::vec::Vec;
use core::cmp::min;

use crate::{OsError, Result};

use super::*;

/* -------------------------------------------------------------------------- */
/*                                   FileSys                                  */
/* -------------------------------------------------------------------------- */

/// An in-memory pseudo file system, wrapping memory buffers
/// with file-like interfaces.
///
/// # Panic
/// This struct only support [`FileSys::open()`], which
/// copy and wrap a byte buffer into a [`File`]. Calls to
/// [`FileSys::close()`], [`FileSys::create()`] and
/// [`FileSys::remove()`] will panic.
pub struct MemFs {
    oft: Mutex<Vec<Weak<Inode>>>,
}

impl FileSys for MemFs {
    type Device = ();
    type Path = Box<[u8]>;

    fn mount(_device: Self::Device) -> Result<Self> {
        Ok(Self {
            oft: Mutex::new(Vec::new()),
        })
    }

    fn unmount(&self) {
        unimplemented!();
    }

    fn open(&self, id: Self::Path) -> Result<File> {
        let buf = Mutex::new(id);
        let vnode = Arc::new(Inode { buf });
        let weak = Arc::downgrade(&vnode);
        self.oft.lock().push(weak);

        Ok(File::new(vnode))
    }

    fn close(&self, _file: File) {
        unimplemented!();
    }

    fn create(&self, _id: Self::Path) -> Result<File> {
        unimplemented!();
    }

    fn remove(&self, _id: Self::Path) -> Result<()> {
        unimplemented!();
    }
}

/* -------------------------------------------------------------------------- */
/*                                    Inode                                   */
/* -------------------------------------------------------------------------- */

// TODO: should it be pub or not
struct Inode {
    buf: Mutex<Box<[u8]>>,
}

impl Vnode for Inode {
    fn inum(&self) -> usize {
        self.buf.lock().as_ptr() as usize
    }

    fn len(&self) -> usize {
        self.buf.lock().len()
    }

    fn read_at(&self, buf: &mut [u8], off: usize) -> Result<usize> {
        // Protect during the whole process.
        let lock = self.buf.lock();
        if off >= lock.len() {
            return Err(OsError::UnexpectedEOF);
        }

        let len = min(lock.len() - off, buf.len());

        buf[..len].copy_from_slice(&lock[off..off + len]);
        Ok(len)
    }

    fn write_at(&self, buf: &[u8], off: usize) -> Result<usize> {
        // Protect during the whole process.
        let mut lock = self.buf.lock();
        if off >= lock.len() {
            return Err(OsError::UnexpectedEOF);
        }

        let len = min(lock.len() - off, buf.len());

        let nb = &mut lock[off..off + len];
        nb.copy_from_slice(&buf[..len]);
        Ok(len)
    }

    fn resize(&self, _size: usize) -> Result<()> {
        unimplemented!();
    }

    fn close(&self) {
        unimplemented!();
    }

    // TODO: Impl deny write for mem file.
    fn deny_write(&self) {}
    fn allow_write(&self) {}
}
