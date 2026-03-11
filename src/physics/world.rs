/****************************************************************************
Rust port of Cocos Creator Physics World
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::math::Vec3;

pub trait World: RefCounted {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsWorldType {
    Bullet = 0,
    Cannon = 1,
    PhysX = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugDrawMode {
    None = 0,
    Wireframe = 1,
    Aabb = 2,
    ContactPoint = 4,
    ContactNormal = 8,
    All = 15,
}

#[derive(Clone)]
pub struct PhysicsRaycastOptions {
    pub collision_mask: u32,
    pub group: u32,
    pub report_trigger: bool,
    pub check_terrain: bool,
    pub check_rigid_body: bool,
    pub single_hit: bool,
}

impl Default for PhysicsRaycastOptions {
    fn default() -> Self {
        PhysicsRaycastOptions {
            collision_mask: u32::MAX,
            group: u32::MAX,
            report_trigger: true,
            check_terrain: true,
            check_rigid_body: true,
            single_hit: true,
        }
    }
}

#[derive(Clone)]
pub struct RaycastHit {
    pub collider: Option<()>,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub bullet: Option<()>,
}

impl RaycastHit {
    pub fn new() -> Self {
        RaycastHit {
            collider: None,
            point: Vec3::ZERO,
            normal: Vec3::ZERO,
            distance: 0.0,
            bullet: None,
        }
    }
}

impl Default for RaycastHit {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct PhysicsWorldConfig {
    pub gravity: Vec3,
    pub world_type: PhysicsWorldType,
    pub fixed_time_step: f32,
    pub max_sub_steps: u32,
    pub sleeping_threshold: f32,
    pub friction_restitution_threshold: f32,
    pub max_velocity: f32,
    pub enable: bool,
    pub allow_sleep: bool,
    pub use_collision_matrix: bool,
}

impl Default for PhysicsWorldConfig {
    fn default() -> Self {
        PhysicsWorldConfig {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            world_type: PhysicsWorldType::Bullet,
            fixed_time_step: 1.0 / 60.0,
            max_sub_steps: 3,
            sleeping_threshold: 0.2,
            friction_restitution_threshold: 0.05,
            max_velocity: 100.0,
            enable: true,
            allow_sleep: true,
            use_collision_matrix: true,
        }
    }
}

pub struct PhysicsWorld {
    config: PhysicsWorldConfig,
    debug_draw_mode: DebugDrawMode,
    wrapped_world: Option<()>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            config: PhysicsWorldConfig::default(),
            debug_draw_mode: DebugDrawMode::None,
            wrapped_world: None,
        }
    }

    pub fn with_config(config: PhysicsWorldConfig) -> Self {
        PhysicsWorld {
            config,
            debug_draw_mode: DebugDrawMode::None,
            wrapped_world: None,
        }
    }

    pub fn step(&mut self, _dt: f32) {}

    pub fn raycast(&self, _origin: Vec3, _direction: Vec3, _distance: f32, _options: &PhysicsRaycastOptions) -> Vec<RaycastHit> {
        Vec::new()
    }

    pub fn raycast_closest(&self, _origin: Vec3, _direction: Vec3, _distance: f32, _options: &PhysicsRaycastOptions) -> Option<Box<RaycastHit>> {
        None
    }

    pub fn get_gravity(&self) -> Vec3 {
        self.config.gravity
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.config.gravity = gravity;
    }

    pub fn get_fixed_time_step(&self) -> f32 {
        self.config.fixed_time_step
    }

    pub fn set_fixed_time_step(&mut self, step: f32) {
        self.config.fixed_time_step = step;
    }

    pub fn get_max_sub_steps(&self) -> u32 {
        self.config.max_sub_steps
    }

    pub fn set_max_sub_steps(&mut self, steps: u32) {
        self.config.max_sub_steps = steps;
    }

    pub fn get_debug_draw_mode(&self) -> DebugDrawMode {
        self.debug_draw_mode
    }

    pub fn set_debug_draw_mode(&mut self, mode: DebugDrawMode) {
        self.debug_draw_mode = mode;
    }

    pub fn enable(&mut self, enable: bool) {
        self.config.enable = enable;
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enable
    }

    pub fn emit_trigger_enter(_a: &(), _b: &()) {}
    pub fn emit_trigger_exit(_a: &(), _b: &()) {}
    pub fn emit_collision_enter(_a: &(), _b: &()) {}
    pub fn emit_collision_exit(_a: &(), _b: &()) {}
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
