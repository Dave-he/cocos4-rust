/****************************************************************************
Rust port of Cocos Creator Renderer Pass System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;
use crate::base::{RefCounted, RefCountedImpl};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassType {
    Compute = 0,
    Graphics = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderQueueType {
    Opaque = 0,
    Transparent = 1,
    UI = 2,
    Overlay = 3,
}

impl Default for RenderQueueType {
    fn default() -> Self {
        RenderQueueType::Opaque
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    SrcAlpha = 2,
    OneMinusSrcAlpha = 3,
    DstAlpha = 4,
    OneMinusDstAlpha = 5,
}

impl Default for BlendFactor {
    fn default() -> Self {
        BlendFactor::One
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareFunc {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

impl Default for CompareFunc {
    fn default() -> Self {
        CompareFunc::Less
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlendStateInfo {
    pub enabled: bool,
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub src_alpha_factor: BlendFactor,
    pub dst_alpha_factor: BlendFactor,
}

impl Default for BlendStateInfo {
    fn default() -> Self {
        BlendStateInfo {
            enabled: false,
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            src_alpha_factor: BlendFactor::One,
            dst_alpha_factor: BlendFactor::Zero,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DepthStencilStateInfo {
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_func: CompareFunc,
    pub stencil_test: bool,
}

impl Default for DepthStencilStateInfo {
    fn default() -> Self {
        DepthStencilStateInfo {
            depth_test: true,
            depth_write: true,
            depth_func: CompareFunc::Less,
            stencil_test: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterizerStateInfo {
    pub cull_mode: CullMode,
    pub is_front_face_ccw: bool,
    pub depth_bias: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    None = 0,
    Front = 1,
    Back = 2,
}

impl Default for CullMode {
    fn default() -> Self {
        CullMode::Back
    }
}

impl Default for RasterizerStateInfo {
    fn default() -> Self {
        RasterizerStateInfo {
            cull_mode: CullMode::Back,
            is_front_face_ccw: false,
            depth_bias: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IPassInfo {
    pub name: String,
    pub pass_type: PassType,
    pub queue: RenderQueueType,
    pub priority: i32,
    pub shader: String,
    pub blend_state: BlendStateInfo,
    pub depth_stencil_state: DepthStencilStateInfo,
    pub rasterizer_state: RasterizerStateInfo,
}

impl Default for IPassInfo {
    fn default() -> Self {
        IPassInfo {
            name: String::new(),
            pass_type: PassType::Graphics,
            queue: RenderQueueType::Opaque,
            priority: 0,
            shader: String::new(),
            blend_state: BlendStateInfo::default(),
            depth_stencil_state: DepthStencilStateInfo::default(),
            rasterizer_state: RasterizerStateInfo::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PassOverrides {
    pub blend_state: Option<BlendStateInfo>,
    pub depth_stencil_state: Option<DepthStencilStateInfo>,
    pub rasterizer_state: Option<RasterizerStateInfo>,
    pub queue: Option<RenderQueueType>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct CallbackPass {
    pub name: String,
    pub priority: i32,
}

impl CallbackPass {
    pub fn new(name: &str, priority: i32) -> Self {
        Self {
            name: name.to_string(),
            priority,
        }
    }
}

#[derive(Debug)]
pub struct Pass {
    pub info: IPassInfo,
    pub properties: HashMap<String, PassProperty>,
    pub hash: u64,
    ref_count: RefCountedImpl,
}

#[derive(Debug, Clone)]
pub enum PassProperty {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Color([f32; 4]),
    Int(i32),
    Bool(bool),
    TextureHandle(u32),
}

impl Pass {
    pub fn new(name: &str) -> Self {
        Pass {
            info: IPassInfo {
                name: name.to_string(),
                ..IPassInfo::default()
            },
            properties: HashMap::new(),
            hash: 0,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn with_info(info: IPassInfo) -> Self {
        let hash = Self::compute_hash(&info);
        Pass {
            info,
            properties: HashMap::new(),
            hash,
            ref_count: RefCountedImpl::new(),
        }
    }

    fn compute_hash(info: &IPassInfo) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        info.name.hash(&mut h);
        info.shader.hash(&mut h);
        (info.queue as u32).hash(&mut h);
        (info.priority as u64).hash(&mut h);
        h.finish()
    }

    pub fn override_pipeline_states(&mut self, overrides: &PassOverrides) {
        if let Some(bs) = &overrides.blend_state {
            self.info.blend_state = bs.clone();
        }
        if let Some(ds) = &overrides.depth_stencil_state {
            self.info.depth_stencil_state = ds.clone();
        }
        if let Some(rs) = &overrides.rasterizer_state {
            self.info.rasterizer_state = rs.clone();
        }
        if let Some(q) = overrides.queue {
            self.info.queue = q;
        }
        if let Some(p) = overrides.priority {
            self.info.priority = p;
        }
        self.hash = Self::compute_hash(&self.info);
    }

    pub fn set_property(&mut self, name: &str, value: PassProperty) {
        self.properties.insert(name.to_string(), value);
    }

    pub fn get_property(&self, name: &str) -> Option<&PassProperty> {
        self.properties.get(name)
    }

    pub fn get_name(&self) -> &str {
        &self.info.name
    }

    pub fn get_queue(&self) -> RenderQueueType {
        self.info.queue
    }

    pub fn get_priority(&self) -> i32 {
        self.info.priority
    }

    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    pub fn is_transparent(&self) -> bool {
        self.info.queue == RenderQueueType::Transparent
            || self.info.blend_state.enabled
    }
}

impl RefCounted for Pass {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

pub trait IPass: RefCounted {
    fn get_info(&self) -> &IPassInfo;
}

pub trait RenderableComponent: RefCounted {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_new() {
        let p = Pass::new("default");
        assert_eq!(p.get_name(), "default");
        assert_eq!(p.get_queue(), RenderQueueType::Opaque);
        assert_eq!(p.get_priority(), 0);
    }

    #[test]
    fn test_pass_with_info() {
        let mut info = IPassInfo::default();
        info.name = "transparent".to_string();
        info.queue = RenderQueueType::Transparent;
        info.priority = 100;
        let p = Pass::with_info(info);
        assert_eq!(p.get_queue(), RenderQueueType::Transparent);
        assert_eq!(p.get_priority(), 100);
        assert!(p.is_transparent());
    }

    #[test]
    fn test_pass_override_states() {
        let mut p = Pass::new("test");
        let old_hash = p.get_hash();
        let overrides = PassOverrides {
            queue: Some(RenderQueueType::Transparent),
            priority: Some(50),
            ..Default::default()
        };
        p.override_pipeline_states(&overrides);
        assert_eq!(p.get_queue(), RenderQueueType::Transparent);
        assert_eq!(p.get_priority(), 50);
        assert_ne!(p.get_hash(), old_hash);
    }

    #[test]
    fn test_pass_properties() {
        let mut p = Pass::new("test");
        p.set_property("opacity", PassProperty::Float(0.5));
        p.set_property("albedo", PassProperty::Vec4([1.0, 0.0, 0.0, 1.0]));
        if let Some(PassProperty::Float(v)) = p.get_property("opacity") {
            assert!((*v - 0.5).abs() < 1e-6);
        } else {
            panic!("expected Float property");
        }
        assert!(p.get_property("missing").is_none());
    }

    #[test]
    fn test_pass_blend_enabled_is_transparent() {
        let mut info = IPassInfo::default();
        info.blend_state.enabled = true;
        let p = Pass::with_info(info);
        assert!(p.is_transparent());
    }

    #[test]
    fn test_pass_ref_count() {
        let p = Pass::new("rc_test");
        assert_eq!(p.get_ref_count(), 1); // starts at 1 (initial reference)
        p.add_ref();
        assert_eq!(p.get_ref_count(), 2);
        p.add_ref();
        assert_eq!(p.get_ref_count(), 3);
        assert!(!p.is_last_reference());
        p.release();
        p.release();
        assert!(p.is_last_reference());
    }
}
