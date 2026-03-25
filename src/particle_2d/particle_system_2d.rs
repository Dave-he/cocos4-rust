use crate::math::{Color, Vec2};
use crate::particle_2d::particle_2d::Particle2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitterMode2D {
    Gravity,
    Radius,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionType2D {
    Free,
    Relative,
    Grouped,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSystem2D {
    pub duration: f32,
    pub emission_rate: f32,
    pub total_particles: usize,
    pub life: f32,
    pub life_var: f32,
    pub angle: f32,
    pub angle_var: f32,
    pub start_color: Color,
    pub end_color: Color,
    pub start_size: f32,
    pub start_size_var: f32,
    pub end_size: f32,
    pub end_size_var: f32,
    pub start_spin: f32,
    pub start_spin_var: f32,
    pub end_spin: f32,
    pub end_spin_var: f32,
    pub gravity: Vec2,
    pub speed: f32,
    pub speed_var: f32,
    pub radial_accel: f32,
    pub tangential_accel: f32,
    pub emitter_mode: EmitterMode2D,
    pub position_type: PositionType2D,
    pub source_pos: Vec2,

    particles: Vec<Particle2D>,
    playing: bool,
    elapsed: f32,
    emit_accumulator: f32,
    rng_state: u64,
}

impl ParticleSystem2D {
    pub fn new() -> Self {
        ParticleSystem2D {
            duration: -1.0,
            emission_rate: 10.0,
            total_particles: 100,
            life: 1.0,
            life_var: 0.0,
            angle: 90.0,
            angle_var: 0.0,
            start_color: Color::new(255, 255, 255, 255),
            end_color: Color::new(255, 255, 255, 0),
            start_size: 10.0,
            start_size_var: 0.0,
            end_size: -1.0,
            end_size_var: 0.0,
            start_spin: 0.0,
            start_spin_var: 0.0,
            end_spin: 0.0,
            end_spin_var: 0.0,
            gravity: Vec2::ZERO,
            speed: 100.0,
            speed_var: 0.0,
            radial_accel: 0.0,
            tangential_accel: 0.0,
            emitter_mode: EmitterMode2D::Gravity,
            position_type: PositionType2D::Free,
            source_pos: Vec2::ZERO,
            particles: Vec::new(),
            playing: false,
            elapsed: 0.0,
            emit_accumulator: 0.0,
            rng_state: 9876543210,
        }
    }

    fn rand_f32(&mut self) -> f32 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state as f32) / (u64::MAX as f32)
    }

    fn rand_range(&mut self, base: f32, var: f32) -> f32 {
        base + (self.rand_f32() * 2.0 - 1.0) * var
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.elapsed = 0.0;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.particles.clear();
        self.elapsed = 0.0;
        self.emit_accumulator = 0.0;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn get_particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }

    fn emit_one(&mut self) {
        let angle_rad = self.rand_range(self.angle, self.angle_var).to_radians();
        let speed = self.rand_range(self.speed, self.speed_var).max(0.0);
        let life = self.rand_range(self.life, self.life_var).max(0.01);
        let size = self.rand_range(self.start_size, self.start_size_var).max(0.0);
        let end_size = if self.end_size < 0.0 { size } else { self.rand_range(self.end_size, self.end_size_var).max(0.0) };

        let mut p = Particle2D::new();
        p.alive = true;
        p.position = self.source_pos;
        p.start_pos = self.source_pos;
        p.velocity = Vec2::new(angle_rad.cos() * speed, angle_rad.sin() * speed);
        p.life = life;
        p.start_life = life;
        p.size = size;
        p.size_delta = (end_size - size) / life;
        p.color = self.start_color;
        p.color_delta_r = (self.end_color.r as f32 - self.start_color.r as f32) / life;
        p.color_delta_g = (self.end_color.g as f32 - self.start_color.g as f32) / life;
        p.color_delta_b = (self.end_color.b as f32 - self.start_color.b as f32) / life;
        p.color_delta_a = (self.end_color.a as f32 - self.start_color.a as f32) / life;
        p.radial_accel = self.radial_accel;
        p.tangential_accel = self.tangential_accel;

        if let Some(slot) = self.particles.iter_mut().find(|p| !p.alive) {
            *slot = p;
        } else if self.particles.len() < self.total_particles {
            self.particles.push(p);
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.playing { return; }

        self.elapsed += dt;
        if self.duration >= 0.0 && self.elapsed >= self.duration {
            if self.get_particle_count() == 0 {
                self.playing = false;
            }
            return;
        }

        self.emit_accumulator += self.emission_rate * dt;
        while self.emit_accumulator >= 1.0 && self.get_particle_count() < self.total_particles {
            self.emit_one();
            self.emit_accumulator -= 1.0;
        }

        let gravity = self.gravity;
        let source = self.source_pos;
        let mode = self.emitter_mode;
        for p in self.particles.iter_mut() {
            if !p.alive { continue; }
            match mode {
                EmitterMode2D::Gravity => p.update_gravity(gravity, dt),
                EmitterMode2D::Radius => p.update_radial(source, dt),
            }
        }
    }

    pub fn get_particles(&self) -> &[Particle2D] {
        &self.particles
    }
}

impl Default for ParticleSystem2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ps2d_new() {
        let ps = ParticleSystem2D::new();
        assert!(!ps.is_playing());
        assert_eq!(ps.get_particle_count(), 0);
    }

    #[test]
    fn test_ps2d_play_stop() {
        let mut ps = ParticleSystem2D::new();
        ps.play();
        assert!(ps.is_playing());
        ps.stop();
        assert!(!ps.is_playing());
        assert_eq!(ps.get_particle_count(), 0);
    }

    #[test]
    fn test_ps2d_emit() {
        let mut ps = ParticleSystem2D::new();
        ps.emission_rate = 100.0;
        ps.life = 5.0;
        ps.play();
        ps.update(0.1);
        assert!(ps.get_particle_count() > 0);
    }

    #[test]
    fn test_ps2d_max_particles() {
        let mut ps = ParticleSystem2D::new();
        ps.total_particles = 5;
        ps.emission_rate = 100.0;
        ps.life = 10.0;
        ps.play();
        ps.update(1.0);
        assert!(ps.get_particle_count() <= 5);
    }

    #[test]
    fn test_ps2d_update_no_play() {
        let mut ps = ParticleSystem2D::new();
        ps.update(1.0);
        assert_eq!(ps.get_particle_count(), 0);
    }

    #[test]
    fn test_ps2d_duration() {
        let mut ps = ParticleSystem2D::new();
        ps.duration = 0.5;
        ps.life = 10.0;
        ps.emission_rate = 10.0;
        ps.play();
        ps.update(1.0);
        assert!(!ps.is_playing());
    }
}
