use super::os_interface::OSInterface;

/// Motion value from device sensors
#[derive(Debug, Clone, Default)]
pub struct MotionValue {
    pub acceleration_x: f32,
    pub acceleration_y: f32,
    pub acceleration_z: f32,
    pub acceleration_including_gravity_x: f32,
    pub acceleration_including_gravity_y: f32,
    pub acceleration_including_gravity_z: f32,
    pub rotation_rate_alpha: f32,
    pub rotation_rate_beta: f32,
    pub rotation_rate_gamma: f32,
}

/// Accelerometer interface
pub trait IAccelerometer: OSInterface {
    /// Enable or disable accelerometer
    fn set_accelerometer_enabled(&mut self, is_enabled: bool);

    /// Set the interval of accelerometer
    fn set_accelerometer_interval(&mut self, interval: f32);

    /// Get the motion value of current device
    fn get_device_motion_value(&self) -> MotionValue;
}
