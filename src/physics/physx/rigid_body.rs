/****************************************************************************
Rust port of Cocos Creator PhysX Rigid Body
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::Vec3;
use super::shared_body::{PhysXSharedBody, SharedBodyConfig, SharedBodyType};
use super::inc::PxTransform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    Static = 0,
    Dynamic = 1,
    Kinematic = 2,
}

impl From<RigidBodyType> for SharedBodyType {
    fn from(t: RigidBodyType) -> Self {
        match t {
            RigidBodyType::Static => SharedBodyType::Static,
            RigidBodyType::Dynamic => SharedBodyType::Dynamic,
            RigidBodyType::Kinematic => SharedBodyType::Kinematic,
        }
    }
}

#[derive(Debug)]
pub struct PhysXRigidBody {
    pub id: u32,
    pub body_type: RigidBodyType,
    pub shared_body_id: u32,
    pub use_gravity: bool,
    pub linear_factor: Vec3,
    pub angular_factor: Vec3,
    pub sleep_threshold: f32,
    pub max_linear_velocity: f32,
    pub max_angular_velocity: f32,
    pub is_awake: bool,
    ref_count: RefCountedImpl,
}

impl PhysXRigidBody {
    pub fn new(id: u32, body_type: RigidBodyType, shared_body_id: u32) -> Self {
        PhysXRigidBody {
            id,
            body_type,
            shared_body_id,
            use_gravity: true,
            linear_factor: Vec3::ONE,
            angular_factor: Vec3::ONE,
            sleep_threshold: 0.005,
            max_linear_velocity: 100.0,
            max_angular_velocity: 7.0,
            is_awake: true,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn get_type(&self) -> RigidBodyType {
        self.body_type
    }

    pub fn set_type(&mut self, body_type: RigidBodyType) {
        self.body_type = body_type;
    }

    pub fn set_use_gravity(&mut self, use_gravity: bool) {
        self.use_gravity = use_gravity;
    }

    pub fn is_using_gravity(&self) -> bool {
        self.use_gravity
    }

    pub fn set_linear_factor(&mut self, factor: Vec3) {
        self.linear_factor = factor;
    }

    pub fn get_linear_factor(&self) -> Vec3 {
        self.linear_factor
    }

    pub fn set_angular_factor(&mut self, factor: Vec3) {
        self.angular_factor = factor;
    }

    pub fn get_angular_factor(&self) -> Vec3 {
        self.angular_factor
    }

    pub fn apply_velocity_to_shared(&self, body: &mut PhysXSharedBody, gravity: Vec3, dt: f32) {
        if !self.is_awake || self.body_type != RigidBodyType::Dynamic {
            return;
        }
        if self.use_gravity {
            let grav_delta = gravity * dt;
            body.linear_velocity = body.linear_velocity + Vec3::new(
                grav_delta.x * self.linear_factor.x,
                grav_delta.y * self.linear_factor.y,
                grav_delta.z * self.linear_factor.z,
            );
        }
        let speed = body.linear_velocity.length();
        if speed > self.max_linear_velocity {
            body.linear_velocity = body.linear_velocity.normalize() * self.max_linear_velocity;
        }
        let sleep_energy = body.linear_velocity.length_squared()
            + body.angular_velocity.length_squared();
        if sleep_energy < self.sleep_threshold * self.sleep_threshold {
            self.is_awake_ref(body);
        }
    }

    fn is_awake_ref(&self, _body: &PhysXSharedBody) {}

    pub fn wake_up(&mut self) {
        self.is_awake = true;
    }

    pub fn put_to_sleep(&mut self) {
        self.is_awake = false;
    }

    pub fn is_sleeping(&self) -> bool {
        !self.is_awake
    }
}

impl RefCounted for PhysXRigidBody {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

pub struct PhysXRigidBodyManager {
    bodies: std::collections::HashMap<u32, PhysXRigidBody>,
    next_id: u32,
}

impl PhysXRigidBodyManager {
    pub fn new() -> Self {
        PhysXRigidBodyManager {
            bodies: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_body(&mut self, body_type: RigidBodyType, shared_body_id: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.bodies.insert(id, PhysXRigidBody::new(id, body_type, shared_body_id));
        id
    }

    pub fn destroy_body(&mut self, id: u32) {
        self.bodies.remove(&id);
    }

    pub fn get_body(&self, id: u32) -> Option<&PhysXRigidBody> {
        self.bodies.get(&id)
    }

    pub fn get_body_mut(&mut self, id: u32) -> Option<&mut PhysXRigidBody> {
        self.bodies.get_mut(&id)
    }

    pub fn get_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn wake_all(&mut self) {
        for body in self.bodies.values_mut() {
            body.wake_up();
        }
    }

    pub fn get_all_ids(&self) -> Vec<u32> {
        self.bodies.keys().copied().collect()
    }
}

impl Default for PhysXRigidBodyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_new() {
        let body = PhysXRigidBody::new(1, RigidBodyType::Dynamic, 10);
        assert_eq!(body.get_type(), RigidBodyType::Dynamic);
        assert!(body.is_using_gravity());
        assert!(!body.is_sleeping());
    }

    #[test]
    fn test_rigid_body_sleep() {
        let mut body = PhysXRigidBody::new(1, RigidBodyType::Dynamic, 10);
        body.put_to_sleep();
        assert!(body.is_sleeping());
        body.wake_up();
        assert!(!body.is_sleeping());
    }

    #[test]
    fn test_rigid_body_linear_factor() {
        let mut body = PhysXRigidBody::new(1, RigidBodyType::Dynamic, 10);
        body.set_linear_factor(Vec3::new(1.0, 0.0, 1.0));
        assert_eq!(body.get_linear_factor(), Vec3::new(1.0, 0.0, 1.0));
    }

    #[test]
    fn test_rigid_body_manager() {
        let mut mgr = PhysXRigidBodyManager::new();
        let id = mgr.create_body(RigidBodyType::Dynamic, 1);
        assert_eq!(mgr.get_count(), 1);
        mgr.destroy_body(id);
        assert_eq!(mgr.get_count(), 0);
    }

    #[test]
    fn test_rigid_body_gravity_apply() {
        let rb = PhysXRigidBody::new(1, RigidBodyType::Dynamic, 1);
        let mut shared = PhysXSharedBody::new(1, SharedBodyConfig::default());
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        rb.apply_velocity_to_shared(&mut shared, gravity, 0.1);
        assert!(shared.linear_velocity.y < 0.0);
    }
}
