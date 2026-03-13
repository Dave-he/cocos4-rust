/****************************************************************************
 Rust port of Cocos Creator Audio System
 Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
 ****************************************************************************/

pub mod decoder;
pub mod utils;

pub use decoder::*;
pub use utils::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioState {
    Initializing,
    Playing,
    Paused,
    Stopped,
    Interrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioType {
    Music,
    Effect,
}

#[derive(Debug, Clone)]
pub struct AudioClip {
    pub url: String,
    pub format: AudioFormat,
    pub duration: f32,
    pub sample_rate: u32,
    pub channel_count: u32,
    pub bits_per_sample: u32,
    pub pcm_data: Vec<u8>,
}

impl AudioClip {
    pub fn new(url: &str) -> Self {
        AudioClip {
            url: url.to_string(),
            format: AudioFormat::Unknown,
            duration: 0.0,
            sample_rate: 44100,
            channel_count: 2,
            bits_per_sample: 16,
            pcm_data: Vec::new(),
        }
    }

    pub fn from_pcm(url: &str, data: Vec<u8>, info: AudioDecoderInfo) -> Self {
        let duration = if info.sample_rate > 0 {
            info.total_frames as f32 / info.sample_rate as f32
        } else {
            0.0
        };
        AudioClip {
            url: url.to_string(),
            format: info.format,
            duration,
            sample_rate: info.sample_rate,
            channel_count: info.channel_count,
            bits_per_sample: info.bits_per_sample,
            pcm_data: data,
        }
    }

    pub fn is_loaded(&self) -> bool {
        !self.pcm_data.is_empty()
    }
}

pub struct AudioSource {
    pub clip: Option<AudioClip>,
    pub volume: f32,
    pub loop_audio: bool,
    pub play_on_awake: bool,
    pub audio_type: AudioType,
    state: AudioState,
    current_time: f32,
    player_id: Option<u32>,
}

impl AudioSource {
    pub fn new() -> Self {
        AudioSource {
            clip: None,
            volume: 1.0,
            loop_audio: false,
            play_on_awake: false,
            audio_type: AudioType::Effect,
            state: AudioState::Stopped,
            current_time: 0.0,
            player_id: None,
        }
    }

    pub fn set_clip(&mut self, clip: AudioClip) {
        self.clip = Some(clip);
    }

    pub fn play(&mut self) -> bool {
        if self.clip.is_none() {
            return false;
        }
        self.state = AudioState::Playing;
        self.current_time = 0.0;
        true
    }

    pub fn pause(&mut self) {
        if self.state == AudioState::Playing {
            self.state = AudioState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == AudioState::Paused {
            self.state = AudioState::Playing;
        }
    }

    pub fn stop(&mut self) {
        self.state = AudioState::Stopped;
        self.current_time = 0.0;
    }

    pub fn get_state(&self) -> AudioState {
        self.state
    }

    pub fn is_playing(&self) -> bool {
        self.state == AudioState::Playing
    }

    pub fn is_paused(&self) -> bool {
        self.state == AudioState::Paused
    }

    pub fn get_current_time(&self) -> f32 {
        self.current_time
    }

    pub fn set_current_time(&mut self, time: f32) {
        if let Some(ref clip) = self.clip {
            self.current_time = time.clamp(0.0, clip.duration);
        }
    }

    pub fn get_duration(&self) -> f32 {
        self.clip.as_ref().map_or(0.0, |c| c.duration)
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_loop(&mut self, loop_audio: bool) {
        self.loop_audio = loop_audio;
    }

    pub fn update(&mut self, dt: f32) {
        if self.state != AudioState::Playing {
            return;
        }
        if let Some(ref clip) = self.clip {
            self.current_time += dt;
            if self.current_time >= clip.duration {
                if self.loop_audio {
                    self.current_time -= clip.duration;
                } else {
                    self.current_time = clip.duration;
                    self.state = AudioState::Stopped;
                }
            }
        }
    }

    pub fn set_player_id(&mut self, id: Option<u32>) {
        self.player_id = id;
    }

    pub fn get_player_id(&self) -> Option<u32> {
        self.player_id
    }
}

impl Default for AudioSource {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AudioPlayer {
    pub id: u32,
    pub source: AudioSource,
    pub finished_callback: Option<Box<dyn Fn(u32) + Send + Sync>>,
}

impl AudioPlayer {
    pub fn new(id: u32) -> Self {
        AudioPlayer {
            id,
            source: AudioSource::new(),
            finished_callback: None,
        }
    }

    pub fn play(&mut self) -> bool {
        self.source.play()
    }

    pub fn pause(&mut self) {
        self.source.pause();
    }

    pub fn resume(&mut self) {
        self.source.resume();
    }

    pub fn stop(&mut self) {
        self.source.stop();
    }

    pub fn update(&mut self, dt: f32) {
        let was_playing = self.source.is_playing();
        self.source.update(dt);
        if was_playing && self.source.get_state() == AudioState::Stopped {
            if let Some(ref cb) = self.finished_callback {
                cb(self.id);
            }
        }
    }

    pub fn set_on_finished<F>(&mut self, callback: F)
    where
        F: Fn(u32) + Send + Sync + 'static,
    {
        self.finished_callback = Some(Box::new(callback));
    }
}

pub struct AudioManager {
    players: HashMap<u32, AudioPlayer>,
    next_id: u32,
    music_volume: f32,
    effect_volume: f32,
    music_mute: bool,
    effect_mute: bool,
    current_music_id: Option<u32>,
}

impl AudioManager {
    pub fn new() -> Self {
        AudioManager {
            players: HashMap::new(),
            next_id: 1,
            music_volume: 1.0,
            effect_volume: 1.0,
            music_mute: false,
            effect_mute: false,
            current_music_id: None,
        }
    }

    pub fn play_effect(&mut self, clip: AudioClip, volume: Option<f32>) -> u32 {
        let id = self.alloc_id();
        let mut player = AudioPlayer::new(id);
        player.source.set_clip(clip);
        player.source.set_volume(volume.unwrap_or(self.effect_volume));
        player.source.audio_type = AudioType::Effect;
        player.play();
        self.players.insert(id, player);
        id
    }

    pub fn play_music(&mut self, clip: AudioClip, loop_music: bool) -> u32 {
        if let Some(music_id) = self.current_music_id {
            self.stop(music_id);
        }
        let id = self.alloc_id();
        let mut player = AudioPlayer::new(id);
        player.source.set_clip(clip);
        player.source.set_volume(self.music_volume);
        player.source.set_loop(loop_music);
        player.source.audio_type = AudioType::Music;
        player.play();
        self.players.insert(id, player);
        self.current_music_id = Some(id);
        id
    }

    pub fn stop(&mut self, id: u32) {
        if let Some(player) = self.players.get_mut(&id) {
            player.stop();
        }
        if self.current_music_id == Some(id) {
            self.current_music_id = None;
        }
        self.players.remove(&id);
    }

    pub fn stop_all_effects(&mut self) {
        let ids: Vec<u32> = self.players
            .values()
            .filter(|p| p.source.audio_type == AudioType::Effect)
            .map(|p| p.id)
            .collect();
        for id in ids {
            self.players.remove(&id);
        }
    }

    pub fn stop_all(&mut self) {
        self.players.clear();
        self.current_music_id = None;
    }

    pub fn pause(&mut self, id: u32) {
        if let Some(player) = self.players.get_mut(&id) {
            player.pause();
        }
    }

    pub fn resume(&mut self, id: u32) {
        if let Some(player) = self.players.get_mut(&id) {
            player.resume();
        }
    }

    pub fn pause_all(&mut self) {
        for player in self.players.values_mut() {
            player.pause();
        }
    }

    pub fn resume_all(&mut self) {
        for player in self.players.values_mut() {
            player.resume();
        }
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        for player in self.players.values_mut() {
            if player.source.audio_type == AudioType::Music {
                player.source.set_volume(if self.music_mute { 0.0 } else { self.music_volume });
            }
        }
    }

    pub fn get_music_volume(&self) -> f32 {
        self.music_volume
    }

    pub fn set_effect_volume(&mut self, volume: f32) {
        self.effect_volume = volume.clamp(0.0, 1.0);
        for player in self.players.values_mut() {
            if player.source.audio_type == AudioType::Effect {
                player.source.set_volume(if self.effect_mute { 0.0 } else { self.effect_volume });
            }
        }
    }

    pub fn get_effect_volume(&self) -> f32 {
        self.effect_volume
    }

    pub fn set_music_mute(&mut self, mute: bool) {
        self.music_mute = mute;
        let volume = if mute { 0.0 } else { self.music_volume };
        for player in self.players.values_mut() {
            if player.source.audio_type == AudioType::Music {
                player.source.set_volume(volume);
            }
        }
    }

    pub fn is_music_mute(&self) -> bool {
        self.music_mute
    }

    pub fn set_effect_mute(&mut self, mute: bool) {
        self.effect_mute = mute;
        let volume = if mute { 0.0 } else { self.effect_volume };
        for player in self.players.values_mut() {
            if player.source.audio_type == AudioType::Effect {
                player.source.set_volume(volume);
            }
        }
    }

    pub fn is_effect_mute(&self) -> bool {
        self.effect_mute
    }

    pub fn is_music_playing(&self) -> bool {
        self.current_music_id
            .and_then(|id| self.players.get(&id))
            .map_or(false, |p| p.source.is_playing())
    }

    pub fn update(&mut self, dt: f32) {
        let mut finished: Vec<u32> = Vec::new();
        for player in self.players.values_mut() {
            player.update(dt);
            if player.source.get_state() == AudioState::Stopped
                && !player.source.loop_audio
            {
                finished.push(player.id);
            }
        }
        for id in finished {
            self.players.remove(&id);
            if self.current_music_id == Some(id) {
                self.current_music_id = None;
            }
        }
    }

    pub fn get_player_count(&self) -> usize {
        self.players.len()
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_clip(duration: f32) -> AudioClip {
        let info = AudioDecoderInfo {
            format: AudioFormat::Wav,
            sample_rate: 44100,
            channel_count: 2,
            bits_per_sample: 16,
            total_frames: (duration * 44100.0) as u64,
        };
        let data = vec![0u8; (info.total_frames * 4) as usize];
        AudioClip::from_pcm("test.wav", data, info)
    }

    #[test]
    fn test_audio_source_play_stop() {
        let mut src = AudioSource::new();
        src.set_clip(make_clip(1.0));
        assert!(src.play());
        assert!(src.is_playing());
        src.stop();
        assert_eq!(src.get_state(), AudioState::Stopped);
    }

    #[test]
    fn test_audio_source_pause_resume() {
        let mut src = AudioSource::new();
        src.set_clip(make_clip(1.0));
        src.play();
        src.pause();
        assert!(src.is_paused());
        src.resume();
        assert!(src.is_playing());
    }

    #[test]
    fn test_audio_source_loop() {
        let mut src = AudioSource::new();
        src.set_clip(make_clip(1.0));
        src.set_loop(true);
        src.play();
        src.update(1.5);
        assert!(src.is_playing());
        assert!(src.get_current_time() < 1.0);
    }

    #[test]
    fn test_audio_source_no_loop_stops() {
        let mut src = AudioSource::new();
        src.set_clip(make_clip(1.0));
        src.set_loop(false);
        src.play();
        src.update(1.5);
        assert_eq!(src.get_state(), AudioState::Stopped);
    }

    #[test]
    fn test_audio_manager_play_effect() {
        let mut mgr = AudioManager::new();
        let clip = make_clip(1.0);
        let id = mgr.play_effect(clip, None);
        assert_eq!(mgr.get_player_count(), 1);
        mgr.stop(id);
        assert_eq!(mgr.get_player_count(), 0);
    }

    #[test]
    fn test_audio_manager_play_music() {
        let mut mgr = AudioManager::new();
        let clip = make_clip(2.0);
        let _id = mgr.play_music(clip, true);
        assert!(mgr.is_music_playing());
    }

    #[test]
    fn test_audio_manager_volume() {
        let mut mgr = AudioManager::new();
        mgr.set_music_volume(0.5);
        assert!((mgr.get_music_volume() - 0.5).abs() < 1e-6);
        mgr.set_effect_volume(0.3);
        assert!((mgr.get_effect_volume() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_audio_manager_mute() {
        let mut mgr = AudioManager::new();
        mgr.set_music_mute(true);
        assert!(mgr.is_music_mute());
        mgr.set_music_mute(false);
        assert!(!mgr.is_music_mute());
    }

    #[test]
    fn test_audio_manager_update_cleans_finished() {
        let mut mgr = AudioManager::new();
        let clip = make_clip(0.1);
        mgr.play_effect(clip, None);
        assert_eq!(mgr.get_player_count(), 1);
        mgr.update(0.2);
        assert_eq!(mgr.get_player_count(), 0);
    }

    #[test]
    fn test_audio_manager_stop_all_effects() {
        let mut mgr = AudioManager::new();
        mgr.play_effect(make_clip(1.0), None);
        mgr.play_effect(make_clip(1.0), None);
        mgr.play_music(make_clip(2.0), false);
        mgr.stop_all_effects();
        assert_eq!(mgr.get_player_count(), 1);
    }

    #[test]
    fn test_audio_clip_from_pcm() {
        let info = AudioDecoderInfo {
            format: AudioFormat::Wav,
            sample_rate: 44100,
            channel_count: 2,
            bits_per_sample: 16,
            total_frames: 44100,
        };
        let data = vec![0u8; 44100 * 4];
        let clip = AudioClip::from_pcm("test.wav", data, info);
        assert!(clip.is_loaded());
        assert!((clip.duration - 1.0).abs() < 1e-4);
    }
}
