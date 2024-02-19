//! VIRTIO Block Device Support
//!
//! This module is a very simple implementation of VIRTIO-v1.2.
//! See the spec for more information.
//!

use alloc::boxed::Box;
use core::{arch, ptr};

use crate::mem::{MMIO_BASE, VM_OFFSET};
use crate::sync::{Lazy, Mutex, Semaphore};

/* -------------------------------------------------------------------------- */
/*                                  INTERFACE                                 */
/* -------------------------------------------------------------------------- */

/// Sector size.
pub const SECTOR_SIZE: usize = 512;

/* -------------------------------------------------------------------------- */
/*                                    MMIO                                    */
/* -------------------------------------------------------------------------- */
// A subset of MMIO Virtio Device Registers.
// RO = Read Only, WO = Write Only, RW = Read Write.
// See section 4.2.2 in the spec for more information.
const MAGIC_VALUE: *const u32 = (MMIO_BASE + 0x0) as _; // RO
const VERSION: *const u32 = (MMIO_BASE + 0x4) as _; // RO
const DEVICE_ID: *const u32 = (MMIO_BASE + 0x8) as _; // RO
const DEVICE_FEATURES: *const u32 = (MMIO_BASE + 0x10) as _; // RO
const DRIVER_FEATURES: *mut u32 = (MMIO_BASE + 0x20) as _; // WO
const QUEUE_SEL: *mut u32 = (MMIO_BASE + 0x30) as _; // WO
const QUEUE_NUM_MAX: *const u32 = (MMIO_BASE + 0x34) as _; // RO
const QUEUE_NUM: *mut u32 = (MMIO_BASE + 0x38) as _; // WO
const QUEUE_READY: *mut u32 = (MMIO_BASE + 0x44) as _; // RW
const QUEUE_NOTIFY: *mut u32 = (MMIO_BASE + 0x50) as _; // WO
const INTERRUPT_STATUS: *const u32 = (MMIO_BASE + 0x60) as _; // RO
const INTERRUPT_ACK: *mut u32 = (MMIO_BASE + 0x64) as _; // WO
const STATUS: *mut u32 = (MMIO_BASE + 0x70) as _; // RW
const QUEUE_DESC_LOW: *mut u32 = (MMIO_BASE + 0x80) as _; // WO
const QUEUE_DESC_HIGH: *mut u32 = (MMIO_BASE + 0x84) as _; // WO
const QUEUE_DRIVER_LOW: *mut u32 = (MMIO_BASE + 0x90) as _; // WO
const QUEUE_DRIVER_HIGH: *mut u32 = (MMIO_BASE + 0x94) as _; // WO
const QUEUE_DEVICE_LOW: *mut u32 = (MMIO_BASE + 0xa0) as _; // WO
const QUEUE_DEVICE_HIGH: *mut u32 = (MMIO_BASE + 0xa4) as _; // WO
const CONFIG: *mut u8 = (MMIO_BASE + 0x100) as _; // RW

// A subset of status fields.
// See section 2.1 in the spec for more information.
bitflags::bitflags! {
    struct Status: u32 {
        const ACKNOWLEDGE = 1;
        const DRIVER = 2;
        const DRIVER_OK = 4;
        const FEATURES_OK = 8;
    }
}

/* -------------------------------------------------------------------------- */
/*                                  VIRTQUEUE                                 */
/* -------------------------------------------------------------------------- */

// A singleton struct representing the virtio device.
// See section 2.7 in the spec for more information.
pub struct Virtio {
    desc_table: *mut [Desc; QUEUE_SIZE as _], // Descriptor table.
    avail: *mut Avail,                        // Available ring.
    used: *mut Used,                          // Used ring.
    capacity: u64,                            // Disk capacity, in 512-byte sectors.
}

// # Safety
//
// Pointers in `Virtio` are only used in this type, and `Virtio` is a singleton type.
// Therefore, These pointers are only used by one thread at a time.
unsafe impl Send for Virtio {}

// According to the spec, this must be a power of 2.
// We only use 3 descriptors at a time, so any size larger than 3 is OK.
const QUEUE_SIZE: u16 = 4;

// Desctriptor.
#[repr(C)]
#[derive(Default)]
struct Desc {
    addr: u64,
    len: u32,
    flag: DescFlag,
    next: u16,
}

// Desctriptor flag.
bitflags::bitflags! {
    #[derive(Default)]
    pub struct DescFlag : u16 {
        const NEXT = 1;
        const WRITE = 2;
    }
}

// Available ring.
#[repr(C)]
#[derive(Default)]
struct Avail {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE as _],
}

// Used ring.
#[repr(C)]
#[derive(Default)]
struct Used {
    flags: u16,
    idx: u16,
    ring: [UsedElem; QUEUE_SIZE as _],
}

// Used ring element.
#[repr(C)]
#[derive(Default)]
struct UsedElem {
    id: u32,
    len: u32,
}

