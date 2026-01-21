use super::os_interface::OSInterface;

/// Battery interface
pub trait IBattery: OSInterface {
    /// Get battery level, only available on iOS and Android
    /// Returns value between 0.0 and 1.0
    fn get_battery_level(&self) -> f32;
}
