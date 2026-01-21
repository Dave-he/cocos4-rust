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

/// RefCounted provides manual reference counting for objects
///
/// Similar to C++'s RefCounted, this allows objects to be shared
/// and have their lifetime managed through addRef/release calls.
///
/// Unlike Rust's Arc, this is a manual reference counting system
/// that more closely matches the C++ API semantics.
///
/// # Safety
///
/// This struct is designed for use as a base class (via composition in Rust).
/// The reference counting operations are thread-safe, but the object
/// being counted must handle its own synchronization if accessed concurrently.
pub struct RefCounted {
    /// Current reference count
    _reference_count: AtomicU32,
}

impl RefCounted {
    /// Create a new RefCounted object
    ///
    /// The reference count starts at 1 (matching C++ semantics where
    /// newly created objects are implicitly owned by their creator)
    pub fn new() -> Self {
        RefCounted {
            _reference_count: AtomicU32::new(1),
        }
    }

    /// Increments the reference count
    ///
    /// This increases the Ref's reference count, indicating another
    /// owner of this object.
    pub fn add_ref(&self) {
        self._reference_count.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrements the reference count
    ///
    /// If the reference count reaches 0, this object should be deleted.
    /// In Rust, the caller is responsible for actually dropping the object
    /// after calling release.
    ///
    /// # Safety
    ///
    /// The caller must ensure that release is called exactly once for
    /// each corresponding add_ref, and that the object is not accessed
    /// after the final release.
    ///
    /// # Panics
    ///
    /// Panics if the reference count is already 0 (indicating
    /// double-release or memory corruption).
    pub fn release(&self) {
        let old_count = self._reference_count.fetch_sub(1, Ordering::SeqCst);
        assert!(
            old_count > 0,
            "RefCounted::release called with zero reference count"
        );
    }

    /// Returns the current reference count
    pub fn get_ref_count(&self) -> u32 {
        self._reference_count.load(Ordering::SeqCst)
    }

    /// Check if this is the last reference
    pub fn is_last_reference(&self) -> bool {
        self._reference_count.load(Ordering::SeqCst) == 1
    }
}

impl Default for RefCounted {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for RefCounted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefCounted")
            .field("reference_count", &self.get_ref_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ref_counted_initialization() {
        let ref_counted = RefCounted::new();
        assert_eq!(ref_counted.get_ref_count(), 1);
    }

    #[test]
    fn test_add_ref() {
        let ref_counted = RefCounted::new();
        ref_counted.add_ref();
        assert_eq!(ref_counted.get_ref_count(), 2);
        ref_counted.add_ref();
        assert_eq!(ref_counted.get_ref_count(), 3);
    }

    #[test]
    fn test_release() {
        let ref_counted = RefCounted::new();
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
        let ref_counted = RefCounted::new();
        ref_counted.release();
        ref_counted.release();
    }

    #[test]
    fn test_is_last_reference() {
        let ref_counted = RefCounted::new();
        assert!(ref_counted.is_last_reference());

        ref_counted.add_ref();
        assert!(!ref_counted.is_last_reference());

        ref_counted.release();
        assert!(ref_counted.is_last_reference());
    }
}
