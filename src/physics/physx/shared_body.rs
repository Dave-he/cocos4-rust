/****************************************************************************
Rust port of Cocos Creator PhysX Shared Body
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::{Vec3, Quaternion};
use super::inc::PxTransform;
use super::filter_shader::FilterData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedBodyType {
    Static = 0,
    Dynamic = 1,
    Kinematic = 2,
}

#[derive(Debug, Clone)]
pub struct SharedBodyConfig {
    pub body_type: SharedBodyType,
    pub filter_data: FilterData,
    pub initial_transform: PxTransform,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

impl Default for SharedBodyConfig {
    fn default() -> Self {
        SharedBodyConfig {
            body_type: SharedBodyType::Dynamic,
            filter_data: FilterData::default(),
            initial_transform: PxTransform::identity(),
            linear_damping: 0.0,
            angular_damping: 0.05,
        }
    }
}

#[derive(Debug)]
pub struct PhysXSharedBody {
    pub id: u32,
    pub config: SharedBodyConfig,
    pub transform: PxTransform,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub mass: f32,
    pub shape_ids: Vec<u32>,
    pub node_uuid: Option<String>,
    pub enabled: bool,
    pub is_sleeping: bool,
    ref_count: RefCountedImpl,
}

impl PhysXSharedBody {
    pub fn new(id: u32, config: SharedBodyConfig) -> Self {
        let transform = config.initial_transform;
        PhysXSharedBody {
            id,
            config,
            transform,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass: 1.0,
            shape_ids: Vec::new(),
            node_uuid: None,
            enabled: true,
            is_sleeping: false,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn get_position(&self) -> Vec3 {
        self.transform.position
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.transform.position = pos;
    }

    pub fn get_rotation(&self) -> Quaternion {
        self.transform.rotation
    }

    pub fn set_rotation(&mut self, rot: Quaternion) {
        self.transform.rotation = rot;
    }

    pub fn get_transform(&self) -> &PxTransform {
        &self.transform
    }

    pub fn set_transform(&mut self, t: PxTransform) {
        self.transform = t;
    }

    pub fn attach_shape(&mut self, shape_id: u32) {
        if !self.shape_ids.contains(&shape_id) {
            self.shape_ids.push(shape_id);
        }
    }

    pub fn detach_shape(&mut self, shape_id: u32) {
        self.shape_ids.retain(|&id| id != shape_id);
    }

    pub fn get_shape_count(&self) -> usize {
        self.shape_ids.len()
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass.max(0.0);
    }

    pub fn apply_force(&mut self, force: Vec3) {
        if self.config.body_type == SharedBodyType::Dynamic && self.enabled && !self.is_sleeping {
            let accel = force / self.mass;
            self.linear_velocity = self.linear_velocity + accel;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.config.body_type == SharedBodyType::Dynamic && self.enabled {
            self.linear_velocity = self.linear_velocity + impulse / self.mass;
        }
    }

    pub fn step(&mut self, dt: f32) {
        if self.config.body_type != SharedBodyType::Dynamic || !self.enabled {
            return;
        }
        let damping = 1.0 - self.config.linear_damping * dt;
        self.linear_velocity = self.linear_velocity * damping.max(0.0);
        self.transform.position = self.transform.position + self.linear_velocity * dt;
    }

    pub fn wake_up(&mut self) {
        self.is_sleeping = false;
    }

    pub fn put_to_sleep(&mut self) {
        self.is_sleeping = true;
        self.linear_velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
    }

    pub fn set_node(&mut self, uuid: Option<String>) {
        self.node_uuid = uuid;
    }
}

impl RefCounted for PhysXSharedBody {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

pub struct PhysXSharedBodyManager {
    bodies: std::collections::HashMap<u32, PhysXSharedBody>,
    next_id: u32,
}

impl PhysXSharedBodyManager {
    pub fn new() -> Self {
        PhysXSharedBodyManager {
            bodies: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_body(&mut self, config: SharedBodyConfig) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.bodies.insert(id, PhysXSharedBody::new(id, config));
        id
    }

    pub fn destroy_body(&mut self, id: u32) {
        self.bodies.remove(&id);
    }

    pub fn get_body(&self, id: u32) -> Option<&PhysXSharedBody> {
        self.bodies.get(&id)
    }

    pub fn get_body_mut(&mut self, id: u32) -> Option<&mut PhysXSharedBody> {
        self.bodies.get_mut(&id)
    }

    pub fn get_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn step_all(&mut self, dt: f32) {
        for body in self.bodies.values_mut() {
            body.step(dt);
        }
    }
}

impl Default for PhysXSharedBodyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_body_new() {
        let body = PhysXSharedBody::new(1, SharedBodyConfig::default());
        assert_eq!(body.id, 1);
        assert!(body.enabled);
        assert_eq!(body.get_position(), Vec3::ZERO);
    }

    #[test]
    fn test_attach_detach_shape() {
        let mut body = PhysXSharedBody::new(1, SharedBodyConfig::default());
        body.attach_shape(10);
        body.attach_shape(20);
        assert_eq!(body.get_shape_count(), 2);
        body.detach_shape(10);
        assert_eq!(body.get_shape_count(), 1);
    }

    #[test]
    fn test_apply_impulse() {
        let mut body = PhysXSharedBody::new(1, SharedBodyConfig::default());
        body.set_mass(2.0);
        body.apply_impulse(Vec3::new(4.0, 0.0, 0.0));
        assert!((body.linear_velocity.x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_step() {
        let mut body = PhysXSharedBody::new(1, SharedBodyConfig::default());
        body.linear_velocity = Vec3::new(1.0, 0.0, 0.0);
        body.step(1.0);
        assert!((body.get_position().x - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_sleep() {
        let mut body = PhysXSharedBody::new(1, SharedBodyConfig::default());
        body.linear_velocity = Vec3::new(5.0, 0.0, 0.0);
        body.put_to_sleep();
        assert!(body.is_sleeping);
        assert_eq!(body.linear_velocity, Vec3::ZERO);
    }

    #[test]
    fn test_manager_create_step() {
        let mut mgr = PhysXSharedBodyManager::new();
        let id = mgr.create_body(SharedBodyConfig::default());
        mgr.get_body_mut(id).unwrap().linear_velocity = Vec3::new(1.0, 0.0, 0.0);
        mgr.step_all(1.0);
        let pos = mgr.get_body(id).unwrap().get_position();
        assert!((pos.x - 1.0).abs() < 1e-5);
    }
}
