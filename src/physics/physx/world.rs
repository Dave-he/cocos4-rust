/****************************************************************************
Rust port of Cocos Creator PhysX World
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::{Vec3, Quaternion};
use crate::core::geometry::Ray;
use crate::physics::world::{PhysicsWorld, PhysicsWorldImpl, PhysicsMaterial, PhysicsRayResult, RaycastOptions, PhysicsDrawFlags};
use super::shared_body::{PhysXSharedBodyManager, SharedBodyConfig, SharedBodyType};
use super::rigid_body::{PhysXRigidBodyManager, RigidBodyType};
use super::event_manager::PhysicsEventManager;
use super::filter_shader::{FilterData, default_filter_shader, FilterAction};

#[derive(Debug)]
pub struct PhysXWorld {
    pub gravity: Vec3,
    pub allow_sleep: bool,
    pub default_material: PhysicsMaterial,
    pub debug_draw_flags: PhysicsDrawFlags,
    pub debug_draw_constraint_size: f32,
    pub shared_body_manager: PhysXSharedBodyManager,
    pub rigid_body_manager: PhysXRigidBodyManager,
    pub event_manager: PhysicsEventManager,
    pub substep_count: u32,
    ref_count: RefCountedImpl,
}

impl PhysXWorld {
    pub fn new() -> Self {
        PhysXWorld {
            gravity: Vec3::new(0.0, -10.0, 0.0),
            allow_sleep: true,
            default_material: PhysicsMaterial::default(),
            debug_draw_flags: PhysicsDrawFlags::None,
            debug_draw_constraint_size: 0.3,
            shared_body_manager: PhysXSharedBodyManager::new(),
            rigid_body_manager: PhysXRigidBodyManager::new(),
            event_manager: PhysicsEventManager::new(),
            substep_count: 0,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn create_dynamic_body(&mut self) -> (u32, u32) {
        let config = SharedBodyConfig {
            body_type: SharedBodyType::Dynamic,
            ..SharedBodyConfig::default()
        };
        let shared_id = self.shared_body_manager.create_body(config);
        let rb_id = self.rigid_body_manager.create_body(RigidBodyType::Dynamic, shared_id);
        (shared_id, rb_id)
    }

    pub fn create_static_body(&mut self) -> u32 {
        let config = SharedBodyConfig {
            body_type: SharedBodyType::Static,
            ..SharedBodyConfig::default()
        };
        self.shared_body_manager.create_body(config)
    }

    pub fn simulate_step(&mut self, dt: f32) {
        self.substep_count += 1;
        let gravity = self.gravity;
        let rb_ids: Vec<u32> = self.rigid_body_manager
            .get_all_ids();
        for rb_id in rb_ids {
            if let Some(rb) = self.rigid_body_manager.get_body(rb_id) {
                let shared_id = rb.shared_body_id;
                if let Some(shared) = self.shared_body_manager.get_body_mut(shared_id) {
                    let lf = rb.linear_factor;
                    let ug = rb.is_using_gravity();
                    if ug && !rb.is_sleeping() {
                        let grav_delta = gravity * dt;
                        shared.linear_velocity = shared.linear_velocity + Vec3::new(
                            grav_delta.x * lf.x,
                            grav_delta.y * lf.y,
                            grav_delta.z * lf.z,
                        );
                    }
                }
            }
        }
        self.shared_body_manager.step_all(dt);
    }
}

impl Default for PhysXWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl RefCounted for PhysXWorld {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

impl PhysicsWorld for PhysXWorld {
    fn get_impl(&self) -> Option<&dyn std::any::Any> {
        Some(self)
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

    fn step(&mut self, fixed_time_step: f32, time_since_last_called: f32, max_sub_steps: i32) {
        let mut elapsed = time_since_last_called;
        let mut steps = 0;
        while elapsed >= fixed_time_step && steps < max_sub_steps {
            self.simulate_step(fixed_time_step);
            elapsed -= fixed_time_step;
            steps += 1;
        }
    }

    fn raycast(&self, world_ray: &Ray, options: &RaycastOptions, results: &mut Vec<PhysicsRayResult>) -> bool {
        false
    }

    fn raycast_closest(&self, world_ray: &Ray, options: &RaycastOptions, out: &mut PhysicsRayResult) -> bool {
        false
    }

    fn sweep_box(&self, _: &Ray, _: &Vec3, _: &Quaternion, _: &RaycastOptions, _: &mut Vec<PhysicsRayResult>) -> bool { false }
    fn sweep_box_closest(&self, _: &Ray, _: &Vec3, _: &Quaternion, _: &RaycastOptions, _: &mut PhysicsRayResult) -> bool { false }
    fn sweep_sphere(&self, _: &Ray, _: f32, _: &RaycastOptions, _: &mut Vec<PhysicsRayResult>) -> bool { false }
    fn sweep_sphere_closest(&self, _: &Ray, _: f32, _: &RaycastOptions, _: &mut PhysicsRayResult) -> bool { false }
    fn sweep_capsule(&self, _: &Ray, _: f32, _: f32, _: &Quaternion, _: &RaycastOptions, _: &mut Vec<PhysicsRayResult>) -> bool { false }
    fn sweep_capsule_closest(&self, _: &Ray, _: f32, _: f32, _: &Quaternion, _: &RaycastOptions, _: &mut PhysicsRayResult) -> bool { false }

    fn emit_events(&mut self) {
        let count = self.event_manager.pending_count();
    }

    fn sync_scene_to_physics(&mut self) {}

    fn sync_after_events(&mut self) {}

    fn destroy(&mut self) {}

    fn get_debug_draw_flags(&self) -> PhysicsDrawFlags { self.debug_draw_flags }
    fn set_debug_draw_flags(&mut self, flags: PhysicsDrawFlags) { self.debug_draw_flags = flags; }
    fn get_debug_draw_constraint_size(&self) -> f32 { self.debug_draw_constraint_size }
    fn set_debug_draw_constraint_size(&mut self, size: f32) { self.debug_draw_constraint_size = size; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physx_world_new() {
        let world = PhysXWorld::new();
        assert_eq!(world.gravity, Vec3::new(0.0, -10.0, 0.0));
        assert!(world.allow_sleep);
    }

    #[test]
    fn test_physx_world_create_dynamic() {
        let mut world = PhysXWorld::new();
        let (shared_id, rb_id) = world.create_dynamic_body();
        assert!(world.shared_body_manager.get_body(shared_id).is_some());
        assert!(world.rigid_body_manager.get_body(rb_id).is_some());
    }

    #[test]
    fn test_physx_world_simulate() {
        let mut world = PhysXWorld::new();
        let (shared_id, _rb_id) = world.create_dynamic_body();
        world.step(1.0 / 60.0, 1.0 / 60.0, 1);
        let body = world.shared_body_manager.get_body(shared_id).unwrap();
        assert!(body.linear_velocity.y < 0.0, "gravity should pull down");
    }

    #[test]
    fn test_physx_world_gravity_change() {
        let mut world = PhysXWorld::new();
        world.set_gravity(Vec3::new(0.0, -20.0, 0.0));
        assert_eq!(world.gravity.y, -20.0);
    }
}
