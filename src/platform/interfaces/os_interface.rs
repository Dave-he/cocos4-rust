/// OS interface - base trait for all platform interfaces
pub trait OSInterface: Send + Sync {}

#[derive(Debug, Default)]
pub struct DefaultOSInterface;

impl OSInterface for DefaultOSInterface {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_interface_default() {
        let _iface = DefaultOSInterface::default();
    }
}
