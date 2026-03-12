/// Window size
pub type Size = (u32, u32);

/// Window flags for window creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowFlags;

impl WindowFlags {
    pub const FULLSCREEN: u32 = 0x0000_0001;
    pub const OPENGL: u32 = 0x0000_0002;
    pub const SHOWN: u32 = 0x0000_0004;
    pub const HIDDEN: u32 = 0x0000_0008;
    pub const BORDERLESS: u32 = 0x0000_0010;
    pub const RESIZABLE: u32 = 0x0000_0020;
    pub const MINIMIZED: u32 = 0x0000_0040;
    pub const MAXIMIZED: u32 = 0x0000_0080;
    pub const INPUT_GRABBED: u32 = 0x0000_0100;
    pub const INPUT_FOCUS: u32 = 0x0000_0200;
    pub const MOUSE_FOCUS: u32 = 0x0000_0400;
    pub const FULLSCREEN_DESKTOP: u32 = Self::FULLSCREEN | 0x0000_1000;
    pub const FOREIGN: u32 = 0x0000_0800;
    pub const ALLOW_HIGHDPI: u32 = 0x0000_2000;
    pub const MOUSE_CAPTURE: u32 = 0x0000_4000;
    pub const ALWAYS_ON_TOP: u32 = 0x0000_8000;
    pub const SKIP_TASKBAR: u32 = 0x0001_0000;
    pub const UTILITY: u32 = 0x0002_0000;
    pub const TOOLTIP: u32 = 0x0004_0000;
    pub const POPUP_MENU: u32 = 0x0008_0000;
    pub const VULKAN: u32 = 0x1000_0000;
}

/// Main window ID constant
pub const MAIN_WINDOW_ID: u32 = 1;

/// System window interface
pub trait ISystemWindow: Send + Sync {
    fn get_window_id(&self) -> u32;
    fn get_window_handle(&self) -> usize;
    fn get_view_size(&self) -> Size;
    fn set_cursor_enabled(&mut self, value: bool);
}
