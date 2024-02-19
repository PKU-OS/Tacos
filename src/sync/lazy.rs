use core::{cell::Cell, ops::Deref};

use crate::sync::OnceCell;

/// A value initialized on the first access. It is thread-safe, and can be used in statics.
///
/// ## Examples
/// ```rust
/// // Suppose it's in the std environment
/// static HASHMAP: Lazy<HashMap<i32, String>> = Lazy::new(|| {
///     println!("initializing");
///     let mut m = HashMap::new();
///     m.insert(13, "Spica".to_string());
///     m.insert(74, "Hoyten".to_string());
///     m
/// });
///
/// fn main() {
///     println!("ready");
///     std::thread::spawn(|| {
///         println!("{:?}", HASHMAP.get(&13));
///     }).join().unwrap();
///     println!("{:?}", HASHMAP.get(&74));
///
///     // Prints:
///     //   ready
///     //   initializing
///     //   Some("Spica")
///     //   Some("Hoyten")
/// }
/// ```
pub struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: Cell<Option<F>>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    pub const fn new(f: F) -> Self {
        Self {
            cell: OnceCell::new(),
            init: Cell::new(Some(f)),
        }
    }

    pub fn get(&self) -> &T {
        self.cell.get_or_init(|| {
            let f = self.init.take().unwrap();
            f()
        })
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.get()
    }
}

unsafe impl<T: Sync + Send, F: Send> Sync for Lazy<T, F> {}
