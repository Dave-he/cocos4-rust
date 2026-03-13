use super::os_interface::OSInterface;

/// Canvas gradient interface
pub trait ICanvasGradient: OSInterface {
    /// Add color stop to gradient
    fn add_color_stop(&mut self, offset: f32, color: &str);
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Text baseline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBaseline {
    Top,
    Middle,
    Bottom,
    Alphabetic,
}

/// Canvas rendering context 2D delegate
pub trait CanvasRenderingContext2DDelegate: Send + Sync {
    /// Canvas buffer size
    type Size: Into<[f32; 2]>;

    /// Recreate buffer with new dimensions
    fn recreate_buffer(&mut self, width: f32, height: f32);

    /// Begin a new path
    fn begin_path(&mut self);

    /// Close the current path
    fn close_path(&mut self);

    /// Move to position
    fn move_to(&mut self, x: f32, y: f32);

    /// Line to position
    fn line_to(&mut self, x: f32, y: f32);

    /// Stroke the current path
    fn stroke(&mut self);

    /// Save context state
    fn save_context(&mut self);

    /// Restore context state
    fn restore_context(&mut self);

    /// Clear rectangle
    fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32);

    /// Fill the current path
    fn fill(&mut self);

    /// Draw rectangle
    fn rect(&mut self, x: f32, y: f32, width: f32, height: f32);

    /// Set line cap style
    fn set_line_cap(&mut self, line_cap: &str);

    /// Set line join style
    fn set_line_join(&mut self, line_join: &str);

    /// Fill image data
    fn fill_image_data(
        &mut self,
        image_data: &[u8],
        image_width: f32,
        image_height: f32,
        offset_x: f32,
        offset_y: f32,
    );

    /// Fill rectangle
    fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32);

    /// Fill text at position
    fn fill_text(&mut self, text: &str, x: f32, y: f32, max_width: f32);

    /// Stroke text at position
    fn stroke_text(&mut self, text: &str, x: f32, y: f32, max_width: f32);

    /// Measure text width
    fn measure_text(&self, text: &str) -> Self::Size;

    /// Update font
    fn update_font(
        &mut self,
        font_name: &str,
        font_size: f32,
        bold: bool,
        italic: bool,
        oblique: bool,
        small_caps: bool,
    );

    /// Set text alignment
    fn set_text_align(&mut self, align: TextAlign);

    /// Set text baseline
    fn set_text_baseline(&mut self, baseline: TextBaseline);

    /// Set fill style as RGBA
    fn set_fill_style_rgba(&mut self, r: u8, g: u8, b: u8, a: u8);

    /// Set stroke style as RGBA
    fn set_stroke_style_rgba(&mut self, r: u8, g: u8, b: u8, a: u8);

    /// Set line width
    fn set_line_width(&mut self, line_width: f32);

    /// Set shadow blur
    fn set_shadow_blur(&mut self, blur: f32);

    /// Set shadow color as RGBA
    fn set_shadow_color(&mut self, r: u8, g: u8, b: u8, a: u8);

    /// Set shadow offset X
    fn set_shadow_offset_x(&mut self, offset_x: f32);

    /// Set shadow offset Y
    fn set_shadow_offset_y(&mut self, offset_y: f32);

    /// Get data reference
    fn get_data_ref(&self) -> &[u8];

    /// Update data
    fn update_data(&mut self);
}

/// Canvas rendering context 2D interface
pub trait ICanvasRenderingContext2D: OSInterface {
    /// Draw rectangle
    fn rect(&mut self, x: f32, y: f32, width: f32, height: f32);

    /// Clear rectangle
    fn clear_rect(&mut self, x: f32, y: f32, width: f32, height: f32);

    /// Fill rectangle
    fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32);

    /// Fill text at position
    fn fill_text(&mut self, text: &str, x: f32, y: f32, max_width: f32);

    /// Stroke text at position
    fn stroke_text(&mut self, text: &str, x: f32, y: f32, max_width: f32);

    /// Measure text
    fn measure_text(&self, text: &str) -> [f32; 2];

    /// Create linear gradient
    fn create_linear_gradient(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
    ) -> Box<dyn ICanvasGradient>;

    /// Save context state
    fn save(&mut self);

    /// Begin new path
    fn begin_path(&mut self);

    /// Close current path
    fn close_path(&mut self);

    /// Move to position
    fn move_to(&mut self, x: f32, y: f32);

    /// Line to position
    fn line_to(&mut self, x: f32, y: f32);

    /// Fill current path
    fn fill(&mut self);

    /// Stroke current path
    fn stroke(&mut self);

    /// Restore context state
    fn restore(&mut self);

    /// Set canvas buffer updated callback
    fn set_canvas_buffer_updated_callback(&mut self, callback: Box<dyn Fn(&[u8]) + Send + Sync>);

    /// Set width
    fn set_width(&mut self, width: f32);

    /// Set height
    fn set_height(&mut self, height: f32);

    /// Set line width
    fn set_line_width(&mut self, line_width: f32);

    /// Set line join
    fn set_line_join(&mut self, line_join: &str);

    /// Set line cap
    fn set_line_cap(&mut self, line_cap: &str);

    /// Set font
    fn set_font(&mut self, font: &str);

    /// Set text alignment
    fn set_text_align(&mut self, text_align: &str);

    /// Set text baseline
    fn set_text_baseline(&mut self, text_baseline: &str);

    /// Set fill style
    fn set_fill_style(&mut self, fill_style: &str);

    /// Set stroke style
    fn set_stroke_style(&mut self, stroke_style: &str);

    /// Set global composite operation
    fn set_global_composite_operation(&mut self, operation: &str);

    /// Set shadow blur
    fn set_shadow_blur(&mut self, blur: f32);

    /// Set shadow color
    fn set_shadow_color(&mut self, shadow_color: &str);

    /// Set shadow offset X
    fn set_shadow_offset_x(&mut self, offset_x: f32);

    /// Set shadow offset Y
    fn set_shadow_offset_y(&mut self, offset_y: f32);

    /// Fill image data
    fn fill_image_data(
        &mut self,
        image_data: &[u8],
        image_width: f32,
        image_height: f32,
        offset_x: f32,
        offset_y: f32,
    );

    /// Translate transform
    fn translate(&mut self, x: f32, y: f32);

    /// Scale transform
    fn scale(&mut self, x: f32, y: f32);

    /// Rotate transform
    fn rotate(&mut self, angle: f32);

    /// Transform matrix
    fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32);

    /// Set transform matrix
    fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32);

    /// Fetch data
    fn fetch_data(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_align_values() {
        assert_ne!(TextAlign::Left, TextAlign::Center);
        assert_ne!(TextAlign::Center, TextAlign::Right);
    }

    #[test]
    fn test_text_baseline_values() {
        assert_ne!(TextBaseline::Top, TextBaseline::Bottom);
        assert_ne!(TextBaseline::Middle, TextBaseline::Alphabetic);
    }
}
