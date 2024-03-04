//! Disk inode.
//!
use alloc::sync::Arc;
use core::convert::TryInto;
use core::ops::{Deref, Drop};
use core::{cmp, mem};

use super::{bytes_to_sectors, Inum, DISKFS};
use crate::device::virtio::{Virtio, SECTOR_SIZE};
use crate::fs::Vnode;
use crate::mem::{Translate, PG_MASK, PG_SIZE};
use crate::sync::Mutex;
use crate::{OsError, Result};

const INODE_PADDING: usize = SECTOR_SIZE - core::mem::size_of::<DiskInodeInner>();
const INODE_MAGIC: u32 = 0x494e4f44;

/// An inode on the disk.
///
/// Size of this must be `SECTOR_SIZE`.
#[repr(C)]
struct DiskInode {
    inner: DiskInodeInner,
    padding: [u8; INODE_PADDING],
}

/// Metadata of on disk inode.
///
/// This struct is used to help calculate padding bytes of an inode.
#[repr(C)]
#[derive(Debug)]
struct DiskInodeInner {
    /// Start sector.
    start: Inum,
    /// Length in bytes.
    len: u32,
    magic: u32,
}

/// In memory inode descriptor.
///
/// Drop when inode leaves memory.
struct InodeDesc {
    /// Sector number. Inumber interchangeably.
    sector: Inum,
    /// Whether to remove this inode on drop.
    removed: bool,
    /// Allocated-bytes - file-len.
    /// Used for lazy shrink.
    shrink_len: u32,
    /// Deny write to a running file.
    deny_write: u32,
}

impl InodeDesc {
    fn new(sector: Inum, shrink_len: u32) -> Self {
        Self {
            sector,
            removed: false,
            deny_write: 0,
            shrink_len,
        }
    }
}

/// Wrapper of in memory inode.
pub struct Inode(Mutex<(InodeDesc, DiskInode)>);

impl Inode {
    /// Tag to remove the inode on drop.
    pub fn remove(&self) {
        self.0.lock().0.removed = true;
    }

    /// Create an inode at `sector` with length of `len`.
    ///
    /// `sector` must be a sector allocated from free map. Also, the content must be
    /// pre allocated from free map. This will not do any sector allocation.
    pub fn create(sector: Inum, start: Inum, len: usize) -> Result<Arc<Self>> {
        // Create file on the disk.
        let sector_num = bytes_to_sectors(len);
        let disk_inode = DiskInode {
            inner: DiskInodeInner {
                start: start as _,
                len: len as _,
                magic: INODE_MAGIC,
            },
            padding: [0; INODE_PADDING],
        };
        unsafe {
            Virtio::write_sector(sector as _, mem::transmute(&disk_inode));
        }

        // Zero the file.
        let zeros = [0; SECTOR_SIZE];
        for i in 0..sector_num {
            Virtio::write_sector((start + i) as _, &zeros);
        }

        let desc = InodeDesc::new(sector, 0);
        Ok(Arc::from(Self(Mutex::new((desc, disk_inode)))))
    }

    /// Open the inode at `sector`.
    ///
    /// # Return
    /// - `Ok(Arc<Inode>)`: successfully opened the inode.
    /// - `Err(InvalidInode)`: failed, specifically, the inode magic is incorrect.
    pub fn open(sector: Inum) -> Result<Arc<Self>> {
        let desc = InodeDesc::new(sector, 0);
        let mut data = DiskInode {
            inner: DiskInodeInner {
                start: 0,
                len: 0,
                magic: 0,
            },
            padding: [0; INODE_PADDING],
        };
        unsafe {
            Virtio::read_sector(sector as _, mem::transmute(&mut data));
        }

        if data.inner.magic != INODE_MAGIC {
            Err(OsError::OpenInvalidInode)
        } else {
            Ok(Arc::from(Self(Mutex::new((desc, data)))))
        }
    }

