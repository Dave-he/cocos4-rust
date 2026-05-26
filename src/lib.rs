#![allow(ambiguous_glob_reexports)]

pub mod agi_minigame;
pub mod application;
pub mod audio;
pub mod base;
pub mod bindings;
pub mod core;
pub mod dragon_bones;
pub mod game;
pub mod gi;
pub mod input;
pub mod math;
pub mod network;
pub mod particle;
pub mod particle_2d;
pub mod physics;
pub mod physics_2d;
pub mod platform;
pub mod primitive;
pub mod profiler;
pub mod renderer;
pub mod scene;
pub mod serialization;
pub mod sorting;
pub mod spine;
pub mod storage;
pub mod terrain;
pub mod tiled_map;
pub mod tween;
pub mod ui;
pub mod xr;

#[path = "2d/mod.rs"]
pub mod _2d;

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

pub use math::*;

pub use core::assets::asset_manager;
pub use core::assets::{Asset, AssetManager, LoadState};
pub use core::event::*;
pub use core::event_target::{EventKey, EventTarget};
pub use core::geometry::*;

pub use platform::interfaces::*;

pub use renderer::core::*;
pub use renderer::frame_graph::*;
pub use renderer::gfx_base::*;
pub use renderer::pipeline::*;

pub use game::{
    Director, DirectorEvent, Game, GameBootstrapContract, GameBootstrapError, GameConfig, GameEvent,
    SceneManager, SceneState,
};
pub use input::{
    EventKeyboard, EventMouse, EventTouch, Input, InputEventType, KeyCode, MouseButton, Touch,
};
pub use particle::{EmitShape, Emitter, Particle, ParticleSystem, ParticleSystemState};
pub use particle_2d::{EmitterMode2D, Particle2D, ParticleSystem2D, PositionType2D};
pub use profiler::{Counter, PerfCounter, Profiler};
pub use serialization::{Deserializer, SerializedValue, Serializer};
pub use sorting::{Sorting, SortingLayerInfo, SortingLayers};
pub use tween::tween::tween as make_tween;
pub use tween::{node_tween, EasingMethod, NodeTweenBuilder, Tween, TweenSystem};
pub use ui::{
    Button, ButtonEventType, ButtonTransition, Layout, LayoutResizeMode, LayoutType, ProgressBar,
    ScrollView, ScrollViewEventType, Toggle, ToggleContainer, Widget, WidgetAlignFlag,
};
