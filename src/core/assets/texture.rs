use super::asset::AssetBase;
use super::asset_enum::{Filter, PixelFormat, WrapMode};

#[derive(Debug)]
pub struct TextureBase {
    pub base: AssetBase,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub mip_filter: Filter,
    pub wrap_s: WrapMode,
    pub wrap_t: WrapMode,
    pub wrap_r: WrapMode,
    pub anisotropy: u32,
    texture_hash: u64,
}

impl TextureBase {
    pub fn new() -> Self {
        TextureBase {
            base: AssetBase::new(),
            width: 1,
            height: 1,
            format: PixelFormat::Rgba8888,
            min_filter: Filter::Linear,
            mag_filter: Filter::Linear,
            mip_filter: Filter::None,
            wrap_s: WrapMode::Repeat,
            wrap_t: WrapMode::Repeat,
            wrap_r: WrapMode::Repeat,
            anisotropy: 0,
            texture_hash: 0,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.update_hash();
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.update_hash();
    }

    pub fn get_pixel_format(&self) -> PixelFormat {
        self.format
    }

    pub fn is_compressed(&self) -> bool {
        self.format.is_compressed()
    }

    pub fn get_anisotropy(&self) -> u32 {
        self.anisotropy
    }

    pub fn set_anisotropy(&mut self, anisotropy: u32) {
        self.anisotropy = anisotropy;
        self.update_hash();
    }

    pub fn set_wrap_mode(&mut self, wrap_s: WrapMode, wrap_t: WrapMode, wrap_r: WrapMode) {
        self.wrap_s = wrap_s;
        self.wrap_t = wrap_t;
        self.wrap_r = wrap_r;
        self.update_hash();
    }

    pub fn set_filters(&mut self, min_filter: Filter, mag_filter: Filter) {
        self.min_filter = min_filter;
        self.mag_filter = mag_filter;
        self.update_hash();
    }

    pub fn set_mip_filter(&mut self, mip_filter: Filter) {
        self.mip_filter = mip_filter;
        self.update_hash();
    }

    pub fn get_hash(&self) -> u64 {
        self.texture_hash
    }

    fn update_hash(&mut self) {
        self.texture_hash = self.compute_hash();
    }

    fn compute_hash(&self) -> u64 {
        let mut h: u64 = 0;
        h = h.wrapping_mul(31).wrapping_add(self.wrap_s as u64);
        h = h.wrapping_mul(31).wrapping_add(self.wrap_t as u64);
        h = h.wrapping_mul(31).wrapping_add(self.wrap_r as u64);
        h = h.wrapping_mul(31).wrapping_add(self.min_filter as u64);
        h = h.wrapping_mul(31).wrapping_add(self.mag_filter as u64);
        h = h.wrapping_mul(31).wrapping_add(self.mip_filter as u64);
        h = h.wrapping_mul(31).wrapping_add(self.anisotropy as u64);
        h
    }
}

impl Default for TextureBase {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Texture2D {
    pub base: TextureBase,
    pub mip_level: u32,
}

impl Texture2D {
    pub fn new() -> Self {
        Texture2D {
            base: TextureBase::new(),
            mip_level: 1,
        }
    }

    pub fn get_mip_level(&self) -> u32 {
        self.mip_level
    }

    pub fn set_mip_level(&mut self, level: u32) {
        self.mip_level = level;
    }
}

impl Default for Texture2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_base_new() {
        let tex = TextureBase::new();
        assert_eq!(tex.width, 1);
        assert_eq!(tex.height, 1);
        assert_eq!(tex.format, PixelFormat::Rgba8888);
        assert_eq!(tex.min_filter, Filter::Linear);
        assert_eq!(tex.mag_filter, Filter::Linear);
        assert_eq!(tex.mip_filter, Filter::None);
        assert_eq!(tex.wrap_s, WrapMode::Repeat);
        assert_eq!(tex.anisotropy, 0);
    }

    #[test]
    fn test_texture_base_set_wrap_mode() {
        let mut tex = TextureBase::new();
        let hash_before = tex.get_hash();
        tex.set_wrap_mode(WrapMode::ClampToEdge, WrapMode::ClampToEdge, WrapMode::Repeat);
        let hash_after = tex.get_hash();
        assert_ne!(hash_before, hash_after);
        assert_eq!(tex.wrap_s, WrapMode::ClampToEdge);
    }

    #[test]
    fn test_texture_base_set_filters() {
        let mut tex = TextureBase::new();
        tex.set_filters(Filter::Nearest, Filter::Nearest);
        assert_eq!(tex.min_filter, Filter::Nearest);
        assert_eq!(tex.mag_filter, Filter::Nearest);
    }

    #[test]
    fn test_texture_base_compressed() {
        let mut tex = TextureBase::new();
        assert!(!tex.is_compressed());
        tex.format = PixelFormat::RgbaEtc2;
        assert!(tex.is_compressed());
    }

    #[test]
    fn test_texture2d_new() {
        let tex = Texture2D::new();
        assert_eq!(tex.mip_level, 1);
        assert_eq!(tex.base.width, 1);
    }
}
