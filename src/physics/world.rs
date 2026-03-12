/****************************************************************************
Rust port of Cocos Creator Physics World
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;
use crate::base::RefCountedImpl;
use crate::math::{Vec3, Quaternion};
use crate::core::geometry::Ray;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsWorldType {
    Bullet = 0,
    Cannon = 1,
    PhysX = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsDrawFlags {
    None = 0,
    Wireframe = 1 << 0,
    Aabb = 1 << 1,
    ContactPoint = 1 << 2,
    ContactNormal = 1 << 3,
    Joint = 1 << 4,
    Shape = 1 << 5,
    All = 0xffffffff,
}

impl Default for PhysicsDrawFlags {
    fn default() -> Self {
        PhysicsDrawFlags::None
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsMaterial {
    pub friction: f32,
    pub restitution: f32,
}

impl Default for PhysicsMaterial {
    fn default() -> Self {
        PhysicsMaterial {
            friction: 0.5,
            restitution: 0.1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsRayResult {
    pub hit_point: Vec3,
    pub distance: f32,
    pub hit_normal: Vec3,
}

impl PhysicsRayResult {
    pub fn new() -> Self {
        PhysicsRayResult {
            hit_point: Vec3::ZERO,
            distance: 0.0,
            hit_normal: Vec3::ZERO,
        }
    }
}

impl Default for PhysicsRayResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RaycastOptions {
    pub mask: u32,
    pub group: i32,
    pub query_trigger: bool,
    pub max_distance: f32,
}

pub trait PhysicsWorld: RefCounted {
    fn get_impl(&self) -> Option<&dyn std::any::Any>;

    fn set_gravity(&mut self, gravity: Vec3);

    fn set_allow_sleep(&mut self, allow: bool);

    fn set_default_material(&mut self, material: PhysicsMaterial);

    fn step(&mut self, fixed_time_step: f32, time_since_last_called: f32, max_sub_steps: i32);

    fn raycast(&self, world_ray: &Ray, options: &RaycastOptions, results: &mut Vec<PhysicsRayResult>) -> bool;

    fn raycast_closest(&self, world_ray: &Ray, options: &RaycastOptions, out: &mut PhysicsRayResult) -> bool;

    fn sweep_box(
        &self,
        world_ray: &Ray,
        half_extents: &Vec3,
        orientation: &Quaternion,
        options: &RaycastOptions,
        results: &mut Vec<PhysicsRayResult>,
    ) -> bool;

    fn sweep_box_closest(
        &self,
        world_ray: &Ray,
        half_extents: &Vec3,
        orientation: &Quaternion,
        options: &RaycastOptions,
        out: &mut PhysicsRayResult,
    ) -> bool;

    fn sweep_sphere(
        &self,
        world_ray: &Ray,
        radius: f32,
        options: &RaycastOptions,
        results: &mut Vec<PhysicsRayResult>,
    ) -> bool;

    fn sweep_sphere_closest(
        &self,
        world_ray: &Ray,
        radius: f32,
        options: &RaycastOptions,
        out: &mut PhysicsRayResult,
    ) -> bool;

    fn sweep_capsule(
        &self,
        world_ray: &Ray,
        radius: f32,
        height: f32,
        orientation: &Quaternion,
        options: &RaycastOptions,
        results: &mut Vec<PhysicsRayResult>,
    ) -> bool;

    fn sweep_capsule_closest(
        &self,
        world_ray: &Ray,
        radius: f32,
        height: f32,
        orientation: &Quaternion,
        options: &RaycastOptions,
        out: &mut PhysicsRayResult,
    ) -> bool;

    fn emit_events(&mut self);

    fn sync_scene_to_physics(&mut self);

    fn sync_after_events(&mut self);

    fn destroy(&mut self);

    fn get_debug_draw_flags(&self) -> PhysicsDrawFlags;

    fn set_debug_draw_flags(&mut self, flags: PhysicsDrawFlags);

    fn get_debug_draw_constraint_size(&self) -> f32;

    fn set_debug_draw_constraint_size(&mut self, size: f32);
}

#[derive(Debug)]
pub struct PhysicsWorldImpl {
    pub gravity: Vec3,
    pub allow_sleep: bool,
    pub default_material: PhysicsMaterial,
    pub debug_draw_flags: PhysicsDrawFlags,
    pub debug_draw_constraint_size: f32,
    pub fixed_time_step: f32,
    pub max_sub_steps: i32,
    pub auto_simulation: bool,
    ref_count: RefCountedImpl,
}

impl Default for PhysicsWorldImpl {
    fn default() -> Self {
        PhysicsWorldImpl {
            gravity: Vec3::new(0.0, -10.0, 0.0),
            allow_sleep: true,
            default_material: PhysicsMaterial::default(),
            debug_draw_flags: PhysicsDrawFlags::None,
            debug_draw_constraint_size: 0.3,
            fixed_time_step: 1.0 / 60.0,
            max_sub_steps: 1,
            auto_simulation: true,
            ref_count: RefCountedImpl::new(),
        }
    }
}

impl PhysicsWorldImpl {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RefCounted for PhysicsWorldImpl {
    fn add_ref(&self) {
        self.ref_count.add_ref();
    }

    fn release(&self) {
        self.ref_count.release();
    }

    fn get_ref_count(&self) -> u32 {
        self.ref_count.get_ref_count()
    }

    fn is_last_reference(&self) -> bool {
        self.ref_count.is_last_reference()
    }
}

impl PhysicsWorld for PhysicsWorldImpl {
    fn get_impl(&self) -> Option<&dyn std::any::Any> {
        None
    }

    fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    fn set_allow_sleep(&mut self, allow: bool) {
        self.allow_sleep = allow;
    }

    fn set_default_material(&mut self, material: PhysicsMaterial) {
        self.default_material = material;
    }

    fn step(&mut self, _fixed_time_step: f32, _time_since_last_called: f32, _max_sub_steps: i32) {
    }

    fn raycast(&self, _world_ray: &Ray, _options: &RaycastOptions, _results: &mut Vec<PhysicsRayResult>) -> bool {
        false
    }

    fn raycast_closest(&self, _world_ray: &Ray, _options: &RaycastOptions, _out: &mut PhysicsRayResult) -> bool {
        false
    }

    fn sweep_box(
        &self,
        _world_ray: &Ray,
        _half_extents: &Vec3,
        _orientation: &Quaternion,
        _options: &RaycastOptions,
        _results: &mut Vec<PhysicsRayResult>,
    ) -> bool {
        false
    }

    fn sweep_box_closest(
        &self,
        _world_ray: &Ray,
        _half_extents: &Vec3,
        _orientation: &Quaternion,
        _options: &RaycastOptions,
        _out: &mut PhysicsRayResult,
    ) -> bool {
        false
    }

    fn sweep_sphere(
        &self,
        _world_ray: &Ray,
        _radius: f32,
        _options: &RaycastOptions,
        _results: &mut Vec<PhysicsRayResult>,
    ) -> bool {
        false
    }

    fn sweep_sphere_closest(
        &self,
        _world_ray: &Ray,
        _radius: f32,
        _options: &RaycastOptions,
        _out: &mut PhysicsRayResult,
    ) -> bool {
        false
    }

    fn sweep_capsule(
        &self,
        _world_ray: &Ray,
        _radius: f32,
        _height: f32,
        _orientation: &Quaternion,
        _options: &RaycastOptions,
        _results: &mut Vec<PhysicsRayResult>,
    ) -> bool {
        false
    }

    fn sweep_capsule_closest(
        &self,
        _world_ray: &Ray,
        _radius: f32,
        _height: f32,
        _orientation: &Quaternion,
        _options: &RaycastOptions,
        _out: &mut PhysicsRayResult,
    ) -> bool {
        false
    }

    fn emit_events(&mut self) {
    }

    fn sync_scene_to_physics(&mut self) {
    }

    fn sync_after_events(&mut self) {
    }

    fn destroy(&mut self) {
    }

    fn get_debug_draw_flags(&self) -> PhysicsDrawFlags {
        self.debug_draw_flags
    }

    fn set_debug_draw_flags(&mut self, flags: PhysicsDrawFlags) {
        self.debug_draw_flags = flags;
    }

    fn get_debug_draw_constraint_size(&self) -> f32 {
        self.debug_draw_constraint_size
    }

    fn set_debug_draw_constraint_size(&mut self, size: f32) {
        self.debug_draw_constraint_size = size;
    }
}
