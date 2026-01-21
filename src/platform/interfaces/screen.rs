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
}
