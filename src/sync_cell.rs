//! Interior mutability that is Sync when `threads` is enabled (Mutex) and not Sync on wasm32 (RefCell).
//! Lets the crate work with gdext's single-threaded path (no Sync) and multithreaded path (Sync).

#[cfg(feature = "threads")]
use std::sync::Mutex;

#[cfg(not(feature = "threads"))]
use std::cell::RefCell;

/// Cell type: wraps `Mutex<T>` when feature `threads` (Sync), else `RefCell<T>` (single-threaded).
#[cfg(feature = "threads")]
#[repr(transparent)]
pub struct SyncCell<T>(Mutex<T>);

#[cfg(not(feature = "threads"))]
#[repr(transparent)]
pub struct SyncCell<T>(RefCell<T>);

impl<T> SyncCell<T> {
    pub fn new(value: T) -> Self {
        #[cfg(feature = "threads")]
        return Self(Mutex::new(value));
        #[cfg(not(feature = "threads"))]
        return Self(RefCell::new(value));
    }
}

/// Run a closure with exclusive access to the inner value.
#[cfg(feature = "threads")]
pub fn with_mut<T, R, F: FnOnce(&mut T) -> R>(cell: &SyncCell<T>, f: F) -> R {
    f(&mut *cell.0.lock().unwrap())
}

#[cfg(not(feature = "threads"))]
pub fn with_mut<T, R, F: FnOnce(&mut T) -> R>(cell: &SyncCell<T>, f: F) -> R {
    f(&mut *cell.0.borrow_mut())
}

impl<T: Default> Default for SyncCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
