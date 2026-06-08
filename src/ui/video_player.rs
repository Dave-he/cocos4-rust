#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoState {
    Idle,
    Loading,
    Playing,
    Paused,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoSourceType {
    Local,
    Remote,
    Streaming,
}

#[derive(Debug, Clone)]
pub struct VideoPlayer {
    pub url: String,
    pub source_type: VideoSourceType,
    pub state: VideoState,
    pub current_time: f32,
    pub duration: f32,
    pub volume: f32,
    pub loop_play: bool,
    pub keep_aspect_ratio: bool,
    pub full_screen_on_awake: bool,
    pub is_playing: bool,
    pub is_fullscreen: bool,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            source_type: VideoSourceType::Local,
            state: VideoState::Idle,
            current_time: 0.0,
            duration: 0.0,
            volume: 1.0,
            loop_play: false,
            keep_aspect_ratio: true,
            full_screen_on_awake: false,
            is_playing: false,
            is_fullscreen: false,
        }
    }

    pub fn play(&mut self) {
        self.state = VideoState::Playing;
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.state = VideoState::Paused;
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.state = VideoState::Stopped;
        self.is_playing = false;
        self.current_time = 0.0;
    }

    pub fn seek(&mut self, time: f32) {
        self.current_time = time.clamp(0.0, self.duration);
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_player_new() {
        let vp = VideoPlayer::new();
        assert_eq!(vp.state, VideoState::Idle);
        assert_eq!(vp.volume, 1.0);
    }

    #[test]
    fn test_video_play_pause_stop() {
        let mut vp = VideoPlayer::new();
        vp.play();
        assert_eq!(vp.state, VideoState::Playing);
        vp.pause();
        assert_eq!(vp.state, VideoState::Paused);
        vp.stop();
        assert_eq!(vp.state, VideoState::Stopped);
        assert_eq!(vp.current_time, 0.0);
    }

    #[test]
    fn test_video_seek() {
        let mut vp = VideoPlayer::new();
        vp.duration = 100.0;
        vp.seek(50.0);
        assert_eq!(vp.current_time, 50.0);
        vp.seek(200.0);
        assert_eq!(vp.current_time, 100.0);
    }

    #[test]
    fn test_video_volume() {
        let mut vp = VideoPlayer::new();
        vp.set_volume(0.5);
        assert_eq!(vp.volume, 0.5);
        vp.set_volume(2.0);
        assert_eq!(vp.volume, 1.0);
    }

    #[test]
    fn test_video_url() {
        let mut vp = VideoPlayer::new();
        vp.set_url("https://example.com/video.mp4");
        assert_eq!(vp.url, "https://example.com/video.mp4");
    }
}
