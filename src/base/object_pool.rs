use std::collections::VecDeque;

pub trait Poolable: Default + Send + Sync {
    fn reset(&mut self);
}

pub struct ObjectPool<T: Poolable> {
    pool: VecDeque<T>,
    capacity: usize,
    total_created: u64,
    total_recycled: u64,
    total_acquired: u64,
}

impl<T: Poolable> ObjectPool<T> {
    pub fn new(capacity: usize) -> Self {
        ObjectPool {
            pool: VecDeque::new(),
            capacity,
            total_created: 0,
            total_recycled: 0,
            total_acquired: 0,
        }
    }

    pub fn with_prewarm(capacity: usize, count: usize) -> Self {
        let mut pool = Self::new(capacity);
        pool.prewarm(count);
        pool
    }

    pub fn prewarm(&mut self, count: usize) {
        let to_create = count.min(self.capacity).saturating_sub(self.pool.len());
        for _ in 0..to_create {
            let obj = T::default();
            self.pool.push_back(obj);
            self.total_created += 1;
        }
    }

    pub fn acquire(&mut self) -> T {
        self.total_acquired += 1;
        if let Some(mut obj) = self.pool.pop_front() {
            obj.reset();
            obj
        } else {
            self.total_created += 1;
            T::default()
        }
    }

    pub fn release(&mut self, mut obj: T) {
        if self.pool.len() < self.capacity {
            obj.reset();
            self.pool.push_back(obj);
            self.total_recycled += 1;
        }
    }

    pub fn available(&self) -> usize {
        self.pool.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity;
        while self.pool.len() > self.capacity {
            self.pool.pop_back();
        }
    }

    pub fn clear(&mut self) {
        self.pool.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    pub fn total_created(&self) -> u64 {
        self.total_created
    }

    pub fn total_recycled(&self) -> u64 {
        self.total_recycled
    }

    pub fn total_acquired(&self) -> u64 {
        self.total_acquired
    }
}

impl<T: Poolable> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Bullet {
        pub x: f32,
        pub y: f32,
        pub active: bool,
        pub damage: f32,
    }

    impl Poolable for Bullet {
        fn reset(&mut self) {
            self.x = 0.0;
            self.y = 0.0;
            self.active = false;
            self.damage = 0.0;
        }
    }

    #[test]
    fn test_pool_new() {
        let pool: ObjectPool<Bullet> = ObjectPool::new(10);
        assert_eq!(pool.available(), 0);
        assert_eq!(pool.capacity(), 10);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_prewarm() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::new(10);
        pool.prewarm(5);
        assert_eq!(pool.available(), 5);
        assert_eq!(pool.total_created(), 5);
    }

    #[test]
    fn test_pool_with_prewarm() {
        let pool: ObjectPool<Bullet> = ObjectPool::with_prewarm(10, 4);
        assert_eq!(pool.available(), 4);
    }

    #[test]
    fn test_pool_acquire_from_pool() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::with_prewarm(10, 3);
        let _b = pool.acquire();
        assert_eq!(pool.available(), 2);
        assert_eq!(pool.total_acquired(), 1);
    }

    #[test]
    fn test_pool_acquire_creates_new_when_empty() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::new(10);
        let _b = pool.acquire();
        assert_eq!(pool.total_created(), 1);
        assert_eq!(pool.total_acquired(), 1);
    }

    #[test]
    fn test_pool_release_returns_to_pool() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::new(10);
        let mut b = pool.acquire();
        b.x = 100.0;
        b.active = true;
        pool.release(b);
        assert_eq!(pool.available(), 1);
        assert_eq!(pool.total_recycled(), 1);
    }

    #[test]
    fn test_pool_release_resets_object() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::new(10);
        let mut b = pool.acquire();
        b.x = 999.0;
        b.damage = 50.0;
        b.active = true;
        pool.release(b);
        let b2 = pool.acquire();
        assert_eq!(b2.x, 0.0);
        assert_eq!(b2.damage, 0.0);
        assert!(!b2.active);
    }

    #[test]
    fn test_pool_release_over_capacity_discards() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::new(2);
        let b1 = pool.acquire();
        let b2 = pool.acquire();
        let b3 = pool.acquire();
        pool.release(b1);
        pool.release(b2);
        pool.release(b3);
        assert_eq!(pool.available(), 2);
        assert_eq!(pool.total_recycled(), 2);
    }

    #[test]
    fn test_pool_set_capacity_shrinks() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::with_prewarm(10, 5);
        pool.set_capacity(3);
        assert_eq!(pool.capacity(), 3);
        assert_eq!(pool.available(), 3);
    }

    #[test]
    fn test_pool_clear() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::with_prewarm(10, 5);
        pool.clear();
        assert_eq!(pool.available(), 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_reuse_cycle() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::new(5);
        for i in 0..20 {
            let mut b = pool.acquire();
            b.x = i as f32;
            b.active = true;
            pool.release(b);
        }
        assert!(pool.total_created() <= 5 + 1);
        assert_eq!(pool.total_acquired(), 20);
    }

    #[test]
    fn test_pool_prewarm_respects_capacity() {
        let mut pool: ObjectPool<Bullet> = ObjectPool::new(3);
        pool.prewarm(100);
        assert_eq!(pool.available(), 3);
    }
}
