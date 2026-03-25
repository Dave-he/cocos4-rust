#![allow(ambiguous_glob_reexports)]

pub mod application;
pub mod audio;
pub mod base;
pub mod core;
pub mod game;
pub mod input;
pub mod math;
pub mod particle;
pub mod particle_2d;
pub mod physics;
pub mod platform;
pub mod profiler;
pub mod renderer;
pub mod scene;
pub mod serialization;
pub mod sorting;
pub mod storage;
pub mod tween;
pub mod ui;
pub mod xr;

#[path = "2d/mod.rs"]
pub mod _2d;

#[path = "3d/mod.rs"]
pub mod _3d;

pub use base::{
    Clonable, Log, LogLevel, LogType, RefCounted, ObjectPool, Poolable,
    TimerManager, TimerHandle,
};

pub use base::util::{
    align_to, clear_lowest_bit, get_bit_position, get_bit_position64, get_lowest_bit,
    get_stacktrace, next_pot, popcount, to_uint,
};

pub use base::value::{Value, ValueMap, ValueMapIntKey, ValueType, ValueVector};

pub use math::*;

pub use core::assets::{AssetManager, Asset, LoadState};
pub use core::event_target::{EventTarget, EventKey};
pub use core::assets::asset_manager;
pub use core::event::*;
pub use core::geometry::*;

pub use platform::interfaces::*;

pub use renderer::core::*;
pub use renderer::frame_graph::*;
pub use renderer::gfx_base::*;
pub use renderer::pipeline::*;

pub use game::{Director, DirectorEvent, Game, GameConfig, GameEvent, SceneManager, SceneState};
pub use input::{Input, KeyCode, MouseButton, Touch, EventKeyboard, EventMouse, EventTouch, InputEventType};
pub use tween::{Tween, TweenSystem, EasingMethod, NodeTweenBuilder, node_tween};
pub use tween::tween::tween as make_tween;
pub use profiler::{Counter, PerfCounter, Profiler};
pub use sorting::{Sorting, SortingLayers, SortingLayerInfo};
pub use particle::{Particle, ParticleSystem, ParticleSystemState, Emitter, EmitShape};
pub use particle_2d::{Particle2D, ParticleSystem2D, EmitterMode2D, PositionType2D};
pub use serialization::{SerializedValue, Serializer, Deserializer};
pub use ui::{
    Button, ButtonTransition, ButtonEventType,
    Layout, LayoutType, LayoutResizeMode,
    ScrollView, ScrollViewEventType,
    Widget, WidgetAlignFlag,
    ProgressBar,
    Toggle, ToggleContainer,
};
