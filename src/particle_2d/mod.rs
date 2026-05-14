#[allow(clippy::module_inception)]
pub mod particle_2d;
pub mod particle_system_2d;

pub use particle_2d::Particle2D;
pub use particle_system_2d::{EmitterMode2D, ParticleSystem2D, PositionType2D};
