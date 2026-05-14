use crate::core::geometry::AABB;
use crate::math::Vec3;

const OCTREE_CHILDREN_NUM: usize = 8;
const DEFAULT_OCTREE_DEPTH: u32 = 8;
const DEFAULT_WORLD_MIN: Vec3 = Vec3 {
    x: -1024.0,
    y: -1024.0,
    z: -1024.0,
};
const DEFAULT_WORLD_MAX: Vec3 = Vec3 {
    x: 1024.0,
    y: 1024.0,
    z: 1024.0,
};
const _OCTREE_BOX_EXPAND_SIZE: f32 = 10.0;

#[derive(Debug)]
pub struct BBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        BBox { min, max }
    }

    pub fn from_aabb(aabb: &AABB) -> Self {
        BBox {
            min: aabb.center - aabb.half_extents,
            max: aabb.center + aabb.half_extents,
        }
    }

    pub fn default() -> Self {
        BBox {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }

    pub fn get_center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn contain_point(&self, point: &Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn contain_box(&self, box_: &BBox) -> bool {
        self.min.x <= box_.min.x
            && self.max.x >= box_.max.x
            && self.min.y <= box_.min.y
            && self.max.y >= box_.max.y
            && self.min.z <= box_.min.z
            && self.max.z >= box_.max.z
    }

    pub fn intersect(&self, box_: &BBox) -> bool {
        self.min.x <= box_.max.x
            && self.max.x >= box_.min.x
            && self.min.y <= box_.max.y
            && self.max.y >= box_.min.y
            && self.min.z <= box_.max.z
            && self.max.z >= box_.min.z
    }

    pub fn get_child_box(&self, index: usize) -> BBox {
        let center = self.get_center();
        let mut min = self.min;
        let mut max = center;
        if index & 4 != 0 {
            min.x = center.x;
            max.x = self.max.x;
        }
        if index & 2 != 0 {
            min.y = center.y;
            max.y = self.max.y;
        }
        if index & 1 != 0 {
            min.z = center.z;
            max.z = self.max.z;
        }
        BBox::new(min, max)
    }
}

pub struct OctreeInfo {
    pub enabled: bool,
    pub min_pos: Vec3,
    pub max_pos: Vec3,
    pub depth: u32,
}

impl Default for OctreeInfo {
    fn default() -> Self {
        OctreeInfo {
            enabled: false,
            min_pos: DEFAULT_WORLD_MIN,
            max_pos: DEFAULT_WORLD_MAX,
            depth: DEFAULT_OCTREE_DEPTH,
        }
    }
}

impl OctreeInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_min_pos(&mut self, val: Vec3) {
        self.min_pos = val;
    }

    pub fn get_min_pos(&self) -> &Vec3 {
        &self.min_pos
    }

    pub fn set_max_pos(&mut self, val: Vec3) {
        self.max_pos = val;
    }

    pub fn get_max_pos(&self) -> &Vec3 {
        &self.max_pos
    }

    pub fn set_depth(&mut self, val: u32) {
        self.depth = val;
    }

    pub fn get_depth(&self) -> u32 {
        self.depth
    }

    pub fn activate(&mut self, resource: &mut Octree) {
        resource.enabled = self.enabled;
        resource.min_pos = self.min_pos;
        resource.max_pos = self.max_pos;
        resource.set_max_depth(self.depth);
    }
}

struct OctreeNode {
    children: [Option<Box<OctreeNode>>; OCTREE_CHILDREN_NUM],
    models: Vec<u64>,
    box_: BBox,
    depth: u32,
    index: u32,
}

impl OctreeNode {
    fn new() -> Self {
        OctreeNode {
            children: Default::default(),
            models: Vec::new(),
            box_: BBox::default(),
            depth: 0,
            index: 0,
        }
    }

    fn set_box(&mut self, box_: BBox) {
        self.box_ = box_;
    }

    fn set_depth(&mut self, depth: u32) {
        self.depth = depth;
    }

    fn set_index(&mut self, index: u32) {
        self.index = index;
    }

    fn get_or_create_child(&mut self, index: usize, max_depth: u32) -> &mut OctreeNode {
        if self.children[index].is_none() && self.depth + 1 < max_depth {
            let child_box = self.box_.get_child_box(index);
            let mut child = Box::new(OctreeNode::new());
            child.set_box(child_box);
            child.set_depth(self.depth + 1);
            child.set_index(index as u32);
            self.children[index] = Some(child);
        }
        self.children[index].as_mut().unwrap()
    }

