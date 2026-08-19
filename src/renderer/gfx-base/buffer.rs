/****************************************************************************
Rust port of Cocos Creator GFX Buffer
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{BufferFlags, BufferUsage, MemoryUsage};

#[derive(Debug, Clone)]
pub struct BufferInfo {
    pub usage: BufferUsage,
    pub mem_usage: MemoryUsage,
    pub size: u32,
    pub stride: u32,
    pub flags: BufferFlags,
}

impl Default for BufferInfo {
    fn default() -> Self {
        BufferInfo {
            usage: BufferUsage::VERTEX,
            mem_usage: MemoryUsage::DEVICE,
            size: 0,
            stride: 0,
            flags: BufferFlags::NONE,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BufferViewInfo {
    pub buffer_id: u32,
    pub offset: u32,
    pub range: u32,
}

#[derive(Debug)]
pub struct GfxBuffer {
    pub id: u32,
    pub info: BufferInfo,
    pub data: Vec<u8>,
}

impl GfxBuffer {
    pub fn new(id: u32, info: BufferInfo) -> Self {
        let size = info.size as usize;
        GfxBuffer {
            id,
            info,
            data: vec![0u8; size],
        }
    }

    pub fn resize(&mut self, new_size: u32) {
        self.info.size = new_size;
        self.data.resize(new_size as usize, 0);
    }

    pub fn update(&mut self, data: &[u8], offset: u32) {
        let start = offset as usize;
        let end = start + data.len();
        if end <= self.data.len() {
            self.data[start..end].copy_from_slice(data);
        }
    }

    pub fn get_size(&self) -> u32 {
        self.info.size
    }

    pub fn get_stride(&self) -> u32 {
        self.info.stride
    }

    pub fn get_count(&self) -> u32 {
        self.info.size.checked_div(self.info.stride).unwrap_or(0)
    }

    pub fn new_view(id: u32, info: BufferViewInfo) -> Self {
        GfxBuffer {
            id,
            info: BufferInfo {
                usage: BufferUsage::NONE,
                mem_usage: MemoryUsage::NONE,
                size: info.range,
                stride: 0,
                flags: BufferFlags::NONE,
            },
            data: Vec::new(),
        }
    }

    pub fn is_view(&self) -> bool {
        self.data.is_empty() && self.info.size > 0
    }

    pub fn flush(&mut self) {}

    pub fn destroy(&mut self) {
        self.data.clear();
        self.info.size = 0;
    }

    pub fn get_usage(&self) -> BufferUsage {
        self.info.usage
    }

    pub fn get_mem_usage(&self) -> MemoryUsage {
        self.info.mem_usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_new() {
        let info = BufferInfo {
            usage: BufferUsage::VERTEX,
            mem_usage: MemoryUsage::DEVICE,
            size: 256,
            stride: 12,
            flags: BufferFlags::NONE,
        };
        let buf = GfxBuffer::new(1, info);
        assert_eq!(buf.get_size(), 256);
        assert_eq!(buf.get_stride(), 12);
        assert_eq!(buf.get_count(), 21);
    }

    #[test]
    fn test_buffer_update() {
        let info = BufferInfo {
            size: 16,
            stride: 4,
            ..Default::default()
        };
        let mut buf = GfxBuffer::new(1, info);
        let data: [u8; 4] = [1, 2, 3, 4];
        buf.update(&data, 0);
        assert_eq!(buf.data[0..4], [1, 2, 3, 4]);
    }

    #[test]
    fn test_buffer_resize() {
        let info = BufferInfo {
            size: 64,
            ..Default::default()
        };
        let mut buf = GfxBuffer::new(1, info);
        assert_eq!(buf.get_size(), 64);
        buf.resize(128);
        assert_eq!(buf.get_size(), 128);
    }
}
