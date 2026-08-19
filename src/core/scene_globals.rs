use crate::math::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AmbientLightMode {
    #[default]
    Flat,
    Gradient,
}

#[derive(Debug, Clone)]
pub struct AmbientLightInfo {
    pub mode: AmbientLightMode,
    pub color: Color,
    pub color_sky: Color,
    pub color_ground: Color,
    pub intensity: f32,
}

impl Default for AmbientLightInfo {
    fn default() -> Self {
        Self {
            mode: AmbientLightMode::Flat,
            color: Color::new(127, 127, 127, 255),
            color_sky: Color::new(51, 127, 204, 255),
            color_ground: Color::new(127, 76, 51, 255),
            intensity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FogType {
    #[default]
    None,
    Linear,
    Exp,
    Exp2,
}

#[derive(Debug, Clone)]
pub struct FogInfo {
    pub fog_type: FogType,
    pub color: Color,
    pub density: f32,
    pub start: f32,
    pub end: f32,
}

impl Default for FogInfo {
    fn default() -> Self {
        Self {
            fog_type: FogType::None,
            color: Color::new(127, 127, 127, 255),
            density: 0.3,
            start: 0.5,
            end: 300.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkyboxInfo {
    pub enabled: bool,
    pub envmap: Option<String>,
    pub use_envmap_color: bool,
    pub envmap_color: Color,
    pub use_diffuse_map: bool,
    pub diffuse_map: Option<String>,
    pub use_reflection_map: bool,
    pub reflection_map: Option<String>,
    pub use_radiance_map: bool,
    pub radiance_map: Option<String>,
    pub exposure: f32,
    pub rotation_angle: f32,
    pub roughness: f32,
    pub applied_diffuse_color: Color,
}

impl Default for SkyboxInfo {
    fn default() -> Self {
        Self {
            enabled: false,
            envmap: None,
            use_envmap_color: false,
            envmap_color: Color::new(51, 51, 51, 255),
            use_diffuse_map: false,
            diffuse_map: None,
            use_reflection_map: false,
            reflection_map: None,
            use_radiance_map: false,
            radiance_map: None,
            exposure: 1.0,
            rotation_angle: 0.0,
            roughness: 0.2,
            applied_diffuse_color: Color::new(51, 51, 51, 255),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowType {
    #[default]
    None,
    Hard,
    Soft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PcfType {
    #[default]
    Hard,
    Soft,
    Soft2x,
    Soft4x,
}

#[derive(Debug, Clone)]
pub struct ShadowsInfo {
    pub shadow_type: ShadowType,
    pub enabled: bool,
    pub pcf: PcfType,
    pub max_received: u32,
    pub size: u32,
    pub distance: f32,
    pub bias: f32,
    pub normal_bias: f32,
    pub saturation: f32,
    pub invisible_others: bool,
    pub csm_level: u32,
    pub csm_layers_opacity: [f32; 4],
    pub csm_custom_bias: [f32; 4],
    pub planar_shadow: bool,
    pub planar_shadow_factor: f32,
}

impl Default for ShadowsInfo {
    fn default() -> Self {
        Self {
            shadow_type: ShadowType::None,
            enabled: false,
            pcf: PcfType::Hard,
            max_received: 0,
            size: 1024,
            distance: 70.0,
            bias: 0.005,
            normal_bias: 0.005,
            saturation: 1.0,
            invisible_others: true,
            csm_level: 1,
            csm_layers_opacity: [0.0; 4],
            csm_custom_bias: [0.0; 4],
            planar_shadow: false,
            planar_shadow_factor: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OctreeInfo {
    pub enabled: bool,
    pub min_depth: u32,
    pub max_depth: u32,
    pub capacity: u32,
}

impl Default for OctreeInfo {
    fn default() -> Self {
        Self { enabled: false, min_depth: 1, max_depth: 8, capacity: 4 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LightProbeMode {
    #[default]
    Baked,
    Realtime,
}

#[derive(Debug, Clone)]
pub struct LightProbeInfo {
    pub enabled: bool,
    pub mode: LightProbeMode,
    pub data: Vec<[f32; 9]>,
}

impl Default for LightProbeInfo {
    fn default() -> Self {
        Self { enabled: false, mode: LightProbeMode::Baked, data: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct PostProcessSettings {
    pub enable_bloom: bool,
    pub bloom_intensity: f32,
    pub bloom_threshold: f32,
    pub enable_fxaa: bool,
    pub enable_tone_mapping: bool,
    pub tone_mapping_exposure: f32,
    pub enable_color_grading: bool,
    pub color_grading_intensity: f32,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            enable_bloom: false,
            bloom_intensity: 0.8,
            bloom_threshold: 1.0,
            enable_fxaa: false,
            enable_tone_mapping: true,
            tone_mapping_exposure: 1.0,
            enable_color_grading: false,
            color_grading_intensity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SceneGlobals {
    pub ambient: AmbientLightInfo,
    pub fog: FogInfo,
    pub skybox: SkyboxInfo,
    pub shadows: ShadowsInfo,
    pub octree: OctreeInfo,
    pub light_probe: LightProbeInfo,
    pub post_process: PostProcessSettings,
    pub skin: Option<String>,
}

impl SceneGlobals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_ambient_color(&mut self, color: Color) {
        self.ambient.color = color;
    }

    pub fn set_ambient_intensity(&mut self, intensity: f32) {
        self.ambient.intensity = intensity;
    }

    pub fn set_fog_type(&mut self, fog_type: FogType) {
        self.fog.fog_type = fog_type;
    }

    pub fn set_fog_color(&mut self, color: Color) {
        self.fog.color = color;
    }

    pub fn set_fog_density(&mut self, density: f32) {
        self.fog.density = density;
    }

    pub fn enable_skybox(&mut self, enabled: bool) {
        self.skybox.enabled = enabled;
    }

    pub fn set_skybox_envmap(&mut self, envmap: Option<String>) {
        self.skybox.envmap = envmap;
        self.skybox.use_envmap_color = self.skybox.envmap.is_some();
    }

    pub fn set_skybox_exposure(&mut self, exposure: f32) {
        self.skybox.exposure = exposure;
    }

    pub fn enable_shadows(&mut self, enabled: bool) {
        self.shadows.enabled = enabled;
        if enabled && self.shadows.shadow_type == ShadowType::None {
            self.shadows.shadow_type = ShadowType::Hard;
        }
        if !enabled {
            self.shadows.shadow_type = ShadowType::None;
        }
    }

    pub fn set_shadow_type(&mut self, shadow_type: ShadowType) {
        self.shadows.shadow_type = shadow_type;
        self.shadows.enabled = shadow_type != ShadowType::None;
    }

    pub fn set_shadow_size(&mut self, size: u32) {
        self.shadows.size = size;
    }

    pub fn enable_octree(&mut self, enabled: bool) {
        self.octree.enabled = enabled;
    }

    pub fn enable_bloom(&mut self, enabled: bool) {
        self.post_process.enable_bloom = enabled;
    }

    pub fn set_bloom_intensity(&mut self, intensity: f32) {
        self.post_process.bloom_intensity = intensity;
    }

    pub fn enable_fxaa(&mut self, enabled: bool) {
        self.post_process.enable_fxaa = enabled;
    }

    pub fn enable_tone_mapping(&mut self, enabled: bool) {
        self.post_process.enable_tone_mapping = enabled;
    }

    pub fn set_exposure(&mut self, exposure: f32) {
        self.post_process.tone_mapping_exposure = exposure;
    }

    pub fn get_ambient_illumination(&self) -> Color {
        let i = self.ambient.intensity;
        let c = self.ambient.color;
        Color::new(
            (c.r as f32 * i) as u8,
            (c.g as f32 * i) as u8,
            (c.b as f32 * i) as u8,
            c.a,
        )
    }

    pub fn get_fog_color_with_density(&self) -> Color {
        let c = self.fog.color;
        let d = self.fog.density;
        Color::new(
            (c.r as f32 * d) as u8,
            (c.g as f32 * d) as u8,
            (c.b as f32 * d) as u8,
            c.a,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_globals_new() {
        let sg = SceneGlobals::new();
        assert_eq!(sg.ambient.mode, AmbientLightMode::Flat);
        assert_eq!(sg.fog.fog_type, FogType::None);
        assert!(!sg.skybox.enabled);
        assert!(!sg.shadows.enabled);
        assert!(!sg.octree.enabled);
    }

    #[test]
    fn test_set_ambient() {
        let mut sg = SceneGlobals::new();
        sg.set_ambient_color(Color::new(255, 0, 0, 255));
        assert_eq!(sg.ambient.color.r, 255);
        sg.set_ambient_intensity(0.8);
        assert!((sg.ambient.intensity - 0.8).abs() < 1e-6);
        let illum = sg.get_ambient_illumination();
        assert_eq!(illum.r, (255.0_f32 * 0.8) as u8);
    }

    #[test]
    fn test_set_fog() {
        let mut sg = SceneGlobals::new();
        sg.set_fog_type(FogType::Exp);
        assert_eq!(sg.fog.fog_type, FogType::Exp);
        sg.set_fog_color(Color::new(127, 127, 127, 255));
        assert_eq!(sg.fog.color.r, 127);
        sg.set_fog_density(0.5);
        assert!((sg.fog.density - 0.5).abs() < 1e-6);
        let fog_color = sg.get_fog_color_with_density();
        assert_eq!(fog_color.r, (127.0_f32 * 0.5) as u8);
    }

    #[test]
    fn test_enable_skybox() {
        let mut sg = SceneGlobals::new();
        sg.enable_skybox(true);
        assert!(sg.skybox.enabled);
        sg.set_skybox_envmap(Some("sky.png".to_string()));
        assert!(sg.skybox.use_envmap_color);
        assert_eq!(sg.skybox.envmap, Some("sky.png".to_string()));
        sg.set_skybox_exposure(2.0);
        assert!((sg.skybox.exposure - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_enable_shadows() {
        let mut sg = SceneGlobals::new();
        sg.enable_shadows(true);
        assert!(sg.shadows.enabled);
        assert_eq!(sg.shadows.shadow_type, ShadowType::Hard);
        sg.set_shadow_type(ShadowType::Soft);
        assert_eq!(sg.shadows.shadow_type, ShadowType::Soft);
        sg.set_shadow_size(2048);
        assert_eq!(sg.shadows.size, 2048);
        sg.enable_shadows(false);
        assert!(!sg.shadows.enabled);
        assert_eq!(sg.shadows.shadow_type, ShadowType::None);
    }

    #[test]
    fn test_octree() {
        let mut sg = SceneGlobals::new();
        sg.enable_octree(true);
        assert!(sg.octree.enabled);
        assert_eq!(sg.octree.max_depth, 8);
    }

    #[test]
    fn test_post_process() {
        let mut sg = SceneGlobals::new();
        sg.enable_bloom(true);
        assert!(sg.post_process.enable_bloom);
        sg.set_bloom_intensity(1.5);
        assert!((sg.post_process.bloom_intensity - 1.5).abs() < 1e-6);
        sg.enable_fxaa(true);
        assert!(sg.post_process.enable_fxaa);
        sg.enable_tone_mapping(false);
        assert!(!sg.post_process.enable_tone_mapping);
        sg.set_exposure(2.0);
        assert!((sg.post_process.tone_mapping_exposure - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_ambient_gradient_mode() {
        let mut sg = SceneGlobals::new();
        sg.ambient.mode = AmbientLightMode::Gradient;
        assert_eq!(sg.ambient.mode, AmbientLightMode::Gradient);
        assert_eq!(sg.ambient.color_sky.r, 51);
        assert_eq!(sg.ambient.color_ground.r, 127);
    }

    #[test]
    fn test_light_probe() {
        let sg = SceneGlobals::new();
        assert!(!sg.light_probe.enabled);
        assert_eq!(sg.light_probe.mode, LightProbeMode::Baked);
    }

    #[test]
    fn test_shadow_csm() {
        let sg = SceneGlobals::new();
        assert_eq!(sg.shadows.csm_level, 1);
        assert_eq!(sg.shadows.csm_layers_opacity, [0.0; 4]);
    }
}
