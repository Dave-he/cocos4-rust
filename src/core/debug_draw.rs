use crate::math::{Vec3, Color};

#[derive(Debug, Clone, PartialEq)]
pub enum DebugShape {
    Line {
        start: Vec3,
        end: Vec3,
        color: Color,
        duration: f32,
    },
    Ray {
        origin: Vec3,
        direction: Vec3,
        length: f32,
        color: Color,
        duration: f32,
    },
    Sphere {
        center: Vec3,
        radius: f32,
        color: Color,
        duration: f32,
        segments: u32,
    },
    Box {
        center: Vec3,
        half_extents: Vec3,
        color: Color,
        duration: f32,
    },
    Cross {
        center: Vec3,
        size: f32,
        color: Color,
        duration: f32,
    },
    Text {
        position: Vec3,
        text: String,
        color: Color,
        duration: f32,
    },
}

impl DebugShape {
    pub fn duration(&self) -> f32 {
        match self {
            DebugShape::Line { duration, .. }
            | DebugShape::Ray { duration, .. }
            | DebugShape::Sphere { duration, .. }
            | DebugShape::Box { duration, .. }
            | DebugShape::Cross { duration, .. }
            | DebugShape::Text { duration, .. } => *duration,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            DebugShape::Line { color, .. }
            | DebugShape::Ray { color, .. }
            | DebugShape::Sphere { color, .. }
            | DebugShape::Box { color, .. }
            | DebugShape::Cross { color, .. }
            | DebugShape::Text { color, .. } => *color,
        }
    }
}

pub struct DebugDraw {
    shapes: Vec<DebugShape>,
    enabled: bool,
    max_shapes: usize,
    elapsed: Vec<f32>,
}

impl DebugDraw {
    pub fn new() -> Self {
        DebugDraw {
            shapes: Vec::new(),
            enabled: true,
            max_shapes: 1000,
            elapsed: Vec::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn draw_line(&mut self, start: Vec3, end: Vec3, color: Color, duration: f32) {
        self.push(DebugShape::Line { start, end, color, duration });
    }

    pub fn draw_ray(&mut self, origin: Vec3, direction: Vec3, length: f32, color: Color, duration: f32) {
        self.push(DebugShape::Ray { origin, direction, length, color, duration });
    }

    pub fn draw_sphere(&mut self, center: Vec3, radius: f32, color: Color, duration: f32) {
        self.push(DebugShape::Sphere { center, radius, color, duration, segments: 16 });
    }

    pub fn draw_box(&mut self, center: Vec3, half_extents: Vec3, color: Color, duration: f32) {
        self.push(DebugShape::Box { center, half_extents, color, duration });
    }

    pub fn draw_cross(&mut self, center: Vec3, size: f32, color: Color, duration: f32) {
        self.push(DebugShape::Cross { center, size, color, duration });
    }

    pub fn draw_text(&mut self, position: Vec3, text: &str, color: Color, duration: f32) {
        self.push(DebugShape::Text { position, text: text.to_string(), color, duration });
    }

    pub fn draw_axes(&mut self, origin: Vec3, scale: f32, duration: f32) {
        self.draw_line(origin, Vec3::new(origin.x + scale, origin.y, origin.z), Color::new(255, 0, 0, 255), duration);
        self.draw_line(origin, Vec3::new(origin.x, origin.y + scale, origin.z), Color::new(0, 255, 0, 255), duration);
        self.draw_line(origin, Vec3::new(origin.x, origin.y, origin.z + scale), Color::new(0, 0, 255, 255), duration);
    }

    pub fn draw_aabb(&mut self, center: Vec3, half: Vec3, color: Color, duration: f32) {
        let min = Vec3::new(center.x - half.x, center.y - half.y, center.z - half.z);
        let max = Vec3::new(center.x + half.x, center.y + half.y, center.z + half.z);
        let corners = [
            Vec3::new(min.x, min.y, min.z), Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z), Vec3::new(min.x, max.y, min.z),
            Vec3::new(min.x, min.y, max.z), Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z), Vec3::new(min.x, max.y, max.z),
        ];
        let edges = [(0,1),(1,2),(2,3),(3,0),(4,5),(5,6),(6,7),(7,4),(0,4),(1,5),(2,6),(3,7)];
        for (a, b) in edges {
            self.draw_line(corners[a], corners[b], color, duration);
        }
    }

    fn push(&mut self, shape: DebugShape) {
        if !self.enabled { return; }
        if self.shapes.len() >= self.max_shapes { return; }
        self.elapsed.push(0.0);
        self.shapes.push(shape);
    }

