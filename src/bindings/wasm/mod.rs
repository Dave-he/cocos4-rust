#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WASM {
    Yes,
    No,
}

pub struct WasmBindgen {}

impl WasmBindgen {
    pub fn new() -> Self { Self {} }

    pub fn is_wasm(&self) -> bool {
        false
    }

    pub fn register_class<F>(&self, _name: &str, _constructor: F) {}
}

pub fn export_function(_name: &str, _f: impl Fn()) {}
pub fn export_constant<T>(_name: &str, _value: T) {}
pub fn import_function(_module: &str, _name: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_bindgen() {
        let wb = WasmBindgen::new();
        assert!(!wb.is_wasm());
    }

    #[test]
    fn test_export_function() {
        export_function("test", || {});
    }

    #[test]
    fn test_export_constant() {
        export_constant("VERSION", "1.0");
    }

    #[test]
    fn test_import_function() {
        import_function("env", "log");
    }

    #[test]
    fn test_register_class() {
        let wb = WasmBindgen::new();
        wb.register_class("Node", || {});
    }
}
