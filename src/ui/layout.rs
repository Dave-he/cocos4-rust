use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    None = 0,
    Horizontal = 1,
    Vertical = 2,
    Grid = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutResizeMode {
    None = 0,
    Container = 1,
    Children = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutPadding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for LayoutPadding {
    fn default() -> Self {
        LayoutPadding { top: 0.0, bottom: 0.0, left: 0.0, right: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct ChildLayout {
    pub position: Vec2,
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub layout_type: LayoutType,
    pub resize_mode: LayoutResizeMode,
    pub direction: LayoutDirection,
    pub spacing_x: f32,
    pub spacing_y: f32,
    pub padding: LayoutPadding,
    pub content_size: Vec2,
    pub cell_size: Vec2,
    pub affected_by_scale: bool,
}

impl Layout {
    pub fn new() -> Self {
        Layout {
            layout_type: LayoutType::None,
            resize_mode: LayoutResizeMode::None,
            direction: LayoutDirection::LeftToRight,
            spacing_x: 0.0,
            spacing_y: 0.0,
            padding: LayoutPadding::default(),
            content_size: Vec2::new(100.0, 100.0),
            cell_size: Vec2::new(40.0, 40.0),
            affected_by_scale: false,
        }
    }

    pub fn update_layout(&self, child_count: usize) -> Vec<ChildLayout> {
        match self.layout_type {
            LayoutType::None => vec![],
            LayoutType::Horizontal => self.layout_horizontal(child_count),
            LayoutType::Vertical => self.layout_vertical(child_count),
            LayoutType::Grid => self.layout_grid(child_count),
        }
    }

    fn layout_horizontal(&self, count: usize) -> Vec<ChildLayout> {
        let mut result = Vec::new();
        let mut x = self.padding.left;
        let y = self.padding.top;
        for _ in 0..count {
            result.push(ChildLayout {
                position: Vec2::new(x, y),
                size: self.cell_size,
            });
            x += self.cell_size.x + self.spacing_x;
        }
        result
    }

    fn layout_vertical(&self, count: usize) -> Vec<ChildLayout> {
        let mut result = Vec::new();
        let x = self.padding.left;
        let mut y = self.padding.top;
        for _ in 0..count {
            result.push(ChildLayout {
                position: Vec2::new(x, y),
                size: self.cell_size,
            });
            y += self.cell_size.y + self.spacing_y;
        }
        result
    }

    fn layout_grid(&self, count: usize) -> Vec<ChildLayout> {
        let mut result = Vec::new();
        let cols = ((self.content_size.x - self.padding.left - self.padding.right)
            / (self.cell_size.x + self.spacing_x)).max(1.0) as usize;
        for i in 0..count {
            let col = i % cols;
            let row = i / cols;
            let x = self.padding.left + col as f32 * (self.cell_size.x + self.spacing_x);
            let y = self.padding.top + row as f32 * (self.cell_size.y + self.spacing_y);
            result.push(ChildLayout {
                position: Vec2::new(x, y),
                size: self.cell_size,
            });
        }
        result
    }

    pub fn get_container_size(&self, child_count: usize) -> Vec2 {
        match self.layout_type {
            LayoutType::Horizontal => Vec2::new(
                self.padding.left + self.padding.right
                    + child_count as f32 * self.cell_size.x
                    + (child_count.saturating_sub(1)) as f32 * self.spacing_x,
                self.padding.top + self.padding.bottom + self.cell_size.y,
            ),
            LayoutType::Vertical => Vec2::new(
                self.padding.left + self.padding.right + self.cell_size.x,
                self.padding.top + self.padding.bottom
                    + child_count as f32 * self.cell_size.y
                    + (child_count.saturating_sub(1)) as f32 * self.spacing_y,
            ),
            _ => self.content_size,
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_new() {
        let l = Layout::new();
        assert_eq!(l.layout_type, LayoutType::None);
    }

    #[test]
    fn test_layout_horizontal() {
        let mut l = Layout::new();
        l.layout_type = LayoutType::Horizontal;
        l.cell_size = Vec2::new(50.0, 50.0);
        l.spacing_x = 10.0;
        let result = l.update_layout(3);
        assert_eq!(result.len(), 3);
        assert!((result[0].position.x - 0.0).abs() < 1e-4);
        assert!((result[1].position.x - 60.0).abs() < 1e-4);
        assert!((result[2].position.x - 120.0).abs() < 1e-4);
    }

    #[test]
    fn test_layout_vertical() {
        let mut l = Layout::new();
        l.layout_type = LayoutType::Vertical;
        l.cell_size = Vec2::new(50.0, 30.0);
        l.spacing_y = 5.0;
        let result = l.update_layout(3);
        assert_eq!(result.len(), 3);
        assert!((result[1].position.y - 35.0).abs() < 1e-4);
        assert!((result[2].position.y - 70.0).abs() < 1e-4);
    }

    #[test]
    fn test_layout_grid() {
        let mut l = Layout::new();
        l.layout_type = LayoutType::Grid;
        l.content_size = Vec2::new(120.0, 200.0);
        l.cell_size = Vec2::new(50.0, 50.0);
        l.spacing_x = 10.0;
        l.spacing_y = 10.0;
        let result = l.update_layout(4);
        assert_eq!(result.len(), 4);
        assert!(result[0].position.x < result[1].position.x);
        assert!((result[2].position.y - 60.0).abs() < 1e-4);
    }

    #[test]
    fn test_layout_padding() {
        let mut l = Layout::new();
        l.layout_type = LayoutType::Horizontal;
        l.padding = LayoutPadding { left: 10.0, top: 5.0, ..Default::default() };
        l.cell_size = Vec2::new(30.0, 30.0);
        let result = l.update_layout(1);
        assert!((result[0].position.x - 10.0).abs() < 1e-4);
        assert!((result[0].position.y - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_layout_none_returns_empty() {
        let l = Layout::new();
        let result = l.update_layout(5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_container_size_horizontal() {
        let mut l = Layout::new();
        l.layout_type = LayoutType::Horizontal;
        l.cell_size = Vec2::new(50.0, 50.0);
        l.spacing_x = 10.0;
        let size = l.get_container_size(3);
        assert!((size.x - 170.0).abs() < 1e-4);
    }
}
