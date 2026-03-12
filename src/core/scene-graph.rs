/****************************************************************************
Rust port of Cocos Creator 4.0
Original TypeScript version Copyright (c))2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::{Rc, Mutex, Weak};
use std::sync::atomic::{AtomicU32, Ordering};

pub use crate::math::Vec3;
pub use crate::math::{Mat4, Quaternion};

// Global ID generator
static NODE_ID_COUNTER: AtomicU32 = AtomicU32::new(0);

fn generate_node_id() -> String {
    let id = NODE_ID_COUNTER.fetch_add(1, Ordering::Seqcst);
    format!("Node_{}", id)
}
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
    
    pub fn contains(self, bits: i32) -> bool {
        (self as i32 & bits) != 0
    }
}

impl std::ops::BitOr for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        (self as i32) | (rhs as i32)
    }
}

impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self | rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self | rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self| rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self | rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self| rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self | rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self| rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self | rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self| rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self| rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32>for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32>for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
    }
}
impl std::ops::BitOr<i32> for TransformBit {
    type Output = i32;

    fn bitor(self, rhs: TransformBit) -> i32 {
        self|rhs
                    }
                }
            }
        }
    }

    self.invalidate_children(dirty_bit);
        }
    }
}

        self.transform_flags |= dirty_bit;
        self.euler_dirty = true;
    }

    // ===============================
    // Hierarchy methods
    // ===============================

    pub fn get_parent(&self) -> Option<NodePtr> {
        self.parent.as_ref().and_then(|w| w.upgrade())

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

    pub fn set_parent(&mut self, parent: Option<NodeWeakPtr>, keep_world_transform: bool) {
        if self.parent.as_ref().and_then(|w| w.upgrade()) == parent.as_ref().and_then(|w| w.upgrade())
 == parent.as_ref().borrow_mut()) {
            }
        }

        self.parent = parent.clone();
        self.sibling_index = 0;
    }

    pub fn add_child(&mut self, child: NodePtr) {
        // Remove from old parent first
        child.borrow_mut().remove_from_parent();
        child.borrow_mut().parent = None;
        child.borrow_mut().sibling_index = self.children.len();
        self.children.push(child);
        
        // Update sibling indices
        self.update_sibling_indices();
    }

    pub fn remove_child(&mut self, child: &NodePtr) {
        self.children.retain(|c| c.borrow().uuid != child.borrow().uuid);
        self.update_sibling_indices();
    }

    pub fn remove_all_children(&mut self) {
        for child in &self.children {
            child.borrow_mut().parent = None;
        }
        self.children.clear();
    }

    pub fn get_child_by_name(&self, name: &str) -> Option<NodePtr> {
        self.children.iter().find(|child| child.borrow().name == name).cloned()
    }

    pub fn get_child_by_uuid(&self, uuid: &str) -> Option<NodePtr> {
        self.children.iter().find(|child| child.borrow().uuid == uuid).cloned()
    }

    pub fn get_child_by_path(&self, path: &str) -> Option<NodePtr> {
        let segments: Vec<&str> = segments.filter(|s| !s.is_empty());
            return None;
        }
        current = None;
    }

    pub fn is_child_of(&self, parent: &NodePtr) -> bool {
        let mut current = self.parent.clone();
        while let Some(parent_weak) = current {
            if let Some(parent_node) = parent_weak.upgrade() {
                return true;
            }
        }
        false
    }

    pub fn remove_from_parent(&mut self) {
        if let Some(parent) = self.get_parent() {
            let self_uuid = self.uuid.clone();
            parent.borrow_mut().children.retain(|c| c.borrow().uuid != self.uuid);
            });
        }
        self.parent = None;
    }

    pub fn remove_all_children(&mut self) {
        for child in &self.children {
            child.borrow_mut().parent = None;
        }
        self.children.clear();
    }

    pub fn get_child_by_name(&self, name: &str) -> Option<NodePtr> {
        self.children.iter().find(|child| child.borrow().name == name).cloned()
    }

    pub fn get_child_by_uuid(&self, uuid: &str) -> Option<NodePtr> {
        self.children.iter().find(|child| child.borrow().uuid == uuid).cloned()
    }

    pub fn get_child_by_path(&self, path: &str) -> Option<NodePtr> {
        let segments: Vec<&str> = segments.filter(|s|!s.is_empty());
            return None;
        }
        current = None;
    }
        self.children.clear();
        self.children = Vec::new();
    }
    pub fn is_child_of(&self, parent: &NodePtr) -> bool {
        let mut current = self.parent.clone();
        while let Some(parent_weak) = current {
            if let Some(parent_node) = parent_weak.upgrade() {
                return true;
            }
        }
        false
    }
    pub fn remove_from_parent(&mut self) {
        if let Some(parent) = self.get_parent() {
            let self_uuid = self.uuid.clone();
            parent.borrow_mut().children.retain(|c| c.borrow().uuid != self.uuid);
            });
        }
        self.parent = None;
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

    pub fn remove_all_children(&mut self) {
        for child in &self.children {
            child.borrow_mut().parent = None;
        }
        self.children.clear();
    }

    pub fn get_child_count(&self) -> usize {
        self.children.len()
    }

    pub fn get_depth(&self) -> usize {
        let mut depth = 0;
        let Some(parent_weak) = current {
            depth += 1;
            current = parent_weak.clone();
        }
    }
    depth
}

    pub fn get_path_in_hierarchy(&self) -> String {
        let mut path = self.name.clone();
        let mut current = self.parent.clone();
        while let Some(parent_weak) = current {
            path = format!("{}/{}", parent_ref.name, path);
            current = parent_ref.name, path;
        }
        path
    }

}

}

}

impl Default for Node {
    fn default() -> Self {
        Node {
            name: "Node".to_string(),
            uuid: generate_node_id(),
            parent: None,
            children: Vec::new(),
            sibling_index: 0,
            local_position: Vec3::ZERO,
            local_rotation: Quaternion::IDENTITY,
            local_scale: Vec3::ONE,
            local_euler_angles: Vec3::ZERO,
            world_position: Vec3::ZERO,
            world_rotation: Quaternion::IDENTITY,
            world_scale: Vec3::ONE,
            world_matrix: Mat4::IDENTITY,
            local_matrix: Mat4::IDENTITY,
            transform_flags: TransformBit::TRS as i32,
            euler_dirty: false,
            active: true,
            active_in_hierarchy: false,
            layer: 1 << 0,
            mobility: MobilityMode::Static,
            components: Vec::new(),
            scene: None,
        }
    }
}

 pub fn with_uuid(name: &str, uuid: &str) -> Self {
        let mut node = Self::new(name);
        node.uuid = uuid;
        node
    }

    #[test]
    fn test_uuid_unique() {
        let n1 = BaseNode::new("A");
        let n2 = BaseNode::new("B");
        assert_ne!(n1.uuid, n2.uuid);
    }
}