/* -------------------------------------------------------------------------- */
/*                               INITIALIZATION                               */
/* -------------------------------------------------------------------------- */

impl Virtio {
    fn init(&mut self) {
        unsafe {
            // Start device initialization.
            // See section 4.2.3.1 in the spec for more information.
            let magic = MAGIC_VALUE.read_volatile();
            assert_eq!(magic, 0x74726976);
            let version = VERSION.read_volatile();
            assert_eq!(version, 0x2);

            // We only support Virtio Block Device.
            // See section 5.2 in the spec for more information.
            let device_id = DEVICE_ID.read_volatile();
            assert_eq!(device_id, 0x2);

            // Reset the device.
            let mut status = Status { bits: 0 };
            STATUS.write_volatile(status.bits());

            // Set the ACKNOWLEDGE status bit.
            status |= Status::ACKNOWLEDGE;
            STATUS.write_volatile(status.bits());

            // Set the DRIVER status bit.
            status |= Status::DRIVER;
            STATUS.write_volatile(status.bits());

            // Negotiate features. We don't support any feature.
            _ = DEVICE_FEATURES.read_volatile();
            DRIVER_FEATURES.write_volatile(0);

            // Finish feature negotiation.
            status |= Status::FEATURES_OK;
            STATUS.write_volatile(status.bits());

            // Ensure the FEATURES_OK status bit is still set.
            status = Status {
                bits: STATUS.read_volatile(),
            };
            assert!(status.contains(Status::FEATURES_OK));

            // Get capacity of the disk.
            let capacity = (CONFIG as *mut u64).read_volatile();
            self.capacity = capacity;

            #[cfg(feature = "debug")]
            kprintln!("Disk capacity: {} * {}B", capacity, SECTOR_SIZE);

            // Select queue 0. We only use queue 0.
            QUEUE_SEL.write_volatile(0);

            // Ensure the queue is not already in use.
            let ready = QUEUE_READY.read_volatile();
            assert_eq!(ready, 0);

            // Negotiate queue size.
            let max_size = QUEUE_NUM_MAX.read_volatile();
            assert!(QUEUE_SIZE <= max_size as _);
            QUEUE_NUM.write_volatile(QUEUE_SIZE as _);

            // Allocate and zero the queues.
            self.desc_table = Box::into_raw(Box::default());
            self.avail = Box::into_raw(Box::default());
            self.used = Box::into_raw(Box::default());
            self.desc_table.write(Default::default());
            self.avail.write(Default::default());
            self.used.write(Default::default());

            // Tell physical addresses of the queues to the device.
            QUEUE_DESC_LOW.write_volatile((self.desc_table as usize - VM_OFFSET) as u32);
            QUEUE_DESC_HIGH.write_volatile((self.desc_table as usize - VM_OFFSET >> 32) as u32);
            QUEUE_DRIVER_LOW.write_volatile((self.avail as usize - VM_OFFSET) as u32);
            QUEUE_DRIVER_HIGH.write_volatile((self.avail as usize - VM_OFFSET >> 32) as u32);
            QUEUE_DEVICE_LOW.write_volatile((self.used as usize - VM_OFFSET) as u32);
            QUEUE_DEVICE_HIGH.write_volatile((self.used as usize - VM_OFFSET >> 32) as u32);

            // The queue is ready after this.
            QUEUE_READY.write_volatile(0x1);

            // The device is live after this.
            status |= Status::DRIVER_OK;
            STATUS.write_volatile(status.bits());
        }
    }

    pub fn get() -> &'static Mutex<Self> {
        static INSTANCE: Lazy<Mutex<Virtio>> = Lazy::new(|| {
            let virtio = Mutex::new(Virtio {
                desc_table: ptr::null_mut(),
                avail: ptr::null_mut(),
                used: ptr::null_mut(),
                capacity: 0,
            });
            virtio.lock().init();
            virtio
        });

        INSTANCE.get()
    }

    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Read a sector from virtio block device.
    /// # Example
    ///
    /// ```
    /// let mut buf = [0; SECTOR_SIZE];
    /// read_sector(0, &mut buf);   // Read from sector 0.
    /// ```
    pub fn read_sector(sector: u64, buf: &mut [u8; SECTOR_SIZE]) {
        Virtio::get().lock().read_sector_impl(sector, buf);
    }

    /// Write a sector to virtio block device.
    /// # Example
    ///
    /// ```
    /// let buf = [0; SECTOR_SIZE];
    /// write_sector(0, &mut buf);  // Write to sector 0.
    /// ```
    pub fn write_sector(sector: u64, buf: &[u8; SECTOR_SIZE]) {
        Virtio::get().lock().write_sector_impl(sector, buf);
    }
}

/* -------------------------------------------------------------------------- */
/*                                READ / WRITE                                */
/* -------------------------------------------------------------------------- */

