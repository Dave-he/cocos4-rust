use std::sync::{Arc, Mutex, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub use crate::math::Vec3;
pub use crate::math::{Mat4, Quaternion};

pub trait Component: Any + Send + Sync {
    fn get_type_id(&self) -> TypeId;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn on_load(&mut self) {}
    fn start(&mut self) {}
    fn update(&mut self, _dt: f32) {}
    fn late_update(&mut self, _dt: f32) {}
    fn on_enable(&mut self) {}
    fn on_disable(&mut self) {}
    fn on_destroy(&mut self) {}
}

pub type ComponentPtr = Box<dyn Component>;

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

    components: HashMap<TypeId, ComponentPtr>,
    event_emitter: NodeEventEmitter,
}

impl std::fmt::Debug for BaseNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseNode")
            .field("name", &self.name)
            .field("uuid", &self.uuid)
            .field("layer", &self.layer)
            .field("active", &self.active)
            .field("mobility", &self.mobility)
            .field("local_position", &self.local_position)
            .field("local_rotation", &self.local_rotation)
            .field("local_scale", &self.local_scale)
            .field("transform_flags", &self.transform_flags)
            .field("child_count", &self.children.len())
            .field("component_count", &self.components.len())
            .finish()
    }
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
            components: HashMap::new(),
            event_emitter: NodeEventEmitter::new(),
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
        self.event_emitter.emit(&NodeEventType::TransformChanged(TransformBit::Position as u32));
    }

    pub fn set_position_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.local_position = Vec3::new(x, y, z);
        self.invalidate(TransformBit::Position);
        self.event_emitter.emit(&NodeEventType::TransformChanged(TransformBit::Position as u32));
    }

    pub fn get_rotation(&self) -> Quaternion {
        self.local_rotation
    }

    pub fn set_rotation(&mut self, rotation: Quaternion) {
        self.local_rotation = rotation;
        self.euler_dirty = true;
        self.invalidate(TransformBit::Rotation);
        self.event_emitter.emit(&NodeEventType::TransformChanged(TransformBit::Rotation as u32));
    }

    pub fn set_rotation_from_euler(&mut self, x: f32, y: f32, z: f32) {
        self.local_euler_angles = Vec3::new(x, y, z);
        self.local_rotation = Quaternion::from_euler(x, y, z);
        self.euler_dirty = false;
        self.invalidate(TransformBit::Rotation);
        self.event_emitter.emit(&NodeEventType::TransformChanged(TransformBit::Rotation as u32));
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
        self.event_emitter.emit(&NodeEventType::TransformChanged(TransformBit::Scale as u32));
    }

    pub fn set_scale_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.local_scale = Vec3::new(x, y, z);
        self.invalidate(TransformBit::Scale);
        self.event_emitter.emit(&NodeEventType::TransformChanged(TransformBit::Scale as u32));
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

    pub fn set_world_rotation(&mut self, rot: Quaternion) {
        self.world_rotation = rot;
        if let Some(parent) = self.get_parent() {
            if let Ok(p) = parent.lock() {
                let parent_inv = p.world_rotation.inverse();
                self.local_rotation = parent_inv * rot;
            }
        } else {
            self.local_rotation = rot;
        }
        self.euler_dirty = true;
        self.invalidate(TransformBit::Rotation);
    }

    pub fn get_world_scale(&self) -> Vec3 {
        self.world_scale
    }

    pub fn set_world_scale(&mut self, scale: Vec3) {
        self.world_scale = scale;
        if let Some(parent) = self.get_parent() {
            if let Ok(p) = parent.lock() {
                let px = p.world_scale.x;
                let py = p.world_scale.y;
                let pz = p.world_scale.z;
                self.local_scale = Vec3::new(
                    if px.abs() > 1e-6 { scale.x / px } else { scale.x },
                    if py.abs() > 1e-6 { scale.y / py } else { scale.y },
                    if pz.abs() > 1e-6 { scale.z / pz } else { scale.z },
                );
            }
        } else {
            self.local_scale = scale;
        }
        self.invalidate(TransformBit::Scale);
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
        self.event_emitter.emit(&NodeEventType::ActiveChanged(active));
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
        self.event_emitter.emit(&NodeEventType::MobilityChanged(mobility));
    }

    pub fn add_component<C: Component + 'static>(&mut self, component: C) {
        let type_id = TypeId::of::<C>();
        self.components.insert(type_id, Box::new(component));
    }

    pub fn get_component<C: Component + 'static>(&self) -> Option<&C> {
        let type_id = TypeId::of::<C>();
        self.components.get(&type_id)?.as_any().downcast_ref::<C>()
    }

    pub fn get_component_mut<C: Component + 'static>(&mut self) -> Option<&mut C> {
        let type_id = TypeId::of::<C>();
        self.components.get_mut(&type_id)?.as_any_mut().downcast_mut::<C>()
    }

    pub fn remove_component<C: Component + 'static>(&mut self) -> Option<ComponentPtr> {
        let type_id = TypeId::of::<C>();
        self.components.remove(&type_id)
    }

    pub fn has_component<C: Component + 'static>(&self) -> bool {
        let type_id = TypeId::of::<C>();
        self.components.contains_key(&type_id)
    }

    pub fn get_component_count(&self) -> usize {
        self.components.len()
    }

    pub fn on_event(&mut self, listener: NodeEventListener) {
        self.event_emitter.on(listener);
    }

    pub fn emit_event(&self, event: &NodeEventType) {
        self.event_emitter.emit(event);
    }

    pub fn clear_event_listeners(&mut self) {
        self.event_emitter.clear();
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

    /// Depth-first traversal of the node hierarchy.
    /// The visitor receives each node as a `&BaseNode`.
    /// Returns early if the visitor returns `false`.
    pub fn visit_nodes<F>(&self, visitor: &mut F) -> bool
    where
        F: FnMut(&BaseNode) -> bool,
    {
        if !visitor(self) {
            return false;
        }
        for child in &self.children {
            if let Ok(c) = child.lock() {
                if !c.visit_nodes(visitor) {
                    return false;
                }
            }
        }
        true
    }

    /// Collect UUIDs of all nodes in the subtree (including self).
    pub fn collect_uuids(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.visit_nodes(&mut |n| {
            result.push(n.uuid.clone());
            true
        });
        result
    }

    /// Find the first node in the subtree satisfying `predicate`.
    pub fn find_node<F>(&self, predicate: F) -> Option<NodePtr>
    where
        F: Fn(&BaseNode) -> bool,
    {
        self.find_node_ref(&predicate)
    }

    fn find_node_ref<F>(&self, predicate: &F) -> Option<NodePtr>
    where
        F: Fn(&BaseNode) -> bool,
    {
        for child in &self.children {
            if let Ok(c) = child.lock() {
                if predicate(&c) {
                    return Some(Arc::clone(child));
                }
                if let Some(found) = c.find_node_ref(predicate) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Collect all NodePtrs in subtree (children only, not self) matching predicate.
    pub fn find_nodes_matching<F>(&self, predicate: &F) -> Vec<NodePtr>
    where
        F: Fn(&BaseNode) -> bool,
    {
        let mut result = Vec::new();
        for child in &self.children {
            if let Ok(c) = child.lock() {
                if predicate(&c) {
                    result.push(Arc::clone(child));
                }
                let sub = c.find_nodes_matching(predicate);
                result.extend(sub);
            }
        }
        result
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
    pub globals: HashMap<String, String>,
}

impl Scene {
    pub fn new(name: &str) -> Self {
        Scene {
            name: name.to_string(),
            root: None,
            auto_release_assets: false,
            globals: HashMap::new(),
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

    pub fn set_global(&mut self, key: &str, value: &str) {
        self.globals.insert(key.to_string(), value.to_string());
    }

    pub fn get_global(&self, key: &str) -> Option<&str> {
        self.globals.get(key).map(|s| s.as_str())
    }

    pub fn get_node_count(&self) -> usize {
        if let Some(ref root) = self.root {
            if let Ok(r) = root.lock() {
                return 1 + count_descendants(&r);
            }
        }
        0
    }

    /// Visit every node in the scene using DFS. Stops early if visitor returns false.
    pub fn for_each_node<F>(&self, mut visitor: F)
    where
        F: FnMut(&BaseNode) -> bool,
    {
        if let Some(ref root) = self.root {
            if let Ok(r) = root.lock() {
                r.visit_nodes(&mut visitor);
            }
        }
    }

    /// Collect all nodes that match a predicate.
    pub fn find_nodes<F>(&self, predicate: F) -> Vec<NodePtr>
    where
        F: Fn(&BaseNode) -> bool,
    {
        let mut result = Vec::new();
        if let Some(ref root) = self.root {
            if let Ok(r) = root.lock() {
                if predicate(&r) {
                    result.push(Arc::clone(root));
                }
                let children_results = r.find_nodes_matching(&predicate);
                result.extend(children_results);
            }
        }
        result
    }
}

fn count_descendants(node: &BaseNode) -> usize {
    let mut count = 0;
    for child in node.get_children() {
        count += 1;
        if let Ok(c) = child.lock() {
            count += count_descendants(&c);
        }
    }
    count
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

impl std::fmt::Debug for NodeEventEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeEventEmitter")
            .field("listener_count", &self.listeners.len())
            .finish()
    }
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

    struct TestComponent {
        pub value: i32,
    }

    impl Component for TestComponent {
        fn get_type_id(&self) -> TypeId {
            TypeId::of::<TestComponent>()
        }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_base_node_add_get_component() {
        let mut node = BaseNode::new("Node");
        node.add_component(TestComponent { value: 42 });
        assert!(node.has_component::<TestComponent>());
        let comp = node.get_component::<TestComponent>();
        assert!(comp.is_some());
        assert_eq!(comp.unwrap().value, 42);
        assert_eq!(node.get_component_count(), 1);
    }

    #[test]
    fn test_base_node_remove_component() {
        let mut node = BaseNode::new("Node");
        node.add_component(TestComponent { value: 10 });
        assert!(node.has_component::<TestComponent>());
        let removed = node.remove_component::<TestComponent>();
        assert!(removed.is_some());
        assert!(!node.has_component::<TestComponent>());
        assert_eq!(node.get_component_count(), 0);
    }

    #[test]
    fn test_base_node_get_component_mut() {
        let mut node = BaseNode::new("Node");
        node.add_component(TestComponent { value: 5 });
        {
            let comp = node.get_component_mut::<TestComponent>().unwrap();
            comp.value = 99;
        }
        assert_eq!(node.get_component::<TestComponent>().unwrap().value, 99);
    }

    #[test]
    fn test_base_node_event_emitter_integration() {
        let mut node = BaseNode::new("Node");
        let changed = Arc::new(Mutex::new(false));
        let changed_clone = Arc::clone(&changed);
        node.on_event(Box::new(move |e| {
            if let NodeEventType::TransformChanged(_) = e {
                *changed_clone.lock().unwrap() = true;
            }
        }));
        node.set_position_xyz(1.0, 2.0, 3.0);
        assert!(*changed.lock().unwrap());
    }

    #[test]
    fn test_base_node_set_world_rotation() {
        let mut node = BaseNode::new("Node");
        let rot = Quaternion::from_axis_angle(&Vec3::UNIT_Y, std::f32::consts::FRAC_PI_2);
        node.set_world_rotation(rot);
        let wr = node.get_world_rotation();
        assert!((wr.x - rot.x).abs() < 1e-5);
        assert!((wr.w - rot.w).abs() < 1e-5);
    }

    #[test]
    fn test_base_node_set_world_scale() {
        let mut node = BaseNode::new("Node");
        node.set_world_scale(Vec3::new(2.0, 3.0, 4.0));
        let ws = node.get_world_scale();
        assert!((ws.x - 2.0).abs() < 1e-5);
        assert!((ws.y - 3.0).abs() < 1e-5);
        assert!((ws.z - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_scene_globals() {
        let mut scene = Scene::new("TestScene");
        scene.set_global("gravity", "-9.8");
        assert_eq!(scene.get_global("gravity"), Some("-9.8"));
        assert_eq!(scene.get_global("nonexistent"), None);
    }

    #[test]
    fn test_scene_node_count() {
        let mut scene = Scene::new("Scene");
        let root = Arc::new(Mutex::new(BaseNode::new("Root")));
        let child = Arc::new(Mutex::new(BaseNode::new("Child")));
        root.lock().unwrap().add_child(Arc::clone(&child));
        scene.set_root(Some(root));
        assert_eq!(scene.get_node_count(), 2);
    }

    #[test]
    fn test_visit_nodes() {
        let mut root = BaseNode::new("Root");
        let child_a = Arc::new(Mutex::new(BaseNode::new("A")));
        let child_b = Arc::new(Mutex::new(BaseNode::new("B")));
        root.add_child(Arc::clone(&child_a));
        root.add_child(Arc::clone(&child_b));

        let mut visited = Vec::new();
        root.visit_nodes(&mut |n| {
            visited.push(n.name.clone());
            true
        });
        // Root is visited first, then children DFS
        assert_eq!(visited, vec!["Root", "A", "B"]);
    }

    #[test]
    fn test_visit_nodes_early_exit() {
        let mut root = BaseNode::new("Root");
        let child_a = Arc::new(Mutex::new(BaseNode::new("A")));
        let child_b = Arc::new(Mutex::new(BaseNode::new("B")));
        root.add_child(Arc::clone(&child_a));
        root.add_child(Arc::clone(&child_b));

        let mut count = 0;
        root.visit_nodes(&mut |_| {
            count += 1;
            count < 2 // stop after 2 visits
        });
        assert_eq!(count, 2);
    }

    #[test]
    fn test_collect_uuids() {
        let mut root = BaseNode::new("Root");
        let child = Arc::new(Mutex::new(BaseNode::new("Child")));
        root.add_child(Arc::clone(&child));

        let uuids = root.collect_uuids();
        assert_eq!(uuids.len(), 2);
        assert!(uuids.contains(&root.uuid));
    }

    #[test]
    fn test_find_node_predicate() {
        let mut root = BaseNode::new("Root");
        let child_a = Arc::new(Mutex::new(BaseNode::new("A")));
        let child_b = Arc::new(Mutex::new(BaseNode::new("B")));
        root.add_child(Arc::clone(&child_a));
        root.add_child(Arc::clone(&child_b));

        let found = root.find_node(|n| n.name == "B");
        assert!(found.is_some());
        assert_eq!(found.unwrap().lock().unwrap().name, "B");

        let not_found = root.find_node(|n| n.name == "C");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_scene_for_each_node() {
        let mut scene = Scene::new("Scene");
        let root = Arc::new(Mutex::new(BaseNode::new("Root")));
        let child_a = Arc::new(Mutex::new(BaseNode::new("A")));
        let child_b = Arc::new(Mutex::new(BaseNode::new("B")));
        root.lock().unwrap().add_child(Arc::clone(&child_a));
        root.lock().unwrap().add_child(Arc::clone(&child_b));
        scene.set_root(Some(root));

        let mut names = Vec::new();
        scene.for_each_node(|n| {
            names.push(n.name.clone());
            true
        });
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"Root".to_string()));
        assert!(names.contains(&"A".to_string()));
        assert!(names.contains(&"B".to_string()));
    }

    #[test]
    fn test_scene_find_nodes() {
        let mut scene = Scene::new("Scene");
        let root = Arc::new(Mutex::new(BaseNode::new("Root")));
        root.lock().unwrap().layer = 1;
        let child_a = Arc::new(Mutex::new(BaseNode::new("A")));
        child_a.lock().unwrap().layer = 2;
        let child_b = Arc::new(Mutex::new(BaseNode::new("B")));
        child_b.lock().unwrap().layer = 1;
        root.lock().unwrap().add_child(Arc::clone(&child_a));
        root.lock().unwrap().add_child(Arc::clone(&child_b));
        scene.set_root(Some(root));

        let layer1_nodes = scene.find_nodes(|n| n.layer == 1);
        assert_eq!(layer1_nodes.len(), 2); // Root + B

        let layer2_nodes = scene.find_nodes(|n| n.layer == 2);
        assert_eq!(layer2_nodes.len(), 1); // A
    }
}
