/****************************************************************************
Rust port of Cocos Creator 3D Skeletal Animation
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Mat4, Quaternion, Vec3};

#[derive(Debug, Clone)]
pub struct JointTransform {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
}

impl JointTransform {
    pub fn new() -> Self {
        JointTransform {
            position: Vec3::ZERO,
            rotation: Quaternion::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_srt(&self.rotation, &self.position, &self.scale)
    }
}

impl Default for JointTransform {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub sample_rate: f32,
    pub joint_names: Vec<String>,
    pub frames: Vec<Vec<JointTransform>>,
}

impl AnimationClip {
    pub fn new(name: &str, duration: f32, sample_rate: f32) -> Self {
        AnimationClip {
            name: name.to_string(),
            duration,
            sample_rate,
            joint_names: Vec::new(),
            frames: Vec::new(),
        }
    }

    pub fn get_frame_count(&self) -> usize {
        (self.duration * self.sample_rate) as usize
    }

    pub fn get_frame_at(&self, frame_index: usize) -> Option<&Vec<JointTransform>> {
        self.frames.get(frame_index)
    }

    pub fn sample(&self, time: f32) -> Vec<JointTransform> {
        if self.frames.is_empty() {
            return Vec::new();
        }

        let frame_count = self.frames.len() as f32;
        let t = (time / self.duration).fract() * (frame_count - 1.0);
        let frame0 = t as usize;
        let frame1 = (frame0 + 1).min(self.frames.len() - 1);
        let alpha = t - frame0 as f32;

        let f0 = &self.frames[frame0];
        let f1 = &self.frames[frame1];

        f0.iter()
            .zip(f1.iter())
            .map(|(a, b)| JointTransform {
                position: Vec3::new(
                    a.position.x + (b.position.x - a.position.x) * alpha,
                    a.position.y + (b.position.y - a.position.y) * alpha,
                    a.position.z + (b.position.z - a.position.z) * alpha,
                ),
                rotation: Quaternion::slerp(&a.rotation, &b.rotation, alpha),
                scale: Vec3::new(
                    a.scale.x + (b.scale.x - a.scale.x) * alpha,
                    a.scale.y + (b.scale.y - a.scale.y) * alpha,
                    a.scale.z + (b.scale.z - a.scale.z) * alpha,
                ),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    Default = 0,
    Normal = 1,
    Loop = 2,
    PingPong = 3,
    ClampForever = 4,
    Once = 5,
}

#[derive(Debug)]
pub struct SkeletalAnimationState {
    pub clip: AnimationClip,
    pub current_time: f32,
    pub state: AnimationState,
    pub wrap_mode: WrapMode,
    pub speed: f32,
    pub weight: f32,
    pub joint_matrices: Vec<Mat4>,
}

impl SkeletalAnimationState {
    pub fn new(clip: AnimationClip) -> Self {
        SkeletalAnimationState {
            clip,
            current_time: 0.0,
            state: AnimationState::Stopped,
            wrap_mode: WrapMode::Loop,
            speed: 1.0,
            weight: 1.0,
            joint_matrices: Vec::new(),
        }
    }

    pub fn play(&mut self) {
        self.state = AnimationState::Playing;
    }

    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    pub fn stop(&mut self) {
        self.state = AnimationState::Stopped;
        self.current_time = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        if self.state != AnimationState::Playing {
            return;
        }

        self.current_time += dt * self.speed;

        match self.wrap_mode {
            WrapMode::Loop | WrapMode::Default => {
                if self.current_time >= self.clip.duration {
                    self.current_time -= self.clip.duration;
                }
            }
            WrapMode::Once | WrapMode::Normal => {
                if self.current_time >= self.clip.duration {
                    self.current_time = self.clip.duration;
                    self.state = AnimationState::Stopped;
                }
            }
            WrapMode::PingPong => {
                let period = self.clip.duration * 2.0;
                self.current_time %= period;
                if self.current_time > self.clip.duration {
                    self.current_time = period - self.current_time;
                }
            }
            WrapMode::ClampForever => {
                self.current_time = self.current_time.min(self.clip.duration);
            }
        }

        let transforms = self.clip.sample(self.current_time);
        self.joint_matrices = transforms.iter().map(|t| t.to_matrix()).collect();
    }

    pub fn get_current_time(&self) -> f32 {
        self.current_time
    }

    pub fn is_playing(&self) -> bool {
        self.state == AnimationState::Playing
    }

    pub fn get_joint_matrix(&self, index: usize) -> Option<&Mat4> {
        self.joint_matrices.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_transform_default() {
        let t = JointTransform::default();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quaternion::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_joint_transform_to_matrix() {
        let t = JointTransform::default();
        let mat = t.to_matrix();
        assert_eq!(mat, Mat4::IDENTITY);
    }

    #[test]
    fn test_animation_clip_new() {
        let clip = AnimationClip::new("walk", 1.0, 30.0);
        assert_eq!(clip.name, "walk");
        assert_eq!(clip.duration, 1.0);
        assert_eq!(clip.sample_rate, 30.0);
        assert_eq!(clip.get_frame_count(), 30);
    }

    #[test]
    fn test_skeletal_animation_state() {
        let clip = AnimationClip::new("idle", 1.0, 30.0);
        let mut state = SkeletalAnimationState::new(clip);
        assert_eq!(state.state, AnimationState::Stopped);

        state.play();
        assert!(state.is_playing());

        state.update(0.5);
        assert_eq!(state.get_current_time(), 0.5);

        state.pause();
        assert_eq!(state.state, AnimationState::Paused);

        state.stop();
        assert_eq!(state.get_current_time(), 0.0);
        assert_eq!(state.state, AnimationState::Stopped);
    }

    #[test]
    fn test_animation_loop_wrap() {
        let clip = AnimationClip::new("loop", 1.0, 30.0);
        let mut state = SkeletalAnimationState::new(clip);
        state.wrap_mode = WrapMode::Loop;
        state.play();

        state.update(1.5);
        assert!(state.get_current_time() < 1.0);
        assert!(state.is_playing());
    }

    #[test]
    fn test_animation_once_wrap() {
        let clip = AnimationClip::new("once", 1.0, 30.0);
        let mut state = SkeletalAnimationState::new(clip);
        state.wrap_mode = WrapMode::Once;
        state.play();

        state.update(1.5);
        assert_eq!(state.state, AnimationState::Stopped);
        assert_eq!(state.get_current_time(), 1.0);
    }
}