// Down'ed by a thread to wait for notification from the disk.
// Up'ed by interrupt handler.
static USED_RING_NOTIFICATION: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(0));

// Part of the block request structure.
// See section 5.2.6 in the spec for more information.
#[repr(C)]
struct BlkReqHeader {
    req_type: BlkReqType,
    reserved: u32,
    sector: u64,
}

// A subset of block request types.
#[repr(u32)]
enum BlkReqType {
    In = 0,
    Out = 1,
}

impl Virtio {
    fn read_sector_impl(&mut self, sector: u64, buf: &mut [u8; SECTOR_SIZE]) {
        // Construct block request header and tailer.
        let header = BlkReqHeader {
            req_type: BlkReqType::In,
            reserved: 0,
            sector,
        };
        let mut status: u8 = 0xff;

        unsafe {
            // Initialize the descriptors. See section 2.7.5 in the spec for more information.
            (*self.desc_table)[0].addr = (ptr::addr_of!(header) as usize - VM_OFFSET) as _;
            (*self.desc_table)[0].len = core::mem::size_of::<BlkReqHeader>() as _;
            (*self.desc_table)[0].flag = DescFlag::NEXT;
            (*self.desc_table)[0].next = 1;
            (*self.desc_table)[1].addr = (buf as *mut _ as usize - VM_OFFSET) as _;
            (*self.desc_table)[1].len = SECTOR_SIZE as _;
            (*self.desc_table)[1].flag = DescFlag::NEXT | DescFlag::WRITE;
            (*self.desc_table)[1].next = 2;
            (*self.desc_table)[2].addr = (ptr::addr_of_mut!(status) as usize - VM_OFFSET) as _;
            (*self.desc_table)[2].len = 1;
            (*self.desc_table)[2].flag = DescFlag::WRITE;
            (*self.desc_table)[2].next = 0; // Actually unnecessary.

            // Supply buffer to the device, and wait for notification.
            self.supply_buffer(0);
            USED_RING_NOTIFICATION.get().down();

            // Check if the operation was successful.
            assert_eq!(status, 0);
            assert_eq!(
                (*self.used).ring[((*self.used).idx.wrapping_sub(1) % QUEUE_SIZE) as usize].len,
                (SECTOR_SIZE + 1) as _
            );

            // Tell the device we've done with the interrupt.
            INTERRUPT_ACK.write_volatile(1);
        }
    }

    // See comments in read_sector() for more information.
    fn write_sector_impl(&mut self, sector: u64, buf: &[u8; SECTOR_SIZE]) {
        let header = BlkReqHeader {
            req_type: BlkReqType::Out,
            reserved: 0,
            sector,
        };
        let mut status: u8 = 0xff;
        unsafe {
            (*self.desc_table)[0].addr = (ptr::addr_of!(header) as usize - VM_OFFSET) as _;
            (*self.desc_table)[0].len = core::mem::size_of::<BlkReqHeader>() as _;
            (*self.desc_table)[0].flag = DescFlag::NEXT;
            (*self.desc_table)[0].next = 1;
            (*self.desc_table)[1].addr = (buf as *const _ as usize - VM_OFFSET) as _;
            (*self.desc_table)[1].len = SECTOR_SIZE as _;
            (*self.desc_table)[1].flag = DescFlag::NEXT;
            (*self.desc_table)[1].next = 2;
            (*self.desc_table)[2].addr = (ptr::addr_of_mut!(status) as usize - VM_OFFSET) as _;
            (*self.desc_table)[2].len = 1;
            (*self.desc_table)[2].flag = DescFlag::WRITE;
            (*self.desc_table)[2].next = 0;

            self.supply_buffer(0);
            USED_RING_NOTIFICATION.get().down();

            assert_eq!(status, 0);
            assert_eq!(
                (*self.used).ring[((*self.used).idx.wrapping_sub(1) % QUEUE_SIZE) as usize].len,
                1
            );

            INTERRUPT_ACK.write_volatile(1);
        }
    }

    // Supply a buffer to the device.
    // See section 2.7.13 in the spec for more information.
    unsafe fn supply_buffer(&mut self, id: u16) {
        // Update the availble ring.
        (*self.avail).ring[((*self.avail).idx % QUEUE_SIZE) as usize] = id;

        // Ensure the device sees the update before next step.
        arch::asm!("fence w,w");

        // Update the availble ring index.
        (*self.avail).idx = (*self.avail).idx.wrapping_add(1);

        // Ensure the device sees the update before next step.
        arch::asm!("fence w,w");

        // Notify the device.
        QUEUE_NOTIFY.write_volatile(0);
    }
}

/// Handle the interrupt.
pub fn handle_interrupt() {
    // Check interrupt status.
    // See section 4.2.3.4 in the spec for more information.
    let status = unsafe { INTERRUPT_STATUS.read_volatile() };
    assert_eq!(status, 1);

    // Wake up the waiting thread.
    USED_RING_NOTIFICATION.get().up();
}
