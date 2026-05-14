#[allow(clippy::module_inception)]
pub mod input;
pub mod types;

pub use input::Input;
pub use types::{
    EventKeyboard, EventMouse, EventTouch, GamepadAxis, GamepadButton, InputEventType, KeyCode,
    MouseButton, Touch,
};