    fn insert(&mut self, model_id: u64, world_bounds: &AABB, max_depth: u32) {
        let model_box = BBox::from_aabb(world_bounds);
        if self.depth + 1 >= max_depth || self.box_.contain_box(&model_box) {
            self.models.push(model_id);
            return;
        }
        let _center = self.box_.get_center();
        for i in 0..OCTREE_CHILDREN_NUM {
            let child_box = self.box_.get_child_box(i);
            if child_box.intersect(&model_box) {
                self.get_or_create_child(i, max_depth)
                    .insert(model_id, world_bounds, max_depth);
            }
        }
    }

    fn remove(&mut self, model_id: u64) {
        self.models.retain(|id| *id != model_id);
        for child in &mut self.children {
            if let Some(c) = child {
                c.remove(model_id);
            }
        }
    }

    fn gather_models(&self, results: &mut Vec<u64>) {
        results.extend(&self.models);
        for child in &self.children {
            if let Some(c) = child {
                c.gather_models(results);
            }
        }
    }
}

pub struct Octree {
    pub enabled: bool,
    root: OctreeNode,
    max_depth: u32,
    total_count: u32,
    pub min_pos: Vec3,
    pub max_pos: Vec3,
}

impl Octree {
    pub fn new() -> Self {
        let mut root = OctreeNode::new();
        root.set_box(BBox::new(DEFAULT_WORLD_MIN, DEFAULT_WORLD_MAX));
        root.set_depth(0);
        Octree {
            enabled: false,
            root,
            max_depth: DEFAULT_OCTREE_DEPTH,
            total_count: 0,
            min_pos: DEFAULT_WORLD_MIN,
            max_pos: DEFAULT_WORLD_MAX,
        }
    }

    pub fn initialize(&mut self, info: &OctreeInfo) {
        self.enabled = info.enabled;
        self.min_pos = info.min_pos;
        self.max_pos = info.max_pos;
        self.max_depth = info.depth;
        self.root.set_box(BBox::new(info.min_pos, info.max_pos));
        self.root.set_depth(0);
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_min_pos(&mut self, val: Vec3) {
        self.min_pos = val;
    }

    pub fn get_min_pos(&self) -> &Vec3 {
        &self.min_pos
    }

    pub fn set_max_pos(&mut self, val: Vec3) {
        self.max_pos = val;
    }

    pub fn get_max_pos(&self) -> &Vec3 {
        &self.max_pos
    }

    pub fn set_max_depth(&mut self, val: u32) {
        self.max_depth = val;
    }

    pub fn get_max_depth(&self) -> u32 {
        self.max_depth
    }

    pub fn insert(&mut self, model_id: u64, world_bounds: &AABB) {
        if !self.enabled {
            return;
        }
        self.root.insert(model_id, world_bounds, self.max_depth);
        self.total_count += 1;
    }

    pub fn remove(&mut self, model_id: u64) {
        if !self.enabled {
            return;
        }
        self.root.remove(model_id);
        self.total_count -= 1;
    }

    pub fn resize(&mut self, min_pos: Vec3, max_pos: Vec3, max_depth: u32) {
        let mut all_models = Vec::new();
        self.root.gather_models(&mut all_models);
        self.min_pos = min_pos;
        self.max_pos = max_pos;
        self.max_depth = max_depth;
        self.root = OctreeNode::new();
        self.root.set_box(BBox::new(min_pos, max_pos));
        self.root.set_depth(0);
    }

    pub fn gather_all_models(&self) -> Vec<u64> {
        let mut results = Vec::new();
        self.root.gather_models(&mut results);
        results
    }
}

impl Default for Octree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octree_new() {
        let octree = Octree::new();
        assert!(!octree.enabled);
        assert_eq!(octree.max_depth, DEFAULT_OCTREE_DEPTH);
    }

    #[test]
    fn test_octree_insert() {
        let mut octree = Octree::new();
        octree.set_enabled(true);
        let aabb = AABB::new(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
        octree.insert(1, &aabb);
        assert_eq!(octree.total_count, 1);
    }

    #[test]
    fn test_octree_remove() {
        let mut octree = Octree::new();
        octree.set_enabled(true);
        let aabb = AABB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        octree.insert(1, &aabb);
        octree.remove(1);
        assert_eq!(octree.total_count, 0);
    }

    #[test]
    fn test_bbox_intersect() {
        let a = BBox::new(Vec3::ZERO, Vec3::ONE);
        let b = BBox::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
        assert!(a.intersect(&b));
    }
}
