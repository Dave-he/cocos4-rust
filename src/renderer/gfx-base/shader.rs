/****************************************************************************
Rust port of Cocos Creator GFX Shader
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::ShaderStageFlagBit;

#[derive(Debug, Clone)]
pub struct ShaderStage {
    pub stage: ShaderStageFlagBit,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub format: super::Format,
    pub is_normalized: bool,
    pub stream: u32,
    pub is_instanced: bool,
    pub location: u32,
}

impl Default for Attribute {
    fn default() -> Self {
        Attribute {
            name: String::new(),
            format: super::Format::Unknown,
            is_normalized: false,
            stream: 0,
            is_instanced: false,
            location: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UniformBlock {
    pub set: u32,
    pub binding: u32,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct UniformSamplerTexture {
    pub set: u32,
    pub binding: u32,
    pub name: String,
    pub tex_type: super::TextureType,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct UniformStorageBuffer {
    pub set: u32,
    pub binding: u32,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct UniformStorageImage {
    pub set: u32,
    pub binding: u32,
    pub name: String,
    pub tex_type: super::TextureType,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct UniformInputAttachment {
    pub set: u32,
    pub binding: u32,
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ShaderInfo {
    pub name: String,
    pub stages: Vec<ShaderStage>,
    pub attributes: Vec<Attribute>,
    pub blocks: Vec<UniformBlock>,
    pub samplers: Vec<UniformSamplerTexture>,
    pub storage_buffers: Vec<UniformStorageBuffer>,
    pub storage_images: Vec<UniformStorageImage>,
    pub subpass_inputs: Vec<UniformInputAttachment>,
}

#[derive(Debug)]
pub struct GfxShader {
    pub id: u32,
    pub info: ShaderInfo,
}

impl GfxShader {
    pub fn new(id: u32, info: ShaderInfo) -> Self {
        GfxShader { id, info }
    }

    pub fn get_name(&self) -> &str {
        &self.info.name
    }

    pub fn get_attribute_count(&self) -> usize {
        self.info.attributes.len()
    }

    pub fn get_block_count(&self) -> usize {
        self.info.blocks.len()
    }

    pub fn get_sampler_count(&self) -> usize {
        self.info.samplers.len()
    }

    pub fn get_storage_buffer_count(&self) -> usize {
        self.info.storage_buffers.len()
    }

    pub fn get_storage_image_count(&self) -> usize {
        self.info.storage_images.len()
    }

    pub fn get_subpass_input_count(&self) -> usize {
        self.info.subpass_inputs.len()
    }

    pub fn destroy(&mut self) {
        self.info.stages.clear();
        self.info.attributes.clear();
        self.info.blocks.clear();
        self.info.samplers.clear();
        self.info.storage_buffers.clear();
        self.info.storage_images.clear();
        self.info.subpass_inputs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_new() {
        let info = ShaderInfo {
            name: "TestShader".to_string(),
            ..Default::default()
        };
        let shader = GfxShader::new(1, info);
        assert_eq!(shader.get_name(), "TestShader");
        assert_eq!(shader.get_attribute_count(), 0);
        assert_eq!(shader.get_block_count(), 0);
    }
}
