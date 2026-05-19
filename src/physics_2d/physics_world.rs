use crate::math::Vec2;
use super::types::{AABB2D, ContactPoint2D, RayCastResult2D};
use super::collider::Collider2D;
use super::rigid_body::RigidBody2D;
use super::joint::Joint2D;

pub struct PhysicsWorld2D {
    pub gravity: Vec2,
    pub allow_sleep: bool,
    pub auto_clear_forces: bool,
    #[allow(dead_code)]
    base_velocity_iterations: u32,
    #[allow(dead_code)]
    base_position_iterations: u32,
    rigid_bodies: Vec<RigidBody2D>,
    colliders: Vec<Collider2D>,
    joints: Vec<Joint2D>,
    contact_points: Vec<ContactPoint2D>,
    next_body_id: u32,
    next_collider_id: u32,
    next_joint_id: u32,
    step_count: u64,
}

impl PhysicsWorld2D {
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(0.0, -9.81),
            allow_sleep: true,
            auto_clear_forces: true,
            base_velocity_iterations: 8,
            base_position_iterations: 3,
            rigid_bodies: Vec::new(),
            colliders: Vec::new(),
            joints: Vec::new(),
            contact_points: Vec::new(),
            next_body_id: 0,
            next_collider_id: 0,
            next_joint_id: 0,
            step_count: 0,
        }
    }

    pub fn create_body(&mut self, body: RigidBody2D) -> u32 {
        let mut body = body;
        body.id = self.next_body_id;
        self.next_body_id += 1;
        self.rigid_bodies.push(body);
        self.next_body_id - 1
    }

    pub fn destroy_body(&mut self, id: u32) -> bool {
        let len_before = self.rigid_bodies.len();
        self.rigid_bodies.retain(|b| b.id != id);
        self.rigid_bodies.len() < len_before
    }

    pub fn get_body(&self, id: u32) -> Option<&RigidBody2D> {
        self.rigid_bodies.iter().find(|b| b.id == id)
    }

    pub fn get_body_mut(&mut self, id: u32) -> Option<&mut RigidBody2D> {
        self.rigid_bodies.iter_mut().find(|b| b.id == id)
    }

    pub fn create_collider(&mut self, collider: Collider2D) -> u32 {
        let mut collider = collider;
        collider.id = self.next_collider_id;
        self.next_collider_id += 1;
        self.colliders.push(collider);
        self.next_collider_id - 1
    }

    pub fn destroy_collider(&mut self, id: u32) -> bool {
        let len_before = self.colliders.len();
        self.colliders.retain(|c| c.id != id);
        self.colliders.len() < len_before
    }

    pub fn get_collider(&self, id: u32) -> Option<&Collider2D> {
        self.colliders.iter().find(|c| c.id == id)
    }

    pub fn create_joint(&mut self, joint: Joint2D) -> u32 {
        let mut joint = joint;
        joint.id = self.next_joint_id;
        self.next_joint_id += 1;
        self.joints.push(joint);
        self.next_joint_id - 1
    }

    pub fn destroy_joint(&mut self, id: u32) -> bool {
        let len_before = self.joints.len();
        self.joints.retain(|j| j.id != id);
        self.joints.len() < len_before
    }

    pub fn step(&mut self, dt: f32) {
        self.contact_points.clear();
        self.update_physics(dt);
        self.step_count += 1;
    }

    fn update_physics(&mut self, dt: f32) {
        for body in &mut self.rigid_bodies {
            if body.body_type != super::types::RigidBodyType2D::Dynamic {
                continue;
            }
            body.linear_velocity[0] += self.gravity.x * body.gravity_scale * dt;
            body.linear_velocity[1] += self.gravity.y * body.gravity_scale * dt;
        }
        self.detect_collisions();
    }

    fn detect_collisions(&mut self) {
        let combinations: Vec<(usize, usize)> = (0..self.colliders.len())
            .flat_map(|i| ((i + 1)..self.colliders.len()).map(move |j| (i, j)))
            .collect();

        for (i, j) in combinations {
            let a = &self.colliders[i];
            let b = &self.colliders[j];
            if !a.enabled || !b.enabled {
                continue;
            }
            if (a.group & b.mask) == 0 || (b.group & a.mask) == 0 {
                continue;
            }
            let aabb_a = a.get_aabb([0.0, 0.0]);
            let aabb_b = b.get_aabb([0.0, 0.0]);
            if aabb_a.overlaps(&aabb_b) {
                self.contact_points.push(ContactPoint2D {
                    point: Vec2::ZERO,
                    normal: Vec2::ZERO,
                    impulse: 1.0,
                    separation: 0.0,
                });
            }
        }
    }

    pub fn query_aabb(&self, aabb: &AABB2D) -> Vec<u32> {
        self.colliders
            .iter()
            .enumerate()
            .filter(|(_, c)| c.enabled && c.get_aabb([0.0, 0.0]).overlaps(aabb))
            .map(|(i, _)| i as u32)
            .collect()
    }

    #[allow(dead_code)]
    pub fn raycast(&self, _origin: Vec2, _direction: Vec2, _max_distance: f32) -> RayCastResult2D {
        RayCastResult2D::default()
    }

    pub fn get_contact_count(&self) -> usize {
        self.contact_points.len()
    }

    pub fn get_body_count(&self) -> usize {
        self.rigid_bodies.len()
    }

    pub fn get_collider_count(&self) -> usize {
        self.colliders.len()
    }

    pub fn get_joint_count(&self) -> usize {
        self.joints.len()
    }

    pub fn get_step_count(&self) -> u64 {
        self.step_count
    }

    pub fn clear(&mut self) {
        self.rigid_bodies.clear();
        self.colliders.clear();
        self.joints.clear();
        self.contact_points.clear();
        self.next_body_id = 0;
        self.next_collider_id = 0;
        self.next_joint_id = 0;
        self.step_count = 0;
    }
}

