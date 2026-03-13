use crate::math::Vec4;

/// Screen orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait = 0,
    LandscapeLeft = -90,
    PortraitUpsideDown = 180,
    LandscapeRight = 90,
}

/// Screen interface
pub trait IScreen {
    /// Get DPI of the device
    fn get_dpi(&self) -> i32;

    /// Get device pixel ratio
    fn get_device_pixel_ratio(&self) -> f32;

    /// Get device orientation
    fn get_device_orientation(&self) -> Orientation;

    /// Check if display stats are being shown
    fn is_display_stats(&self) -> bool;

    /// Set display stats visibility
    fn set_display_stats(&mut self, is_show: bool);

    /// Set whether screen should remain on
    fn set_keep_screen_on(&mut self, keep_screen_on: bool);

    /// Get safe area edge
    /// Returns Vec4(x, y, z, w) meaning Edge(top, left, bottom, right)
    fn get_safe_area_edge(&self) -> Vec4;

    /// Get screen width in logical pixels
    fn get_width(&self) -> u32;

    /// Get screen height in logical pixels
    fn get_height(&self) -> u32;
}

#[derive(Debug)]
pub struct DefaultScreen {
    pub dpi: i32,
    pub pixel_ratio: f32,
    pub orientation: Orientation,
    pub display_stats: bool,
    pub keep_screen_on: bool,
    pub width: u32,
    pub height: u32,
}

impl Default for DefaultScreen {
    fn default() -> Self {
        DefaultScreen {
            dpi: 96,
            pixel_ratio: 1.0,
            orientation: Orientation::LandscapeLeft,
            display_stats: false,
            keep_screen_on: false,
            width: 1920,
            height: 1080,
        }
    }
}

impl DefaultScreen {
    pub fn new(width: u32, height: u32, dpi: i32) -> Self {
        DefaultScreen {
            dpi,
            pixel_ratio: dpi as f32 / 96.0,
            width,
            height,
            ..DefaultScreen::default()
        }
    }
}

impl IScreen for DefaultScreen {
    fn get_dpi(&self) -> i32 { self.dpi }
    fn get_device_pixel_ratio(&self) -> f32 { self.pixel_ratio }
    fn get_device_orientation(&self) -> Orientation { self.orientation }
    fn is_display_stats(&self) -> bool { self.display_stats }
    fn set_display_stats(&mut self, is_show: bool) { self.display_stats = is_show; }
    fn set_keep_screen_on(&mut self, keep_screen_on: bool) { self.keep_screen_on = keep_screen_on; }
    fn get_safe_area_edge(&self) -> Vec4 { Vec4::new(0.0, 0.0, 0.0, 0.0) }
    fn get_width(&self) -> u32 { self.width }
    fn get_height(&self) -> u32 { self.height }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_default() {
        let s = DefaultScreen::default();
        assert_eq!(s.get_dpi(), 96);
        assert!((s.get_device_pixel_ratio() - 1.0).abs() < 1e-6);
        assert_eq!(s.get_width(), 1920);
        assert_eq!(s.get_height(), 1080);
    }

    #[test]
    fn test_screen_display_stats() {
        let mut s = DefaultScreen::default();
        assert!(!s.is_display_stats());
        s.set_display_stats(true);
        assert!(s.is_display_stats());
    }

    #[test]
    fn test_screen_safe_area() {
        let s = DefaultScreen::default();
        let edge = s.get_safe_area_edge();
        assert_eq!(edge, Vec4::new(0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_screen_new_hd() {
        let s = DefaultScreen::new(1280, 720, 96);
        assert_eq!(s.get_width(), 1280);
        assert_eq!(s.get_height(), 720);
    }

    #[test]
    fn test_screen_pixel_ratio_retina() {
        let s = DefaultScreen::new(2560, 1440, 192);
        assert!((s.get_device_pixel_ratio() - 2.0).abs() < 1e-5);
    }
}

