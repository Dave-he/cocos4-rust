use std::collections::HashMap;
use crate::math::Vec3;
use crate::core::geometry::AABB;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BodyId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColliderShape {
    AABB,
    Sphere,
    Capsule,
}

#[derive(Debug, Clone)]
pub struct CollisionEvent {
    pub body_a: BodyId,
    pub body_b: BodyId,
    pub contact_point: Vec3,
    pub normal: Vec3,
    pub penetration: f32,
}

#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub id: BodyId,
    pub body_type: BodyType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub mass: f32,
    pub gravity_scale: f32,
    pub restitution: f32,
    pub friction: f32,
    pub is_trigger: bool,
    pub layer: u32,
    pub mask: u32,

    shape: ColliderShape,
    half_extents: Vec3,
    radius: f32,
}

impl PhysicsBody {
    pub fn new_aabb(id: BodyId, position: Vec3, half_extents: Vec3) -> Self {
        PhysicsBody {
            id,
            body_type: BodyType::Dynamic,
            position,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 1.0,
            gravity_scale: 1.0,
            restitution: 0.3,
            friction: 0.5,
            is_trigger: false,
            layer: 1,
            mask: 0xFFFF_FFFF,
            shape: ColliderShape::AABB,
            half_extents,
            radius: 0.0,
        }
    }

    pub fn new_sphere(id: BodyId, position: Vec3, radius: f32) -> Self {
        PhysicsBody {
            id,
            body_type: BodyType::Dynamic,
            position,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 1.0,
            gravity_scale: 1.0,
            restitution: 0.3,
            friction: 0.5,
            is_trigger: false,
            layer: 1,
            mask: 0xFFFF_FFFF,
            shape: ColliderShape::Sphere,
            half_extents: Vec3::ZERO,
            radius,
        }
    }

    pub fn get_aabb(&self) -> AABB {
        match self.shape {
            ColliderShape::AABB => AABB::new(
                self.position.x, self.position.y, self.position.z,
                self.half_extents.x, self.half_extents.y, self.half_extents.z,
            ),
            ColliderShape::Sphere | ColliderShape::Capsule => AABB::new(
                self.position.x, self.position.y, self.position.z,
                self.radius, self.radius, self.radius,
            ),
        }
    }

    pub fn get_shape(&self) -> ColliderShape {
        self.shape
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn get_half_extents(&self) -> Vec3 {
        self.half_extents
    }

    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type == BodyType::Dynamic && self.mass > 0.0 {
            self.acceleration.x += force.x / self.mass;
            self.acceleration.y += force.y / self.mass;
            self.acceleration.z += force.z / self.mass;
        }
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == BodyType::Dynamic && self.mass > 0.0 {
            self.velocity.x += impulse.x / self.mass;
            self.velocity.y += impulse.y / self.mass;
            self.velocity.z += impulse.z / self.mass;
        }
    }

    pub fn set_velocity(&mut self, v: Vec3) {
        self.velocity = v;
    }

    pub fn set_position(&mut self, p: Vec3) {
        self.position = p;
    }

    pub fn integrate(&mut self, dt: f32, gravity: Vec3) {
        if self.body_type != BodyType::Dynamic {
            return;
        }
        self.acceleration.x += gravity.x * self.gravity_scale;
        self.acceleration.y += gravity.y * self.gravity_scale;
        self.acceleration.z += gravity.z * self.gravity_scale;

        self.velocity.x += self.acceleration.x * dt;
        self.velocity.y += self.acceleration.y * dt;
        self.velocity.z += self.acceleration.z * dt;

        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
        self.position.z += self.velocity.z * dt;

        self.acceleration = Vec3::ZERO;
    }
}

