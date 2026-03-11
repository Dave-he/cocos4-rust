/****************************************************************************
Rust port of Cocos Creator Scene Graph system
Original C++ version Copyright (c) 2022-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::sync::{Arc, RwLock, Weak};

pub use crate::math::Vec3;
pub use crate::math::{Mat4, Quaternion};

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
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        TransformBit(self as i32 | rhs as i32)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestNode {
        name: String,
        parent: Option<Weak<TestNode>>,
        children: Vec<Arc<TestNode>>,
        transform: Transform,
        active: bool,
        active_in_hierarchy: bool,
        layer: u32,
    }

    impl TestNode {
        pub fn new(name: &str) -> Self {
            TestNode {
                name: name.to_string(),
                parent: None,
                children: Vec::new(),
                transform: Transform::default(),
                active: true,
                active_in_hierarchy: true,
                layer: 0,
            }
        }
    }

    impl Node for TestNode {
        fn get_name(&self) -> &str {
            &self.name
        }

        fn get_parent(&self) -> Option<Weak<dyn Node>> {
            self.parent.clone()
        }

        fn set_parent(&mut self, parent: Option<Weak<dyn Node>>) {
            self.parent = parent;
        }

        fn get_children(&self) -> Vec<Arc<dyn Node>> {
            self.children.clone()
        }

        fn add_child(&mut self, child: Arc<dyn Node>) {
            self.children.push(Arc::clone(&child));
        }

        fn remove_child(&mut self, child: &Arc<dyn Node>) {
            self.children.retain(|c| Arc::ptr_eq(c, child));
        }

        fn set_position(&mut self, position: Vec3) {
            self.transform.position = position;
        }

        fn get_position(&self) -> Vec3 {
            self.transform.position
        }

        fn set_rotation(&mut self, rotation: Quaternion) {
            self.transform.rotation = rotation;
        }

        fn get_rotation(&self) -> Quaternion {
            self.transform.rotation
        }

        fn set_scale(&mut self, scale: Vec3) {
            self.transform.scale = scale;
        }

        fn get_scale(&self) -> Vec3 {
            self.transform.scale
        }

        fn get_transform(&self) -> Transform {
            self.transform.clone()
        }

        fn set_transform(&mut self, transform: Transform) {
            self.transform = transform;
        }

        fn is_active(&self) -> bool {
            self.active
        }

        fn set_active(&mut self, active: bool) {
            self.active = active;
            self.update_active_in_hierarchy();
        }

        fn is_active_in_hierarchy(&self) -> bool {
            self.active_in_hierarchy
        }

        fn get_layer(&self) -> u32 {
            self.layer
        }

        fn set_layer(&mut self, layer: u32) {
            self.layer = layer;
        }

        fn is_transform_dirty(&self) -> bool {
            self.active_in_hierarchy
        }

        fn get_uuid(&self) -> String {
            self.name.clone()
        }

        fn update_active_in_hierarchy(&mut self) {
            let parent_active = self
                .parent
                .as_ref()
                .and_then(|p| p.upgrade())
                .map_or(true, |p| p.is_active_in_hierarchy());
            self.active_in_hierarchy = self.active && parent_active;

            for child in &mut self.children {
                if let Some(node) = Arc::get_mut(child) {
                    node.update_active_in_hierarchy();
                }
            }
        }
    }

    #[test]
    fn test_node_hierarchy() {
        let root = TestNode::new("Root");
        let parent = Arc::new(TestNode::new("Parent"));
        let child1 = Arc::new(TestNode::new("Child1"));
        let child2 = Arc::new(TestNode::new("Child2"));

        root.add_child(child1.clone());
        child1.set_parent(Some(Arc::downgrade(&parent)));

        root.add_child(child2.clone());
        child2.set_parent(Some(Arc::downgrade(&parent)));

        assert_eq!(root.get_children().len(), 2);

        assert!(child1.get_parent().is_some());
        assert!(Arc::ptr_eq(&child1, root.get_children()[0]));

        root.remove_child(&child1);
        assert_eq!(root.get_children().len(), 1);
        assert!(child1.get_parent().is_none());
    }

    #[test]
    fn test_transform() {
        let mut node = TestNode::new("TestNode");
        node.set_position(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(node.get_position(), Vec3::new(1.0, 2.0, 3.0));

        node.set_rotation(Quaternion::new(0.0, 0.0, 1.0));
        assert_eq!(node.get_rotation(), Quaternion::new(0.0, 0.0, 1.0));

        node.set_scale(Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(node.get_scale(), Vec3::new(2.0, 2.0, 2.0));

        let transform = node.get_transform();
        assert_eq!(transform.position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_scene() {
        let mut scene = Scene::new();
        let root = Arc::new(TestNode::new("Root"));
        scene.root = Some(Arc::clone(&root));

        assert!(scene.root.is_some());
    }
}
