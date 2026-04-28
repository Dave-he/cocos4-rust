#[allow(clippy::module_inception)]
pub mod particle;
pub mod particle_system;
pub mod emitter;
pub mod modules;
pub mod extra_modules;

pub use particle::Particle;
pub use particle_system::{ParticleSystem, ParticleSystemState};
pub use emitter::{Emitter, EmitShape, EmitterConfig};
pub use modules::{ColorOverLifetime, SizeOverLifetime, VelocityOverLifetime, RotationOverLifetime};
pub use extra_modules::{NoiseModule, TrailModule, TrailPoint};
