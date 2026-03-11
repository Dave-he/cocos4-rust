/****************************************************************************
Rust port of Cocos Creator Scene Graph system
Original C++ version Copyright (c) 2022-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::sync::{Arc, Weak};

pub use crate::math::Vec3;
pub use crate::math::{Mat4, Quaternion};

pub type NodePtr = Arc<dyn Node>;
pub type NodeWeakPtr = Weak<dyn Node>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeSpace {
    Local,
    World,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformBit {
    None = 0,
    Position = 1,
    Rotation = 2,
    Scale = 4,
    Skew = 8,
    RS = 6,
    RSS = 14,
    TRS = 7,
}

impl TransformBit {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl std::ops::BitOr for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as i32 | rhs as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobilityMode {
    Static = 0,
    Stationary = 1,
    Movable = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkewType {
    None = 0,
    Standard = 1,
    Rotational = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
    pub skew: SkewType,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Vec3::ONE,
            skew: SkewType::None,
        }
    }
}

pub trait Node: Send + Sync {
    fn get_name(&self) -> &str;
    fn get_parent(&self) -> Option<Weak<dyn Node>>;
    fn set_parent(&mut self, parent: Option<Weak<dyn Node>>);
    fn get_children(&self) -> Vec<Arc<dyn Node>>;
    fn add_child(&mut self, child: Arc<dyn Node>);
    fn remove_child(&mut self, child: &Arc<dyn Node>);
    fn set_position(&mut self, position: Vec3);
    fn get_position(&self) -> Vec3;
    fn set_rotation(&mut self, rotation: Quaternion);
    fn get_rotation(&self) -> Quaternion;
    fn set_scale(&mut self, scale: Vec3);
    fn get_scale(&self) -> Vec3;
    fn get_transform(&self) -> Transform;
    fn set_transform(&mut self, transform: Transform);

    fn is_active(&self) -> bool;
    fn set_active(&mut self, active: bool);
    fn is_active_in_hierarchy(&self) -> bool;
    fn get_layer(&self) -> u32;
    fn set_layer(&mut self, layer: u32);
    fn is_transform_dirty(&self) -> bool;
    fn get_uuid(&self) -> String;
}

pub struct Scene {
    root: Option<Arc<dyn Node>>,
}

impl Scene {
    pub fn new() -> Self {
        Scene { root: None }
    }

    pub fn get_root(&self) -> Option<&Arc<dyn Node>> {
        self.root.as_ref()
    }

    pub fn set_root(&mut self, root: Option<Arc<dyn Node>>) {
        self.root = root;
    }

    pub fn get_scene(&self) -> Option<Arc<dyn Node>> {
        self.root.clone()
    }

    pub fn find_child(&self, uuid: &str) -> Option<Arc<dyn Node>> {
        if let Some(ref root) = self.root {
            Self::find_child_by_uuid(root.as_ref(), uuid)
        } else {
            None
        }
    }

    fn find_child_by_uuid(node: &dyn Node, uuid: &str) -> Option<Arc<dyn Node>> {
        if node.get_uuid() == uuid {
            return None;
        }
        for child in node.get_children() {
            if child.get_uuid() == uuid {
                return Some(child);
            }
            if let Some(found) = Self::find_child_by_uuid(child.as_ref(), uuid) {
                return Some(found);
            }
        }
        None
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NodeComponent {
    pub id: String,
    pub node: Option<Weak<dyn Node>>,
    pub enabled: bool,
}

impl NodeComponent {
    pub fn new(id: &str) -> Self {
        NodeComponent {
            id: id.to_string(),
            node: None,
            enabled: true,
        }
    }

    pub fn on_load(&mut self) {}
    pub fn start(&mut self) {}
    pub fn update(&mut self, _dt: f32) {}
    pub fn late_update(&mut self, _dt: f32) {}
    pub fn on_enable(&mut self) {}
    pub fn on_disable(&mut self) {}
    pub fn on_destroy(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, Vec3::ZERO);
        assert_eq!(transform.rotation, Quaternion::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
        assert_eq!(transform.skew, SkewType::None);
    }

    #[test]
    fn test_transform_custom() {
        let transform = Transform {
            position: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quaternion::new(0.0, 0.0, 0.0, 1.0),
            scale: Vec3::new(2.0, 2.0, 2.0),
            skew: SkewType::Standard,
        };
        assert_eq!(transform.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.scale, Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_scene_new() {
        let scene = Scene::new();
        assert!(scene.get_root().is_none());
    }

    #[test]
    fn test_transform_bit() {
        assert_eq!(TransformBit::None.as_i32(), 0);
        assert_eq!(TransformBit::Position.as_i32(), 1);
        assert_eq!(TransformBit::Rotation.as_i32(), 2);
        assert_eq!(TransformBit::Scale.as_i32(), 4);
        assert_eq!(TransformBit::TRS.as_i32(), 7);
    }

    #[test]
    fn test_transform_bit_or() {
        let result = TransformBit::Position | TransformBit::Rotation;
        assert_eq!(result, 3);
    }

    #[test]
    fn test_node_space() {
        assert_eq!(NodeSpace::Local, NodeSpace::Local);
        assert_ne!(NodeSpace::Local, NodeSpace::World);
    }

    #[test]
    fn test_mobility_mode() {
        assert_eq!(MobilityMode::Static as i32, 0);
        assert_eq!(MobilityMode::Stationary as i32, 1);
        assert_eq!(MobilityMode::Movable as i32, 2);
    }

    #[test]
    fn test_skew_type() {
        assert_eq!(SkewType::None as i32, 0);
        assert_eq!(SkewType::Standard as i32, 1);
        assert_eq!(SkewType::Rotational as i32, 2);
    }

    #[test]
    fn test_node_component() {
        let mut component = NodeComponent::new("test_component");
        assert_eq!(component.id, "test_component");
        assert!(component.enabled);
        
        component.on_load();
        component.start();
        component.update(0.016);
        component.late_update(0.016);
        component.on_enable();
        component.on_disable();
        component.on_destroy();
    }
}
