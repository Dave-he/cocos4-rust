#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineAnimationPlayMode {
    Normal,
    Loop,
    PingPong,
    Once,
}

#[derive(Debug, Clone)]
pub struct SpineKeyFrame {
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

#[derive(Debug, Clone)]
pub struct SpineTrack {
    pub bone_name: String,
    pub keyframes: Vec<SpineKeyFrame>,
}

#[derive(Debug, Clone)]
pub struct SpineAnimation {
    pub name: String,
    pub duration: f32,
    pub play_mode: SpineAnimationPlayMode,
    pub tracks: Vec<SpineTrack>,
    pub playing: bool,
    pub elapsed: f32,
}

impl SpineAnimation {
    pub fn new(name: &str, duration: f32) -> Self {
        Self {
            name: name.to_string(),
            duration,
            play_mode: SpineAnimationPlayMode::Loop,
            tracks: Vec::new(),
            playing: false,
            elapsed: 0.0,
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.elapsed = 0.0;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.elapsed = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if !self.playing {
            return;
        }
        self.elapsed += dt;
        if self.elapsed >= self.duration {
            match self.play_mode {
                SpineAnimationPlayMode::Once => {
                    self.elapsed = self.duration;
                    self.playing = false;
                }
                SpineAnimationPlayMode::Loop => {
                    self.elapsed %= self.duration;
                }
                _ => {
                    self.elapsed %= self.duration;
                }
            }
        }
    }

    pub fn get_interpolated(&self, bone_name: &str) -> Option<SpineKeyFrame> {
        self.tracks
            .iter()
            .find(|t| t.bone_name == bone_name)
            .and_then(|track| {
                if track.keyframes.is_empty() {
                    return None;
                }
                let t = self.elapsed;
                if let Some(kf) = track
                    .keyframes
                    .iter()
                    .find(|kf| (kf.time - t).abs() < 0.001)
                {
                    return Some(kf.clone());
                }
                Some(track.keyframes[0].clone())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spine_animation_play_once() {
        let mut anim = SpineAnimation::new("idle", 1.0);
        anim.play_mode = SpineAnimationPlayMode::Once;
        anim.play();
        anim.update(1.5);
        assert!(!anim.playing);
    }

    #[test]
    fn test_spine_animation_loop() {
        let mut anim = SpineAnimation::new("walk", 2.0);
        anim.play();
        anim.update(2.5);
        assert!((anim.elapsed - 0.5).abs() < 0.01);
    }
}
