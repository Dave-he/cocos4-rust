/****************************************************************************
Rust port of Cocos Creator Mask Component
Original TS version Copyright (c) 2019-2023 Cocos Core Team
****************************************************************************/

use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaskType {
    #[default]
    GraphicsRect = 0,
    GraphicsEllipse = 1,
    GraphicsStencil = 2,
    SpriteStencil = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StencilStage {
    #[default]
    Disabled = 0,
    Clear = 1,
    EnabledMask = 2,
    VisibleContent = 3,
    ClearInverted = 4,
    EnabledInvertedMask = 5,
}

#[derive(Debug, Clone)]
pub struct Mask {
    pub mask_type: MaskType,
    pub radius: f32,
    pub alpha_threshold: f32,
    pub inverted: bool,
    pub segments: u32,
    pub stencil_stage: StencilStage,
    pub enabled: bool,
    pub node_width: f32,
    pub node_height: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,
    sprite_frame_name: Option<String>,
}

impl Default for Mask {
    fn default() -> Self {
        Self::new()
    }
}

impl Mask {
    pub fn new() -> Self {
        Self {
            mask_type: MaskType::GraphicsRect,
            radius: 0.0,
            alpha_threshold: 0.1,
            inverted: false,
            segments: 64,
            stencil_stage: StencilStage::Disabled,
            enabled: true,
            node_width: 100.0,
            node_height: 100.0,
            anchor_x: 0.5,
            anchor_y: 0.5,
            sprite_frame_name: None,
        }
    }

    pub fn set_type(&mut self, mask_type: MaskType) {
        if self.mask_type == mask_type {
            return;
        }
        self.mask_type = mask_type;
        self.update_graphics();
    }

    pub fn set_radius(&mut self, radius: f32) {
        let clamped = radius.max(0.0);
        if (self.radius - clamped).abs() < f32::EPSILON {
            return;
        }
        self.radius = clamped;
        self.update_graphics();
    }

    pub fn set_alpha_threshold(&mut self, threshold: f32) {
        self.alpha_threshold = threshold.clamp(0.0, 1.0);
    }

    pub fn set_inverted(&mut self, inverted: bool) {
        if self.inverted == inverted {
            return;
        }
        self.inverted = inverted;
        self.update_graphics();
    }

    pub fn set_segments(&mut self, segments: u32) {
        if segments == 0 {
            return;
        }
        self.segments = segments;
        self.update_graphics();
    }

    pub fn set_node_size(&mut self, width: f32, height: f32) {
        self.node_width = width;
        self.node_height = height;
        self.update_graphics();
    }

    pub fn set_anchor(&mut self, x: f32, y: f32) {
        self.anchor_x = x;
        self.anchor_y = y;
        self.update_graphics();
    }

    pub fn hit_test(&self, test_pt: Vec2) -> bool {
        let w = self.node_width;
        let h = self.node_height;
        let ap_x = self.anchor_x;
        let ap_y = self.anchor_y;

        let test_pt = Vec2::new(
            test_pt.x + ap_x * w,
            test_pt.y + ap_y * h,
        );

        let mut result = false;

        match self.mask_type {
            MaskType::GraphicsRect => {
                result = test_pt.x >= 0.0 && test_pt.y >= 0.0
                    && test_pt.x <= w && test_pt.y <= h;
                let radius = self.radius.min(w / 2.0).min(h / 2.0);
                if result && radius > 0.0 {
                    let mut dx = 0.0;
                    let mut dy = 0.0;
                    if test_pt.x < radius {
                        dx = test_pt.x - radius;
                    } else if test_pt.x > w - radius {
                        dx = test_pt.x - (w - radius);
                    }
                    if test_pt.y < radius {
                        dy = test_pt.y - radius;
                    } else if test_pt.y > h - radius {
                        dy = test_pt.y - (h - radius);
                    }
                    result = dx * dx + dy * dy <= radius * radius;
                }
            }
            MaskType::GraphicsStencil | MaskType::SpriteStencil => {
                result = test_pt.x >= 0.0 && test_pt.y >= 0.0
                    && test_pt.x <= w && test_pt.y <= h;
            }
            MaskType::GraphicsEllipse => {
                let rx = w / 2.0;
                let ry = h / 2.0;
                let cx = w / 2.0;
                let cy = h / 2.0;
                if rx > 0.0 && ry > 0.0 {
                    let nx = (test_pt.x - cx) / rx;
                    let ny = (test_pt.y - cy) / ry;
                    result = nx * nx + ny * ny <= 1.0;
                }
            }
        }

        if self.inverted {
            result = !result;
        }

        result
    }

    pub fn update_graphics(&mut self) {
        self.stencil_stage = if self.inverted {
            StencilStage::EnabledInvertedMask
        } else {
            StencilStage::EnabledMask
        };
    }

    pub fn generate_rect_vertices(&self) -> Vec<f32> {
        let w = self.node_width;
        let h = self.node_height;
        let x = -w * self.anchor_x;
        let y = -h * self.anchor_y;
        let radius = self.radius.min(w / 2.0).min(h / 2.0);

        if radius > 0.0 && self.mask_type == MaskType::GraphicsRect {
            self.generate_round_rect_vertices(x, y, w, h, radius)
        } else {
            vec![
                x, y, 0.0,
                x + w, y, 0.0,
                x + w, y + h, 0.0,
                x, y + h, 0.0,
            ]
        }
    }

    fn generate_round_rect_vertices(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) -> Vec<f32> {
        let segments_per_corner = (self.segments / 4).max(4);
        let mut vertices = Vec::new();

        let corners: [(f32, f32, f32); 4] = [
            (x + w - radius, y + h - radius, 0.0),
            (x + radius, y + h - radius, std::f32::consts::FRAC_PI_2),
            (x + radius, y + radius, std::f32::consts::PI),
            (x + w - radius, y + radius, 3.0 * std::f32::consts::FRAC_PI_2),
        ];

        for (cx, cy, start_angle) in &corners {
            for i in 0..=segments_per_corner {
                let angle = start_angle + std::f32::consts::FRAC_PI_2 * (i as f32 / segments_per_corner as f32);
                let px = cx + radius * angle.cos();
                let py = cy + radius * angle.sin();
                vertices.push(px);
                vertices.push(py);
                vertices.push(0.0);
            }
        }

        vertices
    }

    pub fn generate_ellipse_vertices(&self) -> Vec<f32> {
        let w = self.node_width;
        let h = self.node_height;
        let cx = w / 2.0 - w * self.anchor_x;
        let cy = h / 2.0 - h * self.anchor_y;
        let rx = w / 2.0;
        let ry = h / 2.0;
        let segments = self.segments;
        let mut vertices = Vec::new();

        for i in 0..segments {
            let angle = 2.0 * std::f32::consts::PI * (i as f32 / segments as f32);
            let px = cx + rx * angle.cos();
            let py = cy + ry * angle.sin();
            vertices.push(px);
            vertices.push(py);
            vertices.push(0.0);
        }

        vertices
    }

    pub fn get_sprite_frame(&self) -> Option<&str> {
        self.sprite_frame_name.as_deref()
    }

    pub fn set_sprite_frame(&mut self, name: Option<String>) {
        self.sprite_frame_name = name;
        self.update_graphics();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_new() {
        let mask = Mask::new();
        assert_eq!(mask.mask_type, MaskType::GraphicsRect);
        assert_eq!(mask.radius, 0.0);
        assert!(!mask.inverted);
        assert!(mask.enabled);
        assert_eq!(mask.segments, 64);
    }

    #[test]
    fn test_mask_set_type() {
        let mut mask = Mask::new();
        mask.set_type(MaskType::GraphicsEllipse);
        assert_eq!(mask.mask_type, MaskType::GraphicsEllipse);
        assert_eq!(mask.stencil_stage, StencilStage::EnabledMask);
    }

    #[test]
    fn test_mask_set_radius() {
        let mut mask = Mask::new();
        mask.set_radius(10.0);
        assert_eq!(mask.radius, 10.0);
        mask.set_radius(-5.0);
        assert_eq!(mask.radius, 0.0);
    }

    #[test]
    fn test_mask_set_radius_no_update_on_same_value() {
        let mut mask = Mask::new();
        mask.set_radius(5.0);
        let stage_before = mask.stencil_stage;
        mask.set_radius(5.0);
        assert_eq!(mask.stencil_stage, stage_before);
    }

    #[test]
    fn test_mask_hit_test_rect_inside() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        let result = mask.hit_test(Vec2::new(10.0, 10.0));
        assert!(result);
    }

    #[test]
    fn test_mask_hit_test_rect_outside() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        let result = mask.hit_test(Vec2::new(200.0, 200.0));
        assert!(!result);
    }

    #[test]
    fn test_mask_hit_test_rect_rounded_corner_inside() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        mask.set_radius(20.0);
        let result = mask.hit_test(Vec2::new(5.0, 5.0));
        assert!(result);
    }

    #[test]
    fn test_mask_hit_test_rect_rounded_corner_outside() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        mask.set_radius(30.0);
        let result = mask.hit_test(Vec2::new(-44.0, -44.0));
        assert!(!result);
    }

    #[test]
    fn test_mask_hit_test_rect_rounded_corner_boundary() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        mask.set_radius(20.0);
        let result = mask.hit_test(Vec2::new(35.0, 35.0));
        assert!(result);
    }

    #[test]
    fn test_mask_hit_test_ellipse() {
        let mut mask = Mask::new();
        mask.set_type(MaskType::GraphicsEllipse);
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        assert!(mask.hit_test(Vec2::new(0.0, 0.0)));
        assert!(!mask.hit_test(Vec2::new(60.0, 60.0)));
    }

    #[test]
    fn test_mask_hit_test_inverted() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        mask.set_inverted(true);
        let inside = mask.hit_test(Vec2::new(10.0, 10.0));
        assert!(!inside);
        let outside = mask.hit_test(Vec2::new(200.0, 200.0));
        assert!(outside);
    }

    #[test]
    fn test_mask_hit_test_stencil_type() {
        let mut mask = Mask::new();
        mask.set_type(MaskType::GraphicsStencil);
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        assert!(mask.hit_test(Vec2::new(10.0, 10.0)));
        assert!(!mask.hit_test(Vec2::new(200.0, 200.0)));
    }

    #[test]
    fn test_mask_update_graphics() {
        let mut mask = Mask::new();
        mask.set_type(MaskType::GraphicsEllipse);
        assert_eq!(mask.stencil_stage, StencilStage::EnabledMask);
        mask.set_inverted(true);
        assert_eq!(mask.stencil_stage, StencilStage::EnabledInvertedMask);
    }

    #[test]
    fn test_mask_generate_rect_vertices() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        let verts = mask.generate_rect_vertices();
        assert_eq!(verts.len(), 12);
    }

    #[test]
    fn test_mask_generate_round_rect_vertices() {
        let mut mask = Mask::new();
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        mask.set_radius(20.0);
        let verts = mask.generate_rect_vertices();
        assert!(verts.len() > 12);
        assert_eq!(verts.len() % 3, 0);
    }

    #[test]
    fn test_mask_generate_ellipse_vertices() {
        let mut mask = Mask::new();
        mask.set_type(MaskType::GraphicsEllipse);
        mask.set_node_size(100.0, 100.0);
        mask.set_anchor(0.5, 0.5);
        mask.set_segments(32);
        let verts = mask.generate_ellipse_vertices();
        assert_eq!(verts.len(), 32 * 3);
    }

    #[test]
    fn test_mask_set_alpha_threshold() {
        let mut mask = Mask::new();
        mask.set_alpha_threshold(0.5);
        assert_eq!(mask.alpha_threshold, 0.5);
        mask.set_alpha_threshold(2.0);
        assert_eq!(mask.alpha_threshold, 1.0);
        mask.set_alpha_threshold(-1.0);
        assert_eq!(mask.alpha_threshold, 0.0);
    }

    #[test]
    fn test_mask_sprite_frame() {
        let mut mask = Mask::new();
        assert!(mask.get_sprite_frame().is_none());
        mask.set_sprite_frame(Some("frame1".to_string()));
        assert_eq!(mask.get_sprite_frame(), Some("frame1"));
        mask.set_sprite_frame(None);
        assert!(mask.get_sprite_frame().is_none());
    }

    #[test]
    fn test_mask_set_segments() {
        let mut mask = Mask::new();
        mask.set_segments(128);
        assert_eq!(mask.segments, 128);
        mask.set_segments(0);
        assert_eq!(mask.segments, 128);
    }
}
