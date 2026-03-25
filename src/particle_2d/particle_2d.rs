use crate::math::{Color, Vec2};

#[derive(Debug, Clone, PartialEq)]
pub struct Particle2D {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: f32,
    pub size_delta: f32,
    pub rotation: f32,
    pub rotation_delta: f32,
    pub color: Color,
    pub color_delta_r: f32,
    pub color_delta_g: f32,
    pub color_delta_b: f32,
    pub color_delta_a: f32,
    pub life: f32,
    pub start_life: f32,
    pub radial_accel: f32,
    pub tangential_accel: f32,
    pub start_pos: Vec2,
    pub alive: bool,
}

impl Particle2D {
    pub fn new() -> Self {
        Particle2D {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            size: 1.0,
            size_delta: 0.0,
            rotation: 0.0,
            rotation_delta: 0.0,
            color: Color::WHITE,
            color_delta_r: 0.0,
            color_delta_g: 0.0,
            color_delta_b: 0.0,
            color_delta_a: 0.0,
            life: 0.0,
            start_life: 1.0,
            radial_accel: 0.0,
            tangential_accel: 0.0,
            start_pos: Vec2::ZERO,
            alive: false,
        }
    }

    pub fn get_normalized_lifetime(&self) -> f32 {
        if self.start_life <= 0.0 { return 1.0; }
        1.0 - (self.life / self.start_life).clamp(0.0, 1.0)
    }

    pub fn update_gravity(&mut self, gravity: Vec2, dt: f32) {
        if !self.alive { return; }

        self.velocity.x += gravity.x * dt;
        self.velocity.y += gravity.y * dt;
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;

        self.size += self.size_delta * dt;
        self.rotation += self.rotation_delta * dt;

        self.color.r = (self.color.r as f32 + self.color_delta_r * dt).clamp(0.0, 255.0) as u8;
        self.color.g = (self.color.g as f32 + self.color_delta_g * dt).clamp(0.0, 255.0) as u8;
        self.color.b = (self.color.b as f32 + self.color_delta_b * dt).clamp(0.0, 255.0) as u8;
        self.color.a = (self.color.a as f32 + self.color_delta_a * dt).clamp(0.0, 255.0) as u8;

        self.life -= dt;
        if self.life <= 0.0 {
            self.alive = false;
        }
    }

    pub fn update_radial(&mut self, source_pos: Vec2, dt: f32) {
        if !self.alive { return; }

        let dx = self.position.x - source_pos.x;
        let dy = self.position.y - source_pos.y;
        let len = (dx * dx + dy * dy).sqrt().max(1e-6);

        let radial_x = dx / len;
        let radial_y = dy / len;

        let tangential_x = -radial_y;
        let tangential_y = radial_x;

        self.velocity.x += (radial_x * self.radial_accel + tangential_x * self.tangential_accel) * dt;
        self.velocity.y += (radial_y * self.radial_accel + tangential_y * self.tangential_accel) * dt;

        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;

        self.size += self.size_delta * dt;
        self.rotation += self.rotation_delta * dt;

        self.life -= dt;
        if self.life <= 0.0 {
            self.alive = false;
        }
    }
}

impl Default for Particle2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_2d_new() {
        let p = Particle2D::new();
        assert!(!p.alive);
        assert_eq!(p.position, Vec2::ZERO);
    }

    #[test]
    fn test_particle_2d_gravity_update() {
        let mut p = Particle2D::new();
        p.alive = true;
        p.life = 5.0;
        p.start_life = 5.0;
        p.velocity = Vec2::new(10.0, 0.0);
        p.update_gravity(Vec2::new(0.0, -9.8), 1.0);
        assert!((p.position.x - 10.0).abs() < 1e-4);
        assert!((p.velocity.y - (-9.8)).abs() < 1e-4);
    }

    #[test]
    fn test_particle_2d_dies() {
        let mut p = Particle2D::new();
        p.alive = true;
        p.life = 0.1;
        p.start_life = 0.1;
        p.update_gravity(Vec2::ZERO, 1.0);
        assert!(!p.alive);
    }

    #[test]
    fn test_particle_2d_normalized_lifetime() {
        let mut p = Particle2D::new();
        p.life = 0.5;
        p.start_life = 1.0;
        let t = p.get_normalized_lifetime();
        assert!((t - 0.5).abs() < 1e-5);
    }
}
