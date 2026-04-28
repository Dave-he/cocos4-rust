/****************************************************************************
Rust port of Cocos Creator GFX State
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub attributes: Vec<super::VertexAttribute>,
}

#[derive(Debug, Clone, Default)]
pub struct PipelineLayoutInfo {
    pub set_layouts: Vec<u32>,
}

#[derive(Debug)]
pub struct GfxPipelineLayout {
    pub id: u32,
    pub info: PipelineLayoutInfo,
}

impl GfxPipelineLayout {
    pub fn new(id: u32, info: PipelineLayoutInfo) -> Self {
        GfxPipelineLayout { id, info }
    }

    pub fn get_set_layout_count(&self) -> usize {
        self.info.set_layouts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_layout_new() {
        let layout = GfxPipelineLayout::new(1, PipelineLayoutInfo::default());
        assert_eq!(layout.id, 1);
        assert_eq!(layout.get_set_layout_count(), 0);
    }

    #[test]
    fn test_pipeline_layout_with_sets() {
        let layout = GfxPipelineLayout::new(1, PipelineLayoutInfo {
            set_layouts: vec![0, 1, 2],
        });
        assert_eq!(layout.get_set_layout_count(), 3);
    }

    #[test]
    fn test_input_state_default() {
        let state = InputState::default();
        assert!(state.attributes.is_empty());
    }
}
