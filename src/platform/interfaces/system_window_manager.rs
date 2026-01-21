use std::collections::HashMap;

use super::system_window::ISystemWindow;

/// System window info
#[derive(Debug, Clone, Default)]
pub struct SystemWindowInfo {
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub flags: i32,
    pub external_handle: Option<usize>,
}

/// System window map type
pub type SystemWindowMap = HashMap<u32, Box<dyn ISystemWindow>>;

/// System window manager interface
pub trait ISystemWindowManager {
    /// Initialize the NativeWindow environment
    /// Returns 0 on success, -1 on failure
    fn init(&mut self) -> i32;

    /// Process messages at the PAL layer
    fn process_event(&mut self);

    /// Create a system window
    /// Returns the created window or None if failed
    fn create_window(&mut self, info: &SystemWindowInfo) -> Option<Box<dyn ISystemWindow>>;

    /// Find a system window by ID
    fn get_window(&self, window_id: u32) -> Option<&dyn ISystemWindow>;

    /// Get all windows
    fn get_windows(&self) -> &SystemWindowMap;
}
