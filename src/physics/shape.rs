/****************************************************************************
Rust port of Cocos Creator Physics Shape
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeType {
    Unknown = 0,
    Sphere = 1,
    Box = 2,
    Capsule = 3,
    Cylinder = 4,
    Cone = 5,
    Mesh = 6,
    Plane = 7,
    Simplex = 8,
}

#[derive(Debug, Clone)]
pub struct BoxShape {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl BoxShape {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        BoxShape { center, half_extents }
    }
}

impl Default for BoxShape {
    fn default() -> Self {
        BoxShape {
            center: Vec3::ZERO,
            half_extents: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SphereShape {
    pub center: Vec3,
    pub radius: f32,
}

impl SphereShape {
    pub fn new(center: Vec3, radius: f32) -> Self {
        SphereShape { center, radius }
    }
}

impl Default for SphereShape {
    fn default() -> Self {
        SphereShape {
            center: Vec3::ZERO,
            radius: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CapsuleShape {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub direction: u8,
}

impl CapsuleShape {
    pub fn new(center: Vec3, radius: f32, height: f32) -> Self {
        CapsuleShape { center, radius, height, direction: 1 }
    }
}

impl Default for CapsuleShape {
    fn default() -> Self {
        CapsuleShape {
            center: Vec3::ZERO,
            radius: 0.5,
            height: 2.0,
            direction: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CylinderShape {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub direction: u8,
}

impl Default for CylinderShape {
    fn default() -> Self {
        CylinderShape {
            center: Vec3::ZERO,
            radius: 0.5,
            height: 2.0,
            direction: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConeShape {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub direction: u8,
}

impl Default for ConeShape {
    fn default() -> Self {
        ConeShape {
            center: Vec3::ZERO,
            radius: 0.5,
            height: 1.0,
            direction: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaneShape {
    pub normal: Vec3,
    pub constant: f32,
}

impl Default for PlaneShape {
    fn default() -> Self {
        PlaneShape {
            normal: Vec3::new(0.0, 1.0, 0.0),
            constant: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeshShape {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub convex: bool,
}

impl MeshShape {
    pub fn new(vertices: Vec<Vec3>, indices: Vec<u32>, convex: bool) -> Self {
        MeshShape { vertices, indices, convex }
    }
}

#[derive(Debug, Clone)]
pub enum ShapeGeometry {
    Box(BoxShape),
    Sphere(SphereShape),
    Capsule(CapsuleShape),
    Cylinder(CylinderShape),
    Cone(ConeShape),
    Plane(PlaneShape),
    Mesh(MeshShape),
}

impl ShapeGeometry {
    pub fn shape_type(&self) -> ShapeType {
        match self {
            ShapeGeometry::Box(_) => ShapeType::Box,
            ShapeGeometry::Sphere(_) => ShapeType::Sphere,
            ShapeGeometry::Capsule(_) => ShapeType::Capsule,
            ShapeGeometry::Cylinder(_) => ShapeType::Cylinder,
            ShapeGeometry::Cone(_) => ShapeType::Cone,
            ShapeGeometry::Plane(_) => ShapeType::Plane,
            ShapeGeometry::Mesh(_) => ShapeType::Mesh,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsShapeConfig {
    pub geometry: ShapeGeometry,
    pub is_trigger: bool,
    pub contact_offset: f32,
    pub rest_offset: f32,
    pub mask: u32,
    pub group: i32,
}

impl PhysicsShapeConfig {
    pub fn new_sphere(radius: f32) -> Self {
        PhysicsShapeConfig {
            geometry: ShapeGeometry::Sphere(SphereShape::new(Vec3::ZERO, radius)),
            is_trigger: false,
            contact_offset: 0.02,
            rest_offset: 0.0,
            mask: 0xffffffff,
            group: 1,
        }
    }

    pub fn new_box(half_extents: Vec3) -> Self {
        PhysicsShapeConfig {
            geometry: ShapeGeometry::Box(BoxShape::new(Vec3::ZERO, half_extents)),
            is_trigger: false,
            contact_offset: 0.02,
            rest_offset: 0.0,
            mask: 0xffffffff,
            group: 1,
        }
    }

    pub fn new_capsule(radius: f32, height: f32) -> Self {
        PhysicsShapeConfig {
            geometry: ShapeGeometry::Capsule(CapsuleShape::new(Vec3::ZERO, radius, height)),
            is_trigger: false,
            contact_offset: 0.02,
            rest_offset: 0.0,
            mask: 0xffffffff,
            group: 1,
        }
    }
}

#[derive(Debug)]
pub struct PhysicsShapeImpl {
    pub id: u32,
    pub config: PhysicsShapeConfig,
    pub enabled: bool,
    pub node_uuid: Option<String>,
    ref_count: RefCountedImpl,
}

impl PhysicsShapeImpl {
    pub fn new(id: u32, config: PhysicsShapeConfig) -> Self {
        PhysicsShapeImpl {
            id,
            config,
            enabled: true,
            node_uuid: None,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn get_type(&self) -> ShapeType {
        self.config.geometry.shape_type()
    }

    pub fn set_trigger(&mut self, trigger: bool) {
        self.config.is_trigger = trigger;
    }

    pub fn is_trigger(&self) -> bool {
        self.config.is_trigger
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_mask(&mut self, mask: u32) {
        self.config.mask = mask;
    }

    pub fn get_mask(&self) -> u32 {
        self.config.mask
    }

    pub fn set_group(&mut self, group: i32) {
        self.config.group = group;
    }

    pub fn get_group(&self) -> i32 {
        self.config.group
    }

    pub fn set_node(&mut self, uuid: Option<String>) {
        self.node_uuid = uuid;
    }
}

impl RefCounted for PhysicsShapeImpl {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

pub struct PhysicsShapeManager {
    shapes: std::collections::HashMap<u32, PhysicsShapeImpl>,
    next_id: u32,
}

impl PhysicsShapeManager {
    pub fn new() -> Self {
        PhysicsShapeManager {
            shapes: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_shape(&mut self, config: PhysicsShapeConfig) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.shapes.insert(id, PhysicsShapeImpl::new(id, config));
        id
    }

    pub fn destroy_shape(&mut self, id: u32) {
        self.shapes.remove(&id);
    }

    pub fn get_shape(&self, id: u32) -> Option<&PhysicsShapeImpl> {
        self.shapes.get(&id)
    }

    pub fn get_shape_mut(&mut self, id: u32) -> Option<&mut PhysicsShapeImpl> {
        self.shapes.get_mut(&id)
    }

    pub fn get_shape_count(&self) -> usize {
        self.shapes.len()
    }

    pub fn get_shapes_by_group(&self, group: i32) -> Vec<&PhysicsShapeImpl> {
        self.shapes.values().filter(|s| s.config.group == group).collect()
    }
}

impl Default for PhysicsShapeManager {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Shape: RefCounted {
    fn get_type(&self) -> ShapeType;
    fn get_box_shape(&self) -> Option<&BoxShape>;
    fn get_sphere_shape(&self) -> Option<&SphereShape>;
    fn get_capsule_shape(&self) -> Option<&CapsuleShape>;
}

impl Shape for PhysicsShapeImpl {
    fn get_type(&self) -> ShapeType {
        self.config.geometry.shape_type()
    }

    fn get_box_shape(&self) -> Option<&BoxShape> {
        if let ShapeGeometry::Box(ref b) = self.config.geometry { Some(b) } else { None }
    }

    fn get_sphere_shape(&self) -> Option<&SphereShape> {
        if let ShapeGeometry::Sphere(ref s) = self.config.geometry { Some(s) } else { None }
    }

    fn get_capsule_shape(&self) -> Option<&CapsuleShape> {
        if let ShapeGeometry::Capsule(ref c) = self.config.geometry { Some(c) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_shape() {
        let cfg = PhysicsShapeConfig::new_sphere(1.0);
        assert_eq!(cfg.geometry.shape_type(), ShapeType::Sphere);
        if let ShapeGeometry::Sphere(ref s) = cfg.geometry {
            assert!((s.radius - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn test_box_shape() {
        let cfg = PhysicsShapeConfig::new_box(Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(cfg.geometry.shape_type(), ShapeType::Box);
    }

    #[test]
    fn test_capsule_shape() {
        let cfg = PhysicsShapeConfig::new_capsule(0.5, 2.0);
        assert_eq!(cfg.geometry.shape_type(), ShapeType::Capsule);
    }

    #[test]
    fn test_shape_manager_create_destroy() {
        let mut mgr = PhysicsShapeManager::new();
        let id = mgr.create_shape(PhysicsShapeConfig::new_sphere(1.0));
        assert_eq!(mgr.get_shape_count(), 1);
        mgr.destroy_shape(id);
        assert_eq!(mgr.get_shape_count(), 0);
    }

    #[test]
    fn test_shape_manager_get() {
        let mut mgr = PhysicsShapeManager::new();
        let id = mgr.create_shape(PhysicsShapeConfig::new_box(Vec3::new(0.5, 0.5, 0.5)));
        let shape = mgr.get_shape(id);
        assert!(shape.is_some());
        assert_eq!(shape.unwrap().get_type(), ShapeType::Box);
    }

    #[test]
    fn test_shape_trigger() {
        let mut mgr = PhysicsShapeManager::new();
        let id = mgr.create_shape(PhysicsShapeConfig::new_sphere(1.0));
        let shape = mgr.get_shape_mut(id).unwrap();
        assert!(!shape.is_trigger());
        shape.set_trigger(true);
        assert!(shape.is_trigger());
    }

    #[test]
    fn test_shape_group_mask() {
        let mut mgr = PhysicsShapeManager::new();
        let id = mgr.create_shape(PhysicsShapeConfig::new_sphere(1.0));
        let shape = mgr.get_shape_mut(id).unwrap();
        shape.set_group(2);
        shape.set_mask(0x0000ffff);
        assert_eq!(shape.get_group(), 2);
        assert_eq!(shape.get_mask(), 0x0000ffff);
    }

    #[test]
    fn test_shape_by_group() {
        let mut mgr = PhysicsShapeManager::new();
        mgr.create_shape(PhysicsShapeConfig::new_sphere(1.0));
        let id2 = mgr.create_shape(PhysicsShapeConfig::new_box(Vec3::new(1.0, 1.0, 1.0)));
        mgr.get_shape_mut(id2).unwrap().set_group(2);
        let group2 = mgr.get_shapes_by_group(2);
        assert_eq!(group2.len(), 1);
    }
}
