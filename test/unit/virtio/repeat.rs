use crate::device::virtio::{self, Virtio};

pub fn main() {
    let buf1 = [1; virtio::SECTOR_SIZE];
    let buf2 = [0; virtio::SECTOR_SIZE];
    let mut buf3 = [0; virtio::SECTOR_SIZE];

    for s in 0..10 {
        Virtio::write_sector(s, &buf1);
        Virtio::read_sector(s, &mut buf3);
        for i in buf3 {
            assert_eq!(i, 1);
        }

        Virtio::write_sector(s, &buf2);
        Virtio::read_sector(s, &mut buf3);
        for i in buf3 {
            assert_eq!(i, 0);
        }
    }

    kprintln!("Virtio repeat test done.");
}
