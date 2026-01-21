/// Network type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    None,
    Lan,
    Wwan,
}

/// Network interface
pub trait INetwork {
    /// Get current network type
    fn get_network_type(&self) -> NetworkType;
}
