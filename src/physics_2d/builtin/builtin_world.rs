use super::intersection;
use crate::math::Vec2;
use crate::physics_2d::collider::Collider2D;
use crate::physics_2d::joint::Joint2D;
use crate::physics_2d::rigid_body::RigidBody2D;
use crate::physics_2d::types::{
    ColliderType2D, ContactPoint2D, RayCastResult2D, RigidBodyType2D, AABB2D,
};

pub struct BuiltinWorld2D {
    gravity: Vec2,
    rigid_bodies: Vec<RigidBody2D>,
    colliders: Vec<Collider2D>,
    joints: Vec<Joint2D>,
    contact_points: Vec<ContactPoint2D>,
    next_body_id: u32,
    next_collider_id: u32,
    next_joint_id: u32,
    step_count: u64,
    #[allow(dead_code)]
    allow_sleep: bool,
}

impl BuiltinWorld2D {
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(0.0, -9.81),
            rigid_bodies: Vec::new(),
            colliders: Vec::new(),
            joints: Vec::new(),
            contact_points: Vec::new(),
            next_body_id: 0,
            next_collider_id: 0,
            next_joint_id: 0,
            step_count: 0,
            allow_sleep: true,
        }
    }

    pub fn create_rigid_body(&mut self) -> u32 {
        let id = self.next_body_id;
        self.next_body_id += 1;
        let body = RigidBody2D::new(id);
        self.rigid_bodies.push(body);
        id
    }

    pub fn destroy_rigid_body(&mut self, id: u32) -> bool {
        let before = self.rigid_bodies.len();
        self.rigid_bodies.retain(|b| b.id != id);
        self.rigid_bodies.len() < before
    }

    pub fn get_rigid_body(&self, id: u32) -> Option<&RigidBody2D> {
        self.rigid_bodies.iter().find(|b| b.id == id)
    }

    pub fn get_rigid_body_mut(&mut self, id: u32) -> Option<&mut RigidBody2D> {
        self.rigid_bodies.iter_mut().find(|b| b.id == id)
    }

    pub fn create_collider(&mut self, collider: Collider2D) -> u32 {
        let id = self.next_collider_id;
        self.next_collider_id += 1;
        let mut c = collider;
        c.id = id;
        self.colliders.push(c);
        id
    }

    pub fn destroy_collider(&mut self, id: u32) -> bool {
        let before = self.colliders.len();
        self.colliders.retain(|c| c.id != id);
        self.colliders.len() < before
    }

    pub fn create_joint(&mut self, joint: Joint2D) -> u32 {
        let id = self.next_joint_id;
        self.next_joint_id += 1;
        let mut j = joint;
        j.id = id;
        self.joints.push(j);
        id
    }

    pub fn step(&mut self, dt: f32) {
        self.contact_points.clear();
        for body in &mut self.rigid_bodies {
            if body.body_type != RigidBodyType2D::Dynamic || !body.enabled {
                continue;
            }
            body.linear_velocity[0] += self.gravity.x * body.gravity_scale * dt;
            body.linear_velocity[1] += self.gravity.y * body.gravity_scale * dt;
        }
        self.detect_collisions();
        self.step_count += 1;
    }

    fn detect_collisions(&mut self) {
        for i in 0..self.colliders.len() {
            for j in (i + 1)..self.colliders.len() {
                let a = &self.colliders[i];
                let b = &self.colliders[j];
                if !a.enabled || !b.enabled || (a.group & b.mask) == 0 || (b.group & a.mask) == 0 {
                    continue;
                }
                let intersecting = match (a.collider_type, b.collider_type) {
                    (ColliderType2D::Box, ColliderType2D::Box)
                    | (ColliderType2D::Box, ColliderType2D::Polygon)
                    | (ColliderType2D::Polygon, ColliderType2D::Box)
                    | (ColliderType2D::Polygon, ColliderType2D::Polygon) => {
                        let aabb_a = a.get_aabb([0.0, 0.0]);
                        let aabb_b = b.get_aabb([0.0, 0.0]);
                        intersection::aabb_overlap(
                            &aabb_a.min,
                            &aabb_a.max,
                            &aabb_b.min,
                            &aabb_b.max,
                        )
                    }
                    (ColliderType2D::Circle, ColliderType2D::Circle) => {
                        intersection::circle_circle_intersect(
                            &Vec2::new(a.offset[0], a.offset[1]),
                            a.radius,
                            &Vec2::new(b.offset[0], b.offset[1]),
                            b.radius,
                        )
                    }
                    (ColliderType2D::Circle, _) => {
                        let aabb_b = b.get_aabb([0.0, 0.0]);
                        intersection::aabb_circle_intersect(
                            &aabb_b.min,
                            &aabb_b.max,
                            &Vec2::new(a.offset[0], a.offset[1]),
                            a.radius,
                        )
                    }
                    (_, ColliderType2D::Circle) => {
                        let aabb_a = a.get_aabb([0.0, 0.0]);
                        intersection::aabb_circle_intersect(
                            &aabb_a.min,
                            &aabb_a.max,
                            &Vec2::new(b.offset[0], b.offset[1]),
                            b.radius,
                        )
                    }
                    _ => false,
                };
                if intersecting {
                    self.contact_points.push(ContactPoint2D {
                        point: Vec2::ZERO,
                        normal: Vec2::ZERO,
                        impulse: 1.0,
                        separation: 0.0,
                    });
                }
            }
        }
    }

    pub fn raycast(&self, _origin: Vec2, _direction: Vec2, _max_distance: f32) -> RayCastResult2D {
        RayCastResult2D::default()
    }

    pub fn query_aabb(&self, aabb: &AABB2D) -> Vec<u32> {
        self.colliders
            .iter()
            .filter(|c| {
                c.enabled
                    && intersection::aabb_overlap(
                        &c.get_aabb([0.0, 0.0]).min,
                        &c.get_aabb([0.0, 0.0]).max,
                        &aabb.min,
                        &aabb.max,
                    )
            })
            .map(|c| c.id)
            .collect()
    }

    pub fn get_contact_points(&self) -> &[ContactPoint2D] {
        &self.contact_points
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

    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.gravity = Vec2::new(x, y);
    }

    pub fn get_gravity(&self) -> Vec2 {
        self.gravity
    }
}

