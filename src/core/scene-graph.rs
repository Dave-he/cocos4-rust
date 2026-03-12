/****************************************************************************
Rust port of Cocos Creator Scene Graph system
Original C++ version Copyright (c) 2022-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::sync::{Arc, Mutex, Weak};

pub use crate::math::Vec3;
pub use crate::math::{Mat4, Quaternion};

pub type NodePtr = Arc<Mutex<BaseNode>>;
pub type NodeWeakPtr = Weak<Mutex<BaseNode>>;

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

#[derive(Debug)]
pub struct BaseNode {
    pub name: String,
    pub uuid: String,
    pub layer: u32,
    pub active: bool,
    pub transform: Transform,
    pub transform_dirty: bool,
    pub mobility: MobilityMode,
    pub world_matrix: Mat4,
    parent: Option<Weak<Mutex<BaseNode>>>,
    children: Vec<Arc<Mutex<BaseNode>>>,
}

impl BaseNode {
    pub fn new(name: &str) -> Self {
        BaseNode {
            name: name.to_string(),
            uuid: Self::generate_uuid(),
            layer: 0,
            active: true,
            transform: Transform::default(),
            transform_dirty: false,
            mobility: MobilityMode::Movable,
            world_matrix: Mat4::IDENTITY,
            parent: None,
            children: Vec::new(),
        }
    }

    fn generate_uuid() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        format!("node-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn get_parent(&self) -> Option<Arc<Mutex<BaseNode>>> {
        self.parent.as_ref().and_then(|w| w.upgrade())
    }

    pub fn set_parent(&mut self, parent: Option<Weak<Mutex<BaseNode>>>) {
        self.parent = parent;
    }

    pub fn get_children(&self) -> &Vec<Arc<Mutex<BaseNode>>> {
        &self.children
    }

    pub fn add_child(&mut self, child: Arc<Mutex<BaseNode>>) {
        self.children.push(child);
    }

    pub fn remove_child_by_name(&mut self, name: &str) {
        self.children.retain(|c| {
            if let Ok(node) = c.lock() {
                node.name != name
            } else {
                true
            }
        });
    }

    pub fn remove_child_by_uuid(&mut self, uuid: &str) {
        self.children.retain(|c| {
            if let Ok(node) = c.lock() {
                node.uuid != uuid
            } else {
                true
            }
        });
    }

    pub fn get_child_count(&self) -> usize {
        self.children.len()
    }

    pub fn get_position(&self) -> Vec3 {
        self.transform.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.transform.position = position;
        self.transform_dirty = true;
    }

    pub fn set_position_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.transform.position = Vec3::new(x, y, z);
        self.transform_dirty = true;
    }

    pub fn get_rotation(&self) -> Quaternion {
        self.transform.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.transform.rotation = rotation;
        self.transform_dirty = true;
    }

    pub fn get_scale(&self) -> Vec3 {
        self.transform.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.transform.scale = scale;
        self.transform_dirty = true;
    }

    pub fn set_scale_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.transform.scale = Vec3::new(x, y, z);
        self.transform_dirty = true;
    }

    pub fn get_transform(&self) -> Transform {
        self.transform
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
        self.transform_dirty = true;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn is_active_in_hierarchy(&self) -> bool {
        if !self.active {
            return false;
        }
        if let Some(parent) = self.get_parent() {
            if let Ok(p) = parent.lock() {
                return p.is_active_in_hierarchy();
            }
        }
        true
    }

    pub fn get_layer(&self) -> u32 {
        self.layer
    }

    pub fn set_layer(&mut self, layer: u32) {
        self.layer = layer;
    }

    pub fn is_transform_dirty(&self) -> bool {
        self.transform_dirty
    }

    pub fn clear_dirty(&mut self) {
        self.transform_dirty = false;
    }

    pub fn get_mobility(&self) -> MobilityMode {
        self.mobility
    }

    pub fn set_mobility(&mut self, mobility: MobilityMode) {
        self.mobility = mobility;
    }

    pub fn get_world_matrix(&self) -> &Mat4 {
        &self.world_matrix
    }

    pub fn update_world_matrix(&mut self) {
        let t = &self.transform;
        self.world_matrix = Mat4::from_srt(&t.rotation, &t.position, &t.scale);
        self.transform_dirty = false;
    }

    pub fn find_child_by_name(&self, name: &str) -> Option<Arc<Mutex<BaseNode>>> {
        for child in &self.children {
            if let Ok(c) = child.lock() {
                if c.name == name {
                    return Some(Arc::clone(child));
                }
                if let Some(found) = c.find_child_by_name(name) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn find_child_by_uuid(&self, uuid: &str) -> Option<Arc<Mutex<BaseNode>>> {
        for child in &self.children {
            if let Ok(c) = child.lock() {
                if c.uuid == uuid {
                    return Some(Arc::clone(child));
                }
                if let Some(found) = c.find_child_by_uuid(uuid) {
                    return Some(found);
                }
            }
        }
        None
    }
}

impl Default for BaseNode {
    fn default() -> Self {
        Self::new("Node")
    }
}

pub struct Scene {
    pub name: String,
    root: Option<Arc<Mutex<BaseNode>>>,
}

impl Scene {
    pub fn new(name: &str) -> Self {
        Scene {
            name: name.to_string(),
            root: None,
        }
    }

    pub fn get_root(&self) -> Option<&Arc<Mutex<BaseNode>>> {
        self.root.as_ref()
    }

    pub fn set_root(&mut self, root: Option<Arc<Mutex<BaseNode>>>) {
        self.root = root;
    }

    pub fn find_child(&self, uuid: &str) -> Option<Arc<Mutex<BaseNode>>> {
        if let Some(ref root) = self.root {
            if let Ok(r) = root.lock() {
                return r.find_child_by_uuid(uuid);
            }
        }
        None
    }

    pub fn find_child_by_name(&self, name: &str) -> Option<Arc<Mutex<BaseNode>>> {
        if let Some(ref root) = self.root {
            if let Ok(r) = root.lock() {
                return r.find_child_by_name(name);
            }
        }
        None
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Scene")
    }
}

pub struct NodeComponent {
    pub id: String,
    pub node: Option<NodeWeakPtr>,
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
    fn test_base_node_new() {
        let node = BaseNode::new("TestNode");
        assert_eq!(node.get_name(), "TestNode");
        assert!(node.is_active());
        assert_eq!(node.get_layer(), 0);
        assert!(!node.is_transform_dirty());
        assert_eq!(node.get_child_count(), 0);
    }

    #[test]
    fn test_base_node_position() {
        let mut node = BaseNode::new("Node");
        node.set_position_xyz(1.0, 2.0, 3.0);
        assert_eq!(node.get_position(), Vec3::new(1.0, 2.0, 3.0));
        assert!(node.is_transform_dirty());
        node.clear_dirty();
        assert!(!node.is_transform_dirty());
    }

    #[test]
    fn test_base_node_scale() {
        let mut node = BaseNode::new("Node");
        node.set_scale_xyz(2.0, 2.0, 2.0);
        assert_eq!(node.get_scale(), Vec3::new(2.0, 2.0, 2.0));
        assert!(node.is_transform_dirty());
    }

    #[test]
    fn test_base_node_active() {
        let mut node = BaseNode::new("Node");
        assert!(node.is_active());
        node.set_active(false);
        assert!(!node.is_active());
        assert!(!node.is_active_in_hierarchy());
    }

    #[test]
    fn test_base_node_children() {
        let mut parent = BaseNode::new("Parent");
        let child = Arc::new(Mutex::new(BaseNode::new("Child")));
        parent.add_child(Arc::clone(&child));
        assert_eq!(parent.get_child_count(), 1);
        parent.remove_child_by_name("Child");
        assert_eq!(parent.get_child_count(), 0);
    }

    #[test]
    fn test_base_node_find_child() {
        let mut root = BaseNode::new("Root");
        let child1 = Arc::new(Mutex::new(BaseNode::new("Child1")));
        let child2 = Arc::new(Mutex::new(BaseNode::new("Child2")));
        root.add_child(Arc::clone(&child1));
        root.add_child(Arc::clone(&child2));
        let found = root.find_child_by_name("Child1");
        assert!(found.is_some());
        if let Some(n) = found {
            assert_eq!(n.lock().unwrap().get_name(), "Child1");
        }
    }

    #[test]
    fn test_scene_new() {
        let scene = Scene::new("TestScene");
        assert_eq!(scene.name, "TestScene");
        assert!(scene.get_root().is_none());
    }

    #[test]
    fn test_scene_set_root() {
        let mut scene = Scene::new("Scene");
        let root = Arc::new(Mutex::new(BaseNode::new("Root")));
        scene.set_root(Some(root));
        assert!(scene.get_root().is_some());
    }

    #[test]
    fn test_transform_default() {
        let transform = Transform::default();
        assert_eq!(transform.position, Vec3::ZERO);
        assert_eq!(transform.rotation, Quaternion::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
        assert_eq!(transform.skew, SkewType::None);
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

    #[test]
    fn test_uuid_unique() {
        let n1 = BaseNode::new("A");
        let n2 = BaseNode::new("B");
        assert_ne!(n1.uuid, n2.uuid);
    }
}
