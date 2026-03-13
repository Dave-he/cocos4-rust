use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, Ordering};

pub use crate::math::Vec3;
pub use crate::math::{Mat4, Quaternion};

static NODE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn generate_node_id() -> String {
    let id = NODE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("node-{}", id)
}

pub type NodePtr = Arc<Mutex<BaseNode>>;
pub type NodeWeakPtr = Weak<Mutex<BaseNode>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeSpace {
    Local,
    World,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TransformBit {
    None = 0,
    Position = 1 << 0,
    Rotation = 1 << 1,
    Scale = 1 << 2,
    Skew = 1 << 3,
    RS = (1 << 1) | (1 << 2),
    RSS = (1 << 1) | (1 << 2) | (1 << 3),
    TRS = (1 << 0) | (1 << 1) | (1 << 2),
}

impl TransformBit {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn contains(self, bits: u32) -> bool {
        (self as u32 & bits) != 0
    }
}

impl std::ops::BitOr for TransformBit {
    type Output = u32;

    fn bitor(self, rhs: TransformBit) -> u32 {
        (self as u32) | (rhs as u32)
    }
}

impl std::ops::BitOr<u32> for TransformBit {
    type Output = u32;

    fn bitor(self, rhs: u32) -> u32 {
        (self as u32) | rhs
    }
}

impl std::ops::BitAnd for TransformBit {
    type Output = u32;

    fn bitand(self, rhs: TransformBit) -> u32 {
        (self as u32) & (rhs as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobilityMode {
    Static = 0,
    Stationary = 1,
    Movable = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkewType {
    None,
    Skew(f32, f32),
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
    pub mobility: MobilityMode,

    local_position: Vec3,
    local_rotation: Quaternion,
    local_scale: Vec3,
    local_euler_angles: Vec3,
    euler_dirty: bool,

    world_position: Vec3,
    world_rotation: Quaternion,
    world_scale: Vec3,
    world_matrix: Mat4,

    transform_flags: u32,

    parent: Option<NodeWeakPtr>,
    children: Vec<NodePtr>,
    sibling_index: usize,
}

impl BaseNode {
    pub fn new(name: &str) -> Self {
        BaseNode {
            name: name.to_string(),
            uuid: generate_node_id(),
            layer: 1,
            active: true,
            mobility: MobilityMode::Static,
            local_position: Vec3::ZERO,
            local_rotation: Quaternion::IDENTITY,
            local_scale: Vec3::ONE,
            local_euler_angles: Vec3::ZERO,
            euler_dirty: false,
            world_position: Vec3::ZERO,
            world_rotation: Quaternion::IDENTITY,
            world_scale: Vec3::ONE,
            world_matrix: Mat4::IDENTITY,
            transform_flags: TransformBit::TRS as u32,
            parent: None,
            children: Vec::new(),
            sibling_index: 0,
        }
    }

    pub fn with_uuid(name: &str, uuid: &str) -> Self {
        let mut node = Self::new(name);
        node.uuid = uuid.to_string();
        node
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

    pub fn get_parent(&self) -> Option<NodePtr> {
        self.parent.as_ref().and_then(|w| w.upgrade())
    }

    pub fn set_parent_weak(&mut self, parent: Option<NodeWeakPtr>) {
        self.parent = parent;
    }

    pub fn get_children(&self) -> &[NodePtr] {
        &self.children
    }

    pub fn get_child_count(&self) -> usize {
        self.children.len()
    }

    pub fn get_sibling_index(&self) -> usize {
        self.sibling_index
    }

    pub fn add_child(&mut self, child: NodePtr) {
        let idx = self.children.len();
        if let Ok(mut c) = child.lock() {
            c.sibling_index = idx;
        }
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
        self.update_sibling_indices();
    }

    pub fn remove_child_by_uuid(&mut self, uuid: &str) {
        self.children.retain(|c| {
            if let Ok(node) = c.lock() {
                node.uuid != uuid
            } else {
                true
            }
        });
        self.update_sibling_indices();
    }

    pub fn remove_all_children(&mut self) {
        for child in &self.children {
            if let Ok(mut c) = child.lock() {
                c.parent = None;
            }
        }
        self.children.clear();
    }

    fn update_sibling_indices(&mut self) {
        for (i, child) in self.children.iter().enumerate() {
            if let Ok(mut c) = child.lock() {
                c.sibling_index = i;
            }
        }
    }

    pub fn find_child_by_name(&self, name: &str) -> Option<NodePtr> {
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

    pub fn find_child_by_uuid(&self, uuid: &str) -> Option<NodePtr> {
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

    pub fn get_child_by_name(&self, name: &str) -> Option<NodePtr> {
        self.children.iter().find(|c| {
            c.lock().map(|n| n.name == name).unwrap_or(false)
        }).cloned()
    }

    pub fn get_child_by_uuid(&self, uuid: &str) -> Option<NodePtr> {
        self.children.iter().find(|c| {
            c.lock().map(|n| n.uuid == uuid).unwrap_or(false)
        }).cloned()
    }

    pub fn get_child_by_path(&self, path: &str) -> Option<NodePtr> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return None;
        }
        let mut current: Option<NodePtr> = self.get_child_by_name(segments[0]);
        for seg in &segments[1..] {
            match current {
                Some(node) => {
                    let next = node.lock().ok()?.get_child_by_name(seg);
                    current = next;
                }
                None => return None,
            }
        }
        current
    }

    pub fn get_position(&self) -> Vec3 {
        self.local_position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.local_position = position;
        self.invalidate(TransformBit::Position);
    }

    pub fn set_position_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.local_position = Vec3::new(x, y, z);
        self.invalidate(TransformBit::Position);
    }

    pub fn get_rotation(&self) -> Quaternion {
        self.local_rotation
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.local_rotation = rotation;
        self.euler_dirty = true;
        self.invalidate(TransformBit::Rotation);
    }

    pub fn set_rotation_from_euler(&mut self, x: f32, y: f32, z: f32) {
        self.local_euler_angles = Vec3::new(x, y, z);
        self.local_rotation = Quaternion::from_euler(x, y, z);
        self.euler_dirty = false;
        self.invalidate(TransformBit::Rotation);
    }

    pub fn get_euler_angles(&self) -> Vec3 {
        if self.euler_dirty {
            let euler = self.local_rotation.to_euler();
            return Vec3::new(euler.x, euler.y, euler.z);
        }
        self.local_euler_angles
    }

    pub fn get_scale(&self) -> Vec3 {
        self.local_scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.local_scale = scale;
        self.invalidate(TransformBit::Scale);
    }

    pub fn set_scale_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.local_scale = Vec3::new(x, y, z);
        self.invalidate(TransformBit::Scale);
    }

    pub fn get_world_position(&self) -> Vec3 {
        self.world_position
    }

    pub fn set_world_position(&mut self, pos: Vec3) {
        self.world_position = pos;
        if let Some(parent) = self.get_parent() {
            if let Ok(p) = parent.lock() {
                let inv = p.world_matrix.get_inverted();
                self.local_position = self.world_position.transform_mat4(&inv);
            }
        } else {
            self.local_position = pos;
        }
        self.invalidate(TransformBit::Position);
    }

    pub fn get_world_rotation(&self) -> Quaternion {
        self.world_rotation
    }

    pub fn get_world_scale(&self) -> Vec3 {
        self.world_scale
    }

    pub fn get_world_matrix(&self) -> &Mat4 {
        &self.world_matrix
    }

    pub fn get_transform(&self) -> Transform {
        Transform {
            position: self.local_position,
            rotation: self.local_rotation,
            scale: self.local_scale,
            skew: SkewType::None,
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.local_position = transform.position;
        self.local_rotation = transform.rotation;
        self.local_scale = transform.scale;
        self.euler_dirty = true;
        self.invalidate(TransformBit::TRS);
    }

    pub fn update_world_transform(&mut self) {
        if self.transform_flags == TransformBit::None as u32 {
            return;
        }

        if let Some(parent) = self.get_parent() {
            if let Ok(p) = parent.lock() {
                if (self.transform_flags & TransformBit::Position as u32) != 0 {
                    self.world_position = self.local_position.transform_mat4(&p.world_matrix);
                }
                if (self.transform_flags & TransformBit::Rotation as u32) != 0 {
                    self.world_rotation = p.world_rotation * self.local_rotation;
                }
                if (self.transform_flags & TransformBit::Scale as u32) != 0 {
                    self.world_scale = Vec3::new(
                        p.world_scale.x * self.local_scale.x,
                        p.world_scale.y * self.local_scale.y,
                        p.world_scale.z * self.local_scale.z,
                    );
                }
            }
        } else {
            self.world_position = self.local_position;
            self.world_rotation = self.local_rotation;
            self.world_scale = self.local_scale;
        }

        self.world_matrix = Mat4::from_srt(&self.world_rotation, &self.world_position, &self.world_scale);
        self.transform_flags = TransformBit::None as u32;
    }

    fn invalidate(&mut self, dirty_bit: TransformBit) {
        self.transform_flags |= dirty_bit as u32;
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
        self.transform_flags != TransformBit::None as u32
    }

    pub fn clear_dirty(&mut self) {
        self.transform_flags = TransformBit::None as u32;
    }

    pub fn get_mobility(&self) -> MobilityMode {
        self.mobility
    }

    pub fn set_mobility(&mut self, mobility: MobilityMode) {
        self.mobility = mobility;
    }

    pub fn get_depth(&self) -> usize {
        let mut depth = 0;
        let mut current = self.parent.clone();
        while let Some(parent_weak) = current {
            if let Some(parent_node) = parent_weak.upgrade() {
                depth += 1;
                if let Ok(p) = parent_node.lock() {
                    current = p.parent.clone();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        depth
    }

    pub fn get_path_in_hierarchy(&self) -> String {
        let mut parts = vec![self.name.clone()];
        let mut current = self.parent.clone();
        while let Some(parent_weak) = current {
            if let Some(parent_node) = parent_weak.upgrade() {
                if let Ok(p) = parent_node.lock() {
                    parts.push(p.name.clone());
                    current = p.parent.clone();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        parts.reverse();
        parts.join("/")
    }

    pub fn is_child_of(&self, parent: &NodePtr) -> bool {
        let parent_uuid = if let Ok(p) = parent.lock() {
            p.uuid.clone()
        } else {
            return false;
        };
        let mut current = self.parent.clone();
        while let Some(parent_weak) = current {
            if let Some(parent_node) = parent_weak.upgrade() {
                if let Ok(p) = parent_node.lock() {
                    if p.uuid == parent_uuid {
                        return true;
                    }
                    current = p.parent.clone();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        false
    }

    pub fn get_forward(&self) -> Vec3 {
        let r = &self.world_rotation;
        Vec3::new(
            2.0 * (r.x * r.z + r.w * r.y),
            2.0 * (r.y * r.z - r.w * r.x),
            1.0 - 2.0 * (r.x * r.x + r.y * r.y),
        ).get_normalized()
    }

    pub fn get_up(&self) -> Vec3 {
        let r = &self.world_rotation;
        Vec3::new(
            2.0 * (r.x * r.y - r.w * r.z),
            1.0 - 2.0 * (r.x * r.x + r.z * r.z),
            2.0 * (r.y * r.z + r.w * r.x),
        ).get_normalized()
    }

    pub fn get_right(&self) -> Vec3 {
        let r = &self.world_rotation;
        Vec3::new(
            1.0 - 2.0 * (r.y * r.y + r.z * r.z),
            2.0 * (r.x * r.y + r.w * r.z),
            2.0 * (r.x * r.z - r.w * r.y),
        ).get_normalized()
    }
}

impl Default for BaseNode {
    fn default() -> Self {
        Self::new("Node")
    }
}

pub struct Scene {
    pub name: String,
    root: Option<NodePtr>,
    pub auto_release_assets: bool,
}

impl Scene {
    pub fn new(name: &str) -> Self {
        Scene {
            name: name.to_string(),
            root: None,
            auto_release_assets: false,
        }
    }

    pub fn get_root(&self) -> Option<&NodePtr> {
        self.root.as_ref()
    }

    pub fn set_root(&mut self, root: Option<NodePtr>) {
        self.root = root;
    }

    pub fn find_child(&self, uuid: &str) -> Option<NodePtr> {
        if let Some(ref root) = self.root {
            if let Ok(r) = root.lock() {
                return r.find_child_by_uuid(uuid);
            }
        }
        None
    }

    pub fn find_child_by_name(&self, name: &str) -> Option<NodePtr> {
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
    execution_order: i32,
}

impl NodeComponent {
    pub fn new(id: &str) -> Self {
        NodeComponent {
            id: id.to_string(),
            node: None,
            enabled: true,
            execution_order: 0,
        }
    }

    pub fn get_execution_order(&self) -> i32 {
        self.execution_order
    }

    pub fn set_execution_order(&mut self, order: i32) {
        self.execution_order = order;
    }

    pub fn get_node(&self) -> Option<NodePtr> {
        self.node.as_ref().and_then(|w| w.upgrade())
    }

    pub fn on_load(&mut self) {}
    pub fn start(&mut self) {}
    pub fn update(&mut self, _dt: f32) {}
    pub fn late_update(&mut self, _dt: f32) {}
    pub fn on_enable(&mut self) {}
    pub fn on_disable(&mut self) {}
    pub fn on_destroy(&mut self) {}
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeEventType {
    TransformChanged(u32),
    ParentChanged,
    ChildAdded(String),
    ChildRemoved(String),
    ActiveChanged(bool),
    MobilityChanged(MobilityMode),
}

pub type NodeEventListener = Box<dyn Fn(&NodeEventType) + Send + Sync>;

pub struct NodeEventEmitter {
    listeners: Vec<NodeEventListener>,
}

impl NodeEventEmitter {
    pub fn new() -> Self {
        NodeEventEmitter { listeners: Vec::new() }
    }

    pub fn on(&mut self, listener: NodeEventListener) {
        self.listeners.push(listener);
    }

    pub fn emit(&self, event: &NodeEventType) {
        for listener in &self.listeners {
            listener(event);
        }
    }

    pub fn clear(&mut self) {
        self.listeners.clear();
    }
}

impl Default for NodeEventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_node_new() {
        let node = BaseNode::new("TestNode");
        assert_eq!(node.get_name(), "TestNode");
        assert!(node.is_active());
        assert_eq!(node.get_layer(), 1);
        assert!(node.is_transform_dirty());
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
    fn test_uuid_unique() {
        let n1 = BaseNode::new("A");
        let n2 = BaseNode::new("B");
        assert_ne!(n1.uuid, n2.uuid);
    }

    #[test]
    fn test_mobility() {
        let mut node = BaseNode::new("Node");
        assert_eq!(node.get_mobility(), MobilityMode::Static);
        node.set_mobility(MobilityMode::Movable);
        assert_eq!(node.get_mobility(), MobilityMode::Movable);
    }

    #[test]
    fn test_get_child_by_path() {
        let mut root = BaseNode::new("Root");
        let child_a = Arc::new(Mutex::new(BaseNode::new("A")));
        let child_b = Arc::new(Mutex::new(BaseNode::new("B")));
        child_a.lock().unwrap().add_child(Arc::clone(&child_b));
        root.add_child(Arc::clone(&child_a));

        let found = root.get_child_by_path("A/B");
        assert!(found.is_some());
        assert_eq!(found.unwrap().lock().unwrap().get_name(), "B");
    }

    #[test]
    fn test_transform_bit_operations() {
        let pos = TransformBit::Position;
        let rot = TransformBit::Rotation;
        let combined = pos | rot;
        assert_eq!(combined, 3);
        assert!(pos.contains(1));
        assert!(!pos.contains(2));
    }

    #[test]
    fn test_world_transform_no_parent() {
        let mut node = BaseNode::new("Node");
        node.set_position_xyz(10.0, 20.0, 30.0);
        node.update_world_transform();
        assert_eq!(node.get_world_position(), Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_depth() {
        let root = Arc::new(Mutex::new(BaseNode::new("Root")));
        let child = Arc::new(Mutex::new(BaseNode::new("Child")));
        child.lock().unwrap().set_parent_weak(Some(Arc::downgrade(&root)));
        root.lock().unwrap().add_child(Arc::clone(&child));

        assert_eq!(root.lock().unwrap().get_depth(), 0);
        assert_eq!(child.lock().unwrap().get_depth(), 1);
    }

    #[test]
    fn test_path_in_hierarchy() {
        let root = Arc::new(Mutex::new(BaseNode::new("Root")));
        let child = Arc::new(Mutex::new(BaseNode::new("Child")));
        child.lock().unwrap().set_parent_weak(Some(Arc::downgrade(&root)));
        root.lock().unwrap().add_child(Arc::clone(&child));

        assert_eq!(child.lock().unwrap().get_path_in_hierarchy(), "Root/Child");
    }

    #[test]
    fn test_node_component() {
        let mut comp = NodeComponent::new("TestComp");
        assert_eq!(comp.id, "TestComp");
        assert!(comp.enabled);
        assert!(comp.get_node().is_none());

        comp.set_execution_order(5);
        assert_eq!(comp.get_execution_order(), 5);
    }

    #[test]
    fn test_node_event_emitter() {
        let results = Arc::new(Mutex::new(Vec::<String>::new()));
        let mut emitter = NodeEventEmitter::new();

        let r = Arc::clone(&results);
        emitter.on(Box::new(move |e| {
            r.lock().unwrap().push(format!("{:?}", e));
        }));

        emitter.emit(&NodeEventType::ActiveChanged(true));
        emitter.emit(&NodeEventType::ParentChanged);

        let r = results.lock().unwrap();
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_node_event_emitter_clear() {
        let count = Arc::new(Mutex::new(0usize));
        let mut emitter = NodeEventEmitter::new();
        let c = Arc::clone(&count);
        emitter.on(Box::new(move |_| { *c.lock().unwrap() += 1; }));
        emitter.emit(&NodeEventType::ParentChanged);
        assert_eq!(*count.lock().unwrap(), 1);
        emitter.clear();
        emitter.emit(&NodeEventType::ParentChanged);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_node_event_types() {
        let e1 = NodeEventType::TransformChanged(3);
        let e2 = NodeEventType::ChildAdded("child-uuid".to_string());
        let e3 = NodeEventType::MobilityChanged(MobilityMode::Movable);
        assert_ne!(format!("{:?}", e1), format!("{:?}", e2));
        assert_eq!(e3, NodeEventType::MobilityChanged(MobilityMode::Movable));
    }
}
