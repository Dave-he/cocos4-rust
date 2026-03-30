/****************************************************************************
Rust port of Cocos Creator Base Threading
Original C++ version Copyright (c) 2020-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;

// ---------------------------------------------------------------------------
// ReadWriteLock
// ---------------------------------------------------------------------------

pub struct ReadWriteLock<T> {
    inner: RwLock<T>,
}

impl<T> ReadWriteLock<T> {
    pub fn new(value: T) -> Self {
        ReadWriteLock { inner: RwLock::new(value) }
    }

    pub fn lock_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.read().unwrap();
        f(&*guard)
    }

    pub fn lock_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write().unwrap();
        f(&mut *guard)
    }
}

// ---------------------------------------------------------------------------
// Message (single-producer single-consumer message queue)
// ---------------------------------------------------------------------------

pub type MessageFn = Box<dyn FnOnce() + Send>;

pub struct MessageQueue {
    queue: Arc<(Mutex<VecDeque<MessageFn>>, Condvar)>,
    pending: u32,
    written: u32,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            queue: Arc::new((Mutex::new(VecDeque::new()), Condvar::new())),
            pending: 0,
            written: 0,
        }
    }

    pub fn enqueue<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (lock, cvar) = &*self.queue;
        let mut q = lock.lock().unwrap();
        q.push_back(Box::new(f));
        self.written += 1;
        cvar.notify_one();
    }

    pub fn flush_messages(&mut self) {
        let (lock, _) = &*self.queue;
        let mut q = lock.lock().unwrap();
        let msgs: Vec<MessageFn> = q.drain(..).collect();
        drop(q);
        for msg in msgs {
            msg();
        }
        self.pending = 0;
    }

    pub fn get_pending_count(&self) -> u32 {
        let (lock, _) = &*self.queue;
        lock.lock().unwrap().len() as u32
    }

    pub fn get_written_message_count(&self) -> u32 {
        self.written
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ThreadPool
// ---------------------------------------------------------------------------

pub struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    queue: Arc<(Mutex<(VecDeque<Box<dyn FnOnce() + Send>>, bool)>, Condvar)>,
    worker_count: u32,
}

impl ThreadPool {
    pub fn new(worker_count: u32) -> Self {
        let queue: Arc<(Mutex<(VecDeque<Box<dyn FnOnce() + Send>>, bool)>, Condvar)> =
            Arc::new((Mutex::new((VecDeque::new(), false)), Condvar::new()));

        let mut workers = Vec::with_capacity(worker_count as usize);
        for _ in 0..worker_count {
            let q = Arc::clone(&queue);
            let handle = thread::spawn(move || loop {
                let task = {
                    let (lock, cvar) = &*q;
                    let mut state = lock.lock().unwrap();
                    loop {
                        if !state.0.is_empty() {
                            break state.0.pop_front().unwrap();
                        }
                        if state.1 {
                            return;
                        }
                        state = cvar.wait(state).unwrap();
                    }
                };
                task();
            });
            workers.push(handle);
        }

        ThreadPool { workers, queue, worker_count }
    }

    pub fn dispatch<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (lock, cvar) = &*self.queue;
        let mut state = lock.lock().unwrap();
        state.0.push_back(Box::new(f));
        cvar.notify_one();
    }

    pub fn get_worker_count(&self) -> u32 {
        self.worker_count
    }

    pub fn stop(self) {
        {
            let (lock, cvar) = &*self.queue;
            let mut state = lock.lock().unwrap();
            state.1 = true;
            cvar.notify_all();
        }
        for w in self.workers {
            let _ = w.join();
        }
    }
}

// ---------------------------------------------------------------------------
// ThreadSafeCounter
// ---------------------------------------------------------------------------

use std::sync::atomic::{AtomicI32, Ordering};

pub struct ThreadSafeCounter {
    value: AtomicI32,
}

impl ThreadSafeCounter {
    pub fn new(initial: i32) -> Self {
        ThreadSafeCounter { value: AtomicI32::new(initial) }
    }

    pub fn increment(&self) -> i32 {
        self.value.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn decrement(&self) -> i32 {
        self.value.fetch_sub(1, Ordering::SeqCst) - 1
    }

    pub fn get(&self) -> i32 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.value.store(0, Ordering::SeqCst);
    }
}

impl Default for ThreadSafeCounter {
    fn default() -> Self {
        Self::new(0)
    }
}

// ---------------------------------------------------------------------------
// AutoReleasePool (deferred cleanup on next frame)
// ---------------------------------------------------------------------------

pub struct AutoReleasePool {
    objects: Mutex<Vec<Box<dyn FnOnce() + Send>>>,
}

impl AutoReleasePool {
    pub fn new() -> Self {
        AutoReleasePool { objects: Mutex::new(Vec::new()) }
    }

    pub fn add<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.objects.lock().unwrap().push(Box::new(f));
    }

    pub fn drain(&self) {
        let items: Vec<_> = self.objects.lock().unwrap().drain(..).collect();
        for item in items {
            item();
        }
    }

    pub fn count(&self) -> usize {
        self.objects.lock().unwrap().len()
    }
}

impl Default for AutoReleasePool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_read_write_lock_read() {
        let lock = ReadWriteLock::new(42u32);
        let v = lock.lock_read(|x| *x);
        assert_eq!(v, 42);
    }

    #[test]
    fn test_read_write_lock_write() {
        let lock = ReadWriteLock::new(0u32);
        lock.lock_write(|x| *x = 99);
        let v = lock.lock_read(|x| *x);
        assert_eq!(v, 99);
    }

    #[test]
    fn test_message_queue_enqueue_flush() {
        let counter = Arc::new(AtomicU32::new(0));
        let mut mq = MessageQueue::new();
        let c1 = Arc::clone(&counter);
        mq.enqueue(move || { c1.fetch_add(1, Ordering::SeqCst); });
        let c2 = Arc::clone(&counter);
        mq.enqueue(move || { c2.fetch_add(1, Ordering::SeqCst); });
        assert_eq!(mq.get_pending_count(), 2);
        mq.flush_messages();
        assert_eq!(counter.load(Ordering::SeqCst), 2);
        assert_eq!(mq.get_pending_count(), 0);
    }

    #[test]
    fn test_message_queue_written_count() {
        let mut mq = MessageQueue::new();
        mq.enqueue(|| {});
        mq.enqueue(|| {});
        assert_eq!(mq.get_written_message_count(), 2);
    }

    #[test]
    fn test_thread_safe_counter() {
        let counter = ThreadSafeCounter::default();
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.decrement(), 1);
        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_thread_pool_dispatch() {
        let counter = Arc::new(AtomicU32::new(0));
        let pool = ThreadPool::new(2);
        let c = Arc::clone(&counter);
        pool.dispatch(move || { c.fetch_add(1, Ordering::SeqCst); });
        pool.stop();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_auto_release_pool() {
        let pool = AutoReleasePool::new();
        let counter = Arc::new(AtomicU32::new(0));
        let c = Arc::clone(&counter);
        pool.add(move || { c.fetch_add(1, Ordering::SeqCst); });
        assert_eq!(pool.count(), 1);
        pool.drain();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert_eq!(pool.count(), 0);
    }
}
