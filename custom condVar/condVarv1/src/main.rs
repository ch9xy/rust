use mutexv1::*;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering::*};
use std::time::{Instant, Duration};
// atomic-wait
use atomic_wait::wait;
use atomic_wait::wake_one;
use std::thread;

pub struct Condvar {
    counter: AtomicU32,
    num_waiters: AtomicUsize,
}

impl Condvar {
    pub const fn new() -> Self {
        Self { counter: AtomicU32::new(0),
               num_waiters: AtomicUsize::new(0),
        }
    }

    pub fn notify_one(&self) {
        if self.num_waiters.load(Relaxed) > 0 {
            self.counter.fetch_add(1, Relaxed);
            wake_one(&self.counter);
        }
        
    }

    /*
    pub fn notify_all(&self) {
        if self.num_waiters.load(Relaxed) > 0 {
            self.counter.fetch_add(1, Relaxed);
            wake_all(&self.counter);
        }
    }
    */
    
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.num_waiters.fetch_add(1, Relaxed);

        let counter_value = self.counter.load(Relaxed);

        // Unlock the mutex by dropping the guard,
        // but remember the mutex so we can lock it again later.
        let mutex = guard.mutex;
        drop(guard);

        // Wait, but only if the counter hasn't changed since unlocking.
        wait(&self.counter, counter_value);

        self.num_waiters.fetch_sub(1, Relaxed);

        mutex.lock()
    }
}

fn main() {

    test_condvar();

}

fn test_condvar() {
    let mutex = Mutex::new(0);
    let condvar = Condvar::new();
    let mut wakeups = 0;
    println!("hey");
    thread::scope(|s| {
        s.spawn(|| {
            thread::sleep(Duration::from_secs(1));
            *mutex.lock() = 123;
            condvar.notify_one();
        });
        let mut m = mutex.lock();
        while *m < 100 {
            m = condvar.wait(m);

            wakeups += 1;
        }
        assert_eq!(*m, 123);
    });
    assert!(wakeups < 10);
    println!("{}", wakeups);
}