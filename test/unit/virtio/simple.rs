use crate::device::virtio::{self, Virtio};

pub fn main() {
    let mut buf3 = [0; virtio::SECTOR_SIZE];

    Virtio::read_sector(1, &mut buf3);
    for i in buf3 {
        kprint!("{:#x} ", i);
    }

    kprintln!("Virtio simple test done.");
}
