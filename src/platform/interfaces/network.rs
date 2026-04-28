/// Network type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NetworkType {
    #[default]
    None,
    Lan,
    Wwan,
}

impl std::fmt::Display for NetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkType::None => write!(f, "none"),
            NetworkType::Lan => write!(f, "lan"),
            NetworkType::Wwan => write!(f, "wwan"),
        }
    }
}

/// Network interface
pub trait INetwork {
    /// Get current network type
    fn get_network_type(&self) -> NetworkType;

    /// Check if network is available
    fn is_connected(&self) -> bool {
        self.get_network_type() != NetworkType::None
    }

    /// Check if on WiFi/LAN
    fn is_on_wifi(&self) -> bool {
        self.get_network_type() == NetworkType::Lan
    }
}

#[derive(Debug, Default)]
pub struct DefaultNetwork {
    pub network_type: NetworkType,
}

impl DefaultNetwork {
    pub fn new(network_type: NetworkType) -> Self {
        DefaultNetwork { network_type }
    }

    pub fn set_network_type(&mut self, network_type: NetworkType) {
        self.network_type = network_type;
    }
}

impl INetwork for DefaultNetwork {
    fn get_network_type(&self) -> NetworkType {
        self.network_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_none() {
        let n = DefaultNetwork::new(NetworkType::None);
        assert!(!n.is_connected());
        assert!(!n.is_on_wifi());
    }

    #[test]
    fn test_network_lan() {
        let n = DefaultNetwork::new(NetworkType::Lan);
        assert!(n.is_connected());
        assert!(n.is_on_wifi());
    }

    #[test]
    fn test_network_wwan() {
        let n = DefaultNetwork::new(NetworkType::Wwan);
        assert!(n.is_connected());
        assert!(!n.is_on_wifi());
    }

    #[test]
    fn test_network_set_type() {
        let mut n = DefaultNetwork::default();
        assert_eq!(n.get_network_type(), NetworkType::None);
        n.set_network_type(NetworkType::Lan);
        assert_eq!(n.get_network_type(), NetworkType::Lan);
    }

    #[test]
    fn test_network_type_display() {
        assert_eq!(format!("{}", NetworkType::None), "none");
        assert_eq!(format!("{}", NetworkType::Lan), "lan");
        assert_eq!(format!("{}", NetworkType::Wwan), "wwan");
    }
}
