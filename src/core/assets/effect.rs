use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderStageFlagBit {
    None = 0,
    Vertex = 1,
    Fragment = 2,
    Compute = 4,
    All = 7,
}

impl Default for ShaderStageFlagBit {
    fn default() -> Self {
        ShaderStageFlagBit::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniformType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    IVec2,
    IVec3,
    IVec4,
    Mat3,
    Mat4,
    Sampler2D,
    SamplerCube,
}

impl Default for UniformType {
    fn default() -> Self {
        UniformType::Float
    }
}

#[derive(Debug, Clone)]
pub struct UniformInfo {
    pub name: String,
    pub uniform_type: UniformType,
    pub count: u32,
}

impl UniformInfo {
    pub fn new(name: &str, uniform_type: UniformType) -> Self {
        UniformInfo {
            name: name.to_string(),
            uniform_type,
            count: 1,
        }
    }

    pub fn with_count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }
}

#[derive(Debug, Clone)]
pub struct UniformBlock {
    pub name: String,
    pub binding: u32,
    pub set: u32,
    pub members: Vec<UniformInfo>,
}

impl UniformBlock {
    pub fn new(name: &str, binding: u32) -> Self {
        UniformBlock {
            name: name.to_string(),
            binding,
            set: 0,
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, member: UniformInfo) {
        self.members.push(member);
    }
}

#[derive(Debug, Clone)]
pub struct UniformSamplerTexture {
    pub name: String,
    pub binding: u32,
    pub set: u32,
    pub sampler_type: UniformType,
    pub count: u32,
}

impl UniformSamplerTexture {
    pub fn new(name: &str, binding: u32) -> Self {
        UniformSamplerTexture {
            name: name.to_string(),
            binding,
            set: 0,
            sampler_type: UniformType::Sampler2D,
            count: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderStageInfo {
    pub stage: ShaderStageFlagBit,
    pub source: String,
}

impl ShaderStageInfo {
    pub fn new(stage: ShaderStageFlagBit, source: &str) -> Self {
        ShaderStageInfo {
            stage,
            source: source.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderInfo {
    pub name: String,
    pub stages: Vec<ShaderStageInfo>,
    pub attributes: Vec<String>,
    pub blocks: Vec<UniformBlock>,
    pub samplers: Vec<UniformSamplerTexture>,
}

impl ShaderInfo {
    pub fn new(name: &str) -> Self {
        ShaderInfo {
            name: name.to_string(),
            stages: Vec::new(),
            attributes: Vec::new(),
            blocks: Vec::new(),
            samplers: Vec::new(),
        }
    }

    pub fn add_stage(&mut self, stage: ShaderStageInfo) {
        self.stages.push(stage);
    }

    pub fn add_block(&mut self, block: UniformBlock) {
        self.blocks.push(block);
    }

    pub fn add_sampler(&mut self, sampler: UniformSamplerTexture) {
        self.samplers.push(sampler);
    }
}

impl Default for ShaderInfo {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    SrcAlpha = 6,
    OneMinusSrcAlpha = 7,
    DstAlpha = 8,
    OneMinusDstAlpha = 9,
}

impl Default for BlendFactor {
    fn default() -> Self {
        BlendFactor::One
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonFunc {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

impl Default for ComparisonFunc {
    fn default() -> Self {
        ComparisonFunc::Less
    }
}

#[derive(Debug, Clone)]
pub struct BlendState {
    pub blend_src: BlendFactor,
    pub blend_dst: BlendFactor,
    pub blend_src_alpha: BlendFactor,
    pub blend_dst_alpha: BlendFactor,
    pub is_blend: bool,
}

impl Default for BlendState {
    fn default() -> Self {
        BlendState {
            blend_src: BlendFactor::SrcAlpha,
            blend_dst: BlendFactor::OneMinusSrcAlpha,
            blend_src_alpha: BlendFactor::One,
            blend_dst_alpha: BlendFactor::OneMinusSrcAlpha,
            is_blend: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DepthStencilState {
    pub depth_test: bool,
    pub depth_write: bool,
    pub depth_func: ComparisonFunc,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        DepthStencilState {
            depth_test: true,
            depth_write: true,
            depth_func: ComparisonFunc::Less,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    None = 0,
    Front = 1,
    Back = 2,
    FrontAndBack = 3,
}

impl Default for CullMode {
    fn default() -> Self {
        CullMode::Back
    }
}

#[derive(Debug, Clone)]
pub struct RasterizerState {
    pub cull_mode: CullMode,
    pub is_front_face_ccw: bool,
    pub depth_bias: f32,
}

impl Default for RasterizerState {
    fn default() -> Self {
        RasterizerState {
            cull_mode: CullMode::Back,
            is_front_face_ccw: false,
            depth_bias: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EffectPassInfo {
    pub program_name: String,
    pub priority: i32,
    pub primitive: u32,
    pub stage: String,
    pub blend_state: BlendState,
    pub depth_stencil_state: DepthStencilState,
    pub rasterizer_state: RasterizerState,
    pub defines: HashMap<String, String>,
    pub shader: Option<ShaderInfo>,
}

impl EffectPassInfo {
    pub fn new(program_name: &str) -> Self {
        EffectPassInfo {
            program_name: program_name.to_string(),
            priority: 0,
            primitive: 0,
            stage: "default".to_string(),
            blend_state: BlendState::default(),
            depth_stencil_state: DepthStencilState::default(),
            rasterizer_state: RasterizerState::default(),
            defines: HashMap::new(),
            shader: None,
        }
    }

    pub fn set_define(&mut self, name: &str, value: &str) {
        self.defines.insert(name.to_string(), value.to_string());
    }

    pub fn get_define(&self, name: &str) -> Option<&str> {
        self.defines.get(name).map(|s| s.as_str())
    }
}

impl Default for EffectPassInfo {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug, Clone)]
pub struct EffectTechnique {
    pub name: String,
    pub passes: Vec<EffectPassInfo>,
}

impl EffectTechnique {
    pub fn new(name: &str) -> Self {
        EffectTechnique {
            name: name.to_string(),
            passes: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, pass: EffectPassInfo) {
        self.passes.push(pass);
    }

    pub fn get_pass(&self, index: usize) -> Option<&EffectPassInfo> {
        self.passes.get(index)
    }

    pub fn get_pass_count(&self) -> usize {
        self.passes.len()
    }
}

impl Default for EffectTechnique {
    fn default() -> Self {
        Self::new("default")
    }
}

#[derive(Debug)]
pub struct EffectAsset {
    pub name: String,
    pub techniques: Vec<EffectTechnique>,
    pub shaders: Vec<ShaderInfo>,
    pub combo_table: HashMap<String, Vec<String>>,
}

impl EffectAsset {
    pub fn new(name: &str) -> Self {
        EffectAsset {
            name: name.to_string(),
            techniques: Vec::new(),
            shaders: Vec::new(),
            combo_table: HashMap::new(),
        }
    }

    pub fn add_technique(&mut self, technique: EffectTechnique) {
        self.techniques.push(technique);
    }

    pub fn get_technique(&self, index: usize) -> Option<&EffectTechnique> {
        self.techniques.get(index)
    }

    pub fn get_technique_count(&self) -> usize {
        self.techniques.len()
    }

    pub fn add_shader(&mut self, shader: ShaderInfo) {
        self.shaders.push(shader);
    }

    pub fn get_shader(&self, name: &str) -> Option<&ShaderInfo> {
        self.shaders.iter().find(|s| s.name == name)
    }

    pub fn get_pass_by_index(&self, technique_index: usize, pass_index: usize) -> Option<&EffectPassInfo> {
        self.techniques.get(technique_index)?.passes.get(pass_index)
    }

    pub fn get_program_name(&self, technique_index: usize, pass_index: usize) -> Option<&str> {
        Some(&self.get_pass_by_index(technique_index, pass_index)?.program_name)
    }
}

impl Default for EffectAsset {
    fn default() -> Self {
        Self::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_asset_new() {
        let effect = EffectAsset::new("unlit");
        assert_eq!(effect.name, "unlit");
        assert_eq!(effect.get_technique_count(), 0);
    }

    #[test]
    fn test_effect_asset_add_technique() {
        let mut effect = EffectAsset::new("pbr");
        let mut tech = EffectTechnique::new("default");
        tech.add_pass(EffectPassInfo::new("pbr-vs:vert|pbr-fs:frag"));
        effect.add_technique(tech);
        assert_eq!(effect.get_technique_count(), 1);
        assert_eq!(effect.get_technique(0).unwrap().get_pass_count(), 1);
    }

    #[test]
    fn test_effect_pass_info_defines() {
        let mut pass = EffectPassInfo::new("test-program");
        pass.set_define("USE_TEXTURE", "1");
        assert_eq!(pass.get_define("USE_TEXTURE"), Some("1"));
        assert_eq!(pass.get_define("NONEXISTENT"), None);
    }

    #[test]
    fn test_effect_asset_get_program_name() {
        let mut effect = EffectAsset::new("standard");
        let mut tech = EffectTechnique::new("default");
        tech.add_pass(EffectPassInfo::new("standard-vs:vert|standard-fs:frag"));
        effect.add_technique(tech);
        let name = effect.get_program_name(0, 0);
        assert_eq!(name, Some("standard-vs:vert|standard-fs:frag"));
        assert_eq!(effect.get_program_name(0, 99), None);
    }

    #[test]
    fn test_shader_info_add_stage() {
        let mut shader = ShaderInfo::new("test-shader");
        shader.add_stage(ShaderStageInfo::new(ShaderStageFlagBit::Vertex, "void main() {}"));
        shader.add_stage(ShaderStageInfo::new(ShaderStageFlagBit::Fragment, "void main() { gl_FragColor = vec4(1.0); }"));
        assert_eq!(shader.stages.len(), 2);
    }

    #[test]
    fn test_uniform_block() {
        let mut block = UniformBlock::new("CCGlobal", 0);
        block.add_member(UniformInfo::new("cc_time", UniformType::Vec4));
        block.add_member(UniformInfo::new("cc_screenSize", UniformType::Vec4));
        assert_eq!(block.members.len(), 2);
        assert_eq!(block.binding, 0);
    }

    #[test]
    fn test_blend_state_default() {
        let state = BlendState::default();
        assert!(!state.is_blend);
        assert_eq!(state.blend_src, BlendFactor::SrcAlpha);
        assert_eq!(state.blend_dst, BlendFactor::OneMinusSrcAlpha);
    }

    #[test]
    fn test_depth_stencil_state_default() {
        let state = DepthStencilState::default();
        assert!(state.depth_test);
        assert!(state.depth_write);
        assert_eq!(state.depth_func, ComparisonFunc::Less);
    }

    #[test]
    fn test_rasterizer_state_default() {
        let state = RasterizerState::default();
        assert_eq!(state.cull_mode, CullMode::Back);
        assert!(!state.is_front_face_ccw);
    }

    #[test]
    fn test_effect_asset_get_shader() {
        let mut effect = EffectAsset::new("custom");
        let mut shader = ShaderInfo::new("my-vert");
        shader.add_stage(ShaderStageInfo::new(ShaderStageFlagBit::Vertex, "void main() {}"));
        effect.add_shader(shader);
        assert!(effect.get_shader("my-vert").is_some());
        assert!(effect.get_shader("nonexistent").is_none());
    }
}
