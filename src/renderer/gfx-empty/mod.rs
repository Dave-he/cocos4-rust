/****************************************************************************
Rust port of Cocos Creator GFX Empty Backend
All methods are no-op, used for testing and CI environments.
****************************************************************************/

pub mod command_buffer;
pub mod device;

pub use command_buffer::*;
pub use device::*;
