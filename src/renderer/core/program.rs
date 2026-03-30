/****************************************************************************
Rust port of Cocos Creator Renderer Program System
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;
use super::material::MacroRecord;

#[derive(Debug, Clone)]
pub struct IDefineRecord {
    pub name: String,
    pub defines: Vec<MacroRecord>,
    pub offset: i32,
}

#[derive(Debug, Clone, Default)]
pub struct IMacroInfo {
    pub name: String,
    pub value: String,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct ITemplateInfo {
    pub name: String,
    pub defines: Vec<MacroRecord>,
    pub block_sizes: Vec<i32>,
    pub handle_map: HashMap<String, u32>,
    pub sampler_start_binding: i32,
}

impl Default for ITemplateInfo {
    fn default() -> Self {
        ITemplateInfo {
            name: String::new(),
            defines: Vec::new(),
            block_sizes: Vec::new(),
            handle_map: HashMap::new(),
            sampler_start_binding: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramInfo {
    pub name: String,
    pub effect_name: String,
    pub defines: Vec<MacroRecord>,
    pub constant_macros: String,
    pub uber: bool,
}

impl Default for ProgramInfo {
    fn default() -> Self {
        ProgramInfo {
            name: String::new(),
            effect_name: String::new(),
            defines: Vec::new(),
            constant_macros: String::new(),
            uber: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShaderInfo {
    pub name: String,
    pub defines: Vec<MacroRecord>,
}

impl ShaderInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            defines: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// ProgramLib - global shader program manager
// ---------------------------------------------------------------------------

pub struct ProgramLib {
    programs: HashMap<String, ProgramInfo>,
    template_infos: HashMap<String, ITemplateInfo>,
}

impl ProgramLib {
    pub fn new() -> Self {
        ProgramLib {
            programs: HashMap::new(),
            template_infos: HashMap::new(),
        }
    }

    pub fn define(&mut self, shader: ShaderInfo) -> &ProgramInfo {
        let name = shader.name.clone();
        let info = ProgramInfo {
            name: shader.name.clone(),
            defines: shader.defines,
            ..Default::default()
        };
        self.programs.entry(name.clone()).or_insert(info);
        self.programs.get(&name).unwrap()
    }

    pub fn get_template(&self, name: &str) -> Option<&ProgramInfo> {
        self.programs.get(name)
    }

    pub fn get_template_info(&self, name: &str) -> Option<&ITemplateInfo> {
        self.template_infos.get(name)
    }

    pub fn has_program(&self, name: &str) -> bool {
        self.programs.contains_key(name)
    }

    pub fn get_program_count(&self) -> usize {
        self.programs.len()
    }

    pub fn destroy(&mut self) {
        self.programs.clear();
        self.template_infos.clear();
    }

    pub fn get_device_shader_version() -> &'static str {
        "glsl4"
    }
}

impl Default for ProgramLib {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_lib_new() {
        let lib = ProgramLib::new();
        assert_eq!(lib.get_program_count(), 0);
    }

    #[test]
    fn test_program_lib_define() {
        let mut lib = ProgramLib::new();
        let info = lib.define(ShaderInfo::new("standard"));
        assert_eq!(info.name, "standard");
        assert_eq!(lib.get_program_count(), 1);
    }

    #[test]
    fn test_program_lib_has_program() {
        let mut lib = ProgramLib::new();
        assert!(!lib.has_program("standard"));
        lib.define(ShaderInfo::new("standard"));
        assert!(lib.has_program("standard"));
    }

    #[test]
    fn test_program_lib_get_template() {
        let mut lib = ProgramLib::new();
        lib.define(ShaderInfo::new("standard"));
        let tmpl = lib.get_template("standard");
        assert!(tmpl.is_some());
        assert_eq!(tmpl.unwrap().name, "standard");
    }

    #[test]
    fn test_program_lib_destroy() {
        let mut lib = ProgramLib::new();
        lib.define(ShaderInfo::new("unlit"));
        lib.define(ShaderInfo::new("standard"));
        lib.destroy();
        assert_eq!(lib.get_program_count(), 0);
    }

    #[test]
    fn test_program_lib_define_duplicate() {
        let mut lib = ProgramLib::new();
        lib.define(ShaderInfo::new("standard"));
        lib.define(ShaderInfo::new("standard"));
        assert_eq!(lib.get_program_count(), 1);
    }
}
