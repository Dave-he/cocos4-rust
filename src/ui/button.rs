use crate::math::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTransition {
    None,
    Color,
    Sprite,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEventType {
    Click,
}

type ButtonCallback = Box<dyn Fn() + Send + Sync>;

pub struct Button {
    pub interactable: bool,
    pub transition: ButtonTransition,
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    pub normal_scale: f32,
    pub pressed_scale: f32,
    pub duration: f32,
    state: ButtonState,
    click_listeners: Vec<ButtonCallback>,
}

impl Button {
    pub fn new() -> Self {
        Button {
            interactable: true,
            transition: ButtonTransition::Color,
            normal_color: Color::WHITE,
            hover_color: Color::new(200, 200, 200, 255),
            pressed_color: Color::new(150, 150, 150, 255),
            disabled_color: Color::new(100, 100, 100, 255),
            normal_scale: 1.0,
            pressed_scale: 0.9,
            duration: 0.1,
            state: ButtonState::Normal,
            click_listeners: Vec::new(),
        }
    }

    pub fn get_state(&self) -> ButtonState {
        self.state
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.interactable = enabled;
        self.state = if enabled { ButtonState::Normal } else { ButtonState::Disabled };
    }

    pub fn on_click<F: Fn() + Send + Sync + 'static>(&mut self, f: F) {
        self.click_listeners.push(Box::new(f));
    }

    pub fn simulate_hover(&mut self) {
        if !self.interactable { return; }
        self.state = ButtonState::Hover;
    }

    pub fn simulate_press(&mut self) {
        if !self.interactable { return; }
        self.state = ButtonState::Pressed;
    }

    pub fn simulate_release(&mut self) {
        if !self.interactable { return; }
        if self.state == ButtonState::Pressed {
            self.state = ButtonState::Normal;
            self.fire_click();
        }
    }

    pub fn simulate_leave(&mut self) {
        if self.state != ButtonState::Disabled {
            self.state = ButtonState::Normal;
        }
    }

    fn fire_click(&self) {
        for cb in &self.click_listeners {
            cb();
        }
    }

    pub fn get_current_color(&self) -> Color {
        match self.state {
            ButtonState::Normal => self.normal_color,
            ButtonState::Hover => self.hover_color,
            ButtonState::Pressed => self.pressed_color,
            ButtonState::Disabled => self.disabled_color,
        }
    }

    pub fn get_current_scale(&self) -> f32 {
        match self.transition {
            ButtonTransition::Scale => match self.state {
                ButtonState::Pressed => self.pressed_scale,
                _ => self.normal_scale,
            },
            _ => self.normal_scale,
        }
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("interactable", &self.interactable)
            .field("transition", &self.transition)
            .field("state", &self.state)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_button_new() {
        let b = Button::new();
        assert!(b.interactable);
        assert_eq!(b.get_state(), ButtonState::Normal);
    }

    #[test]
    fn test_button_click() {
        let mut b = Button::new();
        let clicked = Arc::new(Mutex::new(false));
        let c = Arc::clone(&clicked);
        b.on_click(move || { *c.lock().unwrap() = true; });
        b.simulate_press();
        b.simulate_release();
        assert!(*clicked.lock().unwrap());
    }

    #[test]
    fn test_button_disabled_no_click() {
        let mut b = Button::new();
        let clicked = Arc::new(Mutex::new(false));
        let c = Arc::clone(&clicked);
        b.on_click(move || { *c.lock().unwrap() = true; });
        b.set_enabled(false);
        b.simulate_press();
        b.simulate_release();
        assert!(!*clicked.lock().unwrap());
    }

    #[test]
    fn test_button_state_transitions() {
        let mut b = Button::new();
        b.simulate_hover();
        assert_eq!(b.get_state(), ButtonState::Hover);
        b.simulate_press();
        assert_eq!(b.get_state(), ButtonState::Pressed);
        b.simulate_release();
        assert_eq!(b.get_state(), ButtonState::Normal);
    }

    #[test]
    fn test_button_disabled_state() {
        let mut b = Button::new();
        b.set_enabled(false);
        assert_eq!(b.get_state(), ButtonState::Disabled);
        assert_eq!(b.get_current_color(), b.disabled_color);
    }

    #[test]
    fn test_button_scale_transition() {
        let mut b = Button::new();
        b.transition = ButtonTransition::Scale;
        b.simulate_press();
        assert!((b.get_current_scale() - 0.9).abs() < 1e-5);
        b.simulate_release();
        assert!((b.get_current_scale() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_button_multi_click_listeners() {
        let mut b = Button::new();
        let count = Arc::new(Mutex::new(0u32));
        for _ in 0..3 {
            let c = Arc::clone(&count);
            b.on_click(move || { *c.lock().unwrap() += 1; });
        }
        b.simulate_press();
        b.simulate_release();
        assert_eq!(*count.lock().unwrap(), 3);
    }
}
