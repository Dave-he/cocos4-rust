/****************************************************************************
Rust port of Cocos Creator PipelineSceneData
Original C++ version Copyright (c) 2020-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::defines::RenderObject;
use super::shadow::ShadowsInfo;
use crate::math::{Color, Vec3, Vec4};

#[derive(Debug, Clone, Default)]
pub struct AmbientInfo {
    pub sky_color: Color,
    pub sky_illum: f32,
    pub ground_albedo: Color,
    pub enabled: bool,
}

impl AmbientInfo {
    pub fn new() -> Self {
        AmbientInfo {
            sky_color: Color::new(51, 128, 204, 255),
            sky_illum: 20000.0,
            ground_albedo: Color::new(51, 51, 51, 255),
            enabled: true,
        }
    }

    pub fn set_sky_color(&mut self, color: Color) {
        self.sky_color = color;
    }

    pub fn get_sky_color(&self) -> Color {
        self.sky_color
    }

    pub fn set_sky_illum(&mut self, illum: f32) {
        self.sky_illum = illum;
    }

    pub fn get_sky_illum(&self) -> f32 {
        self.sky_illum
    }

    pub fn set_ground_albedo(&mut self, albedo: Color) {
        self.ground_albedo = albedo;
    }

    pub fn get_ground_albedo(&self) -> Color {
        self.ground_albedo
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FogType {
    #[default]
    Linear = 0,
    Exp = 1,
    ExpSquared = 2,
    Layered = 3,
}

#[derive(Debug, Clone)]
pub struct FogInfo {
    pub enabled: bool,
    pub fog_color: Color,
    pub fog_type: FogType,
    pub fog_density: f32,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_atten: f32,
    pub fog_top: f32,
    pub fog_range: f32,
}

impl Default for FogInfo {
    fn default() -> Self {
        FogInfo {
            enabled: false,
            fog_color: Color::new(153, 153, 153, 255),
            fog_type: FogType::Linear,
            fog_density: 0.3,
            fog_start: 0.5,
            fog_end: 300.0,
            fog_atten: 5.0,
            fog_top: 1.5,
            fog_range: 1.2,
        }
    }
}

impl FogInfo {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_fog_color(&mut self, color: Color) {
        self.fog_color = color;
    }

    pub fn get_fog_color(&self) -> Color {
        self.fog_color
    }

    pub fn set_fog_type(&mut self, fog_type: FogType) {
        self.fog_type = fog_type;
    }

    pub fn get_fog_type(&self) -> FogType {
        self.fog_type
    }

    pub fn set_fog_density(&mut self, density: f32) {
        self.fog_density = density;
    }

    pub fn get_fog_density(&self) -> f32 {
        self.fog_density
    }

    pub fn set_fog_start(&mut self, start: f32) {
        self.fog_start = start;
    }

    pub fn get_fog_start(&self) -> f32 {
        self.fog_start
    }

    pub fn set_fog_end(&mut self, end: f32) {
        self.fog_end = end;
    }

    pub fn get_fog_end(&self) -> f32 {
        self.fog_end
    }
}

#[derive(Debug, Clone)]
pub struct SkyboxInfo {
    pub enabled: bool,
    pub use_ibl: bool,
    pub use_diffuse_map: bool,
    pub use_hdr: bool,
    pub rotation: f32,
    pub env_lighting_type: u32,
    pub env_map_id: u64,
    pub diffuse_map_id: u64,
    pub specular_map_id: u64,
}

impl Default for SkyboxInfo {
    fn default() -> Self {
        SkyboxInfo {
            enabled: false,
            use_ibl: false,
            use_diffuse_map: false,
            use_hdr: false,
            rotation: 0.0,
            env_lighting_type: 0,
            env_map_id: 0,
            diffuse_map_id: 0,
            specular_map_id: 0,
        }
    }
}

impl SkyboxInfo {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_use_ibl(&mut self, use_ibl: bool) {
        self.use_ibl = use_ibl;
    }

    pub fn is_use_ibl(&self) -> bool {
        self.use_ibl
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }
}

#[derive(Debug)]
pub struct PipelineSceneData {
    pub is_hdr: bool,
    pub shading_scale: f32,
    pub csm_supported: bool,
    pub render_objects: Vec<RenderObject>,
    pub shadows: ShadowsInfo,
    pub ambient: AmbientInfo,
    pub fog: FogInfo,
    pub skybox: SkyboxInfo,
    pub exposure: f32,
    pub sky_color: Vec4,
    pub sky_illum: f32,
    pub main_light_dir: Vec3,
    pub main_light_color: Color,
}

impl PipelineSceneData {
    pub fn new() -> Self {
        PipelineSceneData {
            is_hdr: false,
            shading_scale: 1.0,
            csm_supported: true,
            render_objects: Vec::new(),
            shadows: ShadowsInfo::default(),
            ambient: AmbientInfo::new(),
            fog: FogInfo::default(),
            skybox: SkyboxInfo::default(),
            exposure: 1.0,
            sky_color: Vec4::ONE,
            sky_illum: 20000.0,
            main_light_dir: Vec3::new(0.0, -1.0, 0.0),
            main_light_color: Color::WHITE,
        }
    }

    pub fn activate(&mut self) {}

    pub fn update_pipeline_scene_data(&mut self) {}

    pub fn add_render_object(&mut self, obj: RenderObject) {
        self.render_objects.push(obj);
    }

    pub fn clear_render_objects(&mut self) {
        self.render_objects.clear();
    }

    pub fn get_render_objects(&self) -> &[RenderObject] {
        &self.render_objects
    }

    pub fn set_hdr(&mut self, val: bool) {
        self.is_hdr = val;
    }

    pub fn is_hdr(&self) -> bool {
        self.is_hdr
    }

    pub fn set_shading_scale(&mut self, val: f32) {
        self.shading_scale = val;
    }

    pub fn get_shading_scale(&self) -> f32 {
        self.shading_scale
    }

    pub fn set_csm_supported(&mut self, val: bool) {
        self.csm_supported = val;
    }

    pub fn is_csm_supported(&self) -> bool {
        self.csm_supported
    }

    pub fn get_shadows(&self) -> &ShadowsInfo {
        &self.shadows
    }

    pub fn get_shadows_mut(&mut self) -> &mut ShadowsInfo {
        &mut self.shadows
    }

    pub fn get_ambient(&self) -> &AmbientInfo {
        &self.ambient
    }

    pub fn get_ambient_mut(&mut self) -> &mut AmbientInfo {
        &mut self.ambient
    }

    pub fn get_fog(&self) -> &FogInfo {
        &self.fog
    }

    pub fn get_fog_mut(&mut self) -> &mut FogInfo {
        &mut self.fog
    }

    pub fn get_skybox(&self) -> &SkyboxInfo {
        &self.skybox
    }

    pub fn get_skybox_mut(&mut self) -> &mut SkyboxInfo {
        &mut self.skybox
    }

    pub fn set_exposure(&mut self, val: f32) {
        self.exposure = val;
    }

    pub fn get_exposure(&self) -> f32 {
        self.exposure
    }

    pub fn set_main_light_dir(&mut self, dir: Vec3) {
        self.main_light_dir = dir;
    }

    pub fn get_main_light_dir(&self) -> Vec3 {
        self.main_light_dir
    }

    pub fn set_main_light_color(&mut self, color: Color) {
        self.main_light_color = color;
    }

    pub fn get_main_light_color(&self) -> Color {
        self.main_light_color
    }
}

impl Default for PipelineSceneData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_scene_data_new() {
        let data = PipelineSceneData::new();
        assert!(!data.is_hdr());
        assert_eq!(data.get_shading_scale(), 1.0);
        assert!(data.get_render_objects().is_empty());
    }

    #[test]
    fn test_pipeline_scene_data_render_objects() {
        let mut data = PipelineSceneData::new();
        data.add_render_object(RenderObject {
            depth: 1.0,
            model_id: 42,
        });
        data.add_render_object(RenderObject {
            depth: 2.0,
            model_id: 43,
        });
        assert_eq!(data.get_render_objects().len(), 2);
        data.clear_render_objects();
        assert!(data.get_render_objects().is_empty());
    }

    #[test]
    fn test_pipeline_scene_data_hdr() {
        let mut data = PipelineSceneData::new();
        data.set_hdr(true);
        assert!(data.is_hdr());
        data.set_hdr(false);
        assert!(!data.is_hdr());
    }

    #[test]
    fn test_pipeline_scene_data_exposure() {
        let mut data = PipelineSceneData::new();
        data.set_exposure(2.0);
        assert_eq!(data.get_exposure(), 2.0);
    }

    #[test]
    fn test_pipeline_scene_data_shadows() {
        let mut data = PipelineSceneData::new();
        data.get_shadows_mut().set_enabled(true);
        assert!(data.get_shadows().is_enabled());
    }

    #[test]
    fn test_ambient_default() {
        let ambient = AmbientInfo::new();
        assert!(ambient.is_enabled());
        assert!(ambient.get_sky_illum() > 0.0);
    }

    #[test]
    fn test_fog_default() {
        let fog = FogInfo::default();
        assert!(!fog.is_enabled());
        assert!(fog.get_fog_end() > fog.get_fog_start());
    }
}