    pub fn update(&mut self, dt: f32) {
        if self.shapes.is_empty() { return; }
        let mut i = 0;
        while i < self.shapes.len() {
            self.elapsed[i] += dt;
            if self.elapsed[i] >= self.shapes[i].duration() {
                self.shapes.remove(i);
                self.elapsed.remove(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn clear(&mut self) {
        self.shapes.clear();
        self.elapsed.clear();
    }

    pub fn get_shapes(&self) -> &[DebugShape] {
        &self.shapes
    }

    pub fn shape_count(&self) -> usize {
        self.shapes.len()
    }

    pub fn set_max_shapes(&mut self, max: usize) {
        self.max_shapes = max;
    }
}

impl Default for DebugDraw {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> Color { Color::new(255, 0, 0, 255) }
    fn white() -> Color { Color::WHITE }

    #[test]
    fn test_debug_draw_new() {
        let d = DebugDraw::new();
        assert!(d.is_enabled());
        assert_eq!(d.shape_count(), 0);
    }

    #[test]
    fn test_draw_line() {
        let mut d = DebugDraw::new();
        d.draw_line(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), red(), 1.0);
        assert_eq!(d.shape_count(), 1);
    }

    #[test]
    fn test_draw_ray() {
        let mut d = DebugDraw::new();
        d.draw_ray(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), 5.0, red(), 0.5);
        assert_eq!(d.shape_count(), 1);
    }

    #[test]
    fn test_draw_sphere() {
        let mut d = DebugDraw::new();
        d.draw_sphere(Vec3::ZERO, 1.0, white(), 1.0);
        assert_eq!(d.shape_count(), 1);
    }

    #[test]
    fn test_draw_box() {
        let mut d = DebugDraw::new();
        d.draw_box(Vec3::ZERO, Vec3::new(0.5, 0.5, 0.5), white(), 1.0);
        assert_eq!(d.shape_count(), 1);
    }

    #[test]
    fn test_draw_axes() {
        let mut d = DebugDraw::new();
        d.draw_axes(Vec3::ZERO, 1.0, 0.5);
        assert_eq!(d.shape_count(), 3);
    }

    #[test]
    fn test_draw_aabb_adds_12_lines() {
        let mut d = DebugDraw::new();
        d.draw_aabb(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0), white(), 1.0);
        assert_eq!(d.shape_count(), 12);
    }

    #[test]
    fn test_draw_text() {
        let mut d = DebugDraw::new();
        d.draw_text(Vec3::ZERO, "Hello", white(), 1.0);
        assert_eq!(d.shape_count(), 1);
        if let DebugShape::Text { text, .. } = &d.get_shapes()[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("wrong shape");
        }
    }

    #[test]
    fn test_update_removes_expired() {
        let mut d = DebugDraw::new();
        d.draw_line(Vec3::ZERO, Vec3::UNIT_X, red(), 0.1);
        d.draw_line(Vec3::ZERO, Vec3::UNIT_Y, red(), 10.0);
        d.update(0.2);
        assert_eq!(d.shape_count(), 1);
    }

    #[test]
    fn test_update_keeps_active() {
        let mut d = DebugDraw::new();
        d.draw_line(Vec3::ZERO, Vec3::UNIT_X, red(), 5.0);
        d.update(1.0);
        assert_eq!(d.shape_count(), 1);
    }

    #[test]
    fn test_disabled_no_draw() {
        let mut d = DebugDraw::new();
        d.set_enabled(false);
        d.draw_line(Vec3::ZERO, Vec3::UNIT_X, red(), 1.0);
        assert_eq!(d.shape_count(), 0);
    }

    #[test]
    fn test_clear() {
        let mut d = DebugDraw::new();
        d.draw_line(Vec3::ZERO, Vec3::UNIT_X, red(), 1.0);
        d.draw_sphere(Vec3::ZERO, 1.0, white(), 1.0);
        d.clear();
        assert_eq!(d.shape_count(), 0);
    }

    #[test]
    fn test_max_shapes_limit() {
        let mut d = DebugDraw::new();
        d.set_max_shapes(3);
        for _ in 0..10 {
            d.draw_line(Vec3::ZERO, Vec3::UNIT_X, red(), 1.0);
        }
        assert_eq!(d.shape_count(), 3);
    }

    #[test]
    fn test_shape_color() {
        let mut d = DebugDraw::new();
        d.draw_line(Vec3::ZERO, Vec3::UNIT_X, red(), 1.0);
        assert_eq!(d.get_shapes()[0].color(), red());
    }

    #[test]
    fn test_cross() {
        let mut d = DebugDraw::new();
        d.draw_cross(Vec3::ZERO, 1.0, white(), 1.0);
        assert_eq!(d.shape_count(), 1);
    }
}