    fn resize_inner(desc: &mut InodeDesc, data: &mut DiskInode, size: usize) -> Result<()> {
        let newlen = size as u32;
        let flush_len = |a: &InodeDesc, b: &mut DiskInode| {
            b.inner.len = newlen;
            unsafe {
                Virtio::write_sector(a.sector as _, &mem::transmute_copy(b));
            }
        };

        let mut freemap = DISKFS.free_map.lock();
        if newlen == data.inner.len {
            Ok(())
        }
        // Lazy shrink.
        else if newlen < data.inner.len {
            let shrink = data.inner.len - newlen;
            // The new len is immediately flush to disk.
            // So if the new len is still shorter,
            // we confirm a shorter length than the last
            // newest length (may be shrunk by other threads).
            // So the shrink_len can be updated by adding.
            desc.shrink_len += shrink;
            // Immediately flush to disk.
            flush_len(desc, data);
            Ok(())
        }
        // Extend.
        else {
            // Try to alloc in-place.
            let oldlen = data.inner.len + desc.shrink_len;
            if newlen <= oldlen {
                // No need to re alloc, but should flush new len to disk.
                desc.shrink_len = oldlen - newlen;
                flush_len(desc, data);
                Ok(())
            } else {
                let cnt = bytes_to_sectors(newlen as _) - bytes_to_sectors(oldlen as _);
                let start = data.inner.start + bytes_to_sectors(oldlen as _);
                // Try in-place extend.
                if freemap.in_place_alloc(start, cnt).is_ok() {
                    desc.shrink_len = 0;
                    flush_len(desc, data);
                    return Ok(());
                } else {
                    let cnt = bytes_to_sectors(newlen as _);
                    let old_start = data.inner.start;
                    let new_start = freemap.alloc(cnt)?;
                    // Copy.
                    let mut bounce = [0u8; SECTOR_SIZE];
                    for i in 0..bytes_to_sectors(oldlen as _) {
                        Virtio::read_sector((old_start + i) as _, &mut bounce);
                        Virtio::write_sector((new_start + i) as _, &bounce);
                    }
                    freemap.dealloc(old_start, bytes_to_sectors(oldlen as _));
                    desc.shrink_len = 0;
                    data.inner.start = new_start;
                    flush_len(desc, data);
                    Ok(())
                }
            }
        }
    }
}

impl Vnode for Inode {
    fn inum(&self) -> usize {
        self.0.lock().0.sector as usize
    }

    fn len(&self) -> usize {
        self.0.lock().1.inner.len as _
    }

    fn read_at(&self, buf: &mut [u8], mut off: usize) -> Result<usize> {
        let mut bytes_read = 0;
        let mut buf_left = buf.len(); // Bytes left in `buf`.

        // We must acquire lock during the whole process
        // to avoid being resized by other threads.
        let guard = self.0.lock();
        let (_, data) = &*guard;

        let start = data.inner.start;
        let len = data.inner.len as usize;

        loop {
            // Read from `sector` at `sector_offset`.
            let sector = start as usize + off / SECTOR_SIZE;
            let sector_offset = off % SECTOR_SIZE;

            let inode_left = len.saturating_sub(off); // Bytes left in inode.
            let sector_left = SECTOR_SIZE - sector_offset; // Bytes left in sector.
            let chunk_size = cmp::min(cmp::min(inode_left, sector_left), buf_left); //  Actual bytes to read.
            if chunk_size == 0 {
                break;
            }

            let page_off = (buf.as_ptr() as usize + bytes_read) & PG_MASK;

            if (chunk_size == SECTOR_SIZE) && (page_off <= PG_SIZE - SECTOR_SIZE) {
                // Virtio only supports kernel buffers.
                // So we need to convert the possible user buffer into kernel buffer.
                let buf_kvm: &mut [u8; SECTOR_SIZE] = (&mut buf
                    [bytes_read..bytes_read + SECTOR_SIZE])
                    .translate()
                    .ok_or(OsError::BadPtr)?
                    .try_into()
                    .unwrap();
                Virtio::read_sector(sector as _, buf_kvm);
            } else {
                // We need a bounce buffer.
                let mut bounce = [0; SECTOR_SIZE];
                Virtio::read_sector(sector as _, &mut bounce);
                buf[bytes_read..bytes_read + chunk_size]
                    .copy_from_slice(&bounce[sector_offset..sector_offset + chunk_size]);
            }

            // Advance.
            buf_left -= chunk_size;
            off += chunk_size;
            bytes_read += chunk_size;
        }

        Ok(bytes_read)
    }

