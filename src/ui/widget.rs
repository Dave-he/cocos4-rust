use crate::math::Vec2;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WidgetAlignFlag: u32 {
        const TOP    = 1 << 0;
        const BOTTOM = 1 << 1;
        const LEFT   = 1 << 2;
        const RIGHT  = 1 << 3;
        const HORIZONTAL_CENTER = 1 << 4;
        const VERTICAL_CENTER   = 1 << 5;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetAlignMode {
    OnWindowResize,
    Always,
    Once,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Widget {
    pub align_flags: WidgetAlignFlag,
    pub align_mode: WidgetAlignMode,
    pub is_align_once: bool,
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    pub horizontal_center: f32,
    pub vertical_center: f32,
    pub is_absolute_horizontal: bool,
    pub is_absolute_vertical: bool,
}

impl Widget {
    pub fn new() -> Self {
        Widget {
            align_flags: WidgetAlignFlag::empty(),
            align_mode: WidgetAlignMode::OnWindowResize,
            is_align_once: false,
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
            horizontal_center: 0.0,
            vertical_center: 0.0,
            is_absolute_horizontal: true,
            is_absolute_vertical: true,
        }
    }

    pub fn is_aligned_top(&self) -> bool {
        self.align_flags.contains(WidgetAlignFlag::TOP)
    }

    pub fn is_aligned_bottom(&self) -> bool {
        self.align_flags.contains(WidgetAlignFlag::BOTTOM)
    }

    pub fn is_aligned_left(&self) -> bool {
        self.align_flags.contains(WidgetAlignFlag::LEFT)
    }

    pub fn is_aligned_right(&self) -> bool {
        self.align_flags.contains(WidgetAlignFlag::RIGHT)
    }

    pub fn compute_position(&self, node_size: Vec2, parent_size: Vec2) -> Vec2 {
        let mut x = 0.0f32;
        let mut y = 0.0f32;

        if self.align_flags.contains(WidgetAlignFlag::LEFT)
            && self.align_flags.contains(WidgetAlignFlag::RIGHT)
        {
            x = self.left;
        } else if self.align_flags.contains(WidgetAlignFlag::LEFT) {
            x = self.left;
        } else if self.align_flags.contains(WidgetAlignFlag::RIGHT) {
            x = parent_size.x - node_size.x - self.right;
        } else if self.align_flags.contains(WidgetAlignFlag::HORIZONTAL_CENTER) {
            x = (parent_size.x - node_size.x) / 2.0 + self.horizontal_center;
        }

        if self.align_flags.contains(WidgetAlignFlag::TOP)
            && self.align_flags.contains(WidgetAlignFlag::BOTTOM)
        {
            y = self.top;
        } else if self.align_flags.contains(WidgetAlignFlag::TOP) {
            y = self.top;
        } else if self.align_flags.contains(WidgetAlignFlag::BOTTOM) {
            y = parent_size.y - node_size.y - self.bottom;
        } else if self.align_flags.contains(WidgetAlignFlag::VERTICAL_CENTER) {
            y = (parent_size.y - node_size.y) / 2.0 + self.vertical_center;
        }

        Vec2::new(x, y)
    }

    pub fn compute_size(&self, parent_size: Vec2) -> Option<Vec2> {
        let stretch_h = self.align_flags.contains(WidgetAlignFlag::LEFT)
            && self.align_flags.contains(WidgetAlignFlag::RIGHT);
        let stretch_v = self.align_flags.contains(WidgetAlignFlag::TOP)
            && self.align_flags.contains(WidgetAlignFlag::BOTTOM);

        if stretch_h || stretch_v {
            let w = if stretch_h { parent_size.x - self.left - self.right } else { 0.0 };
            let h = if stretch_v { parent_size.y - self.top - self.bottom } else { 0.0 };
            Some(Vec2::new(w, h))
        } else {
            None
        }
    }
}

impl Default for Widget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_new() {
        let w = Widget::new();
        assert!(w.align_flags.is_empty());
    }

    #[test]
    fn test_widget_align_left() {
        let mut w = Widget::new();
        w.align_flags = WidgetAlignFlag::LEFT;
        w.left = 10.0;
        let pos = w.compute_position(Vec2::new(50.0, 50.0), Vec2::new(200.0, 200.0));
        assert!((pos.x - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_widget_align_right() {
        let mut w = Widget::new();
        w.align_flags = WidgetAlignFlag::RIGHT;
        w.right = 10.0;
        let pos = w.compute_position(Vec2::new(50.0, 50.0), Vec2::new(200.0, 200.0));
        assert!((pos.x - 140.0).abs() < 1e-4);
    }

    #[test]
    fn test_widget_horizontal_center() {
        let mut w = Widget::new();
        w.align_flags = WidgetAlignFlag::HORIZONTAL_CENTER;
        w.horizontal_center = 0.0;
        let pos = w.compute_position(Vec2::new(50.0, 50.0), Vec2::new(200.0, 200.0));
        assert!((pos.x - 75.0).abs() < 1e-4);
    }

    #[test]
    fn test_widget_stretch_horizontal() {
        let mut w = Widget::new();
        w.align_flags = WidgetAlignFlag::LEFT | WidgetAlignFlag::RIGHT;
        w.left = 10.0;
        w.right = 10.0;
        let size = w.compute_size(Vec2::new(200.0, 200.0));
        assert!(size.is_some());
        assert!((size.unwrap().x - 180.0).abs() < 1e-4);
    }

    #[test]
    fn test_widget_no_align_no_stretch() {
        let w = Widget::new();
        let size = w.compute_size(Vec2::new(200.0, 200.0));
        assert!(size.is_none());
    }
}