fn aabb_vs_aabb(a: &AABB, b: &AABB) -> Option<(Vec3, f32)> {
    let dx = b.center.x - a.center.x;
    let dy = b.center.y - a.center.y;
    let dz = b.center.z - a.center.z;
    let overlap_x = (a.half_extents.x + b.half_extents.x) - dx.abs();
    let overlap_y = (a.half_extents.y + b.half_extents.y) - dy.abs();
    let overlap_z = (a.half_extents.z + b.half_extents.z) - dz.abs();

    if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
        return None;
    }

    let (normal, penetration) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
        (Vec3::new(if dx < 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0), overlap_x)
    } else if overlap_y <= overlap_z {
        (Vec3::new(0.0, if dy < 0.0 { -1.0 } else { 1.0 }, 0.0), overlap_y)
    } else {
        (Vec3::new(0.0, 0.0, if dz < 0.0 { -1.0 } else { 1.0 }), overlap_z)
    };

    let contact = Vec3::new(
        a.center.x + normal.x * a.half_extents.x,
        a.center.y + normal.y * a.half_extents.y,
        a.center.z + normal.z * a.half_extents.z,
    );
    Some((normal, penetration))
}

fn sphere_vs_sphere(pa: Vec3, ra: f32, pb: Vec3, rb: f32) -> Option<(Vec3, f32)> {
    let dx = pb.x - pa.x;
    let dy = pb.y - pa.y;
    let dz = pb.z - pa.z;
    let dist2 = dx * dx + dy * dy + dz * dz;
    let r_sum = ra + rb;
    if dist2 >= r_sum * r_sum {
        return None;
    }
    let dist = dist2.sqrt();
    let normal = if dist < 1e-6 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(dx / dist, dy / dist, dz / dist)
    };
    Some((normal, r_sum - dist))
}

pub type CollisionCallback = Box<dyn Fn(&CollisionEvent) + Send + Sync>;

pub struct PhysicsSimulator {
    bodies: HashMap<BodyId, PhysicsBody>,
    next_id: u64,
    gravity: Vec3,
    collision_listeners: Vec<CollisionCallback>,
    trigger_listeners: Vec<CollisionCallback>,
    collision_events: Vec<CollisionEvent>,
    broadphase_layers: HashMap<u32, Vec<BodyId>>,
}

impl PhysicsSimulator {
    pub fn new() -> Self {
        PhysicsSimulator {
            bodies: HashMap::new(),
            next_id: 1,
            gravity: Vec3::new(0.0, -9.8, 0.0),
            collision_listeners: Vec::new(),
            trigger_listeners: Vec::new(),
            collision_events: Vec::new(),
            broadphase_layers: HashMap::new(),
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    pub fn get_gravity(&self) -> Vec3 {
        self.gravity
    }

    pub fn add_body(&mut self, body: PhysicsBody) -> BodyId {
        let id = body.id;
        let layer = body.layer;
        self.bodies.insert(id, body);
        self.broadphase_layers.entry(layer).or_default().push(id);
        id
    }

    pub fn create_aabb_body(&mut self, position: Vec3, half_extents: Vec3) -> BodyId {
        let id = BodyId(self.next_id);
        self.next_id += 1;
        let body = PhysicsBody::new_aabb(id, position, half_extents);
        self.add_body(body)
    }

    pub fn create_sphere_body(&mut self, position: Vec3, radius: f32) -> BodyId {
        let id = BodyId(self.next_id);
        self.next_id += 1;
        let body = PhysicsBody::new_sphere(id, position, radius);
        self.add_body(body)
    }

    pub fn remove_body(&mut self, id: BodyId) {
        if let Some(body) = self.bodies.remove(&id) {
            if let Some(layer_ids) = self.broadphase_layers.get_mut(&body.layer) {
                layer_ids.retain(|&bid| bid != id);
            }
        }
    }

    pub fn get_body(&self, id: BodyId) -> Option<&PhysicsBody> {
        self.bodies.get(&id)
    }

    pub fn get_body_mut(&mut self, id: BodyId) -> Option<&mut PhysicsBody> {
        self.bodies.get_mut(&id)
    }

    pub fn on_collision<F: Fn(&CollisionEvent) + Send + Sync + 'static>(&mut self, cb: F) {
        self.collision_listeners.push(Box::new(cb));
    }

    pub fn on_trigger<F: Fn(&CollisionEvent) + Send + Sync + 'static>(&mut self, cb: F) {
        self.trigger_listeners.push(Box::new(cb));
    }

