/****************************************************************************
Rust port of Cocos Creator Skeletal Animation Utilities
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::Mat4;
use std::sync::{Arc, Weak};
use crate::core::scene_graph::{BaseNode, NodePtr};

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

pub fn get_transform(_node: &NodePtr, _root: &NodePtr) -> Option<std::rc::Weak<IJointTransform>> {
    None
}

pub fn delete_transform(_node: &()) {
}

/// Blend tree node type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendNodeType {
    Leaf,
    Blend1D,
    Blend2D,
    Additive,
}

#[derive(Debug, Clone)]
pub struct BlendNode {
    pub node_type: BlendNodeType,
    pub clip_name: Option<String>,
    pub weight: f32,
    pub threshold: f32,
    pub children: Vec<usize>,
}

impl BlendNode {
    pub fn leaf(clip_name: &str) -> Self {
        BlendNode {
            node_type: BlendNodeType::Leaf,
            clip_name: Some(clip_name.to_string()),
            weight: 1.0,
            threshold: 0.0,
            children: Vec::new(),
        }
    }

    pub fn blend1d(children: Vec<usize>) -> Self {
        BlendNode {
            node_type: BlendNodeType::Blend1D,
            clip_name: None,
            weight: 1.0,
            threshold: 0.0,
            children,
        }
    }
}

pub struct BlendTree {
    pub parameter: f32,
    nodes: Vec<BlendNode>,
    root: usize,
}

impl BlendTree {
    pub fn new() -> Self {
        BlendTree {
            parameter: 0.0,
            nodes: Vec::new(),
            root: 0,
        }
    }

    pub fn add_node(&mut self, node: BlendNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    pub fn set_root(&mut self, idx: usize) {
        self.root = idx;
    }

    pub fn evaluate(&self) -> Vec<(String, f32)> {
        if self.nodes.is_empty() {
            return Vec::new();
        }
        self.eval_node(self.root)
    }

    fn eval_node(&self, idx: usize) -> Vec<(String, f32)> {
        if idx >= self.nodes.len() {
            return Vec::new();
        }
        let node = &self.nodes[idx];
        match node.node_type {
            BlendNodeType::Leaf => {
                if let Some(ref name) = node.clip_name {
                    vec![(name.clone(), node.weight)]
                } else {
                    Vec::new()
                }
            }
            BlendNodeType::Blend1D => {
                if node.children.len() < 2 {
                    return Vec::new();
                }
                let t = self.parameter.clamp(0.0, 1.0);
                let left_idx = node.children[0];
                let right_idx = node.children[1];
                let mut result = Vec::new();
                for (name, w) in self.eval_node(left_idx) {
                    result.push((name, w * (1.0 - t)));
                }
                for (name, w) in self.eval_node(right_idx) {
                    if let Some(existing) = result.iter_mut().find(|(n, _)| n == &name) {
                        existing.1 += w * t;
                    } else {
                        result.push((name, w * t));
                    }
                }
                result
            }
            BlendNodeType::Additive => {
                let mut result = Vec::new();
                for &child_idx in &node.children {
                    for (name, w) in self.eval_node(child_idx) {
                        result.push((name, w));
                    }
                }
                result
            }
            BlendNodeType::Blend2D => Vec::new(),
        }
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for BlendTree {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AnimationManager {
    states: std::collections::HashMap<String, AnimationState>,
    clips: std::collections::HashMap<String, AnimationClip>,
    current_state: Option<String>,
    blend_tree: Option<BlendTree>,
}

impl AnimationManager {
    pub fn new() -> Self {
        AnimationManager {
            states: std::collections::HashMap::new(),
            clips: std::collections::HashMap::new(),
            current_state: None,
            blend_tree: None,
        }
    }

    pub fn add_clip(&mut self, clip: AnimationClip) {
        let name = clip.name.clone();
        let state = AnimationState::new(clip.clone());
        self.clips.insert(name.clone(), clip);
        self.states.insert(name, state);
    }

    pub fn play(&mut self, name: &str) -> bool {
        if !self.states.contains_key(name) {
            return false;
        }
        if let Some(current) = &self.current_state {
            if let Some(state) = self.states.get_mut(current.as_str()) {
                state.reset();
            }
        }
        if let Some(state) = self.states.get_mut(name) {
            state.resume();
            self.current_state = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn stop(&mut self) {
        if let Some(name) = self.current_state.take() {
            if let Some(state) = self.states.get_mut(&name) {
                state.reset();
                state.pause();
            }
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref name) = self.current_state {
            if let Some(state) = self.states.get_mut(name.as_str()) {
                state.pause();
            }
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref name) = self.current_state {
            if let Some(state) = self.states.get_mut(name.as_str()) {
                state.resume();
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(ref name) = self.current_state.clone() {
            if let Some(state) = self.states.get_mut(name.as_str()) {
                state.update(dt);
            }
        }
    }

    pub fn get_current_state(&self) -> Option<&AnimationState> {
        self.current_state.as_ref().and_then(|n| self.states.get(n))
    }

    pub fn is_playing(&self, name: &str) -> bool {
        self.current_state.as_deref() == Some(name)
            && self.states.get(name).map_or(false, |s| !s.paused)
    }

    pub fn get_time(&self) -> f32 {
        self.get_current_state().map_or(0.0, |s| s.time)
    }

    pub fn get_clip_names(&self) -> Vec<&str> {
        self.clips.keys().map(|s| s.as_str()).collect()
    }

    pub fn has_clip(&self, name: &str) -> bool {
        self.clips.contains_key(name)
    }

    pub fn set_blend_tree(&mut self, tree: BlendTree) {
        self.blend_tree = Some(tree);
    }

    pub fn get_blend_tree(&self) -> Option<&BlendTree> {
        self.blend_tree.as_ref()
    }

    pub fn get_blend_tree_mut(&mut self) -> Option<&mut BlendTree> {
        self.blend_tree.as_mut()
    }

    pub fn evaluate_blend_tree(&self) -> Vec<(String, f32)> {
        self.blend_tree.as_ref().map_or(Vec::new(), |t| t.evaluate())
    }

    pub fn crossfade(&mut self, name: &str, duration: f32) -> bool {
        let _ = duration;
        self.play(name)
    }

    pub fn get_clip_count(&self) -> usize {
        self.clips.len()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod animation_tests {
    use super::*;

    fn make_clip(name: &str, duration: f32) -> AnimationClip {
        let mut clip = AnimationClip::new(name);
        clip.duration = duration;
        clip
    }

    #[test]
    fn test_animation_manager_add_play() {
        let mut mgr = AnimationManager::new();
        mgr.add_clip(make_clip("walk", 1.0));
        assert!(mgr.has_clip("walk"));
        assert!(mgr.play("walk"));
        assert!(mgr.is_playing("walk"));
    }

    #[test]
    fn test_animation_manager_stop() {
        let mut mgr = AnimationManager::new();
        mgr.add_clip(make_clip("run", 1.0));
        mgr.play("run");
        mgr.stop();
        assert!(!mgr.is_playing("run"));
    }

    #[test]
    fn test_animation_manager_update() {
        let mut mgr = AnimationManager::new();
        mgr.add_clip(make_clip("idle", 2.0));
        mgr.play("idle");
        mgr.update(0.5);
        assert!((mgr.get_time() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_animation_manager_play_missing() {
        let mut mgr = AnimationManager::new();
        assert!(!mgr.play("nonexistent"));
    }

    #[test]
    fn test_blend_tree_1d() {
        let mut tree = BlendTree::new();
        let left = tree.add_node(BlendNode::leaf("idle"));
        let right = tree.add_node(BlendNode::leaf("walk"));
        let root = tree.add_node(BlendNode::blend1d(vec![left, right]));
        tree.set_root(root);

        tree.parameter = 0.0;
        let result = tree.evaluate();
        assert!(!result.is_empty());
        let idle_w = result.iter().find(|(n, _)| n == "idle").map(|(_, w)| *w).unwrap_or(0.0);
        assert!((idle_w - 1.0).abs() < 1e-5);

        tree.parameter = 1.0;
        let result = tree.evaluate();
        let walk_w = result.iter().find(|(n, _)| n == "walk").map(|(_, w)| *w).unwrap_or(0.0);
        assert!((walk_w - 1.0).abs() < 1e-5);

        tree.parameter = 0.5;
        let result = tree.evaluate();
        let idle_w = result.iter().find(|(n, _)| n == "idle").map(|(_, w)| *w).unwrap_or(0.0);
        let walk_w = result.iter().find(|(n, _)| n == "walk").map(|(_, w)| *w).unwrap_or(0.0);
        assert!((idle_w - 0.5).abs() < 1e-5);
        assert!((walk_w - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_tree_leaf() {
        let mut tree = BlendTree::new();
        let leaf = tree.add_node(BlendNode::leaf("run"));
        tree.set_root(leaf);
        let result = tree.evaluate();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "run");
        assert!((result[0].1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_animation_curve_evaluate() {
        let mut curve = AnimationCurve::new();
        curve.add_keyframe(0.0, 0.0);
        curve.add_keyframe(1.0, 10.0);
        assert!((curve.evaluate(0.0) - 0.0).abs() < 1e-5);
        assert!((curve.evaluate(0.5) - 5.0).abs() < 1e-5);
        assert!((curve.evaluate(1.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_animation_state_loop() {
        let mut clip = AnimationClip::new("test");
        clip.duration = 1.0;
        clip.wrap_mode = WrapMode::Loop;
        let mut state = AnimationState::new(clip);
        state.update(1.5);
        assert!(state.time < 1.0);
        assert!(!state.paused);
    }

    #[test]
    fn test_animation_state_clamp() {
        let mut clip = AnimationClip::new("test");
        clip.duration = 1.0;
        clip.wrap_mode = WrapMode::Normal;
        let mut state = AnimationState::new(clip);
        state.update(2.0);
        assert!((state.time - 1.0).abs() < 1e-5);
        assert!(state.paused);
    }
}
