pub struct LinuxWindow {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub fullscreen: bool,
}

impl LinuxWindow {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self { title: title.into(), width, height, visible: false, fullscreen: false }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_new() {
        let w = LinuxWindow::new("test", 800, 600);
        assert_eq!(w.title, "test");
        assert_eq!(w.width, 800);
    }

    #[test]
    fn test_window_show_hide() {
        let mut w = LinuxWindow::new("test", 800, 600);
        w.show();
        assert!(w.visible);
        w.hide();
        assert!(!w.visible);
    }

    #[test]
    fn test_window_resize() {
        let mut w = LinuxWindow::new("test", 800, 600);
        w.resize(1920, 1080);
        assert_eq!(w.width, 1920);
    }
}
