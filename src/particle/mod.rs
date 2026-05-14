pub mod emitter;
pub mod extra_modules;
pub mod modules;
#[allow(clippy::module_inception)]
pub mod particle;
pub mod particle_system;

pub use emitter::{EmitShape, Emitter, EmitterConfig};
pub use extra_modules::{NoiseModule, TrailModule, TrailPoint};
pub use modules::{
    ColorOverLifetime, RotationOverLifetime, SizeOverLifetime, VelocityOverLifetime,
};
pub use particle::Particle;
pub use particle_system::{ParticleSystem, ParticleSystemState};
