/****************************************************************************
Rust port of Cocos Creator Render Flow
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::defines::RenderFlowType;
use super::render_stage::RenderStage;

#[derive(Debug)]
pub struct RenderFlowInfo {
    pub name: String,
    pub priority: u32,
    pub stages: Vec<RenderStage>,
    pub tag: u32,
}

impl Default for RenderFlowInfo {
    fn default() -> Self {
        RenderFlowInfo {
            name: String::new(),
            priority: 0,
            stages: Vec::new(),
            tag: 0,
        }
    }
}

#[derive(Debug)]
pub struct RenderFlow {
    pub name: String,
    pub priority: u32,
    pub tag: u32,
    pub enabled: bool,
    pub flow_type: RenderFlowType,
    pub stages: Vec<RenderStage>,
}

impl RenderFlow {
    pub fn new(name: &str, priority: u32) -> Self {
        RenderFlow {
            name: name.to_string(),
            priority,
            tag: 0,
            enabled: true,
            flow_type: RenderFlowType::Scene,
            stages: Vec::new(),
        }
    }

    pub fn with_info(info: RenderFlowInfo) -> Self {
        RenderFlow {
            name: info.name,
            priority: info.priority,
            tag: info.tag,
            enabled: true,
            flow_type: RenderFlowType::Scene,
            stages: info.stages,
        }
    }

    pub fn initialize(&mut self, info: RenderFlowInfo) {
        self.name = info.name;
        self.priority = info.priority;
        self.tag = info.tag;
        self.stages = info.stages;
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

    pub fn remove_stage(&mut self, name: &str) {
        self.stages.retain(|s| s.name != name);
    }

    pub fn get_stage(&self, name: &str) -> Option<&RenderStage> {
        self.stages.iter().find(|s| s.name == name)
    }

    pub fn get_stage_mut(&mut self, name: &str) -> Option<&mut RenderStage> {
        self.stages.iter_mut().find(|s| s.name == name)
    }

    pub fn get_stages(&self) -> &[RenderStage] {
        &self.stages
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_priority(&self) -> u32 {
        self.priority
    }

    pub fn set_priority(&mut self, priority: u32) {
        self.priority = priority;
    }

    pub fn get_tag(&self) -> u32 {
        self.tag
    }

    pub fn set_tag(&mut self, tag: u32) {
        self.tag = tag;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_flow_type(&self) -> RenderFlowType {
        self.flow_type
    }

    pub fn set_flow_type(&mut self, flow_type: RenderFlowType) {
        self.flow_type = flow_type;
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
        assert_eq!(flow.get_priority(), 0);
    }

    #[test]
    fn test_render_flow_with_info() {
        let info = RenderFlowInfo {
            name: "shadow".to_string(),
            priority: 10,
            stages: Vec::new(),
            tag: 1,
        };
        let flow = RenderFlow::with_info(info);
        assert_eq!(flow.name, "shadow");
        assert_eq!(flow.get_tag(), 1);
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

    #[test]
    fn test_render_flow_remove_stage() {
        let mut flow = RenderFlow::new("forward", 0);
        flow.add_stage(RenderStage::new("opaque", 0));
        flow.add_stage(RenderStage::new("shadow", 1));
        flow.remove_stage("shadow");
        assert_eq!(flow.get_stages().len(), 1);
    }

    #[test]
    fn test_render_flow_enable_disable() {
        let mut flow = RenderFlow::new("test", 0);
        flow.set_enabled(false);
        assert!(!flow.is_enabled());
        flow.set_enabled(true);
        assert!(flow.is_enabled());
    }

    #[test]
    fn test_render_flow_type() {
        let mut flow = RenderFlow::new("test", 0);
        flow.set_flow_type(RenderFlowType::Postprocess);
        assert_eq!(flow.get_flow_type(), RenderFlowType::Postprocess);
    }
}
