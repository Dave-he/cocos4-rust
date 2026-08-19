#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Default for KeyFrame {
    fn default() -> Self {
        Self {
            time: 0.0,
            tween_easing: 0.0,
            curve: Vec::new(),
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
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
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn get_keyframe_at(&self, time: f32) -> Option<&KeyFrame> {
        self.keyframes.iter().find(|kf| (kf.time - time).abs() < 0.001)
    }

    pub fn get_interpolated(&self, time: f32) -> KeyFrame {
        if self.keyframes.is_empty() {
            return KeyFrame::default();
        }
        if self.keyframes.len() == 1 {
            return self.keyframes[0].clone();
        }

        let mut prev_idx = 0;
        let mut next_idx = 0;

        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= time {
                prev_idx = i;
            }
            if kf.time >= time {
                next_idx = i;
                break;
            }
        }

        if prev_idx == next_idx {
            return self.keyframes[prev_idx].clone();
        }

        let prev = &self.keyframes[prev_idx];
        next_idx = (prev_idx + 1).min(self.keyframes.len() - 1);
        let next = &self.keyframes[next_idx];

        let alpha = if next.time > prev.time {
            (time - prev.time) / (next.time - prev.time)
        } else {
            0.0
        };

        let alpha = if prev.tween_easing != 0.0 && !prev.curve.is_empty() {
            self.apply_bezier_easing(alpha, &prev.curve)
        } else if prev.tween_easing > 0.0 {
            let p = 1.0 + prev.tween_easing * 10.0;
            alpha.powf(p)
        } else if prev.tween_easing < 0.0 {
            let p = 1.0 + (-prev.tween_easing) * 10.0;
            1.0 - (1.0 - alpha).powf(p)
        } else {
            alpha
        };

        let inv = 1.0 - alpha;
        KeyFrame {
            time: prev.time * inv + next.time * alpha,
            tween_easing: 0.0,
            curve: Vec::new(),
            x: prev.x * inv + next.x * alpha,
            y: prev.y * inv + next.y * alpha,
            rotation: prev.rotation * inv + next.rotation * alpha,
            scale_x: prev.scale_x * inv + next.scale_x * alpha,
            scale_y: prev.scale_y * inv + next.scale_y * alpha,
        }
    }

    fn apply_bezier_easing(&self, t: f32, _curve: &[f32]) -> f32 {
        t
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
    ping_pong_forward: bool,
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
            ping_pong_forward: true,
        }
    }

    pub fn add_track(&mut self, track: AnimationTrack) {
        self.tracks.push(track);
    }

    pub fn play(&mut self) {
        self.is_playing = true;
        self.current_time = 0.0;
        self.ping_pong_forward = true;
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
                AnimationPlayMode::PingPong => {
                    if self.ping_pong_forward {
                        self.current_time = self.duration * 2.0 - self.current_time;
                        self.ping_pong_forward = false;
                    } else {
                        self.current_time -= self.duration;
                        self.ping_pong_forward = true;
                    }
                }
                AnimationPlayMode::Normal => {
                    self.current_time = self.duration;
                    self.is_playing = false;
                }
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
    fn test_animation_normal_mode() {
        let mut anim = Animation::new("action", 1.0);
        anim.play_mode = AnimationPlayMode::Normal;
        anim.play();
        anim.advance_time(1.5);
        assert!(!anim.is_playing());
    }

    #[test]
    fn test_animation_ping_pong() {
        let mut anim = Animation::new("bounce", 1.0);
        anim.play_mode = AnimationPlayMode::PingPong;
        anim.play();
        anim.advance_time(1.5);
        assert!(!anim.ping_pong_forward);
        assert!((anim.get_current_time() - 0.5).abs() < 0.01);
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

    #[test]
    fn test_track_interpolation_linear() {
        let mut track = AnimationTrack::new("t", 1.0);
        track.add_keyframe(KeyFrame { time: 0.0, x: 0.0, y: 0.0, ..Default::default() });
        track.add_keyframe(KeyFrame { time: 1.0, x: 100.0, y: 50.0, ..Default::default() });
        let kf = track.get_interpolated(0.5);
        assert!((kf.x - 50.0).abs() < 0.01);
        assert!((kf.y - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_track_interpolation_rotation_scale() {
        let mut track = AnimationTrack::new("r", 1.0);
        track.add_keyframe(KeyFrame { time: 0.0, rotation: 0.0, scale_x: 1.0, scale_y: 1.0, ..Default::default() });
        track.add_keyframe(KeyFrame { time: 1.0, rotation: 90.0, scale_x: 2.0, scale_y: 0.5, ..Default::default() });
        let kf = track.get_interpolated(0.5);
        assert!((kf.rotation - 45.0).abs() < 0.01);
        assert!((kf.scale_x - 1.5).abs() < 0.01);
        assert!((kf.scale_y - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_track_easing() {
        let mut track = AnimationTrack::new("e", 1.0);
        track.add_keyframe(KeyFrame { time: 0.0, x: 0.0, tween_easing: 0.5, ..Default::default() });
        track.add_keyframe(KeyFrame { time: 1.0, x: 100.0, ..Default::default() });
        let kf = track.get_interpolated(0.5);
        assert!(kf.x < 50.0);
    }
}
