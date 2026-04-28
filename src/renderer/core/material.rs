/****************************************************************************
Rust port of Cocos Creator Renderer Material System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;
use crate::base::{RefCounted, RefCountedImpl};
use super::pass::{Pass, PassOverrides, PassProperty};

#[derive(Debug, Clone)]
pub struct MacroRecord {
    pub name: String,
    pub value: String,
}

impl MacroRecord {
    pub fn new(name: &str, value: &str) -> Self {
        MacroRecord { name: name.to_string(), value: value.to_string() }
    }
}

#[derive(Debug, Clone)]
pub struct IMacroInfo {
    pub name: String,
    pub value: String,
    pub is_default: bool,
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub effect_name: String,
    pub passes: Vec<Pass>,
    pub technique_index: usize,
    pub defines: Vec<MacroRecord>,
    pub hash: u64,
    ref_count: RefCountedImpl,
}

impl Material {
    pub fn new(name: &str) -> Self {
        Material {
            name: name.to_string(),
            effect_name: String::new(),
            passes: Vec::new(),
            technique_index: 0,
            defines: Vec::new(),
            hash: 0,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn initialize(&mut self, effect_name: &str, technique_index: usize) {
        self.effect_name = effect_name.to_string();
        self.technique_index = technique_index;
        self.recompute_hash();
    }

    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
        self.recompute_hash();
    }

    pub fn get_pass(&self, index: usize) -> Option<&Pass> {
        self.passes.get(index)
    }

    pub fn get_pass_mut(&mut self, index: usize) -> Option<&mut Pass> {
        self.passes.get_mut(index)
    }

    pub fn get_pass_count(&self) -> usize {
        self.passes.len()
    }

    pub fn clear_passes(&mut self) {
        self.passes.clear();
        self.recompute_hash();
    }

    pub fn set_property(&mut self, name: &str, value: PassProperty) {
        for pass in &mut self.passes {
            pass.set_property(name, value.clone());
        }
    }

    pub fn set_property_on_pass(&mut self, pass_idx: usize, name: &str, value: PassProperty) {
        if let Some(pass) = self.passes.get_mut(pass_idx) {
            pass.set_property(name, value);
        }
    }

    pub fn set_define(&mut self, name: &str, value: &str) {
        if let Some(existing) = self.defines.iter_mut().find(|d| d.name == name) {
            existing.value = value.to_string();
        } else {
            self.defines.push(MacroRecord::new(name, value));
        }
        self.recompute_hash();
    }

    pub fn get_define(&self, name: &str) -> Option<&str> {
        self.defines.iter().find(|d| d.name == name).map(|d| d.value.as_str())
    }

    pub fn recompile_shaders(&mut self, overrides: &[MacroRecord]) {
        for m in overrides {
            self.set_define(&m.name, &m.value);
        }
        self.recompute_hash();
    }

    pub fn override_pipeline_states(&mut self, pass_idx: usize, overrides: &PassOverrides) {
        if let Some(pass) = self.passes.get_mut(pass_idx) {
            pass.override_pipeline_states(overrides);
        }
        self.recompute_hash();
    }

    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    fn recompute_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        self.effect_name.hash(&mut h);
        self.technique_index.hash(&mut h);
        for d in &self.defines {
            d.name.hash(&mut h);
            d.value.hash(&mut h);
        }
        for p in &self.passes {
            p.get_hash().hash(&mut h);
        }
        self.hash = h.finish();
    }
}

impl RefCounted for Material {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

pub struct MaterialPool {
    materials: HashMap<String, Material>,
}

impl MaterialPool {
    pub fn new() -> Self {
        MaterialPool { materials: HashMap::new() }
    }

    pub fn create(&mut self, name: &str) -> &mut Material {
        self.materials.entry(name.to_string()).or_insert_with(|| Material::new(name))
    }

    pub fn get(&self, name: &str) -> Option<&Material> {
        self.materials.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Material> {
        self.materials.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Material> {
        self.materials.remove(name)
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
}

impl Default for MaterialPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::core::pass::{IPassInfo, RenderQueueType};

    fn make_pass(name: &str) -> Pass {
        let info = IPassInfo { name: name.to_string(), ..Default::default() };
        Pass::with_info(info)
    }

    #[test]
    fn test_material_new() {
        let m = Material::new("test_mat");
        assert_eq!(m.name, "test_mat");
        assert_eq!(m.get_pass_count(), 0);
    }

    #[test]
    fn test_material_initialize() {
        let mut m = Material::new("m");
        m.initialize("standard", 0);
        assert_eq!(m.effect_name, "standard");
        assert_eq!(m.technique_index, 0);
    }

    #[test]
    fn test_material_add_pass() {
        let mut m = Material::new("m");
        m.add_pass(make_pass("opaque"));
        m.add_pass(make_pass("shadow"));
        assert_eq!(m.get_pass_count(), 2);
    }

    #[test]
    fn test_material_clear_passes() {
        let mut m = Material::new("m");
        m.add_pass(make_pass("p1"));
        m.clear_passes();
        assert_eq!(m.get_pass_count(), 0);
    }

    #[test]
    fn test_material_hash_changes_on_define() {
        let mut m = Material::new("m");
        m.initialize("pbr", 0);
        let h0 = m.get_hash();
        m.set_define("USE_NORMAL_MAP", "1");
        assert_ne!(m.get_hash(), h0);
    }

    #[test]
    fn test_material_get_define() {
        let mut m = Material::new("m");
        m.set_define("ALPHA_TEST", "1");
        assert_eq!(m.get_define("ALPHA_TEST"), Some("1"));
        assert_eq!(m.get_define("MISSING"), None);
    }

    #[test]
    fn test_material_set_property_all_passes() {
        let mut m = Material::new("m");
        m.add_pass(make_pass("p0"));
        m.add_pass(make_pass("p1"));
        m.set_property("albedo", PassProperty::Vec4([1.0, 0.0, 0.0, 1.0]));
        for i in 0..2 {
            assert!(m.get_pass(i).unwrap().get_property("albedo").is_some());
        }
    }

    #[test]
    fn test_material_recompile_shaders() {
        let mut m = Material::new("m");
        m.recompile_shaders(&[MacroRecord::new("FOG", "1")]);
        assert_eq!(m.get_define("FOG"), Some("1"));
    }

    #[test]
    fn test_material_override_pipeline_states() {
        let mut m = Material::new("m");
        m.add_pass(make_pass("p0"));
        let overrides = PassOverrides {
            queue: Some(RenderQueueType::Transparent),
            priority: Some(10),
            ..Default::default()
        };
        m.override_pipeline_states(0, &overrides);
        assert_eq!(m.get_pass(0).unwrap().get_queue(), RenderQueueType::Transparent);
    }

    #[test]
    fn test_material_ref_count() {
        let m = Material::new("m");
        assert_eq!(m.get_ref_count(), 1); // starts at 1 (initial reference)
        m.add_ref();
        assert_eq!(m.get_ref_count(), 2);
        m.release();
        assert!(m.is_last_reference());
    }

    #[test]
    fn test_material_pool() {
        let mut pool = MaterialPool::new();
        pool.create("mat_a").initialize("pbr", 0);
        pool.create("mat_b").initialize("unlit", 0);
        assert_eq!(pool.len(), 2);
        assert!(pool.get("mat_a").is_some());
        pool.remove("mat_a");
        assert_eq!(pool.len(), 1);
        assert!(!pool.is_empty());
    }
}
