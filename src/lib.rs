#![allow(ambiguous_glob_reexports)]

// ---------------------------------------------------------------------------
// Round 48 — WASM-bindings build gating.
//
// The `wasm-bindings` Cargo feature (added round 48) compiles a thin
// JSON bridge over `agi_minigame::scene_gen` to `wasm32-unknown-unknown`
// for the AGI-miniGame TypeScript layer. Most of cocos4-rust's modules
// depend on native platform APIs (audio device, file system, native
// graphics backends, `0xffffffff` isize enum literals, etc.) that don't
// compile to wasm32; gating those modules behind
// `#[cfg(not(target_arch = "wasm32"))]` lets the wasm build link
// while keeping the default (native) build's surface identical.
//
// `base` is always compiled because `agi_minigame` uses `Value`/`ValueMap`.
// ---------------------------------------------------------------------------

pub mod agi_minigame;
pub mod base;

#[cfg(not(target_arch = "wasm32"))]
pub mod application;
#[cfg(not(target_arch = "wasm32"))]
pub mod audio;
#[cfg(not(target_arch = "wasm32"))]
pub mod bindings;
#[cfg(not(target_arch = "wasm32"))]
pub mod core;
#[cfg(not(target_arch = "wasm32"))]
pub mod dragon_bones;
#[cfg(not(target_arch = "wasm32"))]
pub mod game;
#[cfg(not(target_arch = "wasm32"))]
pub mod gi;
#[cfg(not(target_arch = "wasm32"))]
pub mod input;
#[cfg(not(target_arch = "wasm32"))]
pub mod math;
#[cfg(not(target_arch = "wasm32"))]
pub mod network;
#[cfg(not(target_arch = "wasm32"))]
pub mod particle;
#[cfg(not(target_arch = "wasm32"))]
pub mod particle_2d;
#[cfg(not(target_arch = "wasm32"))]
pub mod physics;
#[cfg(not(target_arch = "wasm32"))]
pub mod physics_2d;
pub use physics_2d::physics_world::DebugDrawFlags2D;
pub mod platform;
#[cfg(not(target_arch = "wasm32"))]
pub mod primitive;
#[cfg(not(target_arch = "wasm32"))]
pub mod profiler;
#[cfg(not(target_arch = "wasm32"))]
pub mod renderer;
#[cfg(not(target_arch = "wasm32"))]
pub mod scene;
#[cfg(not(target_arch = "wasm32"))]
pub mod serialization;
#[cfg(not(target_arch = "wasm32"))]
pub mod sorting;
#[cfg(not(target_arch = "wasm32"))]
pub mod spine;
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod terrain;
#[cfg(not(target_arch = "wasm32"))]
pub mod tiled_map;
#[cfg(not(target_arch = "wasm32"))]
pub mod tween;
#[cfg(not(target_arch = "wasm32"))]
pub mod ui;
#[cfg(not(target_arch = "wasm32"))]
pub mod xr;

#[cfg(not(target_arch = "wasm32"))]
#[path = "2d/mod.rs"]
pub mod _2d;

pub use _2d::mask::{Mask, MaskType, StencilStage as MaskStencilStage};

#[path = "3d/mod.rs"]
pub mod _3d;

pub use base::{
    Clonable, Log, LogLevel, LogType, ObjectPool, Poolable, RefCounted, TimerHandle, TimerManager,
};

pub use base::util::{
    align_to, clear_lowest_bit, get_bit_position, get_bit_position64, get_lowest_bit,
    get_stacktrace, next_pot, popcount, to_uint,
};

pub use base::value::{Value, ValueMap, ValueMapIntKey, ValueType, ValueVector};

// All non-base re-exports are native-only — they reference modules
// the wasm-bindings build does not compile.
#[cfg(not(target_arch = "wasm32"))]
pub use math::*;

#[cfg(not(target_arch = "wasm32"))]
pub use core::assets::asset_manager;
#[cfg(not(target_arch = "wasm32"))]
pub use core::assets::{Asset, AssetManager, LoadState};
#[cfg(not(target_arch = "wasm32"))]
pub use core::event::*;
#[cfg(not(target_arch = "wasm32"))]
pub use core::event_target::{EventKey, EventTarget};
#[cfg(not(target_arch = "wasm32"))]
pub use core::geometry::*;

#[cfg(not(target_arch = "wasm32"))]
pub use platform::interfaces::*;

#[cfg(not(target_arch = "wasm32"))]
pub use renderer::core::*;
#[cfg(not(target_arch = "wasm32"))]
pub use renderer::frame_graph::*;
#[cfg(not(target_arch = "wasm32"))]
pub use renderer::gfx_base::*;
#[cfg(not(target_arch = "wasm32"))]
pub use renderer::pipeline::*;

#[cfg(not(target_arch = "wasm32"))]
pub use game::{
    Director, DirectorEvent, Game, GameBootstrapContract, GameBootstrapError, GameConfig, GameEvent,
    SceneManager, SceneState,
};
#[cfg(not(target_arch = "wasm32"))]
pub use input::{
    EventKeyboard, EventMouse, EventTouch, Input, InputEventType, KeyCode, MouseButton, Touch,
};
#[cfg(not(target_arch = "wasm32"))]
pub use particle::{EmitShape, Emitter, Particle, ParticleSystem, ParticleSystemState};
#[cfg(not(target_arch = "wasm32"))]
pub use particle_2d::{EmitterMode2D, Particle2D, ParticleSystem2D, PositionType2D};
#[cfg(not(target_arch = "wasm32"))]
pub use profiler::{Counter, PerfCounter, Profiler};
#[cfg(not(target_arch = "wasm32"))]
pub use serialization::{Deserializer, SerializedValue, Serializer};
#[cfg(not(target_arch = "wasm32"))]
pub use sorting::{Sorting, SortingLayerInfo, SortingLayers};
#[cfg(not(target_arch = "wasm32"))]
pub use tween::tween::tween as make_tween;
#[cfg(not(target_arch = "wasm32"))]
pub use tween::{node_tween, EasingMethod, NodeTweenBuilder, Tween, TweenSystem};
#[cfg(not(target_arch = "wasm32"))]
pub use ui::{
    Button, ButtonEventType, ButtonTransition, Layout, LayoutResizeMode, LayoutType, ProgressBar,
    ScrollView, ScrollViewEventType, Toggle, ToggleContainer, Widget, WidgetAlignFlag,
};
