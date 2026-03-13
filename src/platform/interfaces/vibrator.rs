use super::os_interface::{OSInterface, DefaultOSInterface};

/// Vibrator interface
pub trait IVibrator: OSInterface {
    /// Vibrate for the specified amount of time
    /// If vibrate is not supported, this has no effect
    /// Some platforms limit to a maximum duration of 5 seconds
    /// Duration is ignored on iOS due to API limitations
    /// duration: duration in seconds
    fn vibrate(&mut self, duration: f32);

    /// Check if vibration is supported
    fn is_vibration_supported(&self) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct DefaultVibrator {
    pub last_duration: f32,
    pub vibration_count: u32,
    _base: DefaultOSInterface,
}

impl DefaultVibrator {
    pub fn new() -> Self {
        DefaultVibrator::default()
    }
}

impl OSInterface for DefaultVibrator {}

impl IVibrator for DefaultVibrator {
    fn vibrate(&mut self, duration: f32) {
        self.last_duration = duration.clamp(0.0, 5.0);
        self.vibration_count += 1;
    }

    fn is_vibration_supported(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibrator_default() {
        let v = DefaultVibrator::new();
        assert!(!v.is_vibration_supported());
        assert_eq!(v.vibration_count, 0);
    }

    #[test]
    fn test_vibrator_vibrate() {
        let mut v = DefaultVibrator::new();
        v.vibrate(1.0);
        assert!((v.last_duration - 1.0).abs() < 1e-6);
        assert_eq!(v.vibration_count, 1);
    }

    #[test]
    fn test_vibrator_clamp_duration() {
        let mut v = DefaultVibrator::new();
        v.vibrate(10.0);
        assert!((v.last_duration - 5.0).abs() < 1e-6);
    }
}
