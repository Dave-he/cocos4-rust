use super::os_interface::{OSInterface, DefaultOSInterface};

/// Battery interface
pub trait IBattery: OSInterface {
    /// Get battery level, only available on iOS and Android
    /// Returns value between 0.0 and 1.0
    fn get_battery_level(&self) -> f32;
}

#[derive(Debug, Default)]
pub struct DefaultBattery {
    level: f32,
    _base: DefaultOSInterface,
}

impl DefaultBattery {
    pub fn new(level: f32) -> Self {
        DefaultBattery { level, _base: DefaultOSInterface }
    }
}

impl OSInterface for DefaultBattery {}

impl IBattery for DefaultBattery {
    fn get_battery_level(&self) -> f32 {
        self.level.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_level() {
        let b = DefaultBattery::new(0.75);
        assert!((b.get_battery_level() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_battery_clamp() {
        let b = DefaultBattery::new(1.5);
        assert!((b.get_battery_level() - 1.0).abs() < 1e-6);
    }
}
