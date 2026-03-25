/****************************************************************************
Rust port of Cocos Creator Audio Utils
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::RefCounted;

pub trait AudioUtils: RefCounted {
    fn get_volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);
    fn is_loop(&self) -> bool;
    fn set_loop(&mut self, loop_audio: bool);
    fn get_current_time(&self) -> f32;
    fn set_current_time(&mut self, time: f32) -> bool;
    fn get_duration(&self) -> f32;
    fn get_state(&self) -> AudioUtilsState;
    fn pause(&mut self);
    fn resume(&mut self);
    fn stop(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioUtilsState {
    Error = -1,
    Initializing = 0,
    Playing = 1,
    Paused = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_utils_state() {
        assert_ne!(AudioUtilsState::Playing, AudioUtilsState::Paused);
        assert_ne!(AudioUtilsState::Error, AudioUtilsState::Initializing);
    }
}
