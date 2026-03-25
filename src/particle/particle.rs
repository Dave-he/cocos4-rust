use crate::math::{Color, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub rotation: Vec3,
    pub angular_velocity: Vec3,
    pub size: Vec3,
    pub color: Color,
    pub life: f32,
    pub start_life: f32,
    pub mass: f32,
    pub frame_index: u32,
    pub random_seed: u32,
    pub alive: bool,
}

impl Particle {
    pub fn new() -> Self {
        Particle {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            rotation: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            size: Vec3::ONE,
            color: Color::WHITE,
            life: 0.0,
            start_life: 1.0,
            mass: 1.0,
            frame_index: 0,
            random_seed: 0,
            alive: false,
        }
    }

    pub fn get_normalized_lifetime(&self) -> f32 {
        if self.start_life <= 0.0 { return 1.0; }
        1.0 - (self.life / self.start_life).clamp(0.0, 1.0)
    }

    pub fn is_expired(&self) -> bool {
        self.life <= 0.0
    }

    pub fn update(&mut self, dt: f32) {
        if !self.alive { return; }
        self.velocity.x += self.acceleration.x * dt;
        self.velocity.y += self.acceleration.y * dt;
        self.velocity.z += self.acceleration.z * dt;

        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
        self.position.z += self.velocity.z * dt;

        self.rotation.x += self.angular_velocity.x * dt;
        self.rotation.y += self.angular_velocity.y * dt;
        self.rotation.z += self.angular_velocity.z * dt;

        self.life -= dt;
        if self.life <= 0.0 {
            self.alive = false;
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_new() {
        let p = Particle::new();
        assert!(!p.alive);
        assert_eq!(p.position, Vec3::ZERO);
    }

    #[test]
    fn test_particle_update() {
        let mut p = Particle::new();
        p.alive = true;
        p.life = 2.0;
        p.start_life = 2.0;
        p.velocity = Vec3::new(10.0, 0.0, 0.0);
        p.update(0.1);
        assert!((p.position.x - 1.0).abs() < 1e-4);
        assert!((p.life - 1.9).abs() < 1e-4);
        assert!(p.alive);
    }

    #[test]
    fn test_particle_dies() {
        let mut p = Particle::new();
        p.alive = true;
        p.life = 0.1;
        p.start_life = 0.1;
        p.update(0.5);
        assert!(!p.alive);
        assert!(p.is_expired());
    }

    #[test]
    fn test_particle_normalized_lifetime() {
        let mut p = Particle::new();
        p.life = 0.5;
        p.start_life = 1.0;
        let t = p.get_normalized_lifetime();
        assert!((t - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_particle_acceleration() {
        let mut p = Particle::new();
        p.alive = true;
        p.life = 10.0;
        p.start_life = 10.0;
        p.acceleration = Vec3::new(0.0, -9.8, 0.0);
        p.update(1.0);
        assert!((p.velocity.y - (-9.8)).abs() < 1e-4);
    }
}
