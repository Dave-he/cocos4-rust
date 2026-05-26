use crate::platform::linux::LinuxWindow;

#[cfg(test)]
mod platform_linux_test {
    use super::*;

    #[test]
    fn linux_window_tracks_fullscreen_and_visibility() {
        let mut window = LinuxWindow::new("demo", 1280, 720);
        assert!(!window.visible);
        assert!(!window.fullscreen);

        window.show();
        window.set_fullscreen(true);

        assert!(window.visible);
        assert!(window.fullscreen);
    }
}