    fn write_at(&self, buf: &[u8], mut off: usize) -> Result<usize> {
        if self.0.lock().0.deny_write > 0 {
            return Err(OsError::InvalidFileMode);
        }

        let mut bytes_written = 0;
        let mut buf_left = buf.len();

        // We must acquire lock during the whole process
        // to avoid being resized by other threads.
        let mut guard = self.0.lock();
        let (desc, data) = &mut *guard;

        let mut start = data.inner.start;
        let mut len = data.inner.len as usize;

        if len < off + buf.len() {
            let newlen = off + buf.len();
            Self::resize_inner(desc, data, newlen)?;
            start = data.inner.start;
            len = data.inner.len as usize;
        }

        loop {
            let sector = start as usize + off / SECTOR_SIZE;
            let sector_offset = off % SECTOR_SIZE;

            let inode_left = len.saturating_sub(off);
            let sector_left = SECTOR_SIZE - sector_offset;
            let chunk_size = cmp::min(cmp::min(inode_left, sector_left), buf_left);
            if chunk_size == 0 {
                break;
            }

            let page_off = (buf.as_ptr() as usize + bytes_written) & PG_MASK;

            if (chunk_size == SECTOR_SIZE) && (page_off <= PG_SIZE - SECTOR_SIZE) {
                // Virtio only supports kernel buffers.
                // So we need to convert the possible user buffer into kernel buffer.
                let buf_kvm: &[u8; SECTOR_SIZE] = (&buf
                    [bytes_written..bytes_written + SECTOR_SIZE])
                    .translate()
                    .ok_or(OsError::BadPtr)?
                    .try_into()
                    .unwrap();
                Virtio::write_sector(sector as _, buf_kvm);
            } else {
                // We need a bounce buffer, preserving old bytes which should not be written.
                let mut bounce = [0; SECTOR_SIZE];
                Virtio::read_sector(sector as _, &mut bounce);
                bounce[sector_offset..sector_offset + chunk_size]
                    .copy_from_slice(&buf[bytes_written..bytes_written + chunk_size]);
                Virtio::write_sector(sector as _, &bounce);
            }

            buf_left -= chunk_size;
            off += chunk_size;
            bytes_written += chunk_size;
        }

        Ok(bytes_written)
    }

    fn resize(&self, newlen: usize) -> Result<()> {
        let mut guard = self.0.lock();
        let (desc, data) = &mut *guard;
        Self::resize_inner(desc, data, newlen)
    }

    fn close(&self) {
        let l = self.0.lock();
        let (desc, data) = l.deref();
        if desc.shrink_len > 0 {
            // Sector count to deallocate.
            // == floor(desc.shrink_len / SECTOR_SIZE)
            let cnt = desc.shrink_len / SECTOR_SIZE as u32;
            // The start sector to deallocate.
            // == inner.start + ceil(inner.len / SECTOR_SIZE)
            let sector =
                data.inner.start + (data.inner.len + SECTOR_SIZE as u32 - 1) / SECTOR_SIZE as u32;
            let mut freemap = DISKFS.free_map.lock();
            freemap.dealloc(sector, cnt);
        }
        if desc.removed {
            // Remove the inode from the disk.
            let mut rootdir = DISKFS.root_dir.lock();
            rootdir
                .remove(desc.sector)
                .expect("Failed to remove from root dir");
            let mut freemap = DISKFS.free_map.lock();
            freemap.dealloc(data.inner.start as _, bytes_to_sectors(data.inner.len as _));
            freemap.dealloc(desc.sector, 1);
        }
    }

    fn deny_write(&self) {
        self.0.lock().0.deny_write += 1;
    }

    fn allow_write(&self) {
        self.0.lock().0.deny_write -= 1;
    }
}

impl Drop for Inode {
    fn drop(&mut self) {
        self.close()
    }
}
