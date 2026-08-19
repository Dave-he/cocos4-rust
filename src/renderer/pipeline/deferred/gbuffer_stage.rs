use super::deferred_types::{DeferredSceneData, GbufferTexture, GbufferTextureId};

pub struct GbufferStage {
    pub enabled: bool,
    pub clear_colors: [[f32; 4]; 4],
    pub gbuffer: Vec<GbufferTexture>,
    width: u32,
    height: u32,
    sample_count: u32,
}

impl GbufferStage {
    pub fn new() -> Self {
        Self {
            enabled: true,
            clear_colors: [[0.0; 4]; 4],
            gbuffer: Vec::new(),
            width: 1920,
            height: 1080,
            sample_count: 1,
        }
    }

    pub fn initialize(&mut self, width: u32, height: u32, sample_count: u32) {
        self.width = width;
        self.height = height;
        self.sample_count = sample_count;
    }

    pub fn create_gbuffer(&mut self) -> Vec<GbufferTexture> {
        self.gbuffer.clear();
        let formats = [0x8058, 0x8058, 0x8058, 0x0002];

        for (i, format) in formats
            .iter()
            .enumerate()
            .take(GbufferTextureId::Count as usize)
        {
            let tex = GbufferTexture::new(i as u32, self.width, self.height, *format);
            self.gbuffer.push(tex);
        }
        self.gbuffer.clone()
    }

    pub fn get_gbuffer_texture(&self, id: GbufferTextureId) -> Option<&GbufferTexture> {
        self.gbuffer.get(id as usize)
    }

    pub fn render(&mut self, scene_data: &DeferredSceneData) -> u32 {
        if !self.enabled || self.gbuffer.is_empty() {
            return 0;
        }
        scene_data.get_light_count() as u32
    }

    pub fn clear(&mut self) {
        self.gbuffer.clear();
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

impl Default for GbufferStage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gbuffer_stage_new() {
        let stage = GbufferStage::new();
        assert!(stage.enabled);
        assert_eq!(stage.gbuffer.len(), 0);
    }

    #[test]
    fn test_gbuffer_create() {
        let mut stage = GbufferStage::new();
        stage.initialize(1920, 1080, 1);
        let gbuffer = stage.create_gbuffer();
        assert_eq!(gbuffer.len(), 4);
        assert_eq!(gbuffer[0].width, 1920);
        assert_eq!(gbuffer[0].height, 1080);
    }

    #[test]
    fn test_gbuffer_get_texture() {
        let mut stage = GbufferStage::new();
        stage.initialize(1280, 720, 1);
        stage.create_gbuffer();
        let albedo = stage.get_gbuffer_texture(GbufferTextureId::Albedo);
        assert!(albedo.is_some());
        assert_eq!(albedo.unwrap().id, 0);
        let depth = stage.get_gbuffer_texture(GbufferTextureId::Depth);
        assert!(depth.is_some());
        assert_eq!(depth.unwrap().id, 3);
    }

    use super::super::deferred_types::DeferredLight as DeferredLightType;

    #[test]
    fn test_gbuffer_render() {
        let mut stage = GbufferStage::new();
        stage.initialize(800, 600, 1);
        stage.create_gbuffer();
        let mut scene = DeferredSceneData::new();
        scene.add_light(DeferredLightType::default());
        let draws = stage.render(&scene);
        assert_eq!(draws, 1);
    }

    #[test]
    fn test_gbuffer_clear() {
        let mut stage = GbufferStage::new();
        stage.initialize(800, 600, 1);
        stage.create_gbuffer();
        assert_eq!(stage.gbuffer.len(), 4);
        stage.clear();
        assert_eq!(stage.gbuffer.len(), 0);
    }
}
