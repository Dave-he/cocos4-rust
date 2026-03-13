/****************************************************************************
Rust port of Cocos Creator UI Model Proxy
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::{Mat4, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIModelType {
    Model = 0,
    Particle = 1,
    Base = 2,
    DragonBones = 3,
    SpineSkeleton = 4,
    MeshRenderer = 5,
}

pub trait UIModelProxy: RefCounted {
    fn get_model_type(&self) -> UIModelType;
    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
    fn update_transform(&mut self, world_matrix: &Mat4);
    fn update_ubo_and_hash(&mut self, scene_offset: u32);
    fn destroy(&mut self);
}

#[derive(Debug)]
pub struct UIModelProxyImpl {
    pub model_type: UIModelType,
    pub enabled: bool,
    pub world_matrix: Mat4,
    pub node_uuid: Option<String>,
    pub scene_offset: u32,
    pub hash: u64,
    ref_count: RefCountedImpl,
}

impl UIModelProxyImpl {
    pub fn new(model_type: UIModelType) -> Self {
        UIModelProxyImpl {
            model_type,
            enabled: true,
            world_matrix: Mat4::IDENTITY,
            node_uuid: None,
            scene_offset: 0,
            hash: 0,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn set_node(&mut self, uuid: Option<String>) {
        self.node_uuid = uuid;
    }

    pub fn get_node(&self) -> Option<&str> {
        self.node_uuid.as_deref()
    }

    pub fn get_world_position(&self) -> Vec3 {
        Vec3::new(
            self.world_matrix.m[12],
            self.world_matrix.m[13],
            self.world_matrix.m[14],
        )
    }
}

impl Default for UIModelProxyImpl {
    fn default() -> Self {
        Self::new(UIModelType::Model)
    }
}

impl RefCounted for UIModelProxyImpl {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

impl UIModelProxy for UIModelProxyImpl {
    fn get_model_type(&self) -> UIModelType { self.model_type }
    fn is_enabled(&self) -> bool { self.enabled }
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }

    fn update_transform(&mut self, world_matrix: &Mat4) {
        self.world_matrix = *world_matrix;
    }

    fn update_ubo_and_hash(&mut self, scene_offset: u32) {
        self.scene_offset = scene_offset;
        let mut h: u64 = 14695981039346656037u64;
        for b in self.world_matrix.m.iter() {
            h ^= b.to_bits() as u64;
            h = h.wrapping_mul(1099511628211u64);
        }
        h ^= scene_offset as u64;
        self.hash = h;
    }

    fn destroy(&mut self) {
        self.enabled = false;
        self.node_uuid = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_model_proxy_new() {
        let proxy = UIModelProxyImpl::default();
        assert_eq!(proxy.get_model_type(), UIModelType::Model);
        assert!(proxy.is_enabled());
    }

    #[test]
    fn test_update_transform() {
        let mut proxy = UIModelProxyImpl::default();
        let mut mat = Mat4::IDENTITY;
        mat.m[12] = 5.0;
        mat.m[13] = 3.0;
        proxy.update_transform(&mat);
        let pos = proxy.get_world_position();
        assert!((pos.x - 5.0).abs() < 1e-6);
        assert!((pos.y - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_update_hash() {
        let mut proxy = UIModelProxyImpl::default();
        proxy.update_ubo_and_hash(42);
        assert_ne!(proxy.hash, 0);
        assert_eq!(proxy.scene_offset, 42);
    }

    #[test]
    fn test_destroy() {
        let mut proxy = UIModelProxyImpl::default();
        proxy.set_node(Some("test-uuid".to_string()));
        proxy.destroy();
        assert!(!proxy.is_enabled());
        assert!(proxy.get_node().is_none());
    }
}