impl Default for BuiltinWorld2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics_2d::collider::Collider2D;
    use crate::physics_2d::joint::Joint2D;
    use crate::physics_2d::types::JointType2D;

    #[test]
    fn test_builtin_world_new() {
        let world = BuiltinWorld2D::new();
        assert_eq!(world.gravity.y, -9.81);
        assert_eq!(world.get_body_count(), 0);
    }

    #[test]
    fn test_create_destroy_body() {
        let mut world = BuiltinWorld2D::new();
        let id = world.create_rigid_body();
        assert_eq!(world.get_body_count(), 1);
        assert!(world.destroy_rigid_body(id));
        assert_eq!(world.get_body_count(), 0);
    }

    #[test]
    fn test_step_physics() {
        let mut world = BuiltinWorld2D::new();
        let id = world.create_rigid_body();
        world.step(1.0 / 60.0);
        assert_eq!(world.get_step_count(), 1);
        let body = world.get_rigid_body(id).unwrap();
        assert!(body.linear_velocity[1] < 0.0);
    }

    #[test]
    fn test_collision_detection() {
        let mut world = BuiltinWorld2D::new();
        let mut c1 = Collider2D::new(0);
        c1.set_as_box(1.0, 1.0);
        let mut c2 = Collider2D::new(0);
        c2.set_as_box(1.0, 1.0);
        world.create_collider(c1);
        world.create_collider(c2);
        world.step(1.0 / 60.0);
    }

    #[test]
    fn test_gravity() {
        let mut world = BuiltinWorld2D::new();
        world.set_gravity(0.0, -20.0);
        assert_eq!(world.get_gravity().y, -20.0);
        world.create_rigid_body();
        world.step(1.0 / 60.0);
    }

    #[test]
    fn test_query_aabb() {
        let mut world = BuiltinWorld2D::new();
        let mut c = Collider2D::new(0);
        c.set_as_box(2.0, 2.0);
        world.create_collider(c);
        let query = AABB2D::new(Vec2::new(-5.0, -5.0), Vec2::new(5.0, 5.0));
        let results = world.query_aabb(&query);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_joints() {
        let mut world = BuiltinWorld2D::new();
        let joint = Joint2D::new(0, JointType2D::Revolute, 1, 2);
        let _id = world.create_joint(joint);
        assert_eq!(world.get_joint_count(), 1);
    }

    #[test]
    fn test_contact_points() {
        let mut world = BuiltinWorld2D::new();
        let mut c1 = Collider2D::new(0);
        c1.set_as_box(1.0, 1.0);
        let mut c2 = Collider2D::new(0);
        c2.set_as_box(1.0, 1.0);
        world.create_collider(c1);
        world.create_collider(c2);
        world.step(1.0 / 60.0);
    }
}
