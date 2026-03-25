use std::collections::{HashMap, HashSet, VecDeque};
use crate::input::types::{
    EventKeyboard, EventMouse, EventTouch, InputEventType, KeyCode, MouseButton, Touch,
};
use crate::math::Vec2;

type KeyboardCallback = Box<dyn Fn(&EventKeyboard) + Send + Sync>;
type MouseCallback = Box<dyn Fn(&EventMouse) + Send + Sync>;
type TouchCallback = Box<dyn Fn(&EventTouch) + Send + Sync>;

pub struct Input {
    keyboard_listeners: Vec<(InputEventType, KeyboardCallback)>,
    mouse_listeners: Vec<(InputEventType, MouseCallback)>,
    touch_listeners: Vec<(InputEventType, TouchCallback)>,

    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_up: HashSet<KeyCode>,

    mouse_position: Vec2,
    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_scroll: Vec2,

    touches: HashMap<u32, Touch>,
    touch_event_queue: VecDeque<EventTouch>,
}

impl Input {
    pub fn new() -> Self {
        Input {
            keyboard_listeners: Vec::new(),
            mouse_listeners: Vec::new(),
            touch_listeners: Vec::new(),
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_up: HashSet::new(),
            mouse_position: Vec2::ZERO,
            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_scroll: Vec2::ZERO,
            touches: HashMap::new(),
            touch_event_queue: VecDeque::new(),
        }
    }

