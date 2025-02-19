use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::time::Instant;
// atomic-wait
use atomic_wait::wait;
use atomic_wait::wake_one;
use std::thread;

pub struct Mutex<T> {
    /// 0: unlocked
    /// 1: locked, no other threads are waiting
    /// 2: locked, other threads are waiting
    state: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

pub struct MutexGuard<'a, T> {
    pub mutex: &'a Mutex<T>,
}

unsafe impl<T:Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut (&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0), // Unlocked state
            value: UnsafeCell::new(value),
        }
    }
 
    pub fn lock(&self) -> MutexGuard<T> {
        if self.state.compare_exchange(0, 1, Acquire, Relaxed).is_err() {
            // the lock was already locked.
            lock_contended(&self.state);
            
        }
        MutexGuard { mutex: self }
    }
}


#[cold]
pub fn lock_contended(state: &AtomicU32) {
    let mut spin_count = 0;

    while state.load(Relaxed) == 1 && spin_count < 100 {
        spin_count += 1;
        std::hint::spin_loop();
    }
    if state.compare_exchange(0, 1, Acquire, Relaxed).is_ok() {
        return;
    }
    while state.swap(2, Acquire) != 0 {
        wait(state, 2);
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.mutex.state.swap(0, Release) == 2 {
            wake_one(&self.mutex.state);
        }
    }
}