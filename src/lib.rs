#![allow(ambiguous_glob_reexports)]

pub mod application;
pub mod audio;
pub mod base;
pub mod core;
pub mod math;
pub mod physics;
pub mod platform;
pub mod renderer;
pub mod scene;
pub mod storage;
pub mod xr;

#[path = "2d/mod.rs"]
pub mod _2d;

#[path = "3d/mod.rs"]
pub mod _3d;

pub use base::{
    Clonable, Log, LogLevel, LogType, RefCounted,
};

pub use base::util::{
    align_to, clear_lowest_bit, get_bit_position, get_bit_position64, get_lowest_bit,
    get_stacktrace, next_pot, popcount, to_uint,
};

pub use base::value::{Value, ValueMap, ValueMapIntKey, ValueType, ValueVector};

pub use math::*;

pub use core::assets::*;
pub use core::event::*;
pub use core::geometry::*;

pub use platform::interfaces::*;

pub use renderer::core::*;
pub use renderer::frame_graph::*;
pub use renderer::gfx_base::*;
pub use renderer::pipeline::*;
