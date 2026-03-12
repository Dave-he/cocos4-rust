use super::asset::AssetBase;
use super::asset_enum::PixelFormat;

#[derive(Debug)]
pub struct ImageAsset {
    pub base: AssetBase,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    pub compressed: bool,
    pub url: String,
    pub mipmap_level_data_size: Vec<u32>,
}

impl ImageAsset {
    pub fn new() -> Self {
        ImageAsset {
            base: AssetBase::new(),
            data: Vec::new(),
            width: 0,
            height: 0,
            format: PixelFormat::Rgba8888,
            compressed: false,
            url: String::new(),
            mipmap_level_data_size: Vec::new(),
        }
    }

    pub fn with_data(width: u32, height: u32, format: PixelFormat, data: Vec<u8>) -> Self {
        let compressed = format.is_compressed();
        ImageAsset {
            base: AssetBase::new(),
            data,
            width,
            height,
            format,
            compressed,
            url: String::new(),
            mipmap_level_data_size: Vec::new(),
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }

    pub fn get_format(&self) -> PixelFormat {
        self.format
    }

    pub fn set_format(&mut self, format: PixelFormat) {
        self.format = format;
        self.compressed = format.is_compressed();
    }

    pub fn is_compressed(&self) -> bool {
        self.compressed
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    pub fn get_mipmap_level_data_size(&self) -> &[u32] {
        &self.mipmap_level_data_size
    }

    pub fn set_mipmap_level_data_size(&mut self, sizes: Vec<u32>) {
        self.mipmap_level_data_size = sizes;
    }

    pub fn expected_data_size(&self) -> usize {
        if self.compressed {
            return self.data.len();
        }
        (self.width * self.height * self.format.bytes_per_pixel()) as usize
    }
}

impl Default for ImageAsset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_asset_new() {
        let img = ImageAsset::new();
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert_eq!(img.format, PixelFormat::Rgba8888);
        assert!(!img.compressed);
    }

    #[test]
    fn test_image_asset_with_data() {
        let data = vec![255u8; 4 * 4 * 4];
        let img = ImageAsset::with_data(4, 4, PixelFormat::Rgba8888, data.clone());
        assert_eq!(img.width, 4);
        assert_eq!(img.height, 4);
        assert_eq!(img.data, data);
        assert!(!img.compressed);
    }

    #[test]
    fn test_image_asset_compressed() {
        let img = ImageAsset::with_data(4, 4, PixelFormat::RgbaEtc2, vec![0; 8]);
        assert!(img.is_compressed());
    }

    #[test]
    fn test_image_asset_expected_size() {
        let data = vec![0u8; 4 * 4 * 4];
        let img = ImageAsset::with_data(4, 4, PixelFormat::Rgba8888, data);
        assert_eq!(img.expected_data_size(), 64);
    }

    #[test]
    fn test_image_asset_set_format() {
        let mut img = ImageAsset::new();
        img.set_format(PixelFormat::RgbaPvrtc4Bppv1);
        assert!(img.is_compressed());
        img.set_format(PixelFormat::Rgba8888);
        assert!(!img.is_compressed());
    }
}
