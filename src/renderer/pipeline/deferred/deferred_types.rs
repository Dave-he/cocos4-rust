#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GbufferTextureId {
    Albedo = 0,
    Normal = 1,
    Emissive = 2,
    Depth = 3,
    Count = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingMode {
    Clustered,
    Tiled,
    ForwardPlus,
}

impl Default for LightingMode {
    fn default() -> Self {
        Self::Tiled
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeferredConfig {
    pub gbuffer_width: u32,
    pub gbuffer_height: u32,
    pub lighting_mode: LightingMode,
    pub enable_bloom: bool,
    pub bloom_intensity: f32,
    pub bloom_threshold: f32,
    pub bloom_soft_knee: f32,
    pub bloom_scatter: f32,
    pub enable_ssao: bool,
    pub ssao_radius: f32,
    pub ssao_bias: f32,
    pub enable_ssr: bool,
}

impl Default for DeferredConfig {
    fn default() -> Self {
        Self {
            gbuffer_width: 1920,
            gbuffer_height: 1080,
            lighting_mode: LightingMode::default(),
            enable_bloom: true,
            bloom_intensity: 0.8,
            bloom_threshold: 1.0,
            bloom_soft_knee: 0.5,
            bloom_scatter: 0.7,
            enable_ssao: false,
            ssao_radius: 0.5,
            ssao_bias: 0.025,
            enable_ssr: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GbufferTexture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub sample_count: u32,
}

impl GbufferTexture {
    pub fn new(id: u32, width: u32, height: u32, format: u32) -> Self {
        Self {
            id,
            width,
            height,
            format,
            sample_count: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderTargetSet {
    pub color_textures: Vec<u32>,
    pub depth_texture: Option<u32>,
    pub width: u32,
    pub height: u32,
}

impl RenderTargetSet {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            color_textures: Vec::new(),
            depth_texture: None,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeferredLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: Option<f32>,
    pub cast_shadows: bool,
}

impl Default for DeferredLight {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 10.0,
            spot_angle: None,
            cast_shadows: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeferredSceneData {
    pub lights: Vec<DeferredLight>,
    pub ambient_light: [f32; 3],
    pub fog_color: [f32; 3],
    pub fog_density: f32,
    pub gbuffer: Vec<GbufferTexture>,
    pub render_targets: Vec<RenderTargetSet>,
    pub config: DeferredConfig,
}

impl DeferredSceneData {
    pub fn new() -> Self {
        Self {
            lights: Vec::new(),
            ambient_light: [0.03, 0.03, 0.03],
            fog_color: [0.5, 0.5, 0.5],
            fog_density: 0.0,
            gbuffer: Vec::new(),
            render_targets: Vec::new(),
            config: DeferredConfig::default(),
        }
    }

    pub fn add_light(&mut self, light: DeferredLight) -> u32 {
        let id = self.lights.len() as u32;
        self.lights.push(light);
        id
    }

    pub fn remove_light(&mut self, index: usize) {
        if index < self.lights.len() {
            self.lights.remove(index);
        }
    }

    pub fn get_light_count(&self) -> usize {
        self.lights.len()
    }

    pub fn clear_lights(&mut self) {
        self.lights.clear();
    }
}
