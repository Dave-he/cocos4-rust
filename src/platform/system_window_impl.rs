/****************************************************************************
Rust implementation of platform-specific SystemWindow
Original C++ version Copyright (c) 2022-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::platform::interfaces::{ISystemWindow, ISystemWindowManager, SystemWindowInfo, SystemWindowMap};

#[derive(Debug)]
pub struct SystemWindow {
    id: u32,
    title: String,
    #[allow(dead_code)]
    x: i32,
    #[allow(dead_code)]
    y: i32,
    width: u32,
    height: u32,
    #[allow(dead_code)]
    flags: u32,
    cursor_enabled: bool,
}

impl SystemWindow {
    pub fn new(id: u32, info: &SystemWindowInfo) -> Self {
        SystemWindow {
            id,
            title: info.title.clone(),
            x: info.x,
            y: info.y,
            width: info.width as u32,
            height: info.height as u32,
            flags: info.flags as u32,
            cursor_enabled: true,
        }
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

impl ISystemWindow for SystemWindow {
    fn get_window_id(&self) -> u32 {
        self.id
    }

    fn get_window_handle(&self) -> usize {
        self.id as usize
    }

    fn get_view_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn set_cursor_enabled(&mut self, value: bool) {
        self.cursor_enabled = value;
    }
}

pub struct SystemWindowManager {
    windows: HashMap<u32, Arc<Mutex<SystemWindow>>>,
    next_id: u32,
}

impl SystemWindowManager {
    pub fn new() -> Self {
        SystemWindowManager {
            windows: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn get_window_by_id(&self, id: u32) -> Option<Arc<Mutex<SystemWindow>>> {
        self.windows.get(&id).cloned()
    }

    pub fn get_window_count(&self) -> usize {
        self.windows.len()
    }
}

impl Default for SystemWindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ISystemWindowManager for SystemWindowManager {
    fn init(&mut self) -> i32 {
        0
    }

    fn process_event(&mut self) {}

    fn create_window(&mut self, info: &SystemWindowInfo) -> Option<Arc<dyn ISystemWindow>> {
        let id = self.next_id;
        self.next_id += 1;
        let window = SystemWindow::new(id, info);
        let arc_window = Arc::new(Mutex::new(window));

        let arc_interface: Arc<dyn ISystemWindow> = Arc::new(SystemWindowProxy {
            id,
            title: info.title.clone(),
            width: info.width as u32,
            height: info.height as u32,
        });

        self.windows.insert(id, arc_window);
        Some(arc_interface)
    }

    fn get_window(&self, _window_id: u32) -> Option<Arc<dyn ISystemWindow>> {
        None
    }

    fn get_windows(&self) -> SystemWindowMap {
        HashMap::new()
    }
}

#[derive(Debug)]
struct SystemWindowProxy {
    id: u32,
    #[allow(dead_code)]
    title: String,
    width: u32,
    height: u32,
}

impl ISystemWindow for SystemWindowProxy {
    fn get_window_id(&self) -> u32 {
        self.id
    }

    fn get_window_handle(&self) -> usize {
        self.id as usize
    }

    fn get_view_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn set_cursor_enabled(&mut self, _value: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_window_new() {
        let info = SystemWindowInfo {
            title: "Test Window".to_string(),
            width: 1280,
            height: 720,
            ..Default::default()
        };
        let win = SystemWindow::new(1, &info);
        assert_eq!(win.get_window_id(), 1);
        assert_eq!(win.get_view_size(), (1280, 720));
        assert_eq!(win.get_title(), "Test Window");
    }

    #[test]
    fn test_system_window_manager_create() {
        let mut mgr = SystemWindowManager::new();
        assert_eq!(mgr.init(), 0);

        let info = SystemWindowInfo {
            title: "Main".to_string(),
            width: 800,
            height: 600,
            ..Default::default()
        };
        let window = mgr.create_window(&info);
        assert!(window.is_some());
        assert_eq!(mgr.get_window_count(), 1);
    }

    #[test]
    fn test_system_window_size() {
        let info = SystemWindowInfo {
            width: 1920,
            height: 1080,
            ..Default::default()
        };
        let mut win = SystemWindow::new(1, &info);
        assert_eq!(win.get_view_size(), (1920, 1080));
        win.set_size(800, 600);
        assert_eq!(win.get_view_size(), (800, 600));
    }
}
