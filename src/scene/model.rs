use crate::math::Mat4;
use crate::core::geometry::AABB;
use super::define::{ModelType, UseReflectionProbeType, CAMERA_DEFAULT_MASK};

#[derive(Debug)]
pub struct Model {
    pub model_id: u64,
    pub model_type: ModelType,
    pub enabled: bool,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub is_shadow_intensity_dirty: bool,
    pub visibility: u32,
    pub priority: u32,
    pub node_uuid: Option<String>,
    pub world_bounds: Option<AABB>,
    pub local_bounds: Option<AABB>,
    pub world_matrix: Mat4,
    pub enable_bounding_box_culling: bool,
    pub use_reflection_probe: UseReflectionProbeType,
    pub reflection_probe_id: i32,
    pub blend_reflection_probe_id: i32,
    pub is_static: bool,
}

impl Model {
    pub fn new(id: u64) -> Self {
        Model {
            model_id: id,
            model_type: ModelType::Default,
            enabled: false,
            cast_shadow: false,
            receive_shadow: false,
            is_shadow_intensity_dirty: false,
            visibility: CAMERA_DEFAULT_MASK,
            priority: 0,
            node_uuid: None,
            world_bounds: None,
            local_bounds: None,
            world_matrix: Mat4::IDENTITY,
            enable_bounding_box_culling: true,
            use_reflection_probe: UseReflectionProbeType::None,
            reflection_probe_id: -1,
            blend_reflection_probe_id: -1,
            is_static: false,
        }
    }

    pub fn initialize(&mut self) {
        self.enabled = false;
        self.cast_shadow = false;
        self.receive_shadow = false;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_cast_shadow(&mut self, cast: bool) {
        self.cast_shadow = cast;
    }

    pub fn is_cast_shadow(&self) -> bool {
        self.cast_shadow
    }

    pub fn set_receive_shadow(&mut self, receive: bool) {
        self.receive_shadow = receive;
    }

    pub fn is_receive_shadow(&self) -> bool {
        self.receive_shadow
    }

    pub fn set_visibility(&mut self, visibility: u32) {
        self.visibility = visibility;
    }

    pub fn get_visibility(&self) -> u32 {
        self.visibility
    }

    pub fn get_model_id(&self) -> u64 {
        self.model_id
    }

    pub fn get_model_type(&self) -> ModelType {
        self.model_type
    }

    pub fn set_world_matrix(&mut self, mat: Mat4) {
        self.world_matrix = mat;
    }

    pub fn get_world_matrix(&self) -> &Mat4 {
        &self.world_matrix
    }

    pub fn set_use_reflection_probe(&mut self, probe: UseReflectionProbeType) {
        self.use_reflection_probe = probe;
    }

    pub fn get_use_reflection_probe(&self) -> UseReflectionProbeType {
        self.use_reflection_probe
    }

    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
    }

    pub fn is_static_model(&self) -> bool {
        self.is_static
    }
}
