use super::render_queue::RenderQueue;

#[derive(Debug)]
pub struct RenderStage {
    pub name: String,
    pub priority: u32,
    pub enabled: bool,
    pub opaque_queue: RenderQueue,
    pub transparent_queue: RenderQueue,
}

impl RenderStage {
    pub fn new(name: &str, priority: u32) -> Self {
        RenderStage {
            name: name.to_string(),
            priority,
            enabled: true,
            opaque_queue: RenderQueue::new(false),
            transparent_queue: RenderQueue::new(true),
        }
    }

    pub fn activate(&mut self) {
        self.enabled = true;
    }

    pub fn destroy(&mut self) {
        self.enabled = false;
        self.opaque_queue.clear();
        self.transparent_queue.clear();
    }

    pub fn clear_queues(&mut self) {
        self.opaque_queue.clear();
        self.transparent_queue.clear();
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_priority(&self) -> u32 {
        self.priority
    }
}

impl Default for RenderStage {
    fn default() -> Self {
        Self::new("default", 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_stage_new() {
        let stage = RenderStage::new("shadow", 1);
        assert_eq!(stage.name, "shadow");
        assert_eq!(stage.priority, 1);
        assert!(stage.enabled);
    }

    #[test]
    fn test_render_stage_destroy() {
        let mut stage = RenderStage::new("test", 0);
        stage.destroy();
        assert!(!stage.enabled);
    }
}