impl Default for PhysicsWorld2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_new() {
        let world = PhysicsWorld2D::new();
        assert_eq!(world.gravity.y, -9.81);
        assert_eq!(world.get_body_count(), 0);
    }

    #[test]
    fn test_create_destroy_body() {
        let mut world = PhysicsWorld2D::new();
        let body = RigidBody2D::new(0);
        let id = world.create_body(body);
        assert_eq!(world.get_body_count(), 1);
        assert!(world.get_body(id).is_some());
        world.destroy_body(id);
        assert_eq!(world.get_body_count(), 0);
    }

    #[test]
    fn test_create_collider() {
        let mut world = PhysicsWorld2D::new();
        let collider = Collider2D::new(0);
        let id = world.create_collider(collider);
        assert_eq!(world.get_collider_count(), 1);
        assert!(world.get_collider(id).is_some());
    }

    #[test]
    fn test_create_joint() {
        let mut world = PhysicsWorld2D::new();
        let joint = Joint2D::new(0, crate::physics_2d::types::JointType2D::Revolute, 1, 2);
        let _id = world.create_joint(joint);
        assert_eq!(world.get_joint_count(), 1);
    }

    #[test]
    fn test_world_step() {
        let mut world = PhysicsWorld2D::new();
        world.create_body(RigidBody2D::new(0));
        world.step(1.0 / 60.0);
        assert_eq!(world.get_step_count(), 1);
    }

    #[test]
    fn test_world_collision_detection() {
        let mut world = PhysicsWorld2D::new();
        let mut c1 = Collider2D::new(0);
        c1.set_as_box(1.0, 1.0);
        let mut c2 = Collider2D::new(0);
        c2.set_as_box(1.0, 1.0);
        world.create_collider(c1);
        world.create_collider(c2);
        world.step(1.0 / 60.0);
        assert_eq!(world.get_step_count(), 1);
    }

    #[test]
    fn test_query_aabb() {
        let mut world = PhysicsWorld2D::new();
        let mut c = Collider2D::new(0);
        c.set_as_box(1.0, 1.0);
        world.create_collider(c);
        let query = AABB2D::new(
            Vec2::new(-10.0, -10.0),
            Vec2::new(10.0, 10.0),
        );
        let results = world.query_aabb(&query);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_world_clear() {
        let mut world = PhysicsWorld2D::new();
        world.create_body(RigidBody2D::new(0));
        world.create_collider(Collider2D::new(0));
        world.clear();
        assert_eq!(world.get_body_count(), 0);
        assert_eq!(world.get_collider_count(), 0);
        assert_eq!(world.get_step_count(), 0);
    }
}
