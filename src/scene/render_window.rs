use super::camera::Camera;

pub struct RenderWindow {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub render_window_id: u32,
    pub is_resized: bool,
    pub cameras: Vec<Camera>,
    pub color_name: String,
    pub depth_stencil_name: String,
}

impl RenderWindow {
    pub fn new() -> Self {
        RenderWindow {
            width: 1,
            height: 1,
            title: String::new(),
            render_window_id: 0,
            is_resized: true,
            cameras: Vec::new(),
            color_name: String::new(),
            depth_stencil_name: String::new(),
        }
    }

    pub fn initialize(&mut self, width: u32, height: u32) -> bool {
        self.width = width;
        self.height = height;
        self.is_resized = true;
        true
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.is_resized = true;
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn attach_camera(&mut self, camera: Camera) {
        self.cameras.push(camera);
    }

    pub fn detach_camera(&mut self, camera_id: u32) {
        self.cameras.retain(|c| c.camera_id != camera_id);
    }

    pub fn clear_cameras(&mut self) {
        self.cameras.clear();
    }

    pub fn sort_cameras(&mut self) {
        self.cameras.sort_by_key(|camera| camera.priority);
    }

    pub fn get_cameras(&self) -> &[Camera] {
        &self.cameras
    }

    pub fn get_render_window_id(&self) -> u32 {
        self.render_window_id
    }

    pub fn get_color_name(&self) -> &str {
        &self.color_name
    }

    pub fn get_depth_stencil_name(&self) -> &str {
        &self.depth_stencil_name
    }

    pub fn is_render_window_resized(&self) -> bool {
        self.is_resized
    }

    pub fn set_render_window_resize_handled(&mut self) {
        self.is_resized = false;
    }

    pub fn extract_render_cameras(&self) -> Vec<&Camera> {
        self.cameras.iter().filter(|c| c.enabled).collect()
    }

    pub fn destroy(&mut self) {
        self.cameras.clear();
    }
}

impl Default for RenderWindow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_window_new() {
        let window = RenderWindow::new();
        assert_eq!(window.width, 1);
        assert_eq!(window.height, 1);
        assert!(window.is_resized);
    }

    #[test]
    fn test_render_window_resize() {
        let mut window = RenderWindow::new();
        window.resize(1920, 1080);
        assert_eq!(window.get_width(), 1920);
        assert_eq!(window.get_height(), 1080);
        assert!(window.is_render_window_resized());
    }

    #[test]
    fn test_render_window_resize_handled() {
        let mut window = RenderWindow::new();
        window.set_render_window_resize_handled();
        assert!(!window.is_render_window_resized());
    }
}
