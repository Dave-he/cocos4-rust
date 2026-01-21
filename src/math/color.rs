use super::Vec4;

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

    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
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
}

impl Default for Color {
    fn default() -> Self {
        Color::WHITE
    }
}
