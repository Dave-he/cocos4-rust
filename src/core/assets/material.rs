use super::asset::AssetBase;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MacroPatch {
    pub name: String,
    pub value: bool,
}

impl MacroPatch {
    pub fn new(name: &str, value: bool) -> Self {
        MacroPatch {
            name: name.to_string(),
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PassInfo {
    pub program_name: String,
    pub priority: i32,
    pub primitive: u32,
    pub stage: String,
    pub defines: HashMap<String, String>,
}

impl PassInfo {
    pub fn new(program_name: &str) -> Self {
        PassInfo {
            program_name: program_name.to_string(),
            priority: 0,
            primitive: 0,
            stage: String::new(),
            defines: HashMap::new(),
        }
    }
}

impl Default for PassInfo {
    fn default() -> Self {
        Self::new("")
    }
}

#[derive(Debug)]
pub struct MaterialInfo {
    pub effect_name: Option<String>,
    pub technique: u32,
}

impl MaterialInfo {
    pub fn new(effect_name: &str) -> Self {
        MaterialInfo {
            effect_name: Some(effect_name.to_string()),
            technique: 0,
        }
    }
}

#[derive(Debug)]
pub struct Material {
    pub base: AssetBase,
    pub effect_name: String,
    pub technique_index: u32,
    passes: Vec<PassInfo>,
    pub hash: u64,
}

impl Material {
    pub fn new() -> Self {
        Material {
            base: AssetBase::new(),
            effect_name: String::new(),
            technique_index: 0,
            passes: Vec::new(),
            hash: 0,
        }
    }

    pub fn initialize(&mut self, info: MaterialInfo) {
        if let Some(name) = info.effect_name {
            self.effect_name = name;
        }
        self.technique_index = info.technique;
        self.update_hash();
    }

    pub fn get_effect_name(&self) -> &str {
        &self.effect_name
    }

    pub fn set_effect_name(&mut self, name: &str) {
        self.effect_name = name.to_string();
        self.update_hash();
    }

    pub fn get_technique_index(&self) -> u32 {
        self.technique_index
    }

    pub fn set_technique_index(&mut self, index: u32) {
        self.technique_index = index;
        self.update_hash();
    }

    pub fn get_pass_count(&self) -> usize {
        self.passes.len()
    }

    pub fn get_pass(&self, index: usize) -> Option<&PassInfo> {
        self.passes.get(index)
    }

    pub fn get_passes(&self) -> &[PassInfo] {
        &self.passes
    }

    pub fn add_pass(&mut self, pass: PassInfo) {
        self.passes.push(pass);
        self.update_hash();
    }

    pub fn remove_pass(&mut self, index: usize) {
        if index < self.passes.len() {
            self.passes.remove(index);
            self.update_hash();
        }
    }

    pub fn clear_passes(&mut self) {
        self.passes.clear();
        self.update_hash();
    }

    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    pub fn set_property(&mut self, pass_index: usize, name: &str, value: &str) {
        if let Some(pass) = self.passes.get_mut(pass_index) {
            pass.defines.insert(name.to_string(), value.to_string());
        }
    }

    pub fn get_property(&self, pass_index: usize, name: &str) -> Option<&String> {
        self.passes.get(pass_index)?.defines.get(name)
    }

    fn update_hash(&mut self) {
        let mut h: u64 = 0;
        h = h.wrapping_mul(31).wrapping_add(self.effect_name.len() as u64);
        h = h.wrapping_mul(31).wrapping_add(self.technique_index as u64);
        h = h.wrapping_mul(31).wrapping_add(self.passes.len() as u64);
        self.hash = h;
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_new() {
        let mat = Material::new();
        assert_eq!(mat.effect_name, "");
        assert_eq!(mat.technique_index, 0);
        assert_eq!(mat.get_pass_count(), 0);
    }

    #[test]
    fn test_material_initialize() {
        let mut mat = Material::new();
        mat.initialize(MaterialInfo::new("unlit"));
        assert_eq!(mat.get_effect_name(), "unlit");
    }

    #[test]
    fn test_material_passes() {
        let mut mat = Material::new();
        let pass = PassInfo::new("builtin-unlit");
        mat.add_pass(pass);
        assert_eq!(mat.get_pass_count(), 1);
        assert_eq!(mat.get_pass(0).unwrap().program_name, "builtin-unlit");
    }

    #[test]
    fn test_material_property() {
        let mut mat = Material::new();
        mat.add_pass(PassInfo::new("test-program"));
        mat.set_property(0, "mainTexture", "white");
        assert_eq!(mat.get_property(0, "mainTexture"), Some(&"white".to_string()));
        assert_eq!(mat.get_property(0, "nonexistent"), None);
    }

    #[test]
    fn test_material_hash_changes() {
        let mut mat = Material::new();
        let hash1 = mat.get_hash();
        mat.set_effect_name("pbr");
        let hash2 = mat.get_hash();
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_material_clear_passes() {
        let mut mat = Material::new();
        mat.add_pass(PassInfo::new("a"));
        mat.add_pass(PassInfo::new("b"));
        assert_eq!(mat.get_pass_count(), 2);
        mat.clear_passes();
        assert_eq!(mat.get_pass_count(), 0);
    }
}
