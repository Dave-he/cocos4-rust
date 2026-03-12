/****************************************************************************
Rust port of Cocos Creator GFX Descriptor Set
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorType {
    Unknown = 0,
    UniformBuffer = 1,
    DynamicUniformBuffer = 2,
    Storage = 4,
    DynamicStorage = 8,
    SamplerTexture = 16,
    Sampler = 32,
    Texture = 64,
    StorageTexture = 128,
    InputAttachment = 256,
}

#[derive(Debug, Clone)]
pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub count: u32,
    pub stage_flags: u32,
}

impl Default for DescriptorSetLayoutBinding {
    fn default() -> Self {
        DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: DescriptorType::UniformBuffer,
            count: 1,
            stage_flags: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DescriptorSetLayoutInfo {
    pub bindings: Vec<DescriptorSetLayoutBinding>,
}

impl Default for DescriptorSetLayoutInfo {
    fn default() -> Self {
        DescriptorSetLayoutInfo {
            bindings: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct GfxDescriptorSetLayout {
    pub id: u32,
    pub info: DescriptorSetLayoutInfo,
}

impl GfxDescriptorSetLayout {
    pub fn new(id: u32, info: DescriptorSetLayoutInfo) -> Self {
        GfxDescriptorSetLayout { id, info }
    }

    pub fn get_binding_count(&self) -> usize {
        self.info.bindings.len()
    }
}

#[derive(Debug)]
pub struct GfxDescriptorSet {
    pub id: u32,
    pub layout_id: u32,
}

impl GfxDescriptorSet {
    pub fn new(id: u32, layout_id: u32) -> Self {
        GfxDescriptorSet { id, layout_id }
    }
}
