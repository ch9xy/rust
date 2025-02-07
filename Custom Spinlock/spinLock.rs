use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

pub struct SpinLock<T> {
        locked: AtomicBool,
        value: UnsafeCell<T>,
    }

unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<T> {
        while self.locked.swap(true, Acquire) {
            std::hint::spin_loop();
        }
        Guard {lock: self }
    }

}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.value.get() }
    } // deallocation of Guard from memory also happens here
}

impl<T> DerefMut for Guard<'_, T> {
    // type Target from Deref trait is inherited here because DerefMut is DerefMut: Deref
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

// We need Sync implementation since our Deref implementation returns a value directly to the T.
// We do not implement Send because there is already an implementation for T at SpinLock.
// If T is Send so the SpinLock is so the Guard is.
unsafe impl<T> Sync for Guard<'_, T> where T: Sync {}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release);
    }
}

