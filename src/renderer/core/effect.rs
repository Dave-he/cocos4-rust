/****************************************************************************\
Rust port of Cocos Creator Renderer Effect System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::pass::{IPassInfo, Pass, PassProperty};
use super::program::{ProgramLib, ShaderInfo};
use crate::base::{RefCounted, RefCountedImpl};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TechniqueInfo {
    pub passes: Vec<IPassInfo>,
}

impl TechniqueInfo {
    pub fn new() -> Self {
        TechniqueInfo { passes: Vec::new() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct EffectInfo {
    pub name: String,
    pub techniques: Vec<TechniqueInfo>,
    pub shaders: Vec<ShaderInfo>,
}

impl EffectInfo {
    pub fn new(name: &str) -> Self {
        EffectInfo {
            name: name.to_string(),
            techniques: Vec::new(),
            shaders: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Effect {
    pub name: String,
    pub techniques: Vec<Vec<Pass>>,
    pub technique_index: usize,
    pub property_layout: HashMap<String, PassProperty>,
    pub default_properties: HashMap<String, PassProperty>,
    ref_count: RefCountedImpl,
}

impl Effect {
    pub fn new(name: &str) -> Self {
        Effect {
            name: name.to_string(),
            techniques: Vec::new(),
            technique_index: 0,
            property_layout: HashMap::new(),
            default_properties: HashMap::new(),
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn initialize(&mut self, info: &EffectInfo, program_lib: &mut ProgramLib) {
        for shader in &info.shaders {
            program_lib.define(shader.clone());
        }
        for technique in &info.techniques {
            let passes: Vec<Pass> = technique
                .passes
                .iter()
                .map(|pass_info| Pass::with_info(pass_info.clone()))
                .collect();
            self.techniques.push(passes);
        }
        self.technique_index = 0;
    }

    pub fn get_technique(&self, idx: usize) -> Option<&Vec<Pass>> {
        self.techniques.get(idx)
    }

    pub fn get_technique_count(&self) -> usize {
        self.techniques.len()
    }

    pub fn set_technique_index(&mut self, idx: usize) {
        if idx < self.techniques.len() {
            self.technique_index = idx;
        }
    }

    pub fn get_current_technique(&self) -> Option<&Vec<Pass>> {
        self.techniques.get(self.technique_index)
    }

    pub fn set_property(&mut self, name: &str, value: PassProperty) {
        self.default_properties
            .insert(name.to_string(), value.clone());
        self.property_layout.insert(name.to_string(), value);
    }

    pub fn get_property(&self, name: &str) -> Option<&PassProperty> {
        self.property_layout.get(name)
    }

    pub fn destroy(&mut self) {
        self.techniques.clear();
        self.property_layout.clear();
        self.default_properties.clear();
    }

    pub fn get_technique_index(&self) -> usize {
        self.technique_index
    }

    pub fn get_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        self.name.hash(&mut h);
        self.technique_index.hash(&mut h);
        h.finish()
    }
}

impl RefCounted for Effect {
    fn add_ref(&self) {
        self.ref_count.add_ref();
    }
    fn release(&self) {
        self.ref_count.release();
    }
    fn get_ref_count(&self) -> u32 {
        self.ref_count.get_ref_count()
    }
    fn is_last_reference(&self) -> bool {
        self.ref_count.is_last_reference()
    }
}

#[derive(Debug)]
pub struct EffectLib {
    effects: HashMap<String, Effect>,
}

impl EffectLib {
    pub fn new() -> Self {
        EffectLib {
            effects: HashMap::new(),
        }
    }

    pub fn register(&mut self, info: EffectInfo, program_lib: &mut ProgramLib) -> &Effect {
        let name = info.name.clone();
        let mut effect = Effect::new(&name);
        effect.initialize(&info, program_lib);
        self.effects.entry(name).or_insert(effect)
    }

    pub fn get(&self, name: &str) -> Option<&Effect> {
        self.effects.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Effect> {
        self.effects.get_mut(name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.effects.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn destroy(&mut self) {
        self.effects.clear();
    }
}

impl Default for EffectLib {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::core::pass::{PassType, RenderQueueType};

    fn make_effect_info() -> EffectInfo {
        let pass_info = IPassInfo {
            name: "opaque".to_string(),
            pass_type: PassType::Graphics,
            queue: RenderQueueType::Opaque,
            shader: "standard-vs/fs".to_string(),
            ..Default::default()
        };
        let technique = TechniqueInfo {
            passes: vec![pass_info],
        };
        let shader = ShaderInfo::new("standard-vs/fs");
        EffectInfo {
            name: "builtin-standard".to_string(),
            techniques: vec![technique],
            shaders: vec![shader],
        }
    }

    #[test]
    fn test_effect_new() {
        let e = Effect::new("test");
        assert_eq!(e.name, "test");
        assert_eq!(e.get_technique_count(), 0);
    }

    #[test]
    fn test_effect_initialize() {
        let info = make_effect_info();
        let mut lib = ProgramLib::new();
        let mut effect = Effect::new("builtin-standard");
        effect.initialize(&info, &mut lib);
        assert_eq!(effect.get_technique_count(), 1);
        assert_eq!(effect.get_technique_index(), 0);
    }

    #[test]
    fn test_effect_technique_access() {
        let info = make_effect_info();
        let mut lib = ProgramLib::new();
        let mut effect = Effect::new("builtin-standard");
        effect.initialize(&info, &mut lib);
        let tech = effect.get_current_technique();
        assert!(tech.is_some());
        assert_eq!(tech.unwrap().len(), 1);
        assert!(effect.get_technique(5).is_none());
    }

    #[test]
    fn test_effect_set_technique_index() {
        let info = make_effect_info();
        let mut lib = ProgramLib::new();
        let mut effect = Effect::new("builtin-standard");
        effect.initialize(&info, &mut lib);
        effect.set_technique_index(5);
        assert_eq!(effect.technique_index, 0);
    }

    #[test]
    fn test_effect_property() {
        let mut e = Effect::new("test");
        e.set_property("albedo", PassProperty::Vec4([1.0, 0.0, 0.0, 1.0]));
        assert!(e.get_property("albedo").is_some());
        assert!(e.get_property("missing").is_none());
    }

    #[test]
    fn test_effect_destroy() {
        let info = make_effect_info();
        let mut lib = ProgramLib::new();
        let mut effect = Effect::new("builtin-standard");
        effect.initialize(&info, &mut lib);
        effect.destroy();
        assert_eq!(effect.get_technique_count(), 0);
    }

    #[test]
    fn test_effect_ref_count() {
        let e = Effect::new("test");
        assert_eq!(e.get_ref_count(), 1);
        e.add_ref();
        assert_eq!(e.get_ref_count(), 2);
        e.release();
        assert!(e.is_last_reference());
    }

    #[test]
    fn test_effect_lib() {
        let mut lib = EffectLib::new();
        let mut prog_lib = ProgramLib::new();
        let info = make_effect_info();
        lib.register(info, &mut prog_lib);
        assert_eq!(lib.len(), 1);
        assert!(lib.has("builtin-standard"));
        assert!(!lib.has("missing"));
        lib.destroy();
        assert!(lib.is_empty());
    }
}
