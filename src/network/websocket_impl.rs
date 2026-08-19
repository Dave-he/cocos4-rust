pub struct WebSocketImpl {
    url: String,
    connected: bool,
    protocols: Vec<String>,
}

impl WebSocketImpl {
    pub fn new(url: &str) -> Self {
        Self { url: url.to_string(), connected: false, protocols: Vec::new() }
    }

    pub fn connect(&mut self) -> bool {
        self.connected = true;
        true
    }

    pub fn close(&mut self) {
        self.connected = false;
    }

    pub fn send(&self, _data: &[u8]) -> Result<usize, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        Ok(0)
    }

    pub fn recv(&self) -> Result<Vec<u8>, String> {
        if !self.connected {
            return Err("Not connected".into());
        }
        Ok(Vec::new())
    }

    pub fn is_connected(&self) -> bool { self.connected }
    pub fn url(&self) -> &str { &self.url }

    pub fn set_protocols(&mut self, protocols: &[&str]) {
        self.protocols = protocols.iter().map(|s| s.to_string()).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_new() {
        let ws = WebSocketImpl::new("ws://localhost:8080");
        assert!(!ws.is_connected());
    }

    #[test]
    fn test_websocket_connect() {
        let mut ws = WebSocketImpl::new("ws://localhost:8080");
        assert!(ws.connect());
        assert!(ws.is_connected());
    }

    #[test]
    fn test_websocket_close() {
        let mut ws = WebSocketImpl::new("ws://localhost:8080");
        ws.connect();
        ws.close();
        assert!(!ws.is_connected());
    }

    #[test]
    fn test_websocket_send_not_connected() {
        let ws = WebSocketImpl::new("ws://localhost:8080");
        assert!(ws.send(b"hello").is_err());
    }

    #[test]
    fn test_websocket_protocols() {
        let mut ws = WebSocketImpl::new("ws://localhost:8080");
        ws.set_protocols(&["chat"]);
    }
}
