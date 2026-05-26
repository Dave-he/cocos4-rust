use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinuxWindow {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub visible: bool,
    pub fullscreen: bool,
    pub position: (i32, i32),
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,
    pub icon_path: Option<PathBuf>,
}

impl LinuxWindow {
    pub fn new(title: &str, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            visible: false,
            fullscreen: false,
            position: (0, 0),
            min_size: None,
            max_size: None,
            icon_path: None,
        }
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

    pub fn set_position(&mut self, x: i32, y: i32) {
        self.position = (x, y);
    }

    pub fn set_min_size(&mut self, width: u32, height: u32) {
        self.min_size = Some((width, height));
    }

    pub fn set_max_size(&mut self, width: u32, height: u32) {
        self.max_size = Some((width, height));
    }

    pub fn set_icon_path(&mut self, path: impl Into<PathBuf>) {
        self.icon_path = Some(path.into());
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
        assert_eq!(w.height, 600);
        assert_eq!(w.position, (0, 0));
        assert!(!w.visible);
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
        assert_eq!(w.height, 1080);
    }

    #[test]
    fn test_window_metadata_updates() {
        let mut w = LinuxWindow::new("test", 800, 600);
        w.set_title("game");
        w.set_position(100, 200);
        w.set_min_size(640, 480);
        w.set_max_size(2560, 1440);
        w.set_icon_path("/tmp/icon.png");

        assert_eq!(w.title, "game");
        assert_eq!(w.position, (100, 200));
        assert_eq!(w.min_size, Some((640, 480)));
        assert_eq!(w.max_size, Some((2560, 1440)));
        assert_eq!(w.icon_path, Some(PathBuf::from("/tmp/icon.png")));
    }
}
