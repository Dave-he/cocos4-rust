#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassKind {
    Raster,
    Compute,
    Copy,
    Move,
    Raytrace,
    Present,
    Resolve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    ManagedBuffer,
    ManagedTexture,
    PersistentBuffer,
    PersistentTexture,
    ImportedBuffer,
    ImportedTexture,
}

impl Default for ResourceKind {
    fn default() -> Self {
        Self::ManagedTexture
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderGraphVersion {
    V1,
}

#[derive(Debug, Clone)]
pub struct SubpassDesc {
    pub inputs: Vec<u32>,
    pub outputs: Vec<u32>,
    pub resolves: Vec<u32>,
    pub preserves: Vec<u32>,
    pub depth_stencil: Option<u32>,
}

impl Default for SubpassDesc {
    fn default() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            resolves: Vec::new(),
            preserves: Vec::new(),
            depth_stencil: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PassDesc {
    pub name: String,
    pub kind: PassKind,
    pub reads: Vec<u32>,
    pub writes: Vec<u32>,
    pub subpasses: Vec<SubpassDesc>,
}

impl Default for PassDesc {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: PassKind::Raster,
            reads: Vec::new(),
            writes: Vec::new(),
            subpasses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderGraphSettings {
    pub enable_barrier_optimization: bool,
    pub enable_memory_aliasing: bool,
    pub max_vertices: u32,
    pub max_descriptor_sets: u32,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceDesc {
    pub name: String,
    pub kind: ResourceKind,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: u32,
    pub sample_count: u32,
    pub mip_levels: u32,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutDesc {
    pub name: String,
    pub bindings: Vec<u32>,
    pub push_constant_ranges: Vec<(u32, u32, u32)>,
}

#[derive(Debug, Clone, Default)]
pub struct DescriptorDB {
    pub sets: Vec<LayoutDesc>,
    pub pipeline_layouts: Vec<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct PipelineLayoutDesc {
    pub set_layouts: Vec<u32>,
    pub push_constant_ranges: Vec<(u32, u32, u32)>,
}
