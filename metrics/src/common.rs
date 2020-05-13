use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::Once;

/// An allocation-optimized string.
///
/// We specify `ScopedString` to attempt to get the best of both worlds: flexibility to provide a
/// static or dynamic (owned) string, while retaining the performance benefits of being able to
/// take ownership of owned strings and borrows of completely static strings.
pub type ScopedString = Cow<'static, str>;

/// Atomically-guarded cell.
#[doc(hidden)]
pub struct OnceCell<T> {
    init: Once,
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceCell<T> {
    /// Creates a new `OnceCell` in the uninitialized state.
    pub const fn new() -> OnceCell<T> {
        OnceCell {
            init: Once::new(),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Gets or initializes the value.
    ///
    /// If the value has not yet been initialized, `f` is run to acquire it, and
    /// stores the value for other callers to utilize.
    ///
    /// All callers rondezvous on an internal atomic guard, so it impossible to see
    /// invalid state.
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: Fn() -> T,
    {
        self.init.call_once(|| {
            let inner = f();
            unsafe {
                (*self.inner.get()) = MaybeUninit::new(inner);
            }
        });

        unsafe { &*(&*self.inner.get()).as_ptr() }
    }
}

unsafe impl<T> Sync for OnceCell<T> where T: Sync {}
