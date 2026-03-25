use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollViewEventType {
    None,
    ScrollToTop,
    ScrollToBottom,
    ScrollToLeft,
    ScrollToRight,
    Scrolling,
    BounceBottom,
    BounceLeft,
    BounceRight,
    BounceTop,
    ScrollEnded,
    ScrollBegan,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollViewBounds {
    pub content_size: Vec2,
    pub view_size: Vec2,
}

type ScrollEventCallback = Box<dyn Fn(ScrollViewEventType) + Send + Sync>;

pub struct ScrollView {
    pub horizontal: bool,
    pub vertical: bool,
    pub inertia: bool,
    pub brake: f32,
    pub elastic: bool,
    pub bounce_duration: f32,
    pub deceleration_rate: f32,

    scroll_offset: Vec2,
    content_size: Vec2,
    view_size: Vec2,
    velocity: Vec2,
    is_scrolling: bool,
    listeners: Vec<(ScrollViewEventType, ScrollEventCallback)>,
}

impl ScrollView {
    pub fn new() -> Self {
        ScrollView {
            horizontal: false,
            vertical: true,
            inertia: true,
            brake: 0.75,
            elastic: true,
            bounce_duration: 0.23,
            deceleration_rate: 0.7,
            scroll_offset: Vec2::ZERO,
            content_size: Vec2::new(100.0, 200.0),
            view_size: Vec2::new(100.0, 100.0),
            velocity: Vec2::ZERO,
            is_scrolling: false,
            listeners: Vec::new(),
        }
    }

    pub fn set_content_size(&mut self, size: Vec2) {
        self.content_size = size;
    }

    pub fn set_view_size(&mut self, size: Vec2) {
        self.view_size = size;
    }

    pub fn get_scroll_offset(&self) -> Vec2 {
        self.scroll_offset
    }

    pub fn scroll_to_top(&mut self, duration: f32) {
        let _ = duration;
        self.scroll_offset.y = 0.0;
        self.emit_event(ScrollViewEventType::ScrollToTop);
    }

    pub fn scroll_to_bottom(&mut self, duration: f32) {
        let _ = duration;
        let max_y = (self.content_size.y - self.view_size.y).max(0.0);
        self.scroll_offset.y = max_y;
        self.emit_event(ScrollViewEventType::ScrollToBottom);
    }

    pub fn scroll_to_left(&mut self, duration: f32) {
        let _ = duration;
        self.scroll_offset.x = 0.0;
        self.emit_event(ScrollViewEventType::ScrollToLeft);
    }

    pub fn scroll_to_right(&mut self, duration: f32) {
        let _ = duration;
        let max_x = (self.content_size.x - self.view_size.x).max(0.0);
        self.scroll_offset.x = max_x;
        self.emit_event(ScrollViewEventType::ScrollToRight);
    }

    pub fn scroll_to_offset(&mut self, offset: Vec2, duration: f32) {
        let _ = duration;
        self.scroll_offset = self.clamp_offset(offset);
        self.emit_event(ScrollViewEventType::Scrolling);
    }

    pub fn scroll_by(&mut self, delta: Vec2) {
        let new_offset = Vec2::new(
            self.scroll_offset.x + delta.x,
            self.scroll_offset.y + delta.y,
        );
        self.scroll_offset = self.clamp_offset(new_offset);
        self.emit_event(ScrollViewEventType::Scrolling);
    }

    fn clamp_offset(&self, offset: Vec2) -> Vec2 {
        if self.elastic {
            offset
        } else {
            Vec2::new(
                offset.x.clamp(0.0, (self.content_size.x - self.view_size.x).max(0.0)),
                offset.y.clamp(0.0, (self.content_size.y - self.view_size.y).max(0.0)),
            )
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.inertia && self.velocity != Vec2::ZERO {
            let dv = Vec2::new(
                self.velocity.x * self.deceleration_rate,
                self.velocity.y * self.deceleration_rate,
            );
            self.scroll_offset = Vec2::new(
                self.scroll_offset.x + dv.x * dt,
                self.scroll_offset.y + dv.y * dt,
            );
            self.scroll_offset = self.clamp_offset(self.scroll_offset);
            self.velocity = Vec2::new(
                self.velocity.x * (1.0 - self.brake),
                self.velocity.y * (1.0 - self.brake),
            );
            let speed = (self.velocity.x * self.velocity.x + self.velocity.y * self.velocity.y).sqrt();
            if speed < 0.1 {
                self.velocity = Vec2::ZERO;
            }
        }
    }

    pub fn on_event<F: Fn(ScrollViewEventType) + Send + Sync + 'static>(
        &mut self,
        event: ScrollViewEventType,
        f: F,
    ) {
        self.listeners.push((event, Box::new(f)));
    }

    fn emit_event(&self, event: ScrollViewEventType) {
        for (e, cb) in &self.listeners {
            if *e == event || *e == ScrollViewEventType::Scrolling {
                cb(event);
            }
        }
    }

    pub fn is_at_top(&self) -> bool {
        self.scroll_offset.y <= 0.0
    }

    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset.y >= (self.content_size.y - self.view_size.y).max(0.0)
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ScrollView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollView")
            .field("horizontal", &self.horizontal)
            .field("vertical", &self.vertical)
            .field("scroll_offset", &self.scroll_offset)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn make_sv() -> ScrollView {
        let mut sv = ScrollView::new();
        sv.set_content_size(Vec2::new(100.0, 500.0));
        sv.set_view_size(Vec2::new(100.0, 100.0));
        sv
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut sv = make_sv();
        sv.scroll_to_bottom(0.0);
        assert_eq!(sv.get_scroll_offset().y, 400.0);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut sv = make_sv();
        sv.scroll_to_bottom(0.0);
        sv.scroll_to_top(0.0);
        assert_eq!(sv.get_scroll_offset().y, 0.0);
    }

    #[test]
    fn test_scroll_by() {
        let mut sv = make_sv();
        sv.elastic = false;
        sv.scroll_by(Vec2::new(0.0, 50.0));
        assert_eq!(sv.get_scroll_offset().y, 50.0);
    }

    #[test]
    fn test_scroll_clamp() {
        let mut sv = make_sv();
        sv.elastic = false;
        sv.scroll_to_offset(Vec2::new(0.0, 1000.0), 0.0);
        assert_eq!(sv.get_scroll_offset().y, 400.0);
    }

    #[test]
    fn test_is_at_top() {
        let mut sv = make_sv();
        assert!(sv.is_at_top());
        sv.scroll_by(Vec2::new(0.0, 10.0));
        assert!(!sv.is_at_top());
    }

    #[test]
    fn test_is_at_bottom() {
        let mut sv = make_sv();
        sv.elastic = false;
        sv.scroll_to_bottom(0.0);
        assert!(sv.is_at_bottom());
    }

    #[test]
    fn test_event_callback() {
        let mut sv = make_sv();
        let received = Arc::new(Mutex::new(false));
        let r = Arc::clone(&received);
        sv.on_event(ScrollViewEventType::ScrollToTop, move |_| {
            *r.lock().unwrap() = true;
        });
        sv.scroll_to_top(0.0);
        assert!(*received.lock().unwrap());
    }
}
