#[derive(Debug)]
pub struct AssetBase {
    pub uuid: String,
    pub name: String,
    pub native: String,
    pub native_url: String,
    pub loaded: bool,
    pub is_default: bool,
    asset_ref_count: u32,
}

impl AssetBase {
    pub fn new() -> Self {
        AssetBase {
            uuid: String::new(),
            name: String::new(),
            native: String::new(),
            native_url: String::new(),
            loaded: true,
            is_default: false,
            asset_ref_count: 0,
        }
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    pub fn set_uuid(&mut self, uuid: &str) {
        self.uuid = uuid.to_string();
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn get_native_url(&self) -> &str {
        &self.native_url
    }

    pub fn add_asset_ref(&mut self) {
        self.asset_ref_count += 1;
    }

    pub fn dec_asset_ref(&mut self) {
        if self.asset_ref_count > 0 {
            self.asset_ref_count -= 1;
        }
    }

    pub fn get_asset_ref_count(&self) -> u32 {
        self.asset_ref_count
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn validate(&self) -> bool {
        true
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn on_loaded(&mut self) {}
}

impl Default for AssetBase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_base_new() {
        let asset = AssetBase::new();
        assert_eq!(asset.uuid, "");
        assert_eq!(asset.name, "");
        assert!(asset.loaded);
        assert!(!asset.is_default);
        assert_eq!(asset.get_asset_ref_count(), 0);
    }

    #[test]
    fn test_asset_base_uuid() {
        let mut asset = AssetBase::new();
        asset.set_uuid("test-uuid-1234");
        assert_eq!(asset.get_uuid(), "test-uuid-1234");
    }

    #[test]
    fn test_asset_base_ref_count() {
        let mut asset = AssetBase::new();
        asset.add_asset_ref();
        asset.add_asset_ref();
        assert_eq!(asset.get_asset_ref_count(), 2);
        asset.dec_asset_ref();
        assert_eq!(asset.get_asset_ref_count(), 1);
        asset.dec_asset_ref();
        asset.dec_asset_ref();
        assert_eq!(asset.get_asset_ref_count(), 0);
    }

    #[test]
    fn test_asset_base_validate() {
        let asset = AssetBase::new();
        assert!(asset.validate());
    }
}
