pub type uint = u32;
pub type ushort = u16;

#[cfg(not(any(
    target_os = "linux",
    target_os = "qnx",
    target_os = "emscripten",
    target_os = "openharmony"
)))]
pub type ulong = u32;

#[cfg(any(
    target_os = "linux",
    target_os = "qnx",
    target_os = "emscripten",
    target_os = "openharmony"
))]
pub type ulong = u64;

pub type FlagBits = u32;

pub type IndexType = i32;

pub const CC_INVALID_INDEX: IndexType = -1;
