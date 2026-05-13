
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatchingSchemes {
    #[default]
    None = 0,
    Instancing = 1,
}

pub struct InstancedAttributeBlock {
    pub buffer: Vec<u8>,
    pub views: Vec<Vec<f32>>,
    pub attributes: Vec<crate::renderer::gfx_base::shader::Attribute>,
}

impl Default for InstancedAttributeBlock {
    fn default() -> Self {
        InstancedAttributeBlock {
            buffer: Vec::new(),
            views: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

pub struct SubModel {
    pub id: i32,
    pub priority: i32,
    pub instanced_world_matrix_index: i32,
    pub instanced_sh_index: i32,
    pub reflection_probe_type: i32,
    pub passes: Vec<crate::renderer::core::pass::Pass>,
    pub shaders: Vec<u64>,
    pub patches: Vec<String>,
    pub batching_scheme: BatchingSchemes,
    pub instanced_attribute_block: InstancedAttributeBlock,
    pub world_bound_descriptor_set: Option<u64>,
    pub descriptor_set: Option<u64>,
    pub input_assembler: Option<u64>,
    pub sub_mesh: Option<u64>,
    pub owner_model_id: Option<u64>,
}

impl SubModel {
    pub fn new() -> Self {
        SubModel {
            id: -1,
            priority: 0,
            instanced_world_matrix_index: -1,
            instanced_sh_index: -1,
            reflection_probe_type: 0,
            passes: Vec::new(),
            shaders: Vec::new(),
            patches: Vec::new(),
            batching_scheme: BatchingSchemes::None,
            instanced_attribute_block: InstancedAttributeBlock::default(),
            world_bound_descriptor_set: None,
            descriptor_set: None,
            input_assembler: None,
            sub_mesh: None,
            owner_model_id: None,
        }
    }

    pub fn initialize(&mut self, passes: Vec<crate::renderer::core::pass::Pass>, patches: Vec<String>) {
        self.passes = passes;
        self.patches = patches;
    }

    pub fn update(&mut self) {}

    pub fn destroy(&mut self) {
        self.passes.clear();
        self.shaders.clear();
        self.patches.clear();
        self.instanced_attribute_block.buffer.clear();
        self.instanced_attribute_block.views.clear();
        self.instanced_attribute_block.attributes.clear();
    }

    pub fn set_descriptor_set(&mut self, ds_id: u64) {
        self.descriptor_set = Some(ds_id);
    }

    pub fn get_descriptor_set(&self) -> Option<u64> {
        self.descriptor_set
    }

    pub fn set_world_bound_descriptor_set(&mut self, ds_id: u64) {
        self.world_bound_descriptor_set = Some(ds_id);
    }

    pub fn get_world_bound_descriptor_set(&self) -> Option<u64> {
        self.world_bound_descriptor_set
    }

    pub fn set_input_assembler(&mut self, ia_id: u64) {
        self.input_assembler = Some(ia_id);
    }

    pub fn get_input_assembler(&self) -> Option<u64> {
        self.input_assembler
    }

    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }

    pub fn get_priority(&self) -> i32 {
        self.priority
    }

    pub fn set_owner(&mut self, model_id: u64) {
        self.owner_model_id = Some(model_id);
    }

    pub fn get_owner(&self) -> Option<u64> {
        self.owner_model_id
    }

    pub fn set_sub_mesh(&mut self, mesh_id: u64) {
        self.sub_mesh = Some(mesh_id);
    }

    pub fn get_sub_mesh(&self) -> Option<u64> {
        self.sub_mesh
    }

    pub fn set_shaders(&mut self, shaders: Vec<u64>) {
        self.shaders = shaders;
    }

    pub fn get_shaders(&self) -> &[u64] {
        &self.shaders
    }

    pub fn get_passes(&self) -> &[crate::renderer::core::pass::Pass] {
        &self.passes
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_batching_scheme(&self) -> BatchingSchemes {
        self.batching_scheme
    }

    pub fn set_batching_scheme(&mut self, scheme: BatchingSchemes) {
        self.batching_scheme = scheme;
    }

    pub fn set_reflection_probe_type(&mut self, val: i32) {
        self.reflection_probe_type = val;
    }

    pub fn get_reflection_probe_type(&self) -> i32 {
        self.reflection_probe_type
    }

    pub fn on_pipeline_state_changed(&mut self) {}

    pub fn on_macro_patches_state_changed(&mut self, patches: Vec<String>) {
        self.patches = patches;
    }

    pub fn on_geometry_changed(&mut self) {}
}

impl Default for SubModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submodel_new() {
        let sub = SubModel::new();
        assert_eq!(sub.id, -1);
        assert!(sub.passes.is_empty());
        assert!(sub.descriptor_set.is_none());
    }

    #[test]
    fn test_submodel_set_priority() {
        let mut sub = SubModel::new();
        sub.set_priority(5);
        assert_eq!(sub.get_priority(), 5);
    }

    #[test]
    fn test_submodel_destroy() {
        let mut sub = SubModel::new();
        sub.shaders.push(1);
        sub.destroy();
        assert!(sub.shaders.is_empty());
    }
}
