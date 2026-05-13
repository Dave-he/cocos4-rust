/****************************************************************************
Rust port of Cocos Creator ForwardFlow
Original C++ version Copyright (c) 2020-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::super::render_flow::{RenderFlow, RenderFlowInfo};

#[derive(Debug)]
pub struct ForwardFlow {
    pub base: RenderFlow,
}

impl ForwardFlow {
    pub fn new() -> Self {
        ForwardFlow {
            base: RenderFlow::new("ForwardFlow", 0),
        }
    }

    pub fn get_initialize_info() -> RenderFlowInfo {
        RenderFlowInfo {
            name: "ForwardFlow".to_string(),
            priority: 0,
            stages: Vec::new(),
            tag: 0,
        }
    }

    pub fn initialize(&mut self, info: RenderFlowInfo) -> bool {
        self.base = RenderFlow::with_info(info);
        true
    }

    pub fn activate(&mut self) {
        self.base.activate();
    }

    pub fn destroy(&mut self) {
        self.base.destroy();
    }

    pub fn render(&mut self, _camera_id: u64) {
    }
}

impl Default for ForwardFlow {
    fn default() -> Self {
        ForwardFlow::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::render_flow::RenderFlowInfo;

    #[test]
    fn test_forward_flow_new() {
        let flow = ForwardFlow::new();
        assert_eq!(flow.base.name, "ForwardFlow");
    }

    #[test]
    fn test_forward_flow_initialize() {
        let mut flow = ForwardFlow::new();
        let info = ForwardFlow::get_initialize_info();
        assert!(flow.initialize(info));
        assert_eq!(flow.base.name, "ForwardFlow");
    }

    #[test]
    fn test_forward_flow_get_initialize_info() {
        let info = ForwardFlow::get_initialize_info();
        assert_eq!(info.name, "ForwardFlow");
    }

    #[test]
    fn test_forward_flow_default() {
        let flow = ForwardFlow::default();
        assert_eq!(flow.base.name, "ForwardFlow");
    }
}