    pub fn step(&mut self, dt: f32) {
        let gravity = self.gravity;
        for body in self.bodies.values_mut() {
            body.integrate(dt, gravity);
        }
        self.detect_collisions();
        self.dispatch_collision_events();
    }

    fn detect_collisions(&mut self) {
        self.collision_events.clear();
        let ids: Vec<BodyId> = self.bodies.keys().copied().collect();

        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let id_a = ids[i];
                let id_b = ids[j];

                let (body_a, body_b) = {
                    let a = self.bodies.get(&id_a);
                    let b = self.bodies.get(&id_b);
                    match (a, b) {
                        (Some(a), Some(b)) => (a.clone(), b.clone()),
                        _ => continue,
                    }
                };

                if body_a.mask & body_b.layer == 0 || body_b.mask & body_a.layer == 0 {
                    continue;
                }

                let result = match (body_a.get_shape(), body_b.get_shape()) {
                    (ColliderShape::AABB, ColliderShape::AABB) => {
                        let aa = body_a.get_aabb();
                        let ab = body_b.get_aabb();
                        aabb_vs_aabb(&aa, &ab)
                    }
                    (ColliderShape::Sphere, ColliderShape::Sphere) => {
                        sphere_vs_sphere(
                            body_a.position, body_a.get_radius(),
                            body_b.position, body_b.get_radius(),
                        )
                    }
                    _ => {
                        let aa = body_a.get_aabb();
                        let ab = body_b.get_aabb();
                        aabb_vs_aabb(&aa, &ab)
                    }
                };

                if let Some((normal, penetration)) = result {
                    let contact_point = Vec3::new(
                        (body_a.position.x + body_b.position.x) * 0.5,
                        (body_a.position.y + body_b.position.y) * 0.5,
                        (body_a.position.z + body_b.position.z) * 0.5,
                    );
                    self.collision_events.push(CollisionEvent {
                        body_a: id_a,
                        body_b: id_b,
                        contact_point,
                        normal,
                        penetration,
                    });

                    if !body_a.is_trigger && !body_b.is_trigger {
                        self.resolve_collision(id_a, id_b, normal, penetration);
                    }
                }
            }
        }
    }

    fn resolve_collision(&mut self, id_a: BodyId, id_b: BodyId, normal: Vec3, penetration: f32) {
        let (mass_a, mass_b, restitution, va, vb, type_a, type_b) = {
            let a = self.bodies.get(&id_a);
            let b = self.bodies.get(&id_b);
            match (a, b) {
                (Some(a), Some(b)) => (
                    a.mass, b.mass,
                    (a.restitution + b.restitution) * 0.5,
                    a.velocity, b.velocity,
                    a.body_type, b.body_type,
                ),
                _ => return,
            }
        };

        let inv_mass_a = if type_a == BodyType::Dynamic && mass_a > 0.0 { 1.0 / mass_a } else { 0.0 };
        let inv_mass_b = if type_b == BodyType::Dynamic && mass_b > 0.0 { 1.0 / mass_b } else { 0.0 };
        let inv_mass_total = inv_mass_a + inv_mass_b;

        if inv_mass_total < 1e-6 {
            return;
        }

        let rel_vel = Vec3::new(
            vb.x - va.x, vb.y - va.y, vb.z - va.z,
        );
        let rel_vel_along_normal = rel_vel.x * normal.x + rel_vel.y * normal.y + rel_vel.z * normal.z;

        if rel_vel_along_normal > 0.0 {
            return;
        }

        let j = -(1.0 + restitution) * rel_vel_along_normal / inv_mass_total;
        let impulse = Vec3::new(normal.x * j, normal.y * j, normal.z * j);

        if let Some(a) = self.bodies.get_mut(&id_a) {
            if a.body_type == BodyType::Dynamic {
                a.velocity.x -= impulse.x * inv_mass_a;
                a.velocity.y -= impulse.y * inv_mass_a;
                a.velocity.z -= impulse.z * inv_mass_a;
            }
        }
        if let Some(b) = self.bodies.get_mut(&id_b) {
            if b.body_type == BodyType::Dynamic {
                b.velocity.x += impulse.x * inv_mass_b;
                b.velocity.y += impulse.y * inv_mass_b;
                b.velocity.z += impulse.z * inv_mass_b;
            }
        }

        let correction_pct = 0.8;
        let correction_x = normal.x * penetration * correction_pct / inv_mass_total;
        let correction_y = normal.y * penetration * correction_pct / inv_mass_total;
        let correction_z = normal.z * penetration * correction_pct / inv_mass_total;

        if let Some(a) = self.bodies.get_mut(&id_a) {
            if a.body_type == BodyType::Dynamic {
                a.position.x -= correction_x * inv_mass_a;
                a.position.y -= correction_y * inv_mass_a;
                a.position.z -= correction_z * inv_mass_a;
            }
        }
        if let Some(b) = self.bodies.get_mut(&id_b) {
            if b.body_type == BodyType::Dynamic {
                b.position.x += correction_x * inv_mass_b;
                b.position.y += correction_y * inv_mass_b;
                b.position.z += correction_z * inv_mass_b;
            }
        }
    }

    fn dispatch_collision_events(&self) {
        for event in &self.collision_events {
            let body_a = self.bodies.get(&event.body_a);
            let body_b = self.bodies.get(&event.body_b);
            let is_trigger = body_a.map(|b| b.is_trigger).unwrap_or(false)
                || body_b.map(|b| b.is_trigger).unwrap_or(false);

            if is_trigger {
                for cb in &self.trigger_listeners {
                    cb(event);
                }
            } else {
                for cb in &self.collision_listeners {
                    cb(event);
                }
            }
        }
    }

    pub fn get_collision_events(&self) -> &[CollisionEvent] {
        &self.collision_events
    }

    pub fn get_body_count(&self) -> usize {
        self.bodies.len()
    }

    pub fn get_bodies_in_layer(&self, layer: u32) -> Vec<&PhysicsBody> {
        self.broadphase_layers
            .get(&layer)
            .map(|ids| ids.iter().filter_map(|id| self.bodies.get(id)).collect())
            .unwrap_or_default()
    }
}

