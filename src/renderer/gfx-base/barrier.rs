/****************************************************************************
Rust port of Cocos Creator GFX Barrier
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessFlagBit {
    None = 0,
    IndirectBuffer = 1 << 0,
    IndexBuffer = 1 << 1,
    VertexBuffer = 1 << 2,
    VertexShaderReadUniform = 1 << 3,
    VertexShaderReadSampledImage = 1 << 4,
    FragmentShaderReadUniform = 1 << 5,
    FragmentShaderReadSampledImage = 1 << 6,
    ColorAttachmentWrite = 1 << 7,
    DepthStencilAttachmentWrite = 1 << 8,
    TransferRead = 1 << 9,
    TransferWrite = 1 << 10,
    HostRead = 1 << 11,
    HostWrite = 1 << 12,
}

#[derive(Debug, Clone)]
pub struct GeneralBarrierInfo {
    pub prev_accesses: u64,
    pub next_accesses: u64,
}

impl Default for GeneralBarrierInfo {
    fn default() -> Self {
        GeneralBarrierInfo {
            prev_accesses: 0,
            next_accesses: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_barrier_default() {
        let b = GeneralBarrierInfo::default();
        assert_eq!(b.prev_accesses, 0);
        assert_eq!(b.next_accesses, 0);
    }

    #[test]
    fn test_texture_barrier_default() {
        let b = TextureBarrierInfo::default();
        assert!(!b.discard_contents);
        assert_eq!(b.range_mip_levels, 1);
        assert_eq!(b.range_layers, 1);
    }

    #[test]
    fn test_buffer_barrier_default() {
        let b = BufferBarrierInfo::default();
        assert!(!b.discard_contents);
        assert_eq!(b.offset, 0);
        assert_eq!(b.size, 0);
    }

    #[test]
    fn test_access_flags() {
        assert_ne!(AccessFlagBit::None as u32, AccessFlagBit::IndexBuffer as u32);
        assert_eq!(AccessFlagBit::None as u32, 0);
    }
}

#[derive(Debug, Clone)]
pub struct TextureBarrierInfo {
    pub prev_accesses: u64,
    pub next_accesses: u64,
    pub discard_contents: bool,
    pub src_queue_type: u32,
    pub dst_queue_type: u32,
    pub texture_id: u32,
    pub range_mip_levels: u32,
    pub range_layers: u32,
}

impl Default for TextureBarrierInfo {
    fn default() -> Self {
        TextureBarrierInfo {
            prev_accesses: 0,
            next_accesses: 0,
            discard_contents: false,
            src_queue_type: 0,
            dst_queue_type: 0,
            texture_id: 0,
            range_mip_levels: 1,
            range_layers: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferBarrierInfo {
    pub prev_accesses: u64,
    pub next_accesses: u64,
    pub offset: u32,
    pub size: u32,
    pub discard_contents: bool,
    pub src_queue_type: u32,
    pub dst_queue_type: u32,
    pub buffer_id: u32,
}

impl Default for BufferBarrierInfo {
    fn default() -> Self {
        BufferBarrierInfo {
            prev_accesses: 0,
            next_accesses: 0,
            offset: 0,
            size: 0,
            discard_contents: false,
            src_queue_type: 0,
            dst_queue_type: 0,
            buffer_id: 0,
        }
    }
}
