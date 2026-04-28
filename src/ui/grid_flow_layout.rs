use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridFlowAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLayoutItem {
    pub position: Vec2,
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct GridLayout {
    pub cell_size: Vec2,
    pub spacing: Vec2,
    pub start_corner: Vec2,
    pub constraint: u32,
    pub constraint_count: u32,
    pub flow_axis: GridFlowAxis,
    pub start_axis: GridFlowAxis,
    pub pad_top: f32,
    pub pad_bottom: f32,
    pub pad_left: f32,
    pub pad_right: f32,
    pub align_horizontal: ContentAlignment,
    pub align_vertical: ContentAlignment,
}

impl GridLayout {
    pub fn new() -> Self {
        GridLayout {
            cell_size: Vec2::new(100.0, 100.0),
            spacing: Vec2::new(0.0, 0.0),
            start_corner: Vec2::ZERO,
            constraint: 0,
            constraint_count: 0,
            flow_axis: GridFlowAxis::Horizontal,
            start_axis: GridFlowAxis::Horizontal,
            pad_top: 0.0,
            pad_bottom: 0.0,
            pad_left: 0.0,
            pad_right: 0.0,
            align_horizontal: ContentAlignment::Start,
            align_vertical: ContentAlignment::Start,
        }
    }

    pub fn calculate(&self, count: usize, container: Vec2) -> Vec<GridLayoutItem> {
        if count == 0 {
            return Vec::new();
        }
        let cols = if self.constraint_count > 0 {
            self.constraint_count as usize
        } else {
            let usable_w = container.x - self.pad_left - self.pad_right;
            let col = ((usable_w + self.spacing.x) / (self.cell_size.x + self.spacing.x)).floor() as usize;
            col.max(1)
        };
        let _rows = count.div_ceil(cols);
        let mut items = Vec::with_capacity(count);
        for i in 0..count {
            let (col, row) = match self.start_axis {
                GridFlowAxis::Horizontal => (i % cols, i / cols),
                GridFlowAxis::Vertical => {
                    let rows_count = count.div_ceil(cols);
                    (i / rows_count, i % rows_count)
                }
            };
            let x = self.pad_left + col as f32 * (self.cell_size.x + self.spacing.x);
            let y = self.pad_top + row as f32 * (self.cell_size.y + self.spacing.y);
            items.push(GridLayoutItem {
                position: Vec2::new(x, y),
                size: self.cell_size,
            });
        }
        items
    }

    pub fn get_content_size(&self, count: usize, container: Vec2) -> Vec2 {
        let items = self.calculate(count, container);
        if items.is_empty() {
            return Vec2::ZERO;
        }
        let max_x = items.iter().map(|i| i.position.x + i.size.x).fold(0.0f32, f32::max);
        let max_y = items.iter().map(|i| i.position.y + i.size.y).fold(0.0f32, f32::max);
        Vec2::new(max_x + self.pad_right, max_y + self.pad_bottom)
    }
}

impl Default for GridLayout {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowWrap {
    NoWrap,
    Wrap,
}

#[derive(Debug, Clone)]
pub struct FlowLayout {
    pub axis: FlowAxis,
    pub wrap: FlowWrap,
    pub spacing: Vec2,
    pub pad_top: f32,
    pub pad_bottom: f32,
    pub pad_left: f32,
    pub pad_right: f32,
    pub align: ContentAlignment,
}

impl FlowLayout {
    pub fn new() -> Self {
        FlowLayout {
            axis: FlowAxis::Horizontal,
            wrap: FlowWrap::Wrap,
            spacing: Vec2::new(4.0, 4.0),
            pad_top: 0.0,
            pad_bottom: 0.0,
            pad_left: 0.0,
            pad_right: 0.0,
            align: ContentAlignment::Start,
        }
    }

    pub fn calculate(&self, item_sizes: &[Vec2], container: Vec2) -> Vec<Vec2> {
        let mut positions = Vec::with_capacity(item_sizes.len());
        let available_w = container.x - self.pad_left - self.pad_right;
        let available_h = container.y - self.pad_top - self.pad_bottom;

        let mut cursor_x = self.pad_left;
        let mut cursor_y = self.pad_top;
        let mut row_max_h = 0.0f32;
        let mut col_max_w = 0.0f32;

        for size in item_sizes {
            match self.axis {
                FlowAxis::Horizontal => {
                    if self.wrap == FlowWrap::Wrap && cursor_x + size.x > self.pad_left + available_w && cursor_x > self.pad_left {
                        cursor_x = self.pad_left;
                        cursor_y += row_max_h + self.spacing.y;
                        row_max_h = 0.0;
                    }
                    positions.push(Vec2::new(cursor_x, cursor_y));
                    cursor_x += size.x + self.spacing.x;
                    row_max_h = row_max_h.max(size.y);
                }
                FlowAxis::Vertical => {
                    if self.wrap == FlowWrap::Wrap && cursor_y + size.y > self.pad_top + available_h && cursor_y > self.pad_top {
                        cursor_y = self.pad_top;
                        cursor_x += col_max_w + self.spacing.x;
                        col_max_w = 0.0;
                    }
                    positions.push(Vec2::new(cursor_x, cursor_y));
                    cursor_y += size.y + self.spacing.y;
                    col_max_w = col_max_w.max(size.x);
                }
            }
        }
        positions
    }
}

impl Default for FlowLayout {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32) -> Vec2 { Vec2::new(x, y) }

    #[test]
    fn test_grid_layout_empty() {
        let gl = GridLayout::new();
        assert!(gl.calculate(0, v(400.0, 400.0)).is_empty());
    }

    #[test]
    fn test_grid_layout_single() {
        let gl = GridLayout::new();
        let items = gl.calculate(1, v(400.0, 400.0));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].position, v(0.0, 0.0));
        assert_eq!(items[0].size, v(100.0, 100.0));
    }

    #[test]
    fn test_grid_layout_row() {
        let mut gl = GridLayout::new();
        gl.constraint_count = 3;
        let items = gl.calculate(3, v(400.0, 400.0));
        assert_eq!(items.len(), 3);
        assert!((items[1].position.x - 100.0).abs() < 1e-5);
        assert!((items[2].position.x - 200.0).abs() < 1e-5);
    }

    #[test]
    fn test_grid_layout_wraps_to_next_row() {
        let mut gl = GridLayout::new();
        gl.constraint_count = 3;
        let items = gl.calculate(4, v(400.0, 400.0));
        assert_eq!(items.len(), 4);
        assert!((items[3].position.y - 100.0).abs() < 1e-5);
        assert!((items[3].position.x - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_grid_layout_with_spacing() {
        let mut gl = GridLayout::new();
        gl.constraint_count = 2;
        gl.spacing = v(10.0, 10.0);
        let items = gl.calculate(2, v(400.0, 400.0));
        assert!((items[1].position.x - 110.0).abs() < 1e-5);
    }

    #[test]
    fn test_grid_layout_with_padding() {
        let mut gl = GridLayout::new();
        gl.constraint_count = 2;
        gl.pad_left = 20.0;
        gl.pad_top = 10.0;
        let items = gl.calculate(1, v(400.0, 400.0));
        assert!((items[0].position.x - 20.0).abs() < 1e-5);
        assert!((items[0].position.y - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_grid_layout_content_size() {
        let mut gl = GridLayout::new();
        gl.constraint_count = 2;
        let size = gl.get_content_size(4, v(400.0, 400.0));
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
    }

    #[test]
    fn test_flow_layout_horizontal() {
        let fl = FlowLayout::new();
        let sizes = vec![v(50.0, 30.0), v(60.0, 30.0), v(40.0, 30.0)];
        let positions = fl.calculate(&sizes, v(300.0, 200.0));
        assert_eq!(positions.len(), 3);
        assert!(positions[1].x > positions[0].x);
    }

    #[test]
    fn test_flow_layout_wraps() {
        let mut fl = FlowLayout::new();
        fl.spacing = v(0.0, 0.0);
        let sizes = vec![v(60.0, 30.0); 5];
        let positions = fl.calculate(&sizes, v(100.0, 200.0));
        assert_eq!(positions.len(), 5);
        let second_row: Vec<_> = positions.iter().filter(|p| p.y > 0.0).collect();
        assert!(!second_row.is_empty());
    }

    #[test]
    fn test_flow_layout_no_wrap() {
        let mut fl = FlowLayout::new();
        fl.wrap = FlowWrap::NoWrap;
        let sizes = vec![v(60.0, 30.0); 5];
        let positions = fl.calculate(&sizes, v(100.0, 200.0));
        assert!(positions.iter().all(|p| p.y < 1.0));
    }

    #[test]
    fn test_flow_layout_vertical() {
        let mut fl = FlowLayout::new();
        fl.axis = FlowAxis::Vertical;
        let sizes = vec![v(50.0, 30.0), v(50.0, 40.0)];
        let positions = fl.calculate(&sizes, v(200.0, 300.0));
        assert!(positions[1].y > positions[0].y);
    }

    #[test]
    fn test_flow_layout_empty() {
        let fl = FlowLayout::new();
        let pos = fl.calculate(&[], v(200.0, 200.0));
        assert!(pos.is_empty());
    }
}
