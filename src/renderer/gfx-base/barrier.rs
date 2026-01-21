/****************************************************************************
Rust port of Cocos Creator GFX Barrier
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

pub trait Barrier: RefCounted {}

pub trait GeneralBarrier: RefCounted {}

pub trait TextureBarrier: RefCounted {}

pub trait BufferBarrier: RefCounted {}
