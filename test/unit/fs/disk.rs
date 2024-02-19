mod chlen;
mod readimg;
mod simple;
mod sync;

pub fn main() {
    #[cfg(feature = "test-fs-disk-simple")]
    {
        simple::main();
        readimg::main().unwrap();
    }
    #[cfg(not(feature = "test-fs-disk-simple"))]
    {
        // chlen::main().unwrap();
        sync::main();
    }
}
