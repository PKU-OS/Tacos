use core::fmt::{Result, Write};

use crate::sbi::{console_putchar, interrupt};

pub struct Stdout;

/// A locked standard output
///
/// `StdoutLock` simply shuts down interrupt when acquired, and restores
/// the previous interrupt setting when dropped.
///
/// ## Examples
/// ```
/// use crate::sbi::console::stdout;
/// let mut output: StdoutLock = stdout().lock();
/// // The two lines below won't be interrupted by any other threads.
/// writeln!(output, "hi, there");
/// writeln!(output, "hi, again");
/// ```
pub struct StdoutLock<'a> {
    /// A reference to the one and only standard output instance
    inner: &'a mut Stdout,
    /// interrupt status before stdout being locked
    intr: bool,
}

/// The one and only Stdout instance
pub fn stdout() -> &'static mut Stdout {
    static mut INSTANCE: Stdout = Stdout;
    unsafe { &mut INSTANCE }
}

impl Stdout {
    /// Lock the Stdout to print message exclusively
    ///
    /// This is a re-entrant lock, allowing called in a nested manner.
    pub fn lock(&self) -> StdoutLock {
        StdoutLock {
            inner: stdout(),
            intr: interrupt::set(false),
        }
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> Result {
        for ch in s.chars() {
            console_putchar(ch as usize);
        }
        Ok(())
    }
}

impl Write for StdoutLock<'_> {
    fn write_str(&mut self, s: &str) -> Result {
        self.inner.write_str(s)
    }
}

impl Drop for StdoutLock<'_> {
    fn drop(&mut self) {
        interrupt::set(self.intr);
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!(crate::sbi::console::stdout().lock(), $($arg)*);
    }}
}

#[macro_export]
macro_rules! kprintln {
    () => {
        kprint!("\n");
    };
    ($($arg:tt)*) => {{
        let _x = crate::sbi::console::stdout().lock();
        kprint!(
            "[\x1B[38;2;129;165;113;1m{} ms\x1B[0m] ",
            crate::sbi::timer::time_ms()
        );
        kprint!($($arg)*);
        kprint!("\n");
    }};
}
