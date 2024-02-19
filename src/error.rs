//! Os Error Handling.
//!

/// Possible errors in the OS
#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum OsError {
    BadPtr = -1,
    UnexpectedEOF = -2,
    NoSuchFile = -3,
    UnknownFormat = -4,
    UserError = -5,
    CreateExistInode = -6,
    OpenInvalidInode = -7,
    DiskSectorAllocFail = -8,
    RootDirFull = -9,
    CstrFormatErr = -10,
    ArgumentTooLong = -11,
    InvalidFileMode = -12,
    FileNotOpened = -13,
}
