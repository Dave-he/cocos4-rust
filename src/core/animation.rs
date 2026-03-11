/****************************************************************************
Rust port of Cocos Creator Skeletal Animation Utilities
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Mat4;
use std::sync::{Arc, Weak};
use crate::core::scene_graph::Node;

pub trait Texture {}

#[derive(Clone)]
pub struct RealTimeJointTexture {
    pub textures: Vec<std::rc::Weak<dyn Texture>>,
    pub buffer: Vec<u8>,
}

impl RealTimeJointTexture {
    pub fn new() -> Self {
        RealTimeJointTexture {
            textures: Vec::new(),
            buffer: Vec::new(),
        }
    }
}

impl Drop for RealTimeJointTexture {
    fn drop(&mut self) {
        self.textures.clear();
        self.buffer.clear();
    }
}

#[derive(Clone)]
pub struct IJointTransform {
    pub local: Mat4,
    pub world: Mat4,
    pub stamp: i32,
    pub parent: Option<std::rc::Weak<IJointTransform>>,
}

impl IJointTransform {
    pub fn new() -> Self {
        IJointTransform {
            local: Mat4::IDENTITY,
            world: Mat4::IDENTITY,
            stamp: -1,
            parent: None,
        }
    }

    pub fn update_world_matrix(&mut self) {
        if let Some(ref parent) = self.parent {
            if let Some(parent) = parent.upgrade() {
                self.world = multiply_mat4(&parent.world, &self.local);
            } else {
                self.world = self.local;
            }
        } else {
            self.world = self.local;
        }
    }
}

impl Default for IJointTransform {
    fn default() -> Self {
        Self::new()
    }
}

fn multiply_mat4(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut result = Mat4::ZERO;
    for i in 0..4 {
        for j in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a.m[i * 4 + k] * b.m[k * 4 + j];
            }
            result.m[i * 4 + j] = sum;
        }
    }
    result
}

#[derive(Clone)]
pub struct AnimationEvent {
    pub frame: f32,
    pub func: String,
    pub params: Vec<String>,
}

impl AnimationEvent {
    pub fn new(frame: f32, func: &str) -> Self {
        AnimationEvent {
            frame,
            func: func.to_string(),
            params: Vec::new(),
        }
    }

    pub fn with_params(mut self, params: Vec<String>) -> Self {
        self.params = params;
        self
    }
}

#[derive(Clone)]
pub struct AnimationCurve {
    pub keys: Vec<f32>,
    pub values: Vec<f32>,
}

impl AnimationCurve {
    pub fn new() -> Self {
        AnimationCurve {
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn add_keyframe(&mut self, time: f32, value: f32) {
        self.keys.push(time);
        self.values.push(value);
    }

    pub fn evaluate(&self, time: f32) -> f32 {
        if self.keys.is_empty() {
            return 0.0;
        }
        
        if self.keys.len() == 1 {
            return self.values[0];
        }

        for i in 0..self.keys.len() - 1 {
            if time >= self.keys[i] && time <= self.keys[i + 1] {
                let t = (time - self.keys[i]) / (self.keys[i + 1] - self.keys[i]);
                return self.values[i] + t * (self.values[i + 1] - self.values[i]);
            }
        }

        *self.values.last().unwrap_or(&0.0)
    }
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f32,
    pub speed: f32,
    pub wrap_mode: WrapMode,
    pub curves: Vec<AnimationCurve>,
    pub events: Vec<AnimationEvent>,
    pub hash: u32,
}

impl AnimationClip {
    pub fn new(name: &str) -> Self {
        AnimationClip {
            name: name.to_string(),
            duration: 0.0,
            speed: 1.0,
            wrap_mode: WrapMode::Loop,
            curves: Vec::new(),
            events: Vec::new(),
            hash: 0,
        }
    }

    pub fn add_curve(&mut self, curve: AnimationCurve) {
        if let Some(last_key) = curve.keys.last() {
            if *last_key > self.duration {
                self.duration = *last_key;
            }
        }
        self.curves.push(curve);
    }

    pub fn add_event(&mut self, event: AnimationEvent) {
        self.events.push(event);
        self.events.sort_by(|a, b| a.frame.partial_cmp(&b.frame).unwrap());
    }

    pub fn sample(&self, time: f32) -> Vec<f32> {
        self.curves.iter().map(|c| c.evaluate(time)).collect()
    }

    pub fn calculate_hash(&mut self) {
        let mut hash: u32 = 0;
        for c in &self.curves {
            for &k in &c.keys {
                hash = hash.wrapping_mul(31).wrapping_add((k * 1000.0) as u32);
            }
            for &v in &c.values {
                hash = hash.wrapping_mul(31).wrapping_add((v * 1000.0) as u32);
            }
        }
        self.hash = hash;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapMode {
    Default = 0,
    Normal = 1,
    Loop = 2,
    PingPong = 3,
    LoopPingPong = 4,
    Clamp = 5,
    LoopClamp = 6,
}

impl Default for WrapMode {
    fn default() -> Self {
        WrapMode::Loop
    }
}

pub struct AnimationState {
    pub clip: AnimationClip,
    pub time: f32,
    pub weight: f32,
    pub paused: bool,
    pub play_count: u32,
    pub current_events: Vec<AnimationEvent>,
}

impl AnimationState {
    pub fn new(clip: AnimationClip) -> Self {
        AnimationState {
            clip,
            time: 0.0,
            weight: 1.0,
            paused: false,
            play_count: 0,
            current_events: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        self.time += dt * self.clip.speed;

        match self.clip.wrap_mode {
            WrapMode::Loop | WrapMode::LoopPingPong | WrapMode::LoopClamp => {
                while self.time > self.clip.duration {
                    self.time -= self.clip.duration;
                    self.play_count += 1;
                }
            }
            WrapMode::PingPong => {
                if self.time > self.clip.duration * 2.0 {
                    self.time = self.time % (self.clip.duration * 2.0);
                    self.play_count += 1;
                }
            }
            _ => {
                if self.time > self.clip.duration {
                    self.time = self.clip.duration;
                    self.paused = true;
                }
            }
        }

        self.update_events();
    }

    fn update_events(&mut self) {
        self.current_events.clear();
        let time = match self.clip.wrap_mode {
            WrapMode::PingPong => {
                if self.time > self.clip.duration {
                    self.clip.duration - (self.time - self.clip.duration)
                } else {
                    self.time
                }
            }
            _ => self.time,
        };

        for event in &self.clip.events {
            if event.frame <= time {
                self.current_events.push(event.clone());
            }
        }
    }

    pub fn get_sample(&self) -> Vec<f32> {
        let time = match self.clip.wrap_mode {
            WrapMode::PingPong => {
                if self.time > self.clip.duration {
                    self.clip.duration - (self.time - self.clip.duration)
                } else {
                    self.time
                }
            }
            _ => self.time,
        };
        self.clip.sample(time)
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn reset(&mut self) {
        self.time = 0.0;
        self.play_count = 0;
        self.paused = false;
        self.current_events.clear();
    }
}

pub fn get_world_matrix(_transform: &mut IJointTransform, _stamp: i32) -> Mat4 {
    Mat4::IDENTITY
}

pub fn get_transform(_node: &dyn Node, _root: &dyn Node) -> Option<std::rc::Weak<IJointTransform>> {
    None
}

pub fn delete_transform(_node: &()) {
}
