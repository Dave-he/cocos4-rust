/****************************************************************************
Rust port of Cocos Creator CCObject System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn all_hide_mask(&self) -> u32 {
        !(Self::ToDestroy
            | Self::Dirty
            | Self::Destroying
            | Self::DontDestroy
            | Self::Deactivating
            | Self::IsPreloadStarted
            | Self::IsOnLoadCalled
            | Self::IsOnLoadStarted
            | Self::IsStartCalled
            | Self::IsRotationLocked
            | Self::IsScaleLocked
            | Self::IsAnchorLocked
            | Self::IsSizeLocked
            | Self::IsPositionLocked)
    }

    pub fn has_any_hide_flag(flags: u32) -> bool {
        (flags & Self::all_hide_mask(0)) != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectFlagBits {
    pub zero: ObjectFlags,
    pub all_hide_mask: u32,
}

impl ObjectFlagBits {
    pub const ALL_HIDE_MASKS: ObjectFlagBits = ObjectFlagBits {
        zero: ObjectFlags::Zero,
        all_hide_mask: ObjectFlags::all_hide_mask(0),
    };
}

pub trait Scriptable {}

pub trait Object: RefCounted {
    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: &str);

    fn set_hide_flags(&mut self, flags: ObjectFlags);
    fn get_hide_flags(&self) -> u32;

    fn is_valid(&self) -> bool;

    fn destroy(&mut self) -> bool;

    fn is_object_valid<T: Object>(&self, obj: &T, strict_mode: bool) -> bool;

    fn to_string(&self) -> String;

    fn get_script_object(&self) -> Option<&dyn Scriptable>;
    fn set_script_object(&mut self, obj: Option<&dyn Scriptable>);
}

pub fn is_object_valid<T: Object>(obj: &T, _strict_mode: bool) -> bool {
    !obj.is_valid() || (_strict_mode && (obj.get_hide_flags() & ObjectFlags::Destroyed as u32) != 0)
}

pub fn deferred_destroy() {}
