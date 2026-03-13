use super::{accelerometer::MotionValue, network::NetworkType, screen::Orientation};
use crate::math::Vec4;

/// Device - provides access to platform-specific device information and features
pub struct Device;

impl Device {
    /// Get DPI of device
    pub fn get_dpi() -> i32 {
        0
    }

    /// Get device pixel ratio
    pub fn get_device_pixel_ratio() -> f32 {
        1.0
    }

    /// Enable or disable accelerometer
    pub fn set_accelerometer_enabled(_is_enabled: bool) {}

    /// Set accelerometer interval
    pub fn set_accelerometer_interval(_interval: f32) {}

    /// Get device motion value
    pub fn get_device_motion_value() -> MotionValue {
        MotionValue::default()
    }

    /// Get device orientation
    pub fn get_device_orientation() -> Orientation {
        Orientation::Portrait
    }

    /// Get device model
    pub fn get_device_model() -> String {
        String::new()
    }

    /// Set whether screen should remain on
    pub fn set_keep_screen_on(_keep_screen_on: bool) {}

    /// Vibrate for specified duration (in seconds)
    pub fn vibrate(_duration: f32) {}

    /// Get battery level (0.0 - 1.0), iOS and Android only
    pub fn get_battery_level() -> f32 {
        1.0
    }

    /// Get network type
    pub fn get_network_type() -> NetworkType {
        NetworkType::None
    }

    /// Get safe area edge as Vec4(x, y, z, w) = Edge(top, left, bottom, right)
    pub fn get_safe_area_edge() -> Vec4 {
        Vec4::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_dpi() {
        let dpi = Device::get_dpi();
        assert_eq!(dpi, 0);
    }

    #[test]
    fn test_device_pixel_ratio() {
        let ratio = Device::get_device_pixel_ratio();
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_device_battery_level() {
        let level = Device::get_battery_level();
        assert!(level >= 0.0 && level <= 1.0);
    }

    #[test]
    fn test_device_safe_area() {
        let edge = Device::get_safe_area_edge();
        assert_eq!(edge.x, 0.0);
        assert_eq!(edge.y, 0.0);
    }

    #[test]
    fn test_device_motion_value_default() {
        let motion = Device::get_device_motion_value();
        assert_eq!(motion.acceleration_x, 0.0);
    }
}
