use super::render_stage::RenderStage;

#[derive(Debug)]
pub struct RenderFlow {
    pub name: String,
    pub priority: u32,
    pub enabled: bool,
    pub stages: Vec<RenderStage>,
}

impl RenderFlow {
    pub fn new(name: &str, priority: u32) -> Self {
        RenderFlow {
            name: name.to_string(),
            priority,
            enabled: true,
            stages: Vec::new(),
        }
    }

    pub fn activate(&mut self) {
        self.enabled = true;
        for stage in &mut self.stages {
            stage.activate();
        }
    }

    pub fn destroy(&mut self) {
        self.enabled = false;
        for stage in &mut self.stages {
            stage.destroy();
        }
        self.stages.clear();
    }

    pub fn add_stage(&mut self, stage: RenderStage) {
        let mut stages = std::mem::take(&mut self.stages);
        stages.push(stage);
        stages.sort_by_key(|s| s.priority);
        self.stages = stages;
    }

    pub fn get_stage(&self, name: &str) -> Option<&RenderStage> {
        self.stages.iter().find(|s| s.name == name)
    }

    pub fn get_stage_mut(&mut self, name: &str) -> Option<&mut RenderStage> {
        self.stages.iter_mut().find(|s| s.name == name)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl Default for RenderFlow {
    fn default() -> Self {
        Self::new("default", 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_flow_new() {
        let flow = RenderFlow::new("forward", 0);
        assert_eq!(flow.name, "forward");
        assert!(flow.enabled);
        assert!(flow.stages.is_empty());
    }

    #[test]
    fn test_render_flow_add_stage() {
        let mut flow = RenderFlow::new("forward", 0);
        flow.add_stage(RenderStage::new("shadow", 1));
        flow.add_stage(RenderStage::new("opaque", 0));
        assert_eq!(flow.stages.len(), 2);
        assert_eq!(flow.stages[0].name, "opaque");
        assert_eq!(flow.stages[1].name, "shadow");
    }

    #[test]
    fn test_render_flow_get_stage() {
        let mut flow = RenderFlow::new("forward", 0);
        flow.add_stage(RenderStage::new("shadow", 0));
        assert!(flow.get_stage("shadow").is_some());
        assert!(flow.get_stage("nonexistent").is_none());
    }
}
