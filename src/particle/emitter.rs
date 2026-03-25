use crate::math::Vec3;
use crate::particle::particle::Particle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitShape {
    Point,
    Sphere,
    Hemisphere,
    Cone,
    Box,
    Circle,
    Edge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmitterConfig {
    pub shape: EmitShape,
    pub radius: f32,
    pub angle: f32,
    pub length: f32,
    pub arc: f32,
    pub spread: f32,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        EmitterConfig {
            shape: EmitShape::Point,
            radius: 1.0,
            angle: 25.0,
            length: 5.0,
            arc: 360.0,
            spread: 0.0,
        }
    }
}

pub struct Emitter {
    pub config: EmitterConfig,
    pub start_speed: f32,
    pub start_life: f32,
    pub start_speed_variation: f32,
    pub start_life_variation: f32,
    rng_state: u64,
}

impl Emitter {
    pub fn new() -> Self {
        Emitter {
            config: EmitterConfig::default(),
            start_speed: 5.0,
            start_life: 1.0,
            start_speed_variation: 0.0,
            start_life_variation: 0.0,
            rng_state: 12345678,
        }
    }

    fn rand_f32(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state as f32) / (u64::MAX as f32)
    }

    fn rand_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.rand_f32() * (max - min)
    }

    pub fn emit_particle(&mut self, position: Vec3) -> Particle {
        let mut p = Particle::new();
        p.alive = true;
        p.position = position;

        let speed = self.start_speed + self.rand_range(
            -self.start_speed_variation,
            self.start_speed_variation,
        );

        p.velocity = self.compute_direction() * speed;
        p.start_life = self.start_life + self.rand_range(
            -self.start_life_variation,
            self.start_life_variation,
        );
        p.start_life = p.start_life.max(0.01);
        p.life = p.start_life;
        p.random_seed = self.rng_state as u32;
        p
    }

    fn compute_direction(&mut self) -> Vec3 {
        match self.config.shape {
            EmitShape::Point => Vec3::new(0.0, 1.0, 0.0),
            EmitShape::Sphere => {
                let theta = self.rand_f32() * std::f32::consts::PI * 2.0;
                let phi = (self.rand_f32() * 2.0 - 1.0).acos();
                Vec3::new(
                    phi.sin() * theta.cos(),
                    phi.sin() * theta.sin(),
                    phi.cos(),
                ).get_normalized()
            }
            EmitShape::Cone => {
                let angle_rad = self.config.angle.to_radians();
                let r = self.rand_f32() * angle_rad.sin();
                let theta = self.rand_f32() * std::f32::consts::PI * 2.0;
                Vec3::new(r * theta.cos(), angle_rad.cos(), r * theta.sin()).get_normalized()
            }
            _ => Vec3::new(0.0, 1.0, 0.0),
        }
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Emitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Emitter")
            .field("config", &self.config)
            .field("start_speed", &self.start_speed)
            .field("start_life", &self.start_life)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emitter_new() {
        let e = Emitter::new();
        assert_eq!(e.config.shape, EmitShape::Point);
    }

    #[test]
    fn test_emitter_emit_particle() {
        let mut e = Emitter::new();
        let p = e.emit_particle(Vec3::ZERO);
        assert!(p.alive);
        assert!(p.life > 0.0);
    }

    #[test]
    fn test_emitter_cone_direction() {
        let mut e = Emitter::new();
        e.config.shape = EmitShape::Cone;
        e.config.angle = 30.0;
        for _ in 0..10 {
            let p = e.emit_particle(Vec3::ZERO);
            let len = (p.velocity.x * p.velocity.x + p.velocity.y * p.velocity.y + p.velocity.z * p.velocity.z).sqrt();
            assert!(len > 0.0);
        }
    }

    #[test]
    fn test_emitter_sphere() {
        let mut e = Emitter::new();
        e.config.shape = EmitShape::Sphere;
        for _ in 0..10 {
            let p = e.emit_particle(Vec3::ZERO);
            let len = (p.velocity.x * p.velocity.x + p.velocity.y * p.velocity.y + p.velocity.z * p.velocity.z).sqrt();
            assert!(len > 0.0);
        }
    }
}
