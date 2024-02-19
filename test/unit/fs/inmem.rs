use alloc::boxed::Box;

use crate::fs::File;
use crate::fs::{inmem::MemFs, FileSys};
use crate::io::prelude::*;
use crate::thread;
use crate::Result;

pub fn main() {
    let fs = &MemFs::mount(()).unwrap();
    base::test(fs);
    // TODO: actually should use wait().
    sync::test(fs);
}

mod sync {
    use super::*;
    pub(super) fn test(fs: &MemFs) {
        const NUM: usize = 10;
        let sync = [0u8; NUM * core::mem::size_of::<usize>()];
        let f = fs.open(Box::from(sync)).unwrap();
        let fw = f.clone();

        thread::spawn("writer", || writer(fw, NUM));
        thread::schedule(); // firstly write sth.
        thread::spawn("reader", || reader(f, NUM));
        for _ in 0..30 {
            thread::schedule();
        }
    }

    fn writer(file: File, num: usize) {
        let ret = usize_writer(file, num);
        kprintln!("[Writer] {:?}", ret);
        assert_eq!(Ok(()), ret);
    }

    fn usize_writer(mut file: File, num: usize) -> Result<()> {
        file.seek(SeekFrom::Start(0))?;
        for i in 0..num {
            file.write_from(i)?;
            kprintln!("[Writer] {:>2} ->", i);
            thread::schedule();
        }
        Ok(())
    }

    fn reader(file: File, num: usize) {
        let ret = usize_reader(file, num);
        kprintln!("[Reader] {:?}", ret);
        assert_eq!(Ok(()), ret);
    }

    fn usize_reader(mut file: File, num: usize) -> Result<()> {
        file.seek(SeekFrom::Start(0))?;
        for i in 0..num {
            let v: usize = file.read_into()?;
            kprintln!("[Reader]    -> {:<2}", v);
            assert_eq!(i, v);
            thread::schedule();
        }
        Ok(())
    }
}

mod base {
    use super::*;
    pub(super) fn test(fs: &MemFs) {
        let raw: [u8; 8] = [0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x89];

        let mut f = fs.open(Box::from(raw)).unwrap();

        let a: usize = f.read_into().expect("fail to call read_into()");
        assert_eq!(a, 0x_89_70_6f_5e_4d_3c_2b_1a_usize);

        assert_eq!(f.stream_position(), Ok(8));
        assert_eq!(f.seek(SeekFrom::Current(1)), Ok(9));
        assert_eq!(f.seek(SeekFrom::End(-10)), Ok(0));
        assert_eq!(f.seek(SeekFrom::Start(4)), Ok(4));

        f.write_from(0x_12_34_u16)
            .expect("fail to call write_from()");
        f.seek(SeekFrom::Start(0)).unwrap();
        let a: usize = f.read_into().unwrap();
        assert_eq!(a, 0x_89_70_12_34_4d_3c_2b_1a_usize);
    }
}
