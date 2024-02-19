//! I/O Interface.
//!

pub mod prelude {
    //! Exposes commonly used I/O interface
    pub use super::{Read, Seek, SeekFrom, Write};
}

use crate::{OsError, Result};

/// Read interface.
///
/// It resembles [`std::io::Read`].
pub trait Read {
    /// Reads into `buf`
    ///
    /// ## Return
    /// - `Ok(usize)`: the actual bytes read (can be 0).
    /// - `Err`: something unexpected happened.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Read the exact number of bytes required to fill `buf``.
    ///
    /// ## Difference with [`std::io::Read::read_exact()`]
    /// The std version will ignore interrupt error, and continue
    /// to read, which is not supported yet.
    ///
    /// ## Errors
    /// If the byte stream is insufficient to fill the buffer,
    /// [`OsError::UnexpectedEOF`] will occur.
    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
                Err(e) => return Err(e),
            }
        }
        if !buf.is_empty() {
            Err(OsError::UnexpectedEOF)
        } else {
            Ok(())
        }
    }

    /// Read a typed value from the byte stream based on
    /// [`Read::read_exact()`] and [`core::mem::size_of()`].
    ///
    /// ## Warning
    /// If failed, file cursor will not be reset.
    ///
    /// ## Errors
    /// When remained bytes in the stream is insufficient,
    /// [`OsError::UnexpectedEOF`] will occur.
    fn read_into<T>(&mut self) -> Result<T> {
        let mut v = alloc::vec![0u8; core::mem::size_of::<T>()];
        self.read_exact(v.as_mut_slice())?;
        let t = unsafe {
            core::mem::transmute_copy(v.as_ptr().cast::<T>().as_ref().ok_or(OsError::BadPtr)?)
        };
        Ok(t)
    }
}

/// Write interface.
///
/// It resembles [`std::io::Write`].
pub trait Write {
    /// Writes into the byte stream from the specified buffer.
    ///
    /// ## Return
    /// - `Ok(usize)`: the actual bytes written (can be 0).
    /// - `Err`: something unexpected happened.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Flush any buffered bytes to its destination
    fn flush(&mut self) -> Result<()>;

    /// Writes the entire buffer to the byte stream.
    ///
    /// ## Difference with [`std::io::Write::write_all()`]
    /// The `std` version will ignore interrupt error, and continue
    /// to write, which is not supported yet.
    ///
    /// ## Errors
    /// If the byte stream cannot hold all the contents (e.g., it cannot
    /// get extended), then [`OsError::UnexpectedEOF`] will occur.
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(OsError::UnexpectedEOF);
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Writes a typed value into the byte stream
    ///
    /// ## Warning
    /// If failed, file cursor will not be reset.
    ///
    /// ## Errors
    /// Fails when the byte stream doesn't have enough space left and cannot be extended.
    fn write_from<T>(&mut self, value: T) -> Result<()> {
        let mut v = alloc::vec![0u8; core::mem::size_of::<T>()];
        unsafe {
            *v.as_mut_ptr().cast::<T>() = core::mem::transmute_copy(&value);
        };
        self.write_all(v.as_slice())?;
        Ok(())
    }
}

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`] trait.
///
/// See also [`std::io::SeekFrom`] for usage of each
/// enumeration value.
pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
}

/// The `Seek` trait provides a cursor which can be moved within a stream of bytes.
///
/// To make things simpler, [`Seek::seek()`] is implemented
/// by default. However, [`Seek::len()`] and [`Seek::pos()`]
/// are required. Also, you are not supposed to call
/// [`Seek::len()`] or [`Seek::pos()`] in most cases.
///
/// ## Example
/// ```rust
/// struct FileHandler {
///     inode: Inode,
///     pos: usize,
/// }
///
/// impl Seek for FileHandler {
///     fn len(&self) -> Result<usize> {
///         Ok(self.inode.len())
///     }
///     fn pos(&mut self) -> Result<&mut usize> {
///         Ok(&mut self.pos)
///     }
/// }
/// ```
pub trait Seek {
    /// Gets the length of the byte stream.
    fn len(&self) -> Result<usize>;

    /// Retrieves a mutable reference of the current position.
    fn pos(&mut self) -> Result<&mut usize>;

    /// Seeks to the specified position.
    ///
    /// It is possible to seek beyond the end of the byte stream
    /// but cannot seek before byte 0.
    ///
    /// ## Return
    /// `Ok(usize)`: the new position.
    /// `Err`: something unexpected happened.
    fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        use self::SeekFrom::*;
        let len = self.len()?;
        let ilen = len as isize;
        let cur = { *(self.pos()?) };

        let new_pos = match pos {
            // std says that it is possible to seek beyond the end,
            // so just use max(0).
            Start(u) => u.max(0),
            End(i) => (ilen + i).max(0) as usize,
            Current(i) => (cur as isize + i).max(0) as usize,
        };
        *self.pos()? = new_pos;
        Ok(new_pos)
    }

    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0)).map(|_| ())
    }

    fn stream_position(&mut self) -> Result<usize> {
        self.seek(SeekFrom::Current(0))
    }
}
