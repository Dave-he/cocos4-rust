use crate::math::Vec3;
use crate::particle::particle::Particle;
use crate::particle::emitter::Emitter;
use crate::particle::modules::{ColorOverLifetime, SizeOverLifetime, VelocityOverLifetime, RotationOverLifetime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleSystemState {
    Stopped,
    Playing,
    Paused,
}

pub struct ParticleSystem {
    pub duration: f32,
    pub looping: bool,
    pub prewarm: bool,
    pub start_delay: f32,
    pub max_particles: usize,
    pub emission_rate: f32,
    pub gravity_modifier: f32,

    pub color_over_lifetime: ColorOverLifetime,
    pub size_over_lifetime: SizeOverLifetime,
    pub velocity_over_lifetime: VelocityOverLifetime,
    pub rotation_over_lifetime: RotationOverLifetime,

    pub emitter: Emitter,

    particles: Vec<Particle>,
    state: ParticleSystemState,
    elapsed: f32,
    emit_accumulator: f32,
    position: Vec3,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            duration: 5.0,
            looping: true,
            prewarm: false,
            start_delay: 0.0,
            max_particles: 100,
            emission_rate: 10.0,
            gravity_modifier: 0.0,
            color_over_lifetime: ColorOverLifetime::new(),
            size_over_lifetime: SizeOverLifetime::new(),
            velocity_over_lifetime: VelocityOverLifetime::new(),
            rotation_over_lifetime: RotationOverLifetime::new(),
            emitter: Emitter::new(),
            particles: Vec::new(),
            state: ParticleSystemState::Stopped,
            elapsed: 0.0,
            emit_accumulator: 0.0,
            position: Vec3::ZERO,
        }
    }

    pub fn play(&mut self) {
        self.state = ParticleSystemState::Playing;
        self.elapsed = 0.0;
    }

    pub fn pause(&mut self) {
        if self.state == ParticleSystemState::Playing {
            self.state = ParticleSystemState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == ParticleSystemState::Paused {
            self.state = ParticleSystemState::Playing;
        }
    }

    pub fn stop(&mut self) {
        self.state = ParticleSystemState::Stopped;
        self.particles.clear();
        self.elapsed = 0.0;
        self.emit_accumulator = 0.0;
    }

    pub fn clear(&mut self) {
        self.particles.clear();
    }

    pub fn get_state(&self) -> ParticleSystemState {
        self.state
    }

    pub fn is_playing(&self) -> bool {
        self.state == ParticleSystemState::Playing
    }

    pub fn get_particle_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn emit_burst(&mut self, count: usize) {
        for _ in 0..count {
            if self.particles.len() < self.max_particles {
                let p = self.emitter.emit_particle(self.position);
                self.particles.push(p);
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.state != ParticleSystemState::Playing {
            return;
        }

        self.elapsed += dt;

        self.emit_accumulator += self.emission_rate * dt;
        while self.emit_accumulator >= 1.0 && self.get_particle_count() < self.max_particles {
            let p = self.emitter.emit_particle(self.position);
            if let Some(slot) = self.particles.iter_mut().find(|p| !p.alive) {
                *slot = p;
            } else if self.particles.len() < self.max_particles {
                self.particles.push(p);
            }
            self.emit_accumulator -= 1.0;
        }

        for p in self.particles.iter_mut() {
            if !p.alive { continue; }

            let t = p.get_normalized_lifetime();

            if self.size_over_lifetime.enabled {
                let s = self.size_over_lifetime.evaluate(t);
                p.size = Vec3::new(s, s, s);
            }

            if self.color_over_lifetime.enabled {
                p.color = self.color_over_lifetime.evaluate(t);
            }

            if self.velocity_over_lifetime.enabled {
                let (vx, vy, vz) = self.velocity_over_lifetime.evaluate(t);
                p.velocity.x += vx * dt;
                p.velocity.y += vy * dt;
                p.velocity.z += vz * dt;
            }

            if self.gravity_modifier != 0.0 {
                p.velocity.y -= self.gravity_modifier * 9.8 * dt;
            }

            p.update(dt);
        }

        self.particles.retain(|p| p.alive || !p.is_expired());

        if self.elapsed >= self.duration {
            if self.looping {
                self.elapsed = 0.0;
            } else {
                if self.get_particle_count() == 0 {
                    self.state = ParticleSystemState::Stopped;
                }
            }
        }
    }

    pub fn get_particles(&self) -> &[Particle] {
        &self.particles
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ParticleSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParticleSystem")
            .field("state", &self.state)
            .field("particle_count", &self.get_particle_count())
            .field("elapsed", &self.elapsed)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_system_new() {
        let ps = ParticleSystem::new();
        assert_eq!(ps.get_state(), ParticleSystemState::Stopped);
        assert_eq!(ps.get_particle_count(), 0);
    }

    #[test]
    fn test_particle_system_play_stop() {
        let mut ps = ParticleSystem::new();
        ps.play();
        assert_eq!(ps.get_state(), ParticleSystemState::Playing);
        ps.stop();
        assert_eq!(ps.get_state(), ParticleSystemState::Stopped);
    }

    #[test]
    fn test_particle_system_emit_burst() {
        let mut ps = ParticleSystem::new();
        ps.play();
        ps.emit_burst(10);
        assert_eq!(ps.get_particle_count(), 10);
    }

    #[test]
    fn test_particle_system_update_emits() {
        let mut ps = ParticleSystem::new();
        ps.emission_rate = 100.0;
        ps.emitter.start_life = 5.0;
        ps.play();
        ps.update(0.1);
        assert!(ps.get_particle_count() > 0);
    }

    #[test]
    fn test_particle_system_pause_resume() {
        let mut ps = ParticleSystem::new();
        ps.play();
        ps.pause();
        assert_eq!(ps.get_state(), ParticleSystemState::Paused);
        ps.resume();
        assert_eq!(ps.get_state(), ParticleSystemState::Playing);
    }

    #[test]
    fn test_particle_system_max_particles() {
        let mut ps = ParticleSystem::new();
        ps.max_particles = 5;
        ps.play();
        ps.emit_burst(100);
        assert!(ps.get_particle_count() <= 5);
    }

    #[test]
    fn test_particle_system_clear() {
        let mut ps = ParticleSystem::new();
        ps.play();
        ps.emit_burst(10);
        ps.clear();
        assert_eq!(ps.get_particle_count(), 0);
    }

    #[test]
    fn test_particle_system_not_playing_no_update() {
        let mut ps = ParticleSystem::new();
        ps.update(1.0);
        assert_eq!(ps.get_particle_count(), 0);
    }
}
