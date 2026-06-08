#[derive(Debug, Clone)]
pub struct WebView {
    pub url: String,
    pub loaded: bool,
    pub visible: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub title: String,
    pub js_interface_enabled: bool,
}

impl WebView {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            loaded: false,
            visible: true,
            can_go_back: false,
            can_go_forward: false,
            title: String::new(),
            js_interface_enabled: false,
        }
    }

    pub fn load_url(&mut self, url: &str) {
        self.url = url.to_string();
        self.loaded = true;
    }

    pub fn load_html(&mut self, _html: &str) {
        self.loaded = true;
    }

    pub fn reload(&mut self) {
        self.loaded = true;
    }

    pub fn stop_loading(&mut self) {}

    pub fn go_back(&mut self) {
        if self.can_go_back {
            self.can_go_forward = true;
        }
    }

    pub fn go_forward(&mut self) {
        if self.can_go_forward {
            self.can_go_back = true;
        }
    }

    pub fn evaluate_js(&self, js: &str) {
        let _ = js;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }
}

impl Default for WebView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview_new() {
        let wv = WebView::new();
        assert!(!wv.is_loaded());
        assert!(wv.visible);
    }

    #[test]
    fn test_webview_load_url() {
        let mut wv = WebView::new();
        wv.load_url("https://example.com");
        assert!(wv.is_loaded());
        assert_eq!(wv.url, "https://example.com");
    }

    #[test]
    fn test_webview_navigation() {
        let mut wv = WebView::new();
        wv.can_go_back = true;
        wv.go_back();
        assert!(wv.can_go_forward);
    }

    #[test]
    fn test_webview_visibility() {
        let mut wv = WebView::new();
        wv.set_visible(false);
        assert!(!wv.visible);
    }

    #[test]
    fn test_webview_evaluate_js() {
        let wv = WebView::new();
        wv.evaluate_js("console.log('test')");
    }
}
