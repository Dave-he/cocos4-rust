use super::os_interface::OSInterface;

/// Vibrator interface
pub trait IVibrator: OSInterface {
    /// Vibrate for the specified amount of time
    /// If vibrate is not supported, this has no effect
    /// Some platforms limit to a maximum duration of 5 seconds
    /// Duration is ignored on iOS due to API limitations
    /// duration: duration in seconds
    fn vibrate(&mut self, duration: f32);
}
