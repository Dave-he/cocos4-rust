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
pub struct ShaderInfo {
    pub name: String,
    pub stages: Vec<ShaderStage>,
    pub attributes: Vec<Attribute>,
    pub blocks: Vec<UniformBlock>,
    pub samplers: Vec<UniformSamplerTexture>,
}

impl Default for ShaderInfo {
    fn default() -> Self {
        ShaderInfo {
            name: String::new(),
            stages: Vec::new(),
            attributes: Vec::new(),
            blocks: Vec::new(),
            samplers: Vec::new(),
        }
    }
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
