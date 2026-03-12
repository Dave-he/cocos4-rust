/****************************************************************************
Rust port of Cocos Creator GFX Device
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{
    BufferInfo, GfxBuffer, GfxSampler, GfxShader, GfxTexture, QueueType, SamplerInfo,
    ShaderInfo, TextureInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStatus {
    Low = 0,
    Medium = 1,
    High = 2,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub max_vertex_attribs: u32,
    pub max_vertex_uniform_vectors: u32,
    pub max_fragment_uniform_vectors: u32,
    pub max_texture_size: u32,
    pub max_texture_units: u32,
    pub max_vertex_texture_units: u32,
    pub max_combined_texture_units: u32,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        DeviceInfo {
            name: "Software Device".to_string(),
            max_vertex_attribs: 16,
            max_vertex_uniform_vectors: 128,
            max_fragment_uniform_vectors: 64,
            max_texture_size: 4096,
            max_texture_units: 16,
            max_vertex_texture_units: 8,
            max_combined_texture_units: 24,
        }
    }
}

#[derive(Debug)]
pub struct GfxDevice {
    pub info: DeviceInfo,
    next_id: u32,
    pub num_draw_calls: u32,
    pub num_instances: u32,
    pub num_tris: u32,
}

impl GfxDevice {
    pub fn new(info: DeviceInfo) -> Self {
        GfxDevice {
            info,
            next_id: 1,
            num_draw_calls: 0,
            num_instances: 0,
            num_tris: 0,
        }
    }

    fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn create_buffer(&mut self, info: BufferInfo) -> GfxBuffer {
        let id = self.next_id();
        GfxBuffer::new(id, info)
    }

    pub fn create_texture(&mut self, info: TextureInfo) -> GfxTexture {
        let id = self.next_id();
        GfxTexture::new(id, info)
    }

    pub fn create_shader(&mut self, info: ShaderInfo) -> GfxShader {
        let id = self.next_id();
        GfxShader::new(id, info)
    }

    pub fn create_sampler(&mut self, info: SamplerInfo) -> GfxSampler {
        let id = self.next_id();
        GfxSampler::new(id, info)
    }

    pub fn get_memory_status(&self) -> MemoryStatus {
        MemoryStatus::Medium
    }

    pub fn get_num_draw_calls(&self) -> u32 {
        self.num_draw_calls
    }

    pub fn get_num_instances(&self) -> u32 {
        self.num_instances
    }

    pub fn get_num_tris(&self) -> u32 {
        self.num_tris
    }

    pub fn reset_stats(&mut self) {
        self.num_draw_calls = 0;
        self.num_instances = 0;
        self.num_tris = 0;
    }

    pub fn get_queue_type(&self) -> QueueType {
        QueueType::Graphics
    }
}

impl Default for GfxDevice {
    fn default() -> Self {
        Self::new(DeviceInfo::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;

    #[test]
    fn test_device_new() {
        let device = GfxDevice::default();
        assert_eq!(device.get_num_draw_calls(), 0);
        assert_eq!(device.get_memory_status(), MemoryStatus::Medium);
    }

    #[test]
    fn test_device_create_buffer() {
        let mut device = GfxDevice::default();
        let buf = device.create_buffer(BufferInfo {
            size: 256,
            stride: 12,
            ..Default::default()
        });
        assert_eq!(buf.get_size(), 256);
        assert!(buf.id > 0);
    }

    #[test]
    fn test_device_create_texture() {
        let mut device = GfxDevice::default();
        let tex = device.create_texture(TextureInfo {
            width: 512,
            height: 512,
            ..Default::default()
        });
        assert_eq!(tex.get_width(), 512);
        assert_eq!(tex.get_height(), 512);
        assert!(tex.id > 0);
    }

    #[test]
    fn test_device_create_shader() {
        let mut device = GfxDevice::default();
        let shader = device.create_shader(ShaderInfo {
            name: "Test".to_string(),
            ..Default::default()
        });
        assert_eq!(shader.get_name(), "Test");
        assert!(shader.id > 0);
    }

    #[test]
    fn test_device_reset_stats() {
        let mut device = GfxDevice::default();
        device.num_draw_calls = 100;
        device.num_tris = 500;
        device.reset_stats();
        assert_eq!(device.get_num_draw_calls(), 0);
        assert_eq!(device.get_num_tris(), 0);
    }
}
