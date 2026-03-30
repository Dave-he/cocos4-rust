/****************************************************************************
Rust port of Cocos Creator Core memop (Pool, RecyclePool, CachedArray)
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

// ---------------------------------------------------------------------------
// Pool<T> - object pool with free-list
// ---------------------------------------------------------------------------

pub struct Pool<T> {
    free_pool: Vec<T>,
    elements_per_batch: usize,
}

impl<T> Pool<T> {
    pub fn new<F>(ctor: F, elements_per_batch: usize) -> Self
    where
        F: Fn() -> T,
    {
        let batch = elements_per_batch.max(1);
        let free_pool = (0..batch).map(|_| ctor()).collect();
        Pool { free_pool, elements_per_batch: batch }
    }

    pub fn alloc(&mut self, ctor: impl Fn() -> T) -> T {
        if self.free_pool.is_empty() {
            self.free_pool = (0..self.elements_per_batch).map(|_| ctor()).collect();
        }
        self.free_pool.pop().unwrap()
    }

    pub fn free(&mut self, obj: T) {
        self.free_pool.push(obj);
    }

    pub fn free_array(&mut self, objs: impl IntoIterator<Item = T>) {
        self.free_pool.extend(objs);
    }

    pub fn destroy(&mut self) {
        self.free_pool.clear();
    }

    pub fn available_count(&self) -> usize {
        self.free_pool.len()
    }
}

// ---------------------------------------------------------------------------
// RecyclePool<T> - reuse-entire pool
// ---------------------------------------------------------------------------

pub struct RecyclePool<T> {
    data: Vec<T>,
    count: usize,
}

impl<T> RecyclePool<T> {
    pub fn new<F>(ctor: F, size: usize) -> Self
    where
        F: Fn() -> T,
    {
        let data = (0..size).map(|_| ctor()).collect();
        RecyclePool { data, count: 0 }
    }

    pub fn get_length(&self) -> usize {
        self.count
    }

    pub fn get_data(&self) -> &[T] {
        &self.data[..self.count]
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }

    pub fn add<F>(&mut self, ctor: F)
    where
        F: Fn() -> T,
    {
        if self.count >= self.data.len() {
            let new_size = (self.data.len() * 2).max(1);
            while self.data.len() < new_size {
                self.data.push(ctor());
            }
        }
        self.count += 1;
    }

    pub fn resize<F>(&mut self, size: usize, ctor: F)
    where
        F: Fn() -> T,
    {
        while self.data.len() < size {
            self.data.push(ctor());
        }
        self.count = self.count.min(size);
    }

    pub fn remove_at(&mut self, idx: usize) {
        if idx < self.count {
            self.data.swap(idx, self.count - 1);
            self.count -= 1;
        }
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.count { Some(&self.data[idx]) } else { None }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.count { Some(&mut self.data[idx]) } else { None }
    }
}

// ---------------------------------------------------------------------------
// CachedArray<T>
// ---------------------------------------------------------------------------

pub struct CachedArray<T: Clone + Default> {
    data: Vec<T>,
    size: usize,
}

impl<T: Clone + Default> CachedArray<T> {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        CachedArray { data: vec![T::default(); cap], size: 0 }
    }

    pub fn push(&mut self, item: T) {
        if self.size >= self.data.len() {
            let new_cap = self.data.len() * 2;
            self.data.resize(new_cap, T::default());
        }
        self.data[self.size] = item;
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size > 0 {
            self.size -= 1;
            Some(self.data[self.size].clone())
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn reserve(&mut self, cap: usize) {
        if cap > self.data.len() {
            self.data.resize(cap, T::default());
        }
    }

    pub fn concat(&mut self, other: &CachedArray<T>) {
        for i in 0..other.size {
            self.push(other.data[i].clone());
        }
    }

    pub fn fast_remove(&mut self, idx: usize) {
        if idx < self.size {
            self.size -= 1;
            self.data[idx] = self.data[self.size].clone();
        }
    }

    pub fn index_of(&self, item: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        (0..self.size).find(|&i| &self.data[i] == item)
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx < self.size { Some(&self.data[idx]) } else { None }
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        if idx < self.size { Some(&mut self.data[idx]) } else { None }
    }
}

impl<T: Clone + Default> std::ops::Index<usize> for CachedArray<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &T {
        &self.data[idx]
    }
}

impl<T: Clone + Default> std::ops::IndexMut<usize> for CachedArray<T> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.data[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_alloc_free() {
        let mut pool = Pool::new(|| 0u32, 4);
        assert_eq!(pool.available_count(), 4);
        let v = pool.alloc(|| 0u32);
        assert_eq!(pool.available_count(), 3);
        pool.free(v);
        assert_eq!(pool.available_count(), 4);
    }

    #[test]
    fn test_pool_auto_expand() {
        let mut pool = Pool::new(|| 0u32, 2);
        pool.alloc(|| 0u32);
        pool.alloc(|| 0u32);
        assert_eq!(pool.available_count(), 0);
        let v = pool.alloc(|| 0u32);
        assert_eq!(v, 0u32);
        assert_eq!(pool.available_count(), 1);
    }

    #[test]
    fn test_pool_free_array() {
        let mut pool = Pool::new(|| 0u32, 2);
        pool.free_array([1u32, 2, 3]);
        assert_eq!(pool.available_count(), 5);
    }

    #[test]
    fn test_recycle_pool_add_reset() {
        let mut pool = RecyclePool::new(|| 0u32, 4);
        assert_eq!(pool.get_length(), 0);
        pool.add(|| 0u32);
        pool.add(|| 0u32);
        assert_eq!(pool.get_length(), 2);
        pool.reset();
        assert_eq!(pool.get_length(), 0);
    }

    #[test]
    fn test_recycle_pool_remove_at() {
        let mut pool = RecyclePool::new(|| 0u32, 4);
        pool.add(|| 1u32);
        pool.add(|| 2u32);
        pool.add(|| 3u32);
        pool.remove_at(1);
        assert_eq!(pool.get_length(), 2);
    }

    #[test]
    fn test_cached_array_push_pop() {
        let mut arr: CachedArray<u32> = CachedArray::new(4);
        arr.push(1);
        arr.push(2);
        arr.push(3);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.pop(), Some(3));
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_cached_array_fast_remove() {
        let mut arr: CachedArray<u32> = CachedArray::new(4);
        arr.push(10);
        arr.push(20);
        arr.push(30);
        arr.fast_remove(0);
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], 30);
    }

    #[test]
    fn test_cached_array_index_of() {
        let mut arr: CachedArray<u32> = CachedArray::new(4);
        arr.push(10);
        arr.push(20);
        arr.push(30);
        assert_eq!(arr.index_of(&20), Some(1));
        assert_eq!(arr.index_of(&99), None);
    }

    #[test]
    fn test_cached_array_concat() {
        let mut a: CachedArray<u32> = CachedArray::new(2);
        let mut b: CachedArray<u32> = CachedArray::new(2);
        a.push(1);
        a.push(2);
        b.push(3);
        b.push(4);
        a.concat(&b);
        assert_eq!(a.len(), 4);
        assert_eq!(a[3], 4);
    }

    #[test]
    fn test_cached_array_clear() {
        let mut arr: CachedArray<u32> = CachedArray::new(4);
        arr.push(1);
        arr.push(2);
        arr.clear();
        assert!(arr.is_empty());
    }
}
