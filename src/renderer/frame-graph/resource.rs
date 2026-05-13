/****************************************************************************
Rust port of Cocos Creator Resource / ResourceEntry / ResourceAllocator
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;
use super::VirtualResourceKind;

#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: crate::renderer::gfx_base::Format,
    pub usage: crate::renderer::gfx_base::TextureUsage,
    pub flags: crate::renderer::gfx_base::TextureFlags,
    pub mip_count: u32,
    pub array_layer: u32,
    pub sample_count: crate::renderer::gfx_base::SampleCount,
    pub texture_type: crate::renderer::gfx_base::TextureType,
}

impl Default for TextureDescriptor {
    fn default() -> Self {
        TextureDescriptor {
            width: 0,
            height: 0,
            depth: 1,
            format: crate::renderer::gfx_base::Format::RGBA8,
            usage: crate::renderer::gfx_base::TextureUsage::SAMPLED
                | crate::renderer::gfx_base::TextureUsage::COLOR_ATTACHMENT,
            flags: crate::renderer::gfx_base::TextureFlags::NONE,
            mip_count: 1,
            array_layer: 1,
            sample_count: crate::renderer::gfx_base::SampleCount::X1,
            texture_type: crate::renderer::gfx_base::TextureType::Tex2D,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    pub size: u32,
    pub usage: crate::renderer::gfx_base::BufferUsage,
    pub memory_usage: crate::renderer::gfx_base::MemoryUsage,
    pub flags: crate::renderer::gfx_base::BufferFlags,
    pub stride: u32,
}

impl Default for BufferDescriptor {
    fn default() -> Self {
        BufferDescriptor {
            size: 0,
            usage: crate::renderer::gfx_base::BufferUsage::VERTEX,
            memory_usage: crate::renderer::gfx_base::MemoryUsage::DEVICE,
            flags: crate::renderer::gfx_base::BufferFlags::NONE,
            stride: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderPassDescriptor {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_stencil_attachment: Option<DepthStencilAttachment>,
    pub subpasses: Vec<SubpassDescriptor>,
    pub dependencies: Vec<SubpassDependency>,
}

#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub format: crate::renderer::gfx_base::Format,
    pub load_op: crate::renderer::gfx_base::LoadOp,
    pub store_op: crate::renderer::gfx_base::StoreOp,
    pub sample_count: crate::renderer::gfx_base::SampleCount,
    pub begin_accesses: crate::renderer::gfx_base::AccessFlags,
    pub end_accesses: crate::renderer::gfx_base::AccessFlags,
}

impl Default for ColorAttachment {
    fn default() -> Self {
        ColorAttachment {
            format: crate::renderer::gfx_base::Format::RGBA8,
            load_op: crate::renderer::gfx_base::LoadOp::Clear,
            store_op: crate::renderer::gfx_base::StoreOp::Store,
            sample_count: crate::renderer::gfx_base::SampleCount::X1,
            begin_accesses: crate::renderer::gfx_base::AccessFlags::NONE,
            end_accesses: crate::renderer::gfx_base::AccessFlags::NONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepthStencilAttachment {
    pub format: crate::renderer::gfx_base::Format,
    pub depth_load_op: crate::renderer::gfx_base::LoadOp,
    pub depth_store_op: crate::renderer::gfx_base::StoreOp,
    pub stencil_load_op: crate::renderer::gfx_base::LoadOp,
    pub stencil_store_op: crate::renderer::gfx_base::StoreOp,
    pub sample_count: crate::renderer::gfx_base::SampleCount,
    pub begin_accesses: crate::renderer::gfx_base::AccessFlags,
    pub end_accesses: crate::renderer::gfx_base::AccessFlags,
}

impl Default for DepthStencilAttachment {
    fn default() -> Self {
        DepthStencilAttachment {
            format: crate::renderer::gfx_base::Format::DepthStencil,
            depth_load_op: crate::renderer::gfx_base::LoadOp::Clear,
            depth_store_op: crate::renderer::gfx_base::StoreOp::Store,
            stencil_load_op: crate::renderer::gfx_base::LoadOp::Clear,
            stencil_store_op: crate::renderer::gfx_base::StoreOp::Store,
            sample_count: crate::renderer::gfx_base::SampleCount::X1,
            begin_accesses: crate::renderer::gfx_base::AccessFlags::NONE,
            end_accesses: crate::renderer::gfx_base::AccessFlags::NONE,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubpassDescriptor {
    pub color_attachments: Vec<u32>,
    pub input_attachments: Vec<u32>,
    pub depth_stencil_attachment: Option<u32>,
    pub resolve_attachments: Vec<u32>,
    pub preserve_attachments: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_accesses: crate::renderer::gfx_base::AccessFlags,
    pub dst_accesses: crate::renderer::gfx_base::AccessFlags,
}

impl Default for SubpassDependency {
    fn default() -> Self {
        SubpassDependency {
            src_subpass: crate::renderer::gfx_base::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_accesses: crate::renderer::gfx_base::AccessFlags::NONE,
            dst_accesses: crate::renderer::gfx_base::AccessFlags::NONE,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FramebufferDescriptor {
    pub render_pass: u32,
    pub color_attachments: Vec<u32>,
    pub depth_stencil_attachment: Option<u32>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Resource {
    id: u32,
    kind: VirtualResourceKind,
    descriptor: ResourceDescriptor,
    device_object_id: Option<u32>,
    imported: bool,
}

#[derive(Debug, Clone)]
pub enum ResourceDescriptor {
    Texture(TextureDescriptor),
    Buffer(BufferDescriptor),
    RenderPass(RenderPassDescriptor),
    Framebuffer(FramebufferDescriptor),
}

impl ResourceDescriptor {
    pub fn texture_desc(&self) -> Option<&TextureDescriptor> {
        match self {
            ResourceDescriptor::Texture(d) => Some(d),
            _ => None,
        }
    }

    pub fn buffer_desc(&self) -> Option<&BufferDescriptor> {
        match self {
            ResourceDescriptor::Buffer(d) => Some(d),
            _ => None,
        }
    }
}

impl Default for ResourceDescriptor {
    fn default() -> Self {
        ResourceDescriptor::Texture(TextureDescriptor::default())
    }
}

impl Resource {
    pub fn new_texture(id: u32, desc: TextureDescriptor) -> Self {
        Resource {
            id,
            kind: VirtualResourceKind::Texture,
            descriptor: ResourceDescriptor::Texture(desc),
            device_object_id: None,
            imported: false,
        }
    }

    pub fn new_buffer(id: u32, desc: BufferDescriptor) -> Self {
        Resource {
            id,
            kind: VirtualResourceKind::Buffer,
            descriptor: ResourceDescriptor::Buffer(desc),
            device_object_id: None,
            imported: false,
        }
    }

    pub fn new_imported(id: u32, kind: VirtualResourceKind, device_object_id: u32) -> Self {
        let descriptor = match kind {
            VirtualResourceKind::Texture => ResourceDescriptor::Texture(TextureDescriptor::default()),
            VirtualResourceKind::Buffer => ResourceDescriptor::Buffer(BufferDescriptor::default()),
        };
        Resource {
            id,
            kind,
            descriptor,
            device_object_id: Some(device_object_id),
            imported: true,
        }
    }

    pub fn create_transient(&mut self) -> Option<u32> {
        if self.imported {
            return None;
        }
        self.device_object_id = Some(self.id);
        self.device_object_id
    }

    pub fn create_persistent(&mut self, device_object_id: u32) {
        self.device_object_id = Some(device_object_id);
    }

    pub fn destroy_transient(&mut self) {
        if !self.imported {
            self.device_object_id = None;
        }
    }

    pub fn destroy_persistent(&mut self) {
        self.device_object_id = None;
    }

    pub fn get_device_object_id(&self) -> Option<u32> {
        self.device_object_id
    }

    pub fn get_descriptor(&self) -> &ResourceDescriptor {
        &self.descriptor
    }

    pub fn is_imported(&self) -> bool {
        self.imported
    }

    pub fn get_kind(&self) -> VirtualResourceKind {
        self.kind
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct ResourceEntry {
    resource: Resource,
    vr_index: u32,
}

impl ResourceEntry {
    pub fn new_transient(resource: Resource, vr_index: u32) -> Self {
        ResourceEntry { resource, vr_index }
    }

    pub fn new_imported(kind: VirtualResourceKind, device_object_id: u32, vr_index: u32) -> Self {
        let resource = Resource::new_imported(vr_index, kind, device_object_id);
        ResourceEntry { resource, vr_index }
    }

    pub fn request(&mut self) -> Option<u32> {
        self.resource.create_transient()
    }

    pub fn release(&mut self) {
        self.resource.destroy_transient();
    }

    pub fn get_device_object_id(&self) -> Option<u32> {
        self.resource.get_device_object_id()
    }

    pub fn get_resource(&self) -> &Resource {
        &self.resource
    }

    pub fn get_resource_mut(&mut self) -> &mut Resource {
        &mut self.resource
    }

    pub fn get_vr_index(&self) -> u32 {
        self.vr_index
    }

    pub fn is_imported(&self) -> bool {
        self.resource.is_imported()
    }
}

#[derive(Debug)]
pub struct ResourcePoolEntry {
    resource_id: u32,
    age: i64,
    descriptor_hash: u64,
}

pub struct ResourceAllocator {
    pools: HashMap<u64, Vec<ResourcePoolEntry>>,
    ages: HashMap<u32, i64>,
    current_age: u64,
    unused_frame_count: u64,
}

impl ResourceAllocator {
    pub fn new(unused_frame_count: u64) -> Self {
        ResourceAllocator {
            pools: HashMap::new(),
            ages: HashMap::new(),
            current_age: 0,
            unused_frame_count,
        }
    }

    pub fn alloc(&mut self, desc_hash: u64) -> u32 {
        if let Some(pool) = self.pools.get_mut(&desc_hash) {
            for entry in pool.iter_mut() {
                if entry.age < 0 || (self.current_age as i64 - entry.age) as u64 >= self.unused_frame_count {
                    entry.age = -1;
                    self.ages.insert(entry.resource_id, -1);
                    return entry.resource_id;
                }
            }
        }

        let resource_id = self.pools.values().flat_map(|v| v.iter().map(|e| e.resource_id)).max().unwrap_or(0) + 1;
        let entry = ResourcePoolEntry {
            resource_id,
            age: -1,
            descriptor_hash: desc_hash,
        };
        self.pools.entry(desc_hash).or_insert_with(Vec::new).push(entry);
        self.ages.insert(resource_id, -1);
        resource_id
    }

    pub fn free(&mut self, resource_id: u32) {
        if let Some(age) = self.ages.get_mut(&resource_id) {
            *age = self.current_age as i64;
        }
    }

    pub fn tick(&mut self) {
        self.current_age += 1;
    }

    pub fn gc(&mut self) {
        let threshold = self.current_age as i64 - self.unused_frame_count as i64;
        for pool in self.pools.values_mut() {
            pool.retain(|entry| entry.age < 0 || entry.age >= threshold);
        }
    }

    pub fn get_pool_size(&self) -> usize {
        self.pools.values().map(|v| v.len()).sum()
    }

    pub fn get_current_age(&self) -> u64 {
        self.current_age
    }
}

impl Default for ResourceAllocator {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_texture() {
        let desc = TextureDescriptor {
            width: 1024,
            height: 1024,
            ..Default::default()
        };
        let r = Resource::new_texture(0, desc);
        assert_eq!(r.get_kind(), VirtualResourceKind::Texture);
        assert!(!r.is_imported());
        assert!(r.get_device_object_id().is_none());
    }

    #[test]
    fn test_resource_buffer() {
        let desc = BufferDescriptor {
            size: 256,
            ..Default::default()
        };
        let r = Resource::new_buffer(1, desc);
        assert_eq!(r.get_kind(), VirtualResourceKind::Buffer);
        assert!(r.get_descriptor().buffer_desc().is_some());
    }

    #[test]
    fn test_resource_imported() {
        let r = Resource::new_imported(2, VirtualResourceKind::Texture, 100);
        assert!(r.is_imported());
        assert_eq!(r.get_device_object_id(), Some(100));
    }

    #[test]
    fn test_resource_transient_lifecycle() {
        let mut r = Resource::new_texture(0, TextureDescriptor::default());
        assert!(r.get_device_object_id().is_none());
        let obj = r.create_transient();
        assert!(obj.is_some());
        r.destroy_transient();
        assert!(r.get_device_object_id().is_none());
    }

    #[test]
    fn test_resource_persistent_lifecycle() {
        let mut r = Resource::new_texture(0, TextureDescriptor::default());
        r.create_persistent(42);
        assert_eq!(r.get_device_object_id(), Some(42));
        r.destroy_persistent();
        assert!(r.get_device_object_id().is_none());
    }

    #[test]
    fn test_resource_entry_transient() {
        let r = Resource::new_texture(0, TextureDescriptor::default());
        let mut entry = ResourceEntry::new_transient(r, 0);
        assert!(!entry.is_imported());
        entry.request();
        assert!(entry.get_device_object_id().is_some());
        entry.release();
        assert!(entry.get_device_object_id().is_none());
    }

    #[test]
    fn test_resource_entry_imported() {
        let mut entry = ResourceEntry::new_imported(VirtualResourceKind::Texture, 100, 0);
        assert!(entry.is_imported());
        assert_eq!(entry.get_device_object_id(), Some(100));
    }

    #[test]
    fn test_resource_allocator_alloc_free() {
        let mut allocator = ResourceAllocator::new(3);
        let id = allocator.alloc(12345);
        assert!(id > 0);
        allocator.free(id);
        allocator.tick();
        allocator.tick();
        allocator.tick();
        allocator.gc();
    }

    #[test]
    fn test_resource_allocator_reuse() {
        let mut allocator = ResourceAllocator::new(10);
        let id1 = allocator.alloc(100);
        allocator.free(id1);
        allocator.tick();
        let id2 = allocator.alloc(100);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_resource_allocator_tick() {
        let mut allocator = ResourceAllocator::new(3);
        assert_eq!(allocator.get_current_age(), 0);
        allocator.tick();
        assert_eq!(allocator.get_current_age(), 1);
    }
}
