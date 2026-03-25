use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyCode {
    None = 0,
    Backspace = 8,
    Tab = 9,
    Enter = 13,
    ShiftLeft = 16,
    ControlLeft = 17,
    AltLeft = 18,
    Pause = 19,
    CapsLock = 20,
    Escape = 27,
    Space = 32,
    PageUp = 33,
    PageDown = 34,
    End = 35,
    Home = 36,
    ArrowLeft = 37,
    ArrowUp = 38,
    ArrowRight = 39,
    ArrowDown = 40,
    Delete = 46,
    Digit0 = 48,
    Digit1 = 49,
    Digit2 = 50,
    Digit3 = 51,
    Digit4 = 52,
    Digit5 = 53,
    Digit6 = 54,
    Digit7 = 55,
    Digit8 = 56,
    Digit9 = 57,
    KeyA = 65,
    KeyB = 66,
    KeyC = 67,
    KeyD = 68,
    KeyE = 69,
    KeyF = 70,
    KeyG = 71,
    KeyH = 72,
    KeyI = 73,
    KeyJ = 74,
    KeyK = 75,
    KeyL = 76,
    KeyM = 77,
    KeyN = 78,
    KeyO = 79,
    KeyP = 80,
    KeyQ = 81,
    KeyR = 82,
    KeyS = 83,
    KeyT = 84,
    KeyU = 85,
    KeyV = 86,
    KeyW = 87,
    KeyX = 88,
    KeyY = 89,
    KeyZ = 90,
    F1 = 112,
    F2 = 113,
    F3 = 114,
    F4 = 115,
    F5 = 116,
    F6 = 117,
    F7 = 118,
    F8 = 119,
    F9 = 120,
    F10 = 121,
    F11 = 122,
    F12 = 123,
    NumLock = 144,
    ScrollLock = 145,
    ShiftRight = 161,
    ControlRight = 162,
    AltRight = 163,
    Semicolon = 186,
    Equal = 187,
    Comma = 188,
    Minus = 189,
    Period = 190,
    Slash = 191,
    Backquote = 192,
    BracketLeft = 219,
    Backslash = 220,
    BracketRight = 221,
    Quote = 222,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
    Button4 = 3,
    Button5 = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputEventType {
    KeyDown,
    KeyUp,
    KeyPressing,
    MouseDown,
    MouseUp,
    MouseMove,
    MouseScroll,
    TouchStart,
    TouchMove,
    TouchEnd,
    TouchCancel,
    GamepadChange,
    HandleInput,
    HmdPose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GamepadButton {
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    LB = 4,
    RB = 5,
    Back = 8,
    Start = 9,
    LeftStick = 10,
    RightStick = 11,
    DpadUp = 12,
    DpadDown = 13,
    DpadLeft = 14,
    DpadRight = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GamepadAxis {
    LeftStickX = 0,
    LeftStickY = 1,
    RightStickX = 2,
    RightStickY = 3,
    LeftTrigger = 4,
    RightTrigger = 5,
}

#[derive(Debug, Clone)]
pub struct Touch {
    pub id: u32,
    pub position: Vec2,
    pub prev_position: Vec2,
    pub start_position: Vec2,
    pub force: f32,
}

impl Touch {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        let pos = Vec2::new(x, y);
        Touch {
            id,
            position: pos,
            prev_position: pos,
            start_position: pos,
            force: 0.0,
        }
    }

    pub fn get_delta(&self) -> Vec2 {
        Vec2::new(
            self.position.x - self.prev_position.x,
            self.position.y - self.prev_position.y,
        )
    }

    pub fn get_start_delta(&self) -> Vec2 {
        Vec2::new(
            self.position.x - self.start_position.x,
            self.position.y - self.start_position.y,
        )
    }

    pub fn update(&mut self, x: f32, y: f32) {
        self.prev_position = self.position;
        self.position = Vec2::new(x, y);
    }
}

#[derive(Debug, Clone)]
pub struct EventKeyboard {
    pub key_code: KeyCode,
    pub event_type: InputEventType,
}

impl EventKeyboard {
    pub fn new(key_code: KeyCode, event_type: InputEventType) -> Self {
        EventKeyboard { key_code, event_type }
    }
}

#[derive(Debug, Clone)]
pub struct EventMouse {
    pub button: MouseButton,
    pub position: Vec2,
    pub delta: Vec2,
    pub scroll_delta: Vec2,
    pub event_type: InputEventType,
}

impl EventMouse {
    pub fn new(button: MouseButton, event_type: InputEventType) -> Self {
        EventMouse {
            button,
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            event_type,
        }
    }

    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Vec2::new(x, y);
        self
    }

    pub fn with_scroll(mut self, dx: f32, dy: f32) -> Self {
        self.scroll_delta = Vec2::new(dx, dy);
        self
    }
}

#[derive(Debug, Clone)]
pub struct EventTouch {
    pub touches: Vec<Touch>,
    pub event_type: InputEventType,
}

impl EventTouch {
    pub fn new(event_type: InputEventType) -> Self {
        EventTouch { touches: Vec::new(), event_type }
    }

    pub fn with_touch(mut self, touch: Touch) -> Self {
        self.touches.push(touch);
        self
    }

    pub fn get_touch(&self) -> Option<&Touch> {
        self.touches.first()
    }

    pub fn get_all_touches(&self) -> &[Touch] {
        &self.touches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_new() {
        let t = Touch::new(0, 100.0, 200.0);
        assert_eq!(t.id, 0);
        assert_eq!(t.position.x, 100.0);
        assert_eq!(t.position.y, 200.0);
    }

    #[test]
    fn test_touch_delta() {
        let mut t = Touch::new(0, 0.0, 0.0);
        t.update(10.0, 20.0);
        let d = t.get_delta();
        assert!((d.x - 10.0).abs() < 1e-5);
        assert!((d.y - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_touch_start_delta() {
        let mut t = Touch::new(0, 5.0, 5.0);
        t.update(15.0, 25.0);
        let d = t.get_start_delta();
        assert!((d.x - 10.0).abs() < 1e-5);
        assert!((d.y - 20.0).abs() < 1e-5);
    }

    #[test]
    fn test_event_mouse_with_position() {
        let e = EventMouse::new(MouseButton::Left, InputEventType::MouseDown)
            .with_position(50.0, 100.0);
        assert_eq!(e.position.x, 50.0);
        assert_eq!(e.position.y, 100.0);
    }

    #[test]
    fn test_event_mouse_scroll() {
        let e = EventMouse::new(MouseButton::Middle, InputEventType::MouseScroll)
            .with_scroll(0.0, -3.0);
        assert!((e.scroll_delta.y - (-3.0)).abs() < 1e-5);
    }

    #[test]
    fn test_event_touch_get_touch() {
        let touch = Touch::new(1, 30.0, 40.0);
        let e = EventTouch::new(InputEventType::TouchStart).with_touch(touch);
        assert!(e.get_touch().is_some());
        assert_eq!(e.get_all_touches().len(), 1);
    }

    #[test]
    fn test_event_keyboard() {
        let e = EventKeyboard::new(KeyCode::Space, InputEventType::KeyDown);
        assert_eq!(e.key_code, KeyCode::Space);
        assert_eq!(e.event_type, InputEventType::KeyDown);
    }
}
