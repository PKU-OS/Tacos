mod fs;
mod malloc;
mod sync;
mod thread;
mod virtio;

pub fn main() {
    #[cfg(any(feature = "test-sync", feature = "test-sync-condvar"))]
    sync::condvar::main();
    #[cfg(any(feature = "test-sync", feature = "test-sync-sema_fifo"))]
    sync::sema_fifo::main();

    #[cfg(any(feature = "test-thread", feature = "test-thread-adder"))]
    thread::adder::main();
    #[cfg(any(feature = "test-thread", feature = "test-thread-block"))]
    thread::block::main();

    // ! This should fail.
    #[cfg(any(feature = "test-thread", feature = "test-thread-bomb"))]
    thread::bomb::main();

    #[cfg(any(feature = "test-thread", feature = "test-thread-spin_yield"))]
    thread::spin_yield::main();
    #[cfg(any(feature = "test-thread", feature = "test-thread-spin_interrupt"))]
    thread::spin_interrupt::main();

    /* ------------------------------- VIRTIO TEST ------------------------------ */

    #[cfg(any(feature = "test-virtio", feature = "test-virtio-simple"))]
    virtio::simple::main();

    #[cfg(any(feature = "test-virtio", feature = "test-virtio-repeat"))]
    virtio::repeat::main();

    #[cfg(feature = "test-mem-malloc")]
    malloc::main();

    #[cfg(feature = "test-fs-inmem")]
    fs::inmem::main();

    #[cfg(feature = "test-fs-disk")]
    fs::disk::main();
}
