use crate::device::virtio::SECTOR_SIZE;
use crate::fs::disk::{Path, DISKFS};
use crate::fs::FileSys;
use crate::io::prelude::*;

pub fn main() {
    let buf = [1; 3 * SECTOR_SIZE];
    let mut buf2 = [0; 4 * SECTOR_SIZE];
    let random_off = SECTOR_SIZE - 100;
    {
        let mut file = DISKFS.create("/disk-simple".into()).unwrap();
        file.set_len(4 * SECTOR_SIZE).unwrap();

        // Go to middle of sector.
        file.seek(SeekFrom::Start(random_off)).unwrap();
        // However the file should still long enough to contain 3 sectors.
        assert!(file.write_all(&buf).is_ok());
        // Go to very beginning;
        file.rewind().unwrap();
        // Can read to end.
        assert!(file.read_exact(&mut buf2).is_ok());
        // Assert value.
        for i in 0..SECTOR_SIZE * 3 {
            assert_eq!(buf2[random_off + i], 1);
        }
        // Drop file and inode.
    }
    {
        // Reopen?
        let _file = DISKFS.open("/disk-simple".into()).unwrap();
        // Remove.
        DISKFS.remove("/disk-simple".into()).unwrap();
    }
    {
        // There isn't such file.
        assert_eq!(Path::exists("/disk-simple".into()), false);
    }
    kprintln!("[DISKFS.SIMPLE] Done.")
}
