#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpineAnimationPlayMode {
    Normal,
    Loop,
    PingPong,
    Once,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineType {
    Rotate,
    Translate,
    Scale,
    Color,
    Attachment,
}

#[derive(Debug, Clone)]
pub struct SpineKeyFrame {
    pub time: f32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub tween_easing: Option<f32>,
    pub timeline_type: TimelineType,
}

impl Default for SpineKeyFrame {
    fn default() -> Self {
        Self {
            time: 0.0,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            tween_easing: None,
            timeline_type: TimelineType::Translate,
        }
    }
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
    pub blend_weight: f32,
    pub ping_pong_forward: bool,
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
            blend_weight: 1.0,
            ping_pong_forward: true,
        }
    }

    pub fn play(&mut self) {
        self.playing = true;
        self.elapsed = 0.0;
        self.ping_pong_forward = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.elapsed = 0.0;
    }

    pub fn set_blend_weight(&mut self, weight: f32) {
        self.blend_weight = weight.clamp(0.0, 1.0);
    }

    pub fn update(&mut self, dt: f32) {
        if !self.playing { return; }
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
                SpineAnimationPlayMode::Normal => {
                    self.elapsed = self.duration;
                    self.playing = false;
                }
                SpineAnimationPlayMode::PingPong => {
                    if self.ping_pong_forward {
                        self.elapsed = self.duration * 2.0 - self.elapsed;
                        self.ping_pong_forward = false;
                    } else {
                        self.elapsed -= self.duration;
                        self.ping_pong_forward = true;
                    }
                }
            }
        }
    }

    pub fn get_interpolated(&self, bone_name: &str) -> Option<SpineKeyFrame> {
        self.tracks.iter().find(|t| t.bone_name == bone_name).and_then(|track| {
            if track.keyframes.is_empty() { return None; }
            if track.keyframes.len() == 1 { return Some(track.keyframes[0].clone()); }

            let t = self.elapsed;
            let mut prev_idx = 0;
            let mut next_idx = 0;

            for (i, kf) in track.keyframes.iter().enumerate() {
                if kf.time <= t {
                    prev_idx = i;
                }
                if kf.time >= t {
                    next_idx = i;
                    break;
                }
            }

            if prev_idx == next_idx {
                return Some(track.keyframes[prev_idx].clone());
            }

            let prev = &track.keyframes[prev_idx];
            next_idx = (prev_idx + 1).min(track.keyframes.len() - 1);
            let next = &track.keyframes[next_idx];

            let alpha = if next.time > prev.time {
                (t - prev.time) / (next.time - prev.time)
            } else {
                0.0
            };

            let alpha = self.apply_easing(alpha, prev.tween_easing);

            Some(self.interpolate_keyframes(prev, next, alpha))
        })
    }

    fn apply_easing(&self, t: f32, easing: Option<f32>) -> f32 {
        match easing {
            None | Some(0.0) => t,
            Some(e) if e > 0.0 => {
                let p = 1.0 + e * 10.0;
                t.powf(p)
            }
            Some(e) => {
                let p = 1.0 + (-e) * 10.0;
                1.0 - (1.0 - t).powf(p)
            }
        }
    }

    fn interpolate_keyframes(&self, a: &SpineKeyFrame, b: &SpineKeyFrame, alpha: f32) -> SpineKeyFrame {
        let inv = 1.0 - alpha;
        SpineKeyFrame {
            time: a.time * inv + b.time * alpha,
            x: a.x * inv + b.x * alpha,
            y: a.y * inv + b.y * alpha,
            rotation: a.rotation * inv + b.rotation * alpha,
            scale_x: a.scale_x * inv + b.scale_x * alpha,
            scale_y: a.scale_y * inv + b.scale_y * alpha,
            tween_easing: None,
            timeline_type: a.timeline_type,
        }
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

    #[test]
    fn test_spine_animation_normal_mode() {
        let mut anim = SpineAnimation::new("action", 1.0);
        anim.play_mode = SpineAnimationPlayMode::Normal;
        anim.play();
        anim.update(1.5);
        assert!(!anim.playing);
        assert_eq!(anim.elapsed, 1.0);
    }

    #[test]
    fn test_spine_animation_ping_pong() {
        let mut anim = SpineAnimation::new("bounce", 1.0);
        anim.play_mode = SpineAnimationPlayMode::PingPong;
        anim.play();
        anim.update(1.5);
        assert!(!anim.ping_pong_forward);
        assert!((anim.elapsed - 0.5).abs() < 0.01);
        anim.update(0.5);
        assert!(anim.ping_pong_forward);
    }

    #[test]
    fn test_spine_interpolation_linear() {
        let mut anim = SpineAnimation::new("test", 2.0);
        let track = SpineTrack {
            bone_name: "bone0".to_string(),
            keyframes: vec![
                SpineKeyFrame { time: 0.0, x: 0.0, y: 0.0, ..Default::default() },
                SpineKeyFrame { time: 1.0, x: 100.0, y: 50.0, ..Default::default() },
                SpineKeyFrame { time: 2.0, x: 200.0, y: 100.0, ..Default::default() },
            ],
        };
        anim.tracks.push(track);
        anim.play();

        anim.elapsed = 0.5;
        let kf = anim.get_interpolated("bone0").unwrap();
        assert!((kf.x - 50.0).abs() < 0.01);
        assert!((kf.y - 25.0).abs() < 0.01);

        anim.elapsed = 1.5;
        let kf = anim.get_interpolated("bone0").unwrap();
        assert!((kf.x - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_spine_interpolation_rotation() {
        let mut anim = SpineAnimation::new("rot", 1.0);
        anim.tracks.push(SpineTrack {
            bone_name: "rot_bone".to_string(),
            keyframes: vec![
                SpineKeyFrame { time: 0.0, rotation: 0.0, timeline_type: TimelineType::Rotate, ..Default::default() },
                SpineKeyFrame { time: 1.0, rotation: 90.0, timeline_type: TimelineType::Rotate, ..Default::default() },
            ],
        });
        anim.play();
        anim.elapsed = 0.5;
        let kf = anim.get_interpolated("rot_bone").unwrap();
        assert!((kf.rotation - 45.0).abs() < 0.01);
    }

    #[test]
    fn test_spine_interpolation_scale() {
        let mut anim = SpineAnimation::new("scale", 1.0);
        anim.tracks.push(SpineTrack {
            bone_name: "scale_bone".to_string(),
            keyframes: vec![
                SpineKeyFrame { time: 0.0, scale_x: 1.0, scale_y: 1.0, timeline_type: TimelineType::Scale, ..Default::default() },
                SpineKeyFrame { time: 1.0, scale_x: 2.0, scale_y: 0.5, timeline_type: TimelineType::Scale, ..Default::default() },
            ],
        });
        anim.play();
        anim.elapsed = 0.5;
        let kf = anim.get_interpolated("scale_bone").unwrap();
        assert!((kf.scale_x - 1.5).abs() < 0.01);
        assert!((kf.scale_y - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_spine_blend_weight() {
        let mut anim = SpineAnimation::new("blend", 1.0);
        anim.set_blend_weight(0.5);
        assert_eq!(anim.blend_weight, 0.5);
        anim.set_blend_weight(2.0);
        assert_eq!(anim.blend_weight, 1.0);
        anim.set_blend_weight(-1.0);
        assert_eq!(anim.blend_weight, 0.0);
    }

    #[test]
    fn test_spine_easing() {
        let anim = SpineAnimation::new("ease", 1.0);
        let linear = anim.apply_easing(0.5, None);
        assert!((linear - 0.5).abs() < 0.001);
        let eased = anim.apply_easing(0.5, Some(0.5));
        assert!(eased < 0.5);
    }

    #[test]
    fn test_spine_single_keyframe() {
        let mut anim = SpineAnimation::new("single", 1.0);
        anim.tracks.push(SpineTrack {
            bone_name: "b".to_string(),
            keyframes: vec![SpineKeyFrame { time: 0.0, x: 42.0, ..Default::default() }],
        });
        let kf = anim.get_interpolated("b").unwrap();
        assert_eq!(kf.x, 42.0);
    }

    #[test]
    fn test_spine_empty_track() {
        let anim = SpineAnimation::new("empty", 1.0);
        assert!(anim.get_interpolated("nonexistent").is_none());
    }
}
