#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    Rgb565 = 1,
    Rgb5A1 = 2,
    Rgba4444 = 3,
    Rgb888 = 4,
    Rgb32F = 5,
    Rgba8888 = 6,
    Rgba32F = 7,
    A8 = 8,
    I8 = 9,
    Ai8 = 10,
    RgbPvrtc2Bppv1 = 11,
    RgbaPvrtc2Bppv1 = 12,
    RgbAPvrtc2Bppv1 = 1024,
    RgbPvrtc4Bppv1 = 13,
    RgbaPvrtc4Bppv1 = 14,
    RgbAPvrtc4Bppv1 = 1025,
    RgbEtc1 = 15,
    RgbaEtc1 = 1026,
    RgbEtc2 = 16,
    RgbaEtc2 = 17,
    RgbaAstc4X4 = 18,
    RgbaAstc5X4 = 19,
    RgbaAstc5X5 = 20,
    RgbaAstc6X5 = 21,
    RgbaAstc6X6 = 22,
    RgbaAstc8X5 = 23,
    RgbaAstc8X6 = 24,
    RgbaAstc8X8 = 25,
    RgbaAstc10X5 = 26,
    RgbaAstc10X6 = 27,
    RgbaAstc10X8 = 28,
    RgbaAstc10X10 = 29,
    RgbaAstc12X10 = 30,
    RgbaAstc12X12 = 31,
}

impl Default for PixelFormat {
    fn default() -> Self {
        PixelFormat::Rgba8888
    }
}

impl PixelFormat {
    pub fn is_compressed(&self) -> bool {
        matches!(
            self,
            PixelFormat::RgbPvrtc2Bppv1
                | PixelFormat::RgbaPvrtc2Bppv1
                | PixelFormat::RgbAPvrtc2Bppv1
                | PixelFormat::RgbPvrtc4Bppv1
                | PixelFormat::RgbaPvrtc4Bppv1
                | PixelFormat::RgbAPvrtc4Bppv1
                | PixelFormat::RgbEtc1
                | PixelFormat::RgbaEtc1
                | PixelFormat::RgbEtc2
                | PixelFormat::RgbaEtc2
                | PixelFormat::RgbaAstc4X4
                | PixelFormat::RgbaAstc5X4
                | PixelFormat::RgbaAstc5X5
                | PixelFormat::RgbaAstc6X5
                | PixelFormat::RgbaAstc6X6
                | PixelFormat::RgbaAstc8X5
                | PixelFormat::RgbaAstc8X6
                | PixelFormat::RgbaAstc8X8
                | PixelFormat::RgbaAstc10X5
                | PixelFormat::RgbaAstc10X6
                | PixelFormat::RgbaAstc10X8
                | PixelFormat::RgbaAstc10X10
                | PixelFormat::RgbaAstc12X10
                | PixelFormat::RgbaAstc12X12
        )
    }

    pub fn has_alpha(&self) -> bool {
        !matches!(
            self,
            PixelFormat::Rgb565
                | PixelFormat::Rgb888
                | PixelFormat::Rgb32F
                | PixelFormat::I8
                | PixelFormat::RgbPvrtc2Bppv1
                | PixelFormat::RgbPvrtc4Bppv1
                | PixelFormat::RgbEtc1
                | PixelFormat::RgbEtc2
        )
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            PixelFormat::A8 | PixelFormat::I8 => 1,
            PixelFormat::Ai8 | PixelFormat::Rgb565 | PixelFormat::Rgb5A1 | PixelFormat::Rgba4444 => 2,
            PixelFormat::Rgb888 => 3,
            PixelFormat::Rgba8888 => 4,
            PixelFormat::Rgb32F => 12,
            PixelFormat::Rgba32F => 16,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrapMode {
    Repeat = 0,
    ClampToEdge = 1,
    MirroredRepeat = 2,
    ClampToBorder = 3,
}

impl Default for WrapMode {
    fn default() -> Self {
        WrapMode::Repeat
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    None = 0,
    Linear = 1,
    Nearest = 2,
}

impl Default for Filter {
    fn default() -> Self {
        Filter::Linear
    }
}
