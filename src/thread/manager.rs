//! Manager of all kernel threads

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem;
use core::ops::DerefMut;

use crate::mem::KernelPgTable;
use crate::sbi::interrupt;
use crate::sync::Lazy;
use crate::thread::{
    schedule, switch, Builder, Mutex, Schedule, Scheduler, Status, Thread, PRI_DEFAULT, PRI_MIN,
};

/* --------------------------------- MANAGER -------------------------------- */
/// Global thread manager, contains a scheduler and a current thread.
pub struct Manager {
    /// Global thread scheduler
    pub scheduler: Mutex<Scheduler>,
    /// The current running thread
    pub current: Mutex<Arc<Thread>>,
    /// All alive and not yet destroyed threads
    all: Mutex<Vec<Arc<Thread>>>,
}

impl Manager {
    pub fn get() -> &'static Self {
        static TMANAGER: Lazy<Manager> = Lazy::new(|| {
            let initial = Arc::new(Thread::new("Initial", 0, PRI_DEFAULT, 0, None, None));
            initial.set_status(Status::Running);

            let manager = Manager {
                scheduler: Mutex::new(Scheduler::default()),
                all: Mutex::new(Vec::from([initial.clone()])),
                current: Mutex::new(initial),
            };

            let idle = Builder::new(|| loop {
                schedule()
            })
            .name("Idle")
            .priority(PRI_MIN)
            .build();
            manager.register(idle);

            manager
        });

        &TMANAGER
    }

    pub(super) fn register(&self, thread: Arc<Thread>) {
        // Register it into the scheduler
        self.scheduler.lock().register(thread.clone());

        // Store it in all list.
        self.all.lock().push(thread.clone());
    }

    /// Choose a `ready` thread to run if possible. If found, do as follows:
    ///
    /// 1. Turn off intr. Mark the `next` thread as [`Running`](Status::Running) and
    /// change manager's current thread.
    ///
    /// 2. Forward the `previous` thread to [`schedule_tail`] through [`switch`].
    /// In [`schedule_tail`], the finishing touches of the schedule is done in the
    /// new chosen thread, including releasing a dead thread's resources.
    ///
    /// 3. Get back from the other thread and restore the intr setting.
    pub fn schedule(&self) {
        let old = interrupt::set(false);

        let next = self.scheduler.lock().schedule();

        // Make sure there's at least one thread runnable.
        assert!(
            self.current.lock().status() == Status::Running || next.is_some(),
            "no thread is ready"
        );

        if let Some(next) = next {
            assert_eq!(next.status(), Status::Ready);
            next.set_status(Status::Running);

            // Update the current thread to the next running thread
            let previous = mem::replace(self.current.lock().deref_mut(), next);
            #[cfg(feature = "debug")]
            kprintln!("[THREAD] switch from {:?}", previous);

            // Retrieve the raw pointers of two threads' context
            let old_ctx = previous.context();
            let new_ctx = self.current.lock().context();

            // WARNING: This function call may not return, so don't expect any value to be dropped.

            unsafe { switch::switch(Arc::into_raw(previous).cast(), old_ctx, new_ctx) }

            // Back to this location (which `ra` points to), indicating that another thread
            // has yielded its control or simply exited. Also, it means now the running
            // thread has been shceudled for more than one time, otherwise it would return
            // to `kernel_thread_entry` (See `create` where the initial context is set).
            //
            // Then, we restore the interrupt setting, and back to where we were before the
            // scheduling, usually inside a trap handler, a method of semaphore, or anywhere
            // `schedule` was invoked.
        }

        interrupt::set(old);
    }

    /// After context switch, now do some finishing touches. We release a thread's
    /// resources if it's about to be destroyed. For a runnable thread, it should
    /// be registered into the scheduler.
    ///
    /// Note: This function is running on the stack of the new thread.
    pub fn schedule_tail(&self, previous: Arc<Thread>) {
        assert!(!interrupt::get());

        #[cfg(feature = "debug")]
        kprintln!("[THREAD] switch to {:?}", *self.current.lock());

        match previous.status() {
            Status::Dying => {
                // A thread's resources should be released at this point
                self.all.lock().retain(|t| t.id() != previous.id());
            }
            Status::Running => {
                previous.set_status(Status::Ready);
                self.scheduler.lock().register(previous);
            }
            Status::Blocked => {}
            Status::Ready => unreachable!(),
        }

        if let Some(pt) = self.current.lock().pagetable.as_ref() {
            pt.lock().activate();
        } else {
            KernelPgTable::get().activate();
        }
    }
}
