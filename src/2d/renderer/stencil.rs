/****************************************************************************
Rust port of Cocos Creator Stencil Manager
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilStage {
    Disabled = 0,
    Clear = 1,
    EnabledMask = 2,
    VisibleContent = 3,
    ClearInverted = 4,
    EnabledInvertedMask = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonFunc {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilOp {
    Zero = 0,
    Keep = 1,
    Replace = 2,
    IncrSat = 3,
    DecrSat = 4,
    Invert = 5,
    IncrWrap = 6,
    DecrWrap = 7,
}

#[derive(Debug, Clone, Copy)]
pub struct StencilStateInfo {
    pub stencil_test: bool,
    pub func: ComparisonFunc,
    pub read_mask: u32,
    pub write_mask: u32,
    pub fail_op: StencilOp,
    pub z_fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub ref_val: u32,
}

impl Default for StencilStateInfo {
    fn default() -> Self {
        StencilStateInfo {
            stencil_test: false,
            func: ComparisonFunc::Always,
            read_mask: 0xFF,
            write_mask: 0xFF,
            fail_op: StencilOp::Keep,
            z_fail_op: StencilOp::Keep,
            pass_op: StencilOp::Keep,
            ref_val: 0,
        }
    }
}

pub trait StencilManager: RefCounted {
    fn get_stage(&self) -> StencilStage;
    fn set_stage(&mut self, stage: StencilStage);
    fn get_mask_stack_size(&self) -> usize;
    fn enter_level(&mut self);
    fn exit_level(&mut self);
    fn get_front_state(&self) -> StencilStateInfo;
    fn get_back_state(&self) -> StencilStateInfo;
    fn reset(&mut self);
    fn get_stencil_ref(&self) -> u32;
}

#[derive(Debug)]
pub struct StencilManagerImpl {
    pub stage: StencilStage,
    pub mask_stack: Vec<u32>,
    pub front_state: StencilStateInfo,
    pub back_state: StencilStateInfo,
    pub max_level: u32,
    ref_count: RefCountedImpl,
}

impl StencilManagerImpl {
    pub fn new() -> Self {
        StencilManagerImpl {
            stage: StencilStage::Disabled,
            mask_stack: Vec::new(),
            front_state: StencilStateInfo::default(),
            back_state: StencilStateInfo::default(),
            max_level: 0,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn get_current_ref(&self) -> u32 {
        (1u32 << self.mask_stack.len()).saturating_sub(1)
    }

    fn update_state(&mut self) {
        let stencil_ref = self.get_current_ref();
        match self.stage {
            StencilStage::Disabled => {
                self.front_state.stencil_test = false;
                self.back_state.stencil_test = false;
            }
            StencilStage::Clear | StencilStage::ClearInverted => {
                self.front_state.stencil_test = true;
                self.front_state.func = ComparisonFunc::Always;
                self.front_state.pass_op = StencilOp::Zero;
                self.front_state.write_mask = 0xFF;
                self.front_state.ref_val = 0;
                self.back_state = self.front_state;
            }
            StencilStage::EnabledMask | StencilStage::EnabledInvertedMask => {
                self.front_state.stencil_test = true;
                self.front_state.func = ComparisonFunc::Always;
                self.front_state.pass_op = StencilOp::Replace;
                self.front_state.write_mask = stencil_ref;
                self.front_state.ref_val = stencil_ref;
                self.back_state = self.front_state;
            }
            StencilStage::VisibleContent => {
                self.front_state.stencil_test = true;
                self.front_state.func = ComparisonFunc::Equal;
                self.front_state.pass_op = StencilOp::Keep;
                self.front_state.read_mask = stencil_ref;
                self.front_state.ref_val = stencil_ref;
                self.back_state = self.front_state;
            }
        }
    }
}

impl Default for StencilManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl RefCounted for StencilManagerImpl {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

impl StencilManager for StencilManagerImpl {
    fn get_stage(&self) -> StencilStage { self.stage }

    fn set_stage(&mut self, stage: StencilStage) {
        self.stage = stage;
        self.update_state();
    }

    fn get_mask_stack_size(&self) -> usize { self.mask_stack.len() }

    fn enter_level(&mut self) {
        let level = self.mask_stack.len() as u32;
        self.mask_stack.push(level);
        if level + 1 > self.max_level {
            self.max_level = level + 1;
        }
        self.update_state();
    }

    fn exit_level(&mut self) {
        self.mask_stack.pop();
        self.update_state();
    }

    fn get_front_state(&self) -> StencilStateInfo { self.front_state }
    fn get_back_state(&self) -> StencilStateInfo { self.back_state }

    fn reset(&mut self) {
        self.stage = StencilStage::Disabled;
        self.mask_stack.clear();
        self.front_state = StencilStateInfo::default();
        self.back_state = StencilStateInfo::default();
        self.max_level = 0;
    }

    fn get_stencil_ref(&self) -> u32 {
        self.get_current_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stencil_new() {
        let sm = StencilManagerImpl::new();
        assert_eq!(sm.get_stage(), StencilStage::Disabled);
        assert_eq!(sm.get_mask_stack_size(), 0);
    }

    #[test]
    fn test_enter_exit_level() {
        let mut sm = StencilManagerImpl::new();
        sm.enter_level();
        assert_eq!(sm.get_mask_stack_size(), 1);
        sm.enter_level();
        assert_eq!(sm.get_mask_stack_size(), 2);
        sm.exit_level();
        assert_eq!(sm.get_mask_stack_size(), 1);
    }

    #[test]
    fn test_set_stage_enabled_mask() {
        let mut sm = StencilManagerImpl::new();
        sm.enter_level();
        sm.set_stage(StencilStage::EnabledMask);
        let front = sm.get_front_state();
        assert!(front.stencil_test);
        assert_eq!(front.func, ComparisonFunc::Always);
        assert_eq!(front.pass_op, StencilOp::Replace);
    }

    #[test]
    fn test_set_stage_visible_content() {
        let mut sm = StencilManagerImpl::new();
        sm.enter_level();
        sm.set_stage(StencilStage::VisibleContent);
        let front = sm.get_front_state();
        assert!(front.stencil_test);
        assert_eq!(front.func, ComparisonFunc::Equal);
    }

    #[test]
    fn test_reset() {
        let mut sm = StencilManagerImpl::new();
        sm.enter_level();
        sm.set_stage(StencilStage::EnabledMask);
        sm.reset();
        assert_eq!(sm.get_stage(), StencilStage::Disabled);
        assert_eq!(sm.get_mask_stack_size(), 0);
    }
}
