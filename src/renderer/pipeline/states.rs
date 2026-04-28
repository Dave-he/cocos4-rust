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
}

impl Default for PipelineStateInfo {
    fn default() -> Self {
        PipelineStateInfo {
            state: PipelineState::Normal,
            bloom_enabled: false,
            hdr_enabled: false,
            shadow_enabled: false,
            postprocess_enabled: false,
        }
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

    pub fn set_current_state(&mut self, state: PipelineStateInfo) {
        self.current_state = state;
    }

    pub fn register_state(&mut self, name: &str, state: PipelineStateInfo) {
        self.states.insert(name.to_string(), state);
    }

    pub fn get_state(&self, name: &str) -> Option<&PipelineStateInfo> {
        self.states.get(name)
    }

    pub fn set_bloom_enabled(&mut self, enabled: bool) {
        self.current_state.bloom_enabled = enabled;
    }

    pub fn set_hdr_enabled(&mut self, enabled: bool) {
        self.current_state.hdr_enabled = enabled;
    }

    pub fn set_shadow_enabled(&mut self, enabled: bool) {
        self.current_state.shadow_enabled = enabled;
    }

    pub fn set_postprocess_enabled(&mut self, enabled: bool) {
        self.current_state.postprocess_enabled = enabled;
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
        assert!(!mgr.get_current_state().bloom_enabled);
        assert!(!mgr.get_current_state().hdr_enabled);
    }

    #[test]
    fn test_pipeline_state_manager_set_bloom() {
        let mut mgr = PipelineStateManager::new();
        mgr.set_bloom_enabled(true);
        assert!(mgr.get_current_state().bloom_enabled);
    }

    #[test]
    fn test_pipeline_state_manager_register() {
        let mut mgr = PipelineStateManager::new();
        let state = PipelineStateInfo { hdr_enabled: true, ..Default::default() };
        mgr.register_state("hdr", state);
        assert!(mgr.get_state("hdr").unwrap().hdr_enabled);
        assert!(mgr.get_state("nonexistent").is_none());
    }
}
