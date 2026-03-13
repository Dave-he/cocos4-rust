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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_set_layout_new() {
        let layout = GfxDescriptorSetLayout::new(1, DescriptorSetLayoutInfo::default());
        assert_eq!(layout.id, 1);
        assert_eq!(layout.get_binding_count(), 0);
    }

    #[test]
    fn test_descriptor_set_layout_with_bindings() {
        let info = DescriptorSetLayoutInfo {
            bindings: vec![
                DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UniformBuffer,
                    count: 1,
                    stage_flags: 0b1,
                },
                DescriptorSetLayoutBinding {
                    binding: 1,
                    descriptor_type: DescriptorType::SamplerTexture,
                    count: 4,
                    stage_flags: 0b10,
                },
            ],
        };
        let layout = GfxDescriptorSetLayout::new(1, info);
        assert_eq!(layout.get_binding_count(), 2);
    }

    #[test]
    fn test_descriptor_set_new() {
        let ds = GfxDescriptorSet::new(5, 1);
        assert_eq!(ds.id, 5);
        assert_eq!(ds.layout_id, 1);
    }

    #[test]
    fn test_descriptor_types() {
        assert_ne!(DescriptorType::UniformBuffer as u32, DescriptorType::Sampler as u32);
        assert_eq!(DescriptorType::Unknown as u32, 0);
    }
}
