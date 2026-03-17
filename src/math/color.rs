use super::Vec4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_array(src: &[u8]) -> Self {
        if src.len() >= 4 {
            Color {
                r: src[0],
                g: src[1],
                b: src[2],
                a: src[3],
            }
        } else {
            Color::BLACK
        }
    }

    pub fn from_u32(val: u32) -> Self {
        Color {
            r: ((val >> 24) & 0xFF) as u8,
            g: ((val >> 16) & 0xFF) as u8,
            b: ((val >> 8) & 0xFF) as u8,
            a: (val & 0xFF) as u8,
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::new(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::new(r, g, b, a))
            }
            _ => None,
        }
    }

    pub fn set(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.r = r;
        self.g = g;
        self.b = b;
        self.a = a;
    }

    pub fn set_from_array(&mut self, array: &[u8]) {
        if array.len() >= 4 {
            self.r = array[0];
            self.g = array[1];
            self.b = array[2];
            self.a = array[3];
        }
    }

    pub fn set_u32(&mut self, val: u32) {
        self.r = ((val >> 24) & 0xFF) as u8;
        self.g = ((val >> 16) & 0xFF) as u8;
        self.b = ((val >> 8) & 0xFF) as u8;
        self.a = (val & 0xFF) as u8;
    }

    pub fn set_from_color(&mut self, c: &Color) {
        *self = *c;
    }

    pub fn to_u32(&self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }

    pub fn to_rgb_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn lerp(from: &Color, to: &Color, ratio: f32) -> Color {
        let r = (from.r as f32 + (to.r as f32 - from.r as f32) * ratio) as u8;
        let g = (from.g as f32 + (to.g as f32 - from.g as f32) * ratio) as u8;
        let b = (from.b as f32 + (to.b as f32 - from.b as f32) * ratio) as u8;
        let a = (from.a as f32 + (to.a as f32 - from.a as f32) * ratio) as u8;
        Color::new(r, g, b, a)
    }

    pub fn equals(&self, other: &Color) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b && self.a == other.a
    }

    pub fn multiply(&mut self, other: &Color) {
        self.r = ((self.r as u32 * other.r as u32) / 255) as u8;
        self.g = ((self.g as u32 * other.g as u32) / 255) as u8;
        self.b = ((self.b as u32 * other.b as u32) / 255) as u8;
        self.a = ((self.a as u32 * other.a as u32) / 255) as u8;
    }

    pub fn add(&mut self, other: &Color) {
        self.r = self.r.saturating_add(other.r);
        self.g = self.g.saturating_add(other.g);
        self.b = self.b.saturating_add(other.b);
        self.a = self.a.saturating_add(other.a);
    }

    pub fn subtract(&mut self, other: &Color) {
        self.r = self.r.saturating_sub(other.r);
        self.g = self.g.saturating_sub(other.g);
        self.b = self.b.saturating_sub(other.b);
        self.a = self.a.saturating_sub(other.a);
    }

    pub fn get_grayscale(&self) -> u8 {
        ((self.r as u32 + self.g as u32 + self.b as u32) / 3) as u8
    }

    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < 1e-6 {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let h = if (max - r).abs() < 1e-6 {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if (max - g).abs() < 1e-6 {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } / 6.0;
        (h * 360.0, s, l)
    }

    pub fn from_hsl(h: f32, s: f32, l: f32, a: u8) -> Self {
        if s.abs() < 1e-6 {
            let v = (l * 255.0) as u8;
            return Color::new(v, v, v, a);
        }
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        let hue = h / 360.0;
        let hue_to_rgb = |mut t: f32| -> f32 {
            if t < 0.0 { t += 1.0; }
            if t > 1.0 { t -= 1.0; }
            if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
            if t < 1.0 / 2.0 { return q; }
            if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
            p
        };
        Color::new(
            (hue_to_rgb(hue + 1.0 / 3.0) * 255.0) as u8,
            (hue_to_rgb(hue) * 255.0) as u8,
            (hue_to_rgb(hue - 1.0 / 3.0) * 255.0) as u8,
            a,
        )
    }

    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let v = max;
        let s = if max < 1e-6 { 0.0 } else { d / max };
        if d < 1e-6 {
            return (0.0, s, v);
        }
        let h = if (max - r).abs() < 1e-6 {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if (max - g).abs() < 1e-6 {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        } / 6.0;
        (h * 360.0, s, v)
    }

    pub fn from_hsv(h: f32, s: f32, v: f32, a: u8) -> Self {
        if s.abs() < 1e-6 {
            let val = (v * 255.0) as u8;
            return Color::new(val, val, val, a);
        }
        let h = (h / 60.0).rem_euclid(6.0);
        let i = h as u32;
        let f = h - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);
        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        Color::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, a)
    }

    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const GRAY: Color = Color {
        r: 127,
        g: 127,
        b: 127,
        a: 255,
    };
    pub const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    pub const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Color = Color {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
        a: 255,
    };
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Color(r:{}, g:{}, b:{}, a:{})", self.r, self.g, self.b, self.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let c = Color::new(255, 128, 64, 32);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 32);
    }

    #[test]
    fn test_color_from_u32() {
        let c = Color::from_u32(0xFF804020);
        assert_eq!(c.r, 0xFF);
        assert_eq!(c.g, 0x80);
        assert_eq!(c.b, 0x40);
        assert_eq!(c.a, 0x20);
    }

    #[test]
    fn test_color_to_u32() {
        let c = Color::new(0xFF, 0x80, 0x40, 0x20);
        assert_eq!(c.to_u32(), 0xFF804020);
    }

    #[test]
    fn test_color_from_hex() {
        let c = Color::from_hex("#FF8040").unwrap();
        assert_eq!(c.r, 0xFF);
        assert_eq!(c.g, 0x80);
        assert_eq!(c.b, 0x40);
        assert_eq!(c.a, 255);

        let c = Color::from_hex("#FF804020").unwrap();
        assert_eq!(c.r, 0xFF);
        assert_eq!(c.g, 0x80);
        assert_eq!(c.b, 0x40);
        assert_eq!(c.a, 0x20);
    }

    #[test]
    fn test_color_to_hex() {
        let c = Color::new(0xFF, 0x80, 0x40, 0x20);
        assert_eq!(c.to_hex(), "#FF804020");
        assert_eq!(c.to_rgb_hex(), "#FF8040");
    }

    #[test]
    fn test_color_lerp() {
        let from = Color::new(0, 0, 0, 0);
        let to = Color::new(255, 255, 255, 255);
        let result = Color::lerp(&from, &to, 0.5);
        assert!(result.r >= 127 && result.r <= 128);
        assert!(result.g >= 127 && result.g <= 128);
        assert!(result.b >= 127 && result.b <= 128);
        assert!(result.a >= 127 && result.a <= 128);
    }

    #[test]
    fn test_color_equals() {
        let c1 = Color::new(100, 100, 100, 100);
        let c2 = Color::new(100, 100, 100, 100);
        let c3 = Color::new(200, 200, 200, 200);
        assert!(c1.equals(&c2));
        assert!(!c1.equals(&c3));
    }

    #[test]
    fn test_color_multiply() {
        let mut c = Color::new(200, 200, 200, 200);
        let other = Color::new(128, 128, 128, 128);
        c.multiply(&other);
        assert!(c.r >= 100 && c.r <= 101);
        assert!(c.g >= 100 && c.g <= 101);
        assert!(c.b >= 100 && c.b <= 101);
        assert!(c.a >= 100 && c.a <= 101);
    }

    #[test]
    fn test_color_grayscale() {
        let c = Color::new(100, 100, 100, 255);
        assert_eq!(c.get_grayscale(), 100);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE, Color::new(255, 255, 255, 255));
        assert_eq!(Color::BLACK, Color::new(0, 0, 0, 255));
        assert_eq!(Color::RED, Color::new(255, 0, 0, 255));
        assert_eq!(Color::GREEN, Color::new(0, 255, 0, 255));
        assert_eq!(Color::BLUE, Color::new(0, 0, 255, 255));
    }

    #[test]
    fn test_color_to_hsl() {
        let c = Color::new(255, 0, 0, 255);
        let (h, s, l) = c.to_hsl();
        assert!((h - 0.0).abs() < 1.0 || (h - 360.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_from_hsl_roundtrip() {
        let original = Color::new(100, 150, 200, 255);
        let (h, s, l) = original.to_hsl();
        let restored = Color::from_hsl(h, s, l, 255);
        assert!((original.r as i32 - restored.r as i32).abs() <= 2);
        assert!((original.g as i32 - restored.g as i32).abs() <= 2);
        assert!((original.b as i32 - restored.b as i32).abs() <= 2);
    }

    #[test]
    fn test_color_to_hsv() {
        let c = Color::new(255, 0, 0, 255);
        let (h, s, v) = c.to_hsv();
        assert!((h - 0.0).abs() < 1.0 || (h - 360.0).abs() < 1.0);
        assert!((s - 1.0).abs() < 0.01);
        assert!((v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_from_hsv_roundtrip() {
        let original = Color::new(100, 150, 200, 255);
        let (h, s, v) = original.to_hsv();
        let restored = Color::from_hsv(h, s, v, 255);
        assert!((original.r as i32 - restored.r as i32).abs() <= 2);
        assert!((original.g as i32 - restored.g as i32).abs() <= 2);
        assert!((original.b as i32 - restored.b as i32).abs() <= 2);
    }

    #[test]
    fn test_color_display() {
        let c = Color::new(255, 128, 64, 255);
        assert_eq!(format!("{}", c), "Color(r:255, g:128, b:64, a:255)");
    }
}
