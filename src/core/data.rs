use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use crate::base::RefCounted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ObjectFlags {
    Zero = 0,
    Destroyed = 1 << 0,
    RealDestroyed = 1 << 1,
    ToDestroy = 1 << 2,
    DontSave = 1 << 3,
    EditOnly = 1 << 4,
    Dirty = 1 << 5,
    Destroying = 1 << 6,
    DontDestroy = 1 << 7,
    Deactivating = 1 << 8,
    IsPreloadStarted = 1 << 13,
    IsOnLoadCalled = 1 << 14,
    IsOnLoadStarted = 1 << 15,
    IsStartCalled = 1 << 16,
    IsRotationLocked = 1 << 17,
    IsScaleLocked = 1 << 18,
    IsAnchorLocked = 1 << 19,
    IsSizeLocked = 1 << 20,
    IsPositionLocked = 1 << 21,
    IsReplicated = 1 << 22,
    IsClientLoad = 1 << 23,
    IsSkipTransformUpdate = 1 << 24,
}

impl ObjectFlags {
    pub const ALL_HIDE_MASK: u32 = !(
        ObjectFlags::ToDestroy as u32
        | ObjectFlags::Dirty as u32
        | ObjectFlags::Destroying as u32
        | ObjectFlags::DontDestroy as u32
        | ObjectFlags::Deactivating as u32
        | ObjectFlags::IsPreloadStarted as u32
        | ObjectFlags::IsOnLoadCalled as u32
        | ObjectFlags::IsOnLoadStarted as u32
        | ObjectFlags::IsStartCalled as u32
        | ObjectFlags::IsRotationLocked as u32
        | ObjectFlags::IsScaleLocked as u32
        | ObjectFlags::IsAnchorLocked as u32
        | ObjectFlags::IsSizeLocked as u32
        | ObjectFlags::IsPositionLocked as u32
    );

    pub fn all_hide_mask() -> u32 {
        Self::ALL_HIDE_MASK
    }

    pub fn has_any_hide_flag(flags: u32) -> bool {
        (flags & Self::ALL_HIDE_MASK) != 0
    }
}

static OBJECT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_object_id() -> u64 {
    OBJECT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub trait Scriptable {}

pub trait Object: RefCounted {
    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: &str);
    fn get_id(&self) -> u64;
    fn set_hide_flags(&mut self, flags: u32);
    fn get_hide_flags(&self) -> u32;
    fn is_valid(&self) -> bool;
    fn destroy(&mut self) -> bool;
    fn to_string(&self) -> String;
}

#[derive(Debug)]
pub struct CCObject {
    pub name: String,
    id: u64,
    ref_count: AtomicU32,
    flags: u32,
}

impl CCObject {
    pub fn new(name: &str) -> Self {
        CCObject {
            name: name.to_string(),
            id: generate_object_id(),
            ref_count: AtomicU32::new(1),
            flags: 0,
        }
    }

    pub fn is_destroyed(&self) -> bool {
        (self.flags & ObjectFlags::Destroyed as u32) != 0
    }

    pub fn is_to_destroy(&self) -> bool {
        (self.flags & ObjectFlags::ToDestroy as u32) != 0
    }

    pub fn is_destroying(&self) -> bool {
        (self.flags & ObjectFlags::Destroying as u32) != 0
    }

    pub fn dont_destroy(&self) -> bool {
        (self.flags & ObjectFlags::DontDestroy as u32) != 0
    }

    pub fn set_flag(&mut self, flag: ObjectFlags, value: bool) {
        if value {
            self.flags |= flag as u32;
        } else {
            self.flags &= !(flag as u32);
        }
    }

    pub fn has_flag(&self, flag: ObjectFlags) -> bool {
        (self.flags & flag as u32) != 0
    }
}

impl Object for CCObject {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_id(&self) -> u64 {
        self.id
    }

    fn set_hide_flags(&mut self, flags: u32) {
        self.flags = (self.flags & !ObjectFlags::ALL_HIDE_MASK) | (flags & ObjectFlags::ALL_HIDE_MASK);
    }

    fn get_hide_flags(&self) -> u32 {
        self.flags & ObjectFlags::ALL_HIDE_MASK
    }

    fn is_valid(&self) -> bool {
        !self.is_destroyed()
    }

    fn destroy(&mut self) -> bool {
        if self.is_to_destroy() || self.is_destroyed() {
            return false;
        }
        self.set_flag(ObjectFlags::ToDestroy, true);
        DEFERRED_DESTROY_QUEUE.with(|q| {
            q.borrow_mut().push(self.id);
        });
        true
    }

    fn to_string(&self) -> String {
        format!("[CCObject] {} (id={})", self.name, self.id)
    }
}

