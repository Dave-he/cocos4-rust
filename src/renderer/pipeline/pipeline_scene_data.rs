/****************************************************************************
Rust port of Cocos Creator PipelineSceneData
Original C++ version Copyright (c) 2020-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::defines::RenderObject;
use super::shadow::ShadowsInfo;

#[derive(Debug, Clone, Default)]
pub struct AmbientInfo {
    pub sky_color: crate::math::Color,
    pub sky_illum: f32,
    pub ground_albedo: crate::math::Color,
    pub enabled: bool,
}

impl AmbientInfo {
    pub fn new() -> Self {
        AmbientInfo {
            sky_color: crate::math::Color::new(51, 128, 204, 255),
            sky_illum: 20000.0,
            ground_albedo: crate::math::Color::new(51, 51, 51, 255),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FogInfo {
    pub enabled: bool,
    pub fog_color: crate::math::Color,
    pub fog_type: u32,
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
            fog_color: crate::math::Color::new(153, 153, 153, 255),
            fog_type: 0,
            fog_density: 0.3,
            fog_start: 0.5,
            fog_end: 300.0,
            fog_atten: 5.0,
            fog_top: 1.5,
            fog_range: 1.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkyboxInfo {
    pub enabled: bool,
    pub use_ibl: bool,
    pub use_diffuse_map: bool,
    pub use_hdr: bool,
    pub rotation: f32,
}

impl Default for SkyboxInfo {
    fn default() -> Self {
        SkyboxInfo {
            enabled: false,
            use_ibl: false,
            use_diffuse_map: false,
            use_hdr: false,
            rotation: 0.0,
        }
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
        data.add_render_object(RenderObject { depth: 1.0, model_id: 42 });
        data.add_render_object(RenderObject { depth: 2.0, model_id: 43 });
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
    fn test_pipeline_scene_data_shading_scale() {
        let mut data = PipelineSceneData::new();
        data.set_shading_scale(0.5);
        assert_eq!(data.get_shading_scale(), 0.5);
    }

    #[test]
    fn test_pipeline_scene_data_shadows() {
        let mut data = PipelineSceneData::new();
        data.get_shadows_mut().set_enabled(true);
        assert!(data.get_shadows().enabled);
    }

    #[test]
    fn test_ambient_default() {
        let ambient = AmbientInfo::new();
        assert!(ambient.enabled);
        assert!(ambient.sky_illum > 0.0);
    }

    #[test]
    fn test_fog_default() {
        let fog = FogInfo::default();
        assert!(!fog.enabled);
        assert!(fog.fog_end > fog.fog_start);
    }
}
