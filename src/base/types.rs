pub type Uint = u32;
pub type Ushort = u16;

#[cfg(not(any(
    target_os = "linux",
    target_os = "emscripten"
)))]
pub type Ulong = u32;

#[cfg(any(
    target_os = "linux",
    target_os = "emscripten"
))]
pub type Ulong = u64;

pub type FlagBits = u32;

pub type IndexType = i32;

pub const CC_INVALID_INDEX: IndexType = -1;