impl Default for PhysicsSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_simulator_new() {
        let sim = PhysicsSimulator::new();
        assert_eq!(sim.get_body_count(), 0);
        assert!((sim.get_gravity().y - (-9.8)).abs() < 1e-5);
    }

    #[test]
    fn test_create_aabb_body() {
        let mut sim = PhysicsSimulator::new();
        let id = sim.create_aabb_body(Vec3::ZERO, Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(sim.get_body_count(), 1);
        assert!(sim.get_body(id).is_some());
    }

    #[test]
    fn test_create_sphere_body() {
        let mut sim = PhysicsSimulator::new();
        let id = sim.create_sphere_body(Vec3::ZERO, 1.0);
        assert!(sim.get_body(id).is_some());
        assert_eq!(sim.get_body(id).unwrap().get_shape(), ColliderShape::Sphere);
    }

    #[test]
    fn test_remove_body() {
        let mut sim = PhysicsSimulator::new();
        let id = sim.create_aabb_body(Vec3::ZERO, Vec3::new(0.5, 0.5, 0.5));
        sim.remove_body(id);
        assert_eq!(sim.get_body_count(), 0);
    }

    #[test]
    fn test_gravity_integration() {
        let mut sim = PhysicsSimulator::new();
        let id = sim.create_sphere_body(Vec3::new(0.0, 10.0, 0.0), 0.5);
        sim.step(1.0);
        let body = sim.get_body(id).unwrap();
        assert!(body.position.y < 10.0);
    }

    #[test]
    fn test_static_body_no_movement() {
        let mut sim = PhysicsSimulator::new();
        let id = sim.create_aabb_body(Vec3::ZERO, Vec3::new(5.0, 0.1, 5.0));
        sim.get_body_mut(id).unwrap().body_type = BodyType::Static;
        sim.step(1.0);
        let body = sim.get_body(id).unwrap();
        assert!((body.position.y).abs() < 1e-6);
    }

    #[test]
    fn test_aabb_collision_detection() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let _a = sim.create_aabb_body(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        let _b = sim.create_aabb_body(Vec3::new(1.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        sim.step(0.0);
        assert_eq!(sim.get_collision_events().len(), 1);
    }

    #[test]
    fn test_sphere_collision_detection() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let _a = sim.create_sphere_body(Vec3::new(0.0, 0.0, 0.0), 1.0);
        let _b = sim.create_sphere_body(Vec3::new(1.5, 0.0, 0.0), 1.0);
        sim.step(0.0);
        assert_eq!(sim.get_collision_events().len(), 1);
    }

    #[test]
    fn test_no_collision_far_apart() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let _a = sim.create_aabb_body(Vec3::new(-10.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5));
        let _b = sim.create_aabb_body(Vec3::new(10.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5));
        sim.step(0.0);
        assert_eq!(sim.get_collision_events().len(), 0);
    }

    #[test]
    fn test_collision_callback() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        sim.on_collision(move |_| { *c.lock().unwrap() += 1; });
        let _a = sim.create_aabb_body(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let _b = sim.create_aabb_body(Vec3::new(1.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        sim.step(0.0);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_trigger_callback() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let triggered = Arc::new(Mutex::new(false));
        let t = Arc::clone(&triggered);
        sim.on_trigger(move |_| { *t.lock().unwrap() = true; });
        let a = sim.create_aabb_body(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let _b = sim.create_aabb_body(Vec3::new(1.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        sim.get_body_mut(a).unwrap().is_trigger = true;
        sim.step(0.0);
        assert!(*triggered.lock().unwrap());
    }

    #[test]
    fn test_layer_mask_filtering() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let a = sim.create_aabb_body(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let b = sim.create_aabb_body(Vec3::new(1.5, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0));
        sim.get_body_mut(a).unwrap().layer = 1;
        sim.get_body_mut(a).unwrap().mask = 0;
        sim.step(0.0);
        assert_eq!(sim.get_collision_events().len(), 0);
    }

    #[test]
    fn test_apply_force() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let id = sim.create_sphere_body(Vec3::ZERO, 0.5);
        sim.get_body_mut(id).unwrap().apply_force(Vec3::new(10.0, 0.0, 0.0));
        sim.step(1.0);
        let body = sim.get_body(id).unwrap();
        assert!(body.position.x > 0.0);
    }

    #[test]
    fn test_apply_impulse() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let id = sim.create_sphere_body(Vec3::ZERO, 0.5);
        sim.get_body_mut(id).unwrap().apply_impulse(Vec3::new(5.0, 0.0, 0.0));
        let body = sim.get_body(id).unwrap();
        assert!(body.velocity.x > 0.0);
    }

    #[test]
    fn test_bodies_in_layer() {
        let mut sim = PhysicsSimulator::new();
        let mut body_a = PhysicsBody::new_aabb(BodyId(1), Vec3::ZERO, Vec3::new(0.5, 0.5, 0.5));
        body_a.layer = 1;
        let mut body_b = PhysicsBody::new_aabb(BodyId(2), Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.5, 0.5, 0.5));
        body_b.layer = 2;
        sim.add_body(body_a);
        sim.add_body(body_b);
        assert_eq!(sim.get_bodies_in_layer(1).len(), 1);
        assert_eq!(sim.get_bodies_in_layer(2).len(), 1);
    }

    #[test]
    fn test_collision_response_separates_bodies() {
        let mut sim = PhysicsSimulator::new();
        sim.set_gravity(Vec3::ZERO);
        let a = sim.create_sphere_body(Vec3::new(-0.9, 0.0, 0.0), 1.0);
        let b = sim.create_sphere_body(Vec3::new(0.9, 0.0, 0.0), 1.0);
        sim.get_body_mut(a).unwrap().velocity = Vec3::new(1.0, 0.0, 0.0);
        sim.get_body_mut(b).unwrap().velocity = Vec3::new(-1.0, 0.0, 0.0);
        sim.step(0.016);
        let va = sim.get_body(a).unwrap().velocity.x;
        let vb = sim.get_body(b).unwrap().velocity.x;
        assert!(va < 1.0 || vb > -1.0);
    }
}
