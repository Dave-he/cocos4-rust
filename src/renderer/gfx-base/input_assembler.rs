/****************************************************************************
Rust port of Cocos Creator GFX Input Assembler
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::Format;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    Position = 0,
    Normal = 1,
    Tangent = 2,
    Bitangent = 3,
    Weights = 4,
    Joints = 5,
    Color = 6,
    TexCoord = 7,
    TexCoord1 = 8,
    TexCoord2 = 9,
    TexCoord3 = 10,
    TexCoord4 = 11,
    TexCoord5 = 12,
    TexCoord6 = 13,
    TexCoord7 = 14,
    TexCoord8 = 15,
    BatchingId = 16,
    Count = 17,
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub name: String,
    pub semantic: Attribute,
    pub format: Format,
    pub binding: u32,
    pub offset: u32,
    pub is_normalized: bool,
    pub is_instanced: bool,
    pub location: u32,
}

impl Default for VertexAttribute {
    fn default() -> Self {
        VertexAttribute {
            name: "a_position".to_string(),
            semantic: Attribute::Position,
            format: Format::RGB32F,
            binding: 0,
            offset: 0,
            is_normalized: false,
            is_instanced: false,
            location: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputAssemblerInfo {
    pub attributes: Vec<VertexAttribute>,
    pub vertex_buffers: Vec<u32>,
    pub index_buffer: Option<u32>,
    pub indirect_buffer: Option<u32>,
}

impl Default for InputAssemblerInfo {
    fn default() -> Self {
        InputAssemblerInfo {
            attributes: Vec::new(),
            vertex_buffers: Vec::new(),
            index_buffer: None,
            indirect_buffer: None,
        }
    }
}

#[derive(Debug)]
pub struct GfxInputAssembler {
    pub id: u32,
    pub info: InputAssemblerInfo,
    pub draw_count: u32,
    pub first_index: u32,
    pub index_count: u32,
    pub vertex_count: u32,
}

impl GfxInputAssembler {
    pub fn new(id: u32, info: InputAssemblerInfo) -> Self {
        GfxInputAssembler {
            id,
            info,
            draw_count: 1,
            first_index: 0,
            index_count: 0,
            vertex_count: 0,
        }
    }

    pub fn get_attribute_count(&self) -> usize {
        self.info.attributes.len()
    }

    pub fn has_index_buffer(&self) -> bool {
        self.info.index_buffer.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_assembler_new() {
        let mut info = InputAssemblerInfo::default();
        info.attributes.push(VertexAttribute {
            name: "a_position".to_string(),
            semantic: Attribute::Position,
            format: Format::RGB32F,
            ..Default::default()
        });
        let ia = GfxInputAssembler::new(1, info);
        assert_eq!(ia.get_attribute_count(), 1);
        assert!(!ia.has_index_buffer());
    }
}
