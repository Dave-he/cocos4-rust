/****************************************************************************
Rust port of Cocos Creator Physics Rigid Body
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Vec3;
use crate::core::scene_graph::NodeWeakPtr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
    Dynamic = 0,
    Kinematic = 1,
    Static = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventType {
    Enter,
    Stay,
    Exit,
}

#[derive(Debug, Clone)]
pub struct CollisionContact {
    pub point: Vec3,
    pub normal: Vec3,
    pub separation: f32,
    pub impulse: Vec3,
}

impl CollisionContact {
    pub fn new(point: Vec3, normal: Vec3, separation: f32) -> Self {
        CollisionContact {
            point,
            normal,
            separation,
            impulse: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub event_type: CollisionEventType,
    pub other_id: u32,
    pub contacts: Vec<CollisionContact>,
}

#[derive(Debug, Clone)]
pub struct RigidBody {
    pub node: Option<NodeWeakPtr>,
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub use_gravity: bool,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub linear_factor: Vec3,
    pub angular_factor: Vec3,
    is_sleeping: bool,
    pending_force: Vec3,
    pending_impulse: Vec3,
    pending_torque: Vec3,
}

impl RigidBody {
    pub fn new() -> Self {
        RigidBody {
            node: None,
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            use_gravity: true,
            linear_damping: 0.0,
            angular_damping: 0.05,
            linear_factor: Vec3::ONE,
            angular_factor: Vec3::ONE,
            is_sleeping: false,
            pending_force: Vec3::ZERO,
            pending_impulse: Vec3::ZERO,
            pending_torque: Vec3::ZERO,
        }
    }

    pub fn with_node(node: NodeWeakPtr) -> Self {
        RigidBody {
            node: Some(node),
            ..Self::new()
        }
    }

    pub fn set_body_type(&mut self, body_type: RigidBodyType) {
        self.body_type = body_type;
    }

    pub fn set_mass(&mut self, mass: f32) {
        if mass > 0.0 {
            self.mass = mass;
        }
    }

    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
        self.wake_up();
    }

    pub fn set_angular_velocity(&mut self, angular_velocity: Vec3) {
        self.angular_velocity = angular_velocity;
        self.wake_up();
    }

    pub fn set_linear_factor(&mut self, factor: Vec3) {
        self.linear_factor = factor;
    }

    pub fn set_angular_factor(&mut self, factor: Vec3) {
        self.angular_factor = factor;
    }

    pub fn set_use_gravity(&mut self, use_gravity: bool) {
        self.use_gravity = use_gravity;
    }

    pub fn set_linear_damping(&mut self, damping: f32) {
        self.linear_damping = damping.clamp(0.0, 1.0);
    }

    pub fn set_angular_damping(&mut self, damping: f32) {
        self.angular_damping = damping.clamp(0.0, 1.0);
    }

    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type != RigidBodyType::Dynamic {
            return;
        }
        self.pending_force += force;
        self.wake_up();
    }

    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3) {
        if self.body_type != RigidBodyType::Dynamic {
            return;
        }
        self.pending_force += force;
        let torque = Vec3::cross_vecs(&point, &force);
        self.pending_torque += torque;
        self.wake_up();
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type != RigidBodyType::Dynamic {
            return;
        }
        self.pending_impulse += impulse;
        self.wake_up();
    }

    pub fn apply_torque(&mut self, torque: Vec3) {
        if self.body_type != RigidBodyType::Dynamic {
            return;
        }
        self.pending_torque += torque;
        self.wake_up();
    }

    pub fn clear_forces(&mut self) {
        self.pending_force = Vec3::ZERO;
        self.pending_impulse = Vec3::ZERO;
        self.pending_torque = Vec3::ZERO;
    }

    pub fn integrate(&mut self, dt: f32, gravity: Vec3) {
        if self.body_type != RigidBodyType::Dynamic || self.is_sleeping {
            return;
        }

        let inv_mass = 1.0 / self.mass;

        if self.use_gravity {
            self.velocity += gravity * dt;
        }

        let force_accel = self.pending_force * inv_mass;
        self.velocity += force_accel * dt;
        self.velocity += self.pending_impulse * inv_mass;

        let lf = self.linear_factor;
        self.velocity.x *= lf.x;
        self.velocity.y *= lf.y;
        self.velocity.z *= lf.z;

        let linear_damp = (1.0 - self.linear_damping * dt).max(0.0);
        self.velocity *= linear_damp;

        let torque_accel = self.pending_torque * inv_mass;
        let af = self.angular_factor;
        self.angular_velocity += torque_accel * dt;
        self.angular_velocity.x *= af.x;
        self.angular_velocity.y *= af.y;
        self.angular_velocity.z *= af.z;

        let angular_damp = (1.0 - self.angular_damping * dt).max(0.0);
        self.angular_velocity *= angular_damp;

        self.pending_force = Vec3::ZERO;
        self.pending_impulse = Vec3::ZERO;
        self.pending_torque = Vec3::ZERO;

        let speed_sq = self.velocity.length_squared() + self.angular_velocity.length_squared();
        if speed_sq < 1e-6 {
            self.is_sleeping = true;
        }
    }

    pub fn wake_up(&mut self) {
        self.is_sleeping = false;
    }

    pub fn put_to_sleep(&mut self) {
        self.is_sleeping = true;
        self.velocity = Vec3::ZERO;
        self.angular_velocity = Vec3::ZERO;
    }

    pub fn is_sleeping(&self) -> bool {
        self.is_sleeping
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_new() {
        let rb = RigidBody::new();
        assert_eq!(rb.body_type, RigidBodyType::Dynamic);
        assert_eq!(rb.mass, 1.0);
        assert!(!rb.is_sleeping());
        assert!(rb.use_gravity);
    }

    #[test]
    fn test_rigid_body_apply_force() {
        let mut rb = RigidBody::new();
        rb.apply_force(Vec3::new(10.0, 0.0, 0.0));
        rb.integrate(1.0, Vec3::ZERO);
        assert!(rb.velocity.x > 0.0);
    }

    #[test]
    fn test_rigid_body_apply_impulse() {
        let mut rb = RigidBody::new();
        rb.apply_impulse(Vec3::new(5.0, 0.0, 0.0));
        rb.integrate(1.0 / 60.0, Vec3::ZERO);
        assert_eq!(rb.velocity.x, 5.0);
    }

    #[test]
    fn test_rigid_body_gravity() {
        let mut rb = RigidBody::new();
        let gravity = Vec3::new(0.0, -9.8, 0.0);
        rb.integrate(1.0, gravity);
        assert!(rb.velocity.y < 0.0);
    }

    #[test]
    fn test_rigid_body_no_gravity_when_disabled() {
        let mut rb = RigidBody::new();
        rb.set_use_gravity(false);
        rb.integrate(1.0, Vec3::new(0.0, -9.8, 0.0));
        assert_eq!(rb.velocity.y, 0.0);
    }

    #[test]
    fn test_rigid_body_sleep_wake() {
        let mut rb = RigidBody::new();
        rb.put_to_sleep();
        assert!(rb.is_sleeping());
        rb.wake_up();
        assert!(!rb.is_sleeping());
    }

    #[test]
    fn test_rigid_body_static_ignores_force() {
        let mut rb = RigidBody::new();
        rb.set_body_type(RigidBodyType::Static);
        rb.apply_force(Vec3::new(100.0, 0.0, 0.0));
        rb.integrate(1.0, Vec3::new(0.0, -9.8, 0.0));
        assert_eq!(rb.velocity, Vec3::ZERO);
    }

    #[test]
    fn test_rigid_body_linear_factor() {
        let mut rb = RigidBody::new();
        rb.set_linear_factor(Vec3::new(1.0, 0.0, 1.0));
        rb.apply_impulse(Vec3::new(5.0, 5.0, 5.0));
        rb.integrate(1.0 / 60.0, Vec3::ZERO);
        assert!(rb.velocity.x > 0.0);
        assert_eq!(rb.velocity.y, 0.0);
        assert!(rb.velocity.z > 0.0);
    }

    #[test]
    fn test_collision_contact() {
        let contact = CollisionContact::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            0.01,
        );
        assert_eq!(contact.point, Vec3::new(1.0, 0.0, 0.0));
        assert!((contact.separation - 0.01).abs() < 1e-6);
        assert_eq!(contact.impulse, Vec3::ZERO);
    }

    #[test]
    fn test_rigid_body_set_mass_positive_only() {
        let mut rb = RigidBody::new();
        rb.set_mass(-1.0);
        assert_eq!(rb.mass, 1.0);
        rb.set_mass(5.0);
        assert_eq!(rb.mass, 5.0);
    }
}
