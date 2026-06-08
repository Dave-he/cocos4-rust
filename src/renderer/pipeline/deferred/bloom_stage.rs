use super::deferred_types::DeferredConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BloomPass {
    Prefilter,
    Downsample(u32),
    Upsample(u32),
    Combine,
}

pub struct BloomStage {
    pub enabled: bool,
    pub threshold: f32,
    pub intensity: f32,
    pub scatter: f32,
    pub soft_knee: f32,
    pub max_iterations: u32,
    current_iterations: u32,
    passes: Vec<BloomPass>,
}

impl BloomStage {
    pub fn new() -> Self {
        Self {
            enabled: true,
            threshold: 1.0,
            intensity: 0.8,
            scatter: 0.7,
            soft_knee: 0.5,
            max_iterations: 5,
            current_iterations: 0,
            passes: Vec::new(),
        }
    }

    pub fn configure(&mut self, config: &DeferredConfig) {
        self.enabled = config.enable_bloom;
        self.threshold = config.bloom_threshold;
        self.intensity = config.bloom_intensity;
        self.scatter = config.bloom_scatter;
        self.soft_knee = config.bloom_soft_knee;
    }

    pub fn build_passes(&mut self, width: u32, height: u32) {
        self.passes.clear();
        self.passes.push(BloomPass::Prefilter);

        let mut w = width / 2;
        let mut h = height / 2;
        let mut level = 0;
        while w >= 4 && h >= 4 && level < self.max_iterations {
            self.passes.push(BloomPass::Downsample(level));
            level += 1;
            w /= 2;
            h /= 2;
        }
        self.current_iterations = level;

        for i in (0..level).rev() {
            self.passes.push(BloomPass::Upsample(i));
        }
        self.passes.push(BloomPass::Combine);
    }

    pub fn get_pass_count(&self) -> usize {
        self.passes.len()
    }

    pub fn get_pass(&self, index: usize) -> Option<&BloomPass> {
        self.passes.get(index)
    }

    pub fn render(&mut self, width: u32, height: u32) -> u32 {
        if !self.enabled {
            return 0;
        }
        self.build_passes(width, height);
        self.passes.len() as u32
    }

    pub fn get_current_level(&self) -> u32 {
        self.current_iterations
    }

    pub fn reset(&mut self) {
        self.passes.clear();
        self.current_iterations = 0;
    }
}

impl Default for BloomStage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_stage_new() {
        let stage = BloomStage::new();
        assert!(stage.enabled);
        assert_eq!(stage.threshold, 1.0);
        assert_eq!(stage.max_iterations, 5);
    }

    #[test]
    fn test_bloom_configure() {
        let mut stage = BloomStage::new();
        let config = DeferredConfig {
            enable_bloom: true,
            bloom_threshold: 1.2,
            bloom_intensity: 1.5,
            bloom_soft_knee: 0.4,
            bloom_scatter: 0.6,
            ..Default::default()
        };
        stage.configure(&config);
        assert_eq!(stage.threshold, 1.2);
        assert_eq!(stage.intensity, 1.5);
    }

    #[test]
    fn test_bloom_build_passes() {
        let mut stage = BloomStage::new();
        stage.build_passes(1920, 1080);
        let count = stage.get_pass_count();
        assert!(count >= 5);
        assert_eq!(*stage.get_pass(0).unwrap(), BloomPass::Prefilter);
        assert_eq!(*stage.get_pass(count - 1).unwrap(), BloomPass::Combine);
    }

    #[test]
    fn test_bloom_build_passes_small_screen() {
        let mut stage = BloomStage::new();
        stage.build_passes(64, 64);
        let count = stage.get_pass_count();
        assert!(count >= 4);
    }

    #[test]
    fn test_bloom_iteration_level() {
        let mut stage = BloomStage::new();
        stage.build_passes(1920, 1080);
        assert!(stage.get_current_level() >= 4);
    }

    #[test]
    fn test_bloom_pass_ordering() {
        let mut stage = BloomStage::new();
        stage.build_passes(1920, 1080);
        let count = stage.get_pass_count();
        let passes: Vec<BloomPass> = (0..count).map(|i| *stage.get_pass(i).unwrap()).collect();

        assert_eq!(passes[0], BloomPass::Prefilter);
        let last_downsample = passes
            .iter()
            .position(|p| matches!(p, BloomPass::Downsample(_)))
            .unwrap();
        assert!(last_downsample > 0);
    }
}
