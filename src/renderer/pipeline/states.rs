/****************************************************************************
Rust port of Cocos Creator Pipeline State Manager
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PipelineState {
    #[default]
    Normal = 0,
    Skybox = 1,
    Postprocess = 2,
}

#[derive(Debug, Clone)]
pub struct PipelineStateInfo {
    pub state: PipelineState,
    pub bloom_enabled: bool,
    pub hdr_enabled: bool,
    pub shadow_enabled: bool,
    pub postprocess_enabled: bool,
    pub cluster_enabled: bool,
}

impl Default for PipelineStateInfo {
    fn default() -> Self {
        PipelineStateInfo {
            state: PipelineState::Normal,
            bloom_enabled: false,
            hdr_enabled: false,
            shadow_enabled: false,
            postprocess_enabled: false,
            cluster_enabled: false,
        }
    }
}

impl PipelineStateInfo {
    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.bloom_enabled = enabled;
    }

    pub fn is_bloom_enabled(&self) -> bool {
        self.bloom_enabled
    }

    pub fn set_hdr_enabled(&mut self, enabled: bool) {
        self.hdr_enabled = enabled;
    }

    pub fn is_hdr_enabled(&self) -> bool {
        self.hdr_enabled
    }

    pub fn set_shadow_enabled(&mut self, enabled: bool) {
        self.shadow_enabled = enabled;
    }

    pub fn is_shadow_enabled(&self) -> bool {
        self.shadow_enabled
    }

    pub fn set_postprocess_enabled(&mut self, enabled: bool) {
        self.postprocess_enabled = enabled;
    }

    pub fn is_postprocess_enabled(&self) -> bool {
        self.postprocess_enabled
    }

    pub fn set_cluster_enabled(&mut self, enabled: bool) {
        self.cluster_enabled = enabled;
    }

    pub fn is_cluster_enabled(&self) -> bool {
        self.cluster_enabled
    }
}

#[derive(Debug)]
pub struct PipelineStateManager {
    states: HashMap<String, PipelineStateInfo>,
    current_state: PipelineStateInfo,
}

impl PipelineStateManager {
    pub fn new() -> Self {
        PipelineStateManager {
            states: HashMap::new(),
            current_state: PipelineStateInfo::default(),
        }
    }

    pub fn get_current_state(&self) -> &PipelineStateInfo {
        &self.current_state
    }

    pub fn get_current_state_mut(&mut self) -> &mut PipelineStateInfo {
        &mut self.current_state
    }

    pub fn set_current_state(&mut self, state: PipelineStateInfo) {
        self.current_state = state;
    }

    pub fn register_state(&mut self, name: &str, state: PipelineStateInfo) {
        self.states.insert(name.to_string(), state);
    }

    pub fn unregister_state(&mut self, name: &str) {
        self.states.remove(name);
    }

    pub fn get_state(&self, name: &str) -> Option<&PipelineStateInfo> {
        self.states.get(name)
    }

    pub fn get_state_mut(&mut self, name: &str) -> Option<&mut PipelineStateInfo> {
        self.states.get_mut(name)
    }

    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.current_state.bloom_enabled = enabled;
    }

    pub fn is_bloom_enabled(&self) -> bool {
        self.current_state.bloom_enabled
    }

    pub fn set_hdr_enabled(&mut self, enabled: bool) {
        self.current_state.hdr_enabled = enabled;
    }

    pub fn is_hdr_enabled(&self) -> bool {
        self.current_state.hdr_enabled
    }

    pub fn set_shadow_enabled(&mut self, enabled: bool) {
        self.current_state.shadow_enabled = enabled;
    }

    pub fn is_shadow_enabled(&self) -> bool {
        self.current_state.shadow_enabled
    }

    pub fn set_postprocess_enabled(&mut self, enabled: bool) {
        self.current_state.postprocess_enabled = enabled;
    }

    pub fn is_postprocess_enabled(&self) -> bool {
        self.current_state.postprocess_enabled
    }

    pub fn set_cluster_enabled(&mut self, enabled: bool) {
        self.current_state.cluster_enabled = enabled;
    }

    pub fn is_cluster_enabled(&self) -> bool {
        self.current_state.cluster_enabled
    }

    pub fn reset(&mut self) {
        self.current_state = PipelineStateInfo::default();
        self.states.clear();
    }
}

impl Default for PipelineStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_state_manager_new() {
        let mgr = PipelineStateManager::new();
        assert!(!mgr.is_bloom_enabled());
        assert!(!mgr.is_hdr_enabled());
        assert!(!mgr.is_shadow_enabled());
    }

    #[test]
    fn test_pipeline_state_manager_set_bloom() {
        let mut mgr = PipelineStateManager::new();
        mgr.set_bloom_enabled(true);
        assert!(mgr.is_bloom_enabled());
    }

    #[test]
    fn test_pipeline_state_manager_register() {
        let mut mgr = PipelineStateManager::new();
        let state = PipelineStateInfo {
            hdr_enabled: true,
            ..Default::default()
        };
        mgr.register_state("hdr", state);
        assert!(mgr.get_state("hdr").unwrap().is_hdr_enabled());
        assert!(mgr.get_state("nonexistent").is_none());
    }

    #[test]
    fn test_pipeline_state_manager_unregister() {
        let mut mgr = PipelineStateManager::new();
        mgr.register_state("hdr", PipelineStateInfo::default());
        mgr.unregister_state("hdr");
        assert!(mgr.get_state("hdr").is_none());
    }

    #[test]
    fn test_pipeline_state_manager_reset() {
        let mut mgr = PipelineStateManager::new();
        mgr.set_bloom_enabled(true);
        mgr.register_state("test", PipelineStateInfo::default());
        mgr.reset();
        assert!(!mgr.is_bloom_enabled());
        assert!(mgr.get_state("test").is_none());
    }

    #[test]
    fn test_pipeline_state_info() {
        let mut info = PipelineStateInfo::default();
        info.set_cluster_enabled(true);
        assert!(info.is_cluster_enabled());
    }
}
