use alloc::string::ToString;
use alloc::vec::Vec;

use crate::fs::disk::DISKFS;
use crate::fs::FileSys;
use crate::thread;
use crate::userproc;

const LEN: usize = 11;
const KILLED_USERPROC: [&str; LEN] = [
    // lab2 tests
    "bad-load",
    "bad-load2",
    "bad-jump",
    "bad-jump2",
    "bad-store",
    "bad-store2",
    // lab3 tests
    "mmap-unmap",
    "mmap-zero",
    "pt-bad-addr",
    "pt-grow-bad",
    "pt-write-code",
];
const KILLED_EXIT: isize = -1;
const NORMAL_EXIT: isize = 0;

pub fn main(cmd: &str) {
    kprintln!("Executing command {}", cmd);

    let argv: Vec<_> = cmd.split(" ").map(|s| s.to_string()).collect();
    let name = argv[0].clone();
    let file = DISKFS.open(name.as_str().into()).unwrap();

    let r = userproc::wait(userproc::execute(file, argv)).unwrap();
    if KILLED_USERPROC.iter().find(|n| name.eq(**n)).is_some() {
        assert_eq!(r, KILLED_EXIT);
    } else {
        assert_eq!(r, NORMAL_EXIT);
    }

    thread::schedule();
}
