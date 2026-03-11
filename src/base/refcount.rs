/****************************************************************************
 Rust port of Cocos Creator RefCounted system
 Original C++ version Copyright (c) 2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::sync::atomic::{AtomicU32, Ordering};

/// Clonable trait interface
/// Allows objects to be cloned in a type-erased manner
pub trait Clonable: std::fmt::Debug {
    /// Returns a boxed copy of the object
    fn clone_box(&self) -> Box<dyn Clonable>;
}

/// RefCounted trait provides manual reference counting for objects
///
/// Similar to C++'s RefCounted, this allows objects to be shared
/// and have their lifetime managed through addRef/release calls.
///
/// Unlike Rust's Arc, this is a manual reference counting system
/// that more closely matches the C++ API semantics.
pub trait RefCounted: std::fmt::Debug {
    /// Increments the reference count
    fn add_ref(&self);

    /// Decrements the reference count
    fn release(&self);

    /// Returns the current reference count
    fn get_ref_count(&self) -> u32;

    /// Check if this is the last reference
    fn is_last_reference(&self) -> bool;
}

/// A default implementation of RefCounted using atomic reference counting
pub struct RefCountedImpl {
    reference_count: AtomicU32,
}

impl RefCountedImpl {
    pub fn new() -> Self {
        RefCountedImpl {
            reference_count: AtomicU32::new(1),
        }
    }
}

impl Default for RefCountedImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl RefCounted for RefCountedImpl {
    fn add_ref(&self) {
        self.reference_count.fetch_add(1, Ordering::SeqCst);
    }

    fn release(&self) {
        let old_count = self.reference_count.fetch_sub(1, Ordering::SeqCst);
        assert!(
            old_count > 0,
            "RefCounted::release called with zero reference count"
        );
    }

    fn get_ref_count(&self) -> u32 {
        self.reference_count.load(Ordering::SeqCst)
    }

    fn is_last_reference(&self) -> bool {
        self.reference_count.load(Ordering::SeqCst) == 1
    }
}

impl std::fmt::Debug for RefCountedImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefCountedImpl")
            .field("reference_count", &self.get_ref_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_counted_initialization() {
        let ref_counted = RefCountedImpl::new();
        assert_eq!(ref_counted.get_ref_count(), 1);
    }

    #[test]
    fn test_add_ref() {
        let ref_counted = RefCountedImpl::new();
        ref_counted.add_ref();
        assert_eq!(ref_counted.get_ref_count(), 2);
        ref_counted.add_ref();
        assert_eq!(ref_counted.get_ref_count(), 3);
    }

    #[test]
    fn test_release() {
        let ref_counted = RefCountedImpl::new();
        ref_counted.add_ref();
        ref_counted.add_ref();
        assert_eq!(ref_counted.get_ref_count(), 3);

        ref_counted.release();
        assert_eq!(ref_counted.get_ref_count(), 2);
        ref_counted.release();
        assert_eq!(ref_counted.get_ref_count(), 1);
    }

    #[test]
    #[should_panic(expected = "RefCounted::release called with zero reference count")]
    fn test_double_release_panics() {
        let ref_counted = RefCountedImpl::new();
        ref_counted.release();
        ref_counted.release();
    }

    #[test]
    fn test_is_last_reference() {
        let ref_counted = RefCountedImpl::new();
        assert!(ref_counted.is_last_reference());

        ref_counted.add_ref();
        assert!(!ref_counted.is_last_reference());

        ref_counted.release();
        assert!(ref_counted.is_last_reference());
    }
}
