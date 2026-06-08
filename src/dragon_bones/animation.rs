#[derive(Debug, Clone, Copy)]
pub enum AnimationPlayMode {
    Normal,
    NoTween,
    Once,
    Loop,
    PingPong,
}

#[derive(Debug, Clone)]
pub struct KeyFrame {
    pub time: f32,
    pub tween_easing: f32,
    pub curve: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct AnimationTrack {
    pub name: String,
    pub duration: f32,
    pub frame_rate: f32,
    pub keyframes: Vec<KeyFrame>,
    pub play_count: i32,
    pub scale: f32,
}

impl AnimationTrack {
    pub fn new(name: &str, duration: f32) -> Self {
        Self {
            name: name.to_string(),
            duration,
            frame_rate: 30.0,
            keyframes: Vec::new(),
            play_count: 0,
            scale: 1.0,
        }
    }

    pub fn add_keyframe(&mut self, frame: KeyFrame) {
        self.keyframes.push(frame);
    }

    pub fn get_keyframe_at(&self, time: f32) -> Option<&KeyFrame> {
        self.keyframes
            .iter()
            .find(|kf| (kf.time - time).abs() < 0.001)
    }

    pub fn get_total_frames(&self) -> usize {
        (self.duration * self.frame_rate) as usize
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub play_mode: AnimationPlayMode,
    pub tracks: Vec<AnimationTrack>,
    pub auto_tween: bool,
    is_playing: bool,
    current_time: f32,
}

impl Animation {
    pub fn new(name: &str, duration: f32) -> Self {
        Self {
            name: name.to_string(),
            duration,
            play_mode: AnimationPlayMode::Loop,
            tracks: Vec::new(),
            auto_tween: true,
            is_playing: false,
            current_time: 0.0,
        }
    }

    pub fn add_track(&mut self, track: AnimationTrack) {
        self.tracks.push(track);
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.current_time = 0.0;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_time = 0.0;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn advance_time(&mut self, dt: f32) {
        if !self.is_playing {
            return;
        }
        self.current_time += dt;
        if self.current_time >= self.duration {
            match self.play_mode {
                AnimationPlayMode::Once => {
                    self.current_time = self.duration;
                    self.is_playing = false;
                }
                AnimationPlayMode::Loop => {
                    self.current_time %= self.duration;
                }
                AnimationPlayMode::PingPong => {}
                _ => {
                    self.current_time %= self.duration;
                }
            }
        }
    }

    pub fn get_current_time(&self) -> f32 {
        self.current_time
    }

    pub fn get_progress(&self) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            self.current_time / self.duration
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_new() {
        let anim = Animation::new("walk", 2.0);
        assert_eq!(anim.name, "walk");
        assert_eq!(anim.duration, 2.0);
        assert!(!anim.is_playing());
    }

    #[test]
    fn test_animation_play_once() {
        let mut anim = Animation::new("idle", 1.0);
        anim.play_mode = AnimationPlayMode::Once;
        anim.play();
        assert!(anim.is_playing());
        anim.advance_time(1.5);
        assert!(!anim.is_playing());
        assert_eq!(anim.get_current_time(), 1.0);
    }

    #[test]
    fn test_animation_loop() {
        let mut anim = Animation::new("run", 2.0);
        anim.play();
        anim.advance_time(2.5);
        assert_eq!(anim.get_current_time(), 0.5);
    }

    #[test]
    fn test_animation_track() {
        let track = AnimationTrack::new("translate", 1.0);
        assert_eq!(track.name, "translate");
        assert_eq!(track.get_total_frames(), 30);
    }

    #[test]
    fn test_animation_progress() {
        let mut anim = Animation::new("test", 2.0);
        anim.play();
        anim.advance_time(1.0);
        assert!((anim.get_progress() - 0.5).abs() < 0.01);
    }
}