    pub fn on_keyboard<F: Fn(&EventKeyboard) + Send + Sync + 'static>(
        &mut self,
        event_type: InputEventType,
        cb: F,
    ) {
        self.keyboard_listeners.push((event_type, Box::new(cb)));
    }

    pub fn on_mouse<F: Fn(&EventMouse) + Send + Sync + 'static>(
        &mut self,
        event_type: InputEventType,
        cb: F,
    ) {
        self.mouse_listeners.push((event_type, Box::new(cb)));
    }

    pub fn on_touch<F: Fn(&EventTouch) + Send + Sync + 'static>(
        &mut self,
        event_type: InputEventType,
        cb: F,
    ) {
        self.touch_listeners.push((event_type, Box::new(cb)));
    }

    pub fn off_keyboard(&mut self, event_type: &InputEventType) {
        self.keyboard_listeners.retain(|(e, _)| e != event_type);
    }

    pub fn off_mouse(&mut self, event_type: &InputEventType) {
        self.mouse_listeners.retain(|(e, _)| e != event_type);
    }

    pub fn off_touch(&mut self, event_type: &InputEventType) {
        self.touch_listeners.retain(|(e, _)| e != event_type);
    }

    pub fn dispatch_key_down(&mut self, key_code: KeyCode) {
        if !self.keys_pressed.contains(&key_code) {
            self.keys_down.insert(key_code);
        }
        self.keys_pressed.insert(key_code);
        let event = EventKeyboard::new(key_code, InputEventType::KeyDown);
        for (et, cb) in &self.keyboard_listeners {
            if *et == InputEventType::KeyDown {
                cb(&event);
            }
        }
    }

    pub fn dispatch_key_up(&mut self, key_code: KeyCode) {
        self.keys_pressed.remove(&key_code);
        self.keys_up.insert(key_code);
        let event = EventKeyboard::new(key_code, InputEventType::KeyUp);
        for (et, cb) in &self.keyboard_listeners {
            if *et == InputEventType::KeyUp {
                cb(&event);
            }
        }
    }

    pub fn dispatch_mouse_down(&mut self, button: MouseButton, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y);
        self.mouse_buttons_down.insert(button);
        self.mouse_buttons_pressed.insert(button);
        let event = EventMouse::new(button, InputEventType::MouseDown).with_position(x, y);
        for (et, cb) in &self.mouse_listeners {
            if *et == InputEventType::MouseDown {
                cb(&event);
            }
        }
    }

    pub fn dispatch_mouse_up(&mut self, button: MouseButton, x: f32, y: f32) {
        self.mouse_buttons_pressed.remove(&button);
        let event = EventMouse::new(button, InputEventType::MouseUp).with_position(x, y);
        for (et, cb) in &self.mouse_listeners {
            if *et == InputEventType::MouseUp {
                cb(&event);
            }
        }
    }

    pub fn dispatch_mouse_move(&mut self, x: f32, y: f32) {
        let old = self.mouse_position;
        self.mouse_position = Vec2::new(x, y);
        let mut event = EventMouse::new(MouseButton::Left, InputEventType::MouseMove)
            .with_position(x, y);
        event.delta = Vec2::new(x - old.x, y - old.y);
        for (et, cb) in &self.mouse_listeners {
            if *et == InputEventType::MouseMove {
                cb(&event);
            }
        }
    }

    pub fn dispatch_mouse_scroll(&mut self, dx: f32, dy: f32) {
        self.mouse_scroll = Vec2::new(dx, dy);
        let event = EventMouse::new(MouseButton::Left, InputEventType::MouseScroll)
            .with_scroll(dx, dy);
        for (et, cb) in &self.mouse_listeners {
            if *et == InputEventType::MouseScroll {
                cb(&event);
            }
        }
    }

    pub fn dispatch_touch_start(&mut self, id: u32, x: f32, y: f32) {
        let touch = Touch::new(id, x, y);
        self.touches.insert(id, touch.clone());
        let event = EventTouch::new(InputEventType::TouchStart).with_touch(touch);
        for (et, cb) in &self.touch_listeners {
            if *et == InputEventType::TouchStart {
                cb(&event);
            }
        }
    }

    pub fn dispatch_touch_move(&mut self, id: u32, x: f32, y: f32) {
        if let Some(touch) = self.touches.get_mut(&id) {
            touch.update(x, y);
            let touch_clone = touch.clone();
            let event = EventTouch::new(InputEventType::TouchMove).with_touch(touch_clone);
            for (et, cb) in &self.touch_listeners {
                if *et == InputEventType::TouchMove {
                    cb(&event);
                }
            }
        }
    }

    pub fn dispatch_touch_end(&mut self, id: u32) {
        if let Some(touch) = self.touches.remove(&id) {
            let event = EventTouch::new(InputEventType::TouchEnd).with_touch(touch);
            for (et, cb) in &self.touch_listeners {
                if *et == InputEventType::TouchEnd {
                    cb(&event);
                }
            }
        }
    }

    pub fn dispatch_touch_cancel(&mut self, id: u32) {
        if let Some(touch) = self.touches.remove(&id) {
            let event = EventTouch::new(InputEventType::TouchCancel).with_touch(touch);
            for (et, cb) in &self.touch_listeners {
                if *et == InputEventType::TouchCancel {
                    cb(&event);
                }
            }
        }
    }

    pub fn frame_end(&mut self) {
        self.keys_down.clear();
        self.keys_up.clear();
        self.mouse_buttons_down.clear();
        self.mouse_scroll = Vec2::ZERO;
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    pub fn is_key_pressing(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_key_up(&self, key: KeyCode) -> bool {
        self.keys_up.contains(&key)
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    pub fn is_mouse_button_pressing(&self, button: MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub fn get_mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn get_mouse_scroll(&self) -> Vec2 {
        self.mouse_scroll
    }

    pub fn get_touch(&self, id: u32) -> Option<&Touch> {
        self.touches.get(&id)
    }

    pub fn get_touch_count(&self) -> usize {
        self.touches.len()
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_input_new() {
        let input = Input::new();
        assert_eq!(input.get_touch_count(), 0);
        assert_eq!(input.get_mouse_position(), Vec2::ZERO);
    }

    #[test]
    fn test_key_down_up() {
        let mut input = Input::new();
        input.dispatch_key_down(KeyCode::Space);
        assert!(input.is_key_down(KeyCode::Space));
        assert!(input.is_key_pressing(KeyCode::Space));

        input.dispatch_key_up(KeyCode::Space);
        assert!(!input.is_key_pressing(KeyCode::Space));
        assert!(input.is_key_up(KeyCode::Space));
    }

    #[test]
    fn test_frame_end_clears_transient() {
        let mut input = Input::new();
        input.dispatch_key_down(KeyCode::Enter);
        input.frame_end();
        assert!(!input.is_key_down(KeyCode::Enter));
        assert!(!input.is_key_up(KeyCode::Enter));
    }

    #[test]
    fn test_mouse_down_up() {
        let mut input = Input::new();
        input.dispatch_mouse_down(MouseButton::Left, 100.0, 200.0);
        assert!(input.is_mouse_button_down(MouseButton::Left));
        assert!(input.is_mouse_button_pressing(MouseButton::Left));
        assert_eq!(input.get_mouse_position(), Vec2::new(100.0, 200.0));

        input.dispatch_mouse_up(MouseButton::Left, 100.0, 200.0);
        assert!(!input.is_mouse_button_pressing(MouseButton::Left));
    }

    #[test]
    fn test_mouse_scroll() {
        let mut input = Input::new();
        input.dispatch_mouse_scroll(0.0, -3.0);
        let s = input.get_mouse_scroll();
        assert!((s.y - (-3.0)).abs() < 1e-5);
    }

    #[test]
    fn test_touch_lifecycle() {
        let mut input = Input::new();
        input.dispatch_touch_start(0, 50.0, 100.0);
        assert_eq!(input.get_touch_count(), 1);
        input.dispatch_touch_move(0, 60.0, 110.0);
        assert_eq!(input.get_touch(0).unwrap().position.x, 60.0);
        input.dispatch_touch_end(0);
        assert_eq!(input.get_touch_count(), 0);
    }

    #[test]
    fn test_touch_cancel() {
        let mut input = Input::new();
        input.dispatch_touch_start(1, 0.0, 0.0);
        input.dispatch_touch_cancel(1);
        assert_eq!(input.get_touch_count(), 0);
    }

    #[test]
    fn test_keyboard_callback() {
        let mut input = Input::new();
        let received = Arc::new(Mutex::new(None::<KeyCode>));
        let r = Arc::clone(&received);
        input.on_keyboard(InputEventType::KeyDown, move |e| {
            *r.lock().unwrap() = Some(e.key_code);
        });
        input.dispatch_key_down(KeyCode::KeyA);
        assert_eq!(*received.lock().unwrap(), Some(KeyCode::KeyA));
    }

    #[test]
    fn test_mouse_callback() {
        let mut input = Input::new();
        let pos = Arc::new(Mutex::new(Vec2::ZERO));
        let p = Arc::clone(&pos);
        input.on_mouse(InputEventType::MouseDown, move |e| {
            *p.lock().unwrap() = e.position;
        });
        input.dispatch_mouse_down(MouseButton::Right, 30.0, 40.0);
        let got = *pos.lock().unwrap();
        assert!((got.x - 30.0).abs() < 1e-5);
    }

    #[test]
    fn test_touch_callback() {
        let mut input = Input::new();
        let id = Arc::new(Mutex::new(999u32));
        let i = Arc::clone(&id);
        input.on_touch(InputEventType::TouchStart, move |e| {
            if let Some(t) = e.get_touch() {
                *i.lock().unwrap() = t.id;
            }
        });
        input.dispatch_touch_start(5, 0.0, 0.0);
        assert_eq!(*id.lock().unwrap(), 5);
    }

    #[test]
    fn test_off_keyboard() {
        let mut input = Input::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        input.on_keyboard(InputEventType::KeyDown, move |_| {
            *c.lock().unwrap() += 1;
        });
        input.off_keyboard(&InputEventType::KeyDown);
        input.dispatch_key_down(KeyCode::Space);
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn test_multi_touch() {
        let mut input = Input::new();
        input.dispatch_touch_start(0, 10.0, 10.0);
        input.dispatch_touch_start(1, 20.0, 20.0);
        assert_eq!(input.get_touch_count(), 2);
        input.dispatch_touch_end(0);
        assert_eq!(input.get_touch_count(), 1);
    }
}
