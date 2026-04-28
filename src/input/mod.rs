pub mod types;
#[allow(clippy::module_inception)]
pub mod input;

pub use types::{
    KeyCode, MouseButton, Touch, EventKeyboard, EventMouse, EventTouch,
    InputEventType, GamepadButton, GamepadAxis,
};
pub use input::Input;