impl RefCounted for CCObject {
    fn add_ref(&self) {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    fn release(&self) {
        let prev = self.ref_count.fetch_sub(1, Ordering::Release);
        if prev == 1 {
            std::sync::atomic::fence(Ordering::Acquire);
        }
    }

    fn get_ref_count(&self) -> u32 {
        self.ref_count.load(Ordering::Relaxed)
    }

    fn is_last_reference(&self) -> bool {
        self.ref_count.load(Ordering::Relaxed) == 1
    }
}

impl Default for CCObject {
    fn default() -> Self {
        Self::new("Object")
    }
}

std::thread_local! {
    static DEFERRED_DESTROY_QUEUE: std::cell::RefCell<Vec<u64>> = std::cell::RefCell::new(Vec::new());
}

pub fn deferred_destroy_tick() {
    DEFERRED_DESTROY_QUEUE.with(|q| {
        q.borrow_mut().clear();
    });
}

pub fn deferred_destroy() {
    deferred_destroy_tick();
}

pub fn is_object_valid<T: Object>(obj: &T, strict_mode: bool) -> bool {
    if !obj.is_valid() {
        return false;
    }
    if strict_mode && (obj.get_hide_flags() & ObjectFlags::Destroyed as u32) != 0 {
        return false;
    }
    true
}

pub struct ObjectPool<T> {
    objects: Vec<Option<Arc<Mutex<T>>>>,
    free_indices: Vec<usize>,
}

impl<T> ObjectPool<T> {
    pub fn new() -> Self {
        ObjectPool {
            objects: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        ObjectPool {
            objects: Vec::with_capacity(capacity),
            free_indices: Vec::new(),
        }
    }

    pub fn alloc(&mut self, obj: T) -> usize {
        if let Some(idx) = self.free_indices.pop() {
            self.objects[idx] = Some(Arc::new(Mutex::new(obj)));
            idx
        } else {
            let idx = self.objects.len();
            self.objects.push(Some(Arc::new(Mutex::new(obj))));
            idx
        }
    }

    pub fn free(&mut self, idx: usize) {
        if idx < self.objects.len() {
            self.objects[idx] = None;
            self.free_indices.push(idx);
        }
    }

    pub fn get(&self, idx: usize) -> Option<Arc<Mutex<T>>> {
        self.objects.get(idx)?.as_ref().cloned()
    }

    pub fn len(&self) -> usize {
        self.objects.len() - self.free_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, Arc<Mutex<T>>)> + '_ {
        self.objects.iter().enumerate().filter_map(|(i, o)| {
            o.as_ref().map(|arc| (i, Arc::clone(arc)))
        })
    }
}

impl<T> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub type ObjectHandle = usize;

pub const INVALID_HANDLE: ObjectHandle = usize::MAX;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccobject_new() {
        let obj = CCObject::new("TestObject");
        assert_eq!(obj.get_name(), "TestObject");
        assert!(obj.is_valid());
        assert!(!obj.is_destroyed());
        assert!(!obj.is_to_destroy());
        assert_eq!(obj.get_ref_count(), 1);
    }

    #[test]
    fn test_ccobject_id_unique() {
        let o1 = CCObject::new("A");
        let o2 = CCObject::new("B");
        assert_ne!(o1.get_id(), o2.get_id());
    }

    #[test]
    fn test_ccobject_flags() {
        let mut obj = CCObject::new("Obj");
        assert!(!obj.has_flag(ObjectFlags::Dirty));
        obj.set_flag(ObjectFlags::Dirty, true);
        assert!(obj.has_flag(ObjectFlags::Dirty));
        obj.set_flag(ObjectFlags::Dirty, false);
        assert!(!obj.has_flag(ObjectFlags::Dirty));
    }

    #[test]
    fn test_ccobject_destroy() {
        let mut obj = CCObject::new("ToDestroy");
        assert!(obj.is_valid());
        let result = obj.destroy();
        assert!(result);
        assert!(obj.is_to_destroy());

        let result2 = obj.destroy();
        assert!(!result2);
    }

    #[test]
    fn test_ccobject_ref_count() {
        let obj = CCObject::new("Ref");
        assert_eq!(obj.get_ref_count(), 1);
        obj.add_ref();
        assert_eq!(obj.get_ref_count(), 2);
        obj.release();
        assert_eq!(obj.get_ref_count(), 1);
    }

    #[test]
    fn test_ccobject_name() {
        let mut obj = CCObject::new("OldName");
        assert_eq!(obj.get_name(), "OldName");
        obj.set_name("NewName");
        assert_eq!(obj.get_name(), "NewName");
    }

    #[test]
    fn test_ccobject_hide_flags() {
        let mut obj = CCObject::new("Obj");
        obj.set_hide_flags(ObjectFlags::EditOnly as u32);
        assert!(obj.get_hide_flags() & ObjectFlags::EditOnly as u32 != 0);
    }

    #[test]
    fn test_is_object_valid() {
        let obj = CCObject::new("Valid");
        assert!(is_object_valid(&obj, false));
        assert!(is_object_valid(&obj, true));
    }

    #[test]
    fn test_object_pool_alloc_free() {
        let mut pool: ObjectPool<i32> = ObjectPool::new();
        let h1 = pool.alloc(10);
        let _h2 = pool.alloc(20);
        assert_eq!(pool.len(), 2);

        let v1 = pool.get(h1).unwrap();
        assert_eq!(*v1.lock().unwrap(), 10);

        pool.free(h1);
        assert_eq!(pool.len(), 1);

        let h3 = pool.alloc(30);
        assert_eq!(h3, h1);
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_object_pool_iter() {
        let mut pool: ObjectPool<String> = ObjectPool::new();
        pool.alloc("a".to_string());
        pool.alloc("b".to_string());
        pool.alloc("c".to_string());
        let count = pool.iter().count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_object_pool_empty() {
        let pool: ObjectPool<i32> = ObjectPool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_deferred_destroy_tick() {
        let mut obj = CCObject::new("DeferredObj");
        obj.destroy();
        deferred_destroy_tick();
    }

    #[test]
    fn test_object_flags_all_hide_mask() {
        let mask = ObjectFlags::all_hide_mask();
        assert_ne!(mask, 0);
    }
}
