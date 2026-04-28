/****************************************************************************
Rust port of Cocos Creator Root Manager
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::scene::render_scene::{RenderScene, RenderSceneInfo};
use crate::scene::camera::Camera;
use crate::scene::model::Model;
use crate::scene::directional_light::DirectionalLight;
use crate::scene::lights::{SpotLight, PointLight, SphereLight};
use crate::renderer::pipeline::render_pipeline::RenderPipeline;
use crate::renderer::core::material::MaterialPool;

#[derive(Debug, Clone, Default)]
pub struct RootInfo {
    pub enable_hdr: bool,
}

#[derive(Debug, Clone)]
pub struct RenderWindowInfo {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for RenderWindowInfo {
    fn default() -> Self {
        RenderWindowInfo {
            title: String::from("Main Window"),
            width: 960,
            height: 640,
        }
    }
}

#[derive(Debug)]
pub struct RenderWindow {
    pub id: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub is_offscreen: bool,
}

impl RenderWindow {
    pub fn new(id: u32, info: &RenderWindowInfo) -> Self {
        RenderWindow {
            id,
            title: info.title.clone(),
            width: info.width,
            height: info.height,
            is_offscreen: false,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

pub struct Root {
    pub initialized: bool,
    pub enable_hdr: bool,
    pub frame_time: f64,
    pub cumulative_time: f64,
    pub frame_count: u64,
    pub accumulated_dt: f32,

    windows: Vec<RenderWindow>,
    main_window_id: Option<u32>,
    next_window_id: u32,

    scenes: Vec<RenderScene>,
    pipeline: Option<RenderPipeline>,

    pub materials: MaterialPool,
}

impl Root {
    pub fn new() -> Self {
        Root {
            initialized: false,
            enable_hdr: false,
            frame_time: 0.0,
            cumulative_time: 0.0,
            frame_count: 0,
            accumulated_dt: 0.0,
            windows: Vec::new(),
            main_window_id: None,
            next_window_id: 1,
            scenes: Vec::new(),
            pipeline: None,
            materials: MaterialPool::new(),
        }
    }

    pub fn initialize(&mut self, info: &RootInfo) -> bool {
        if self.initialized {
            return false;
        }
        self.enable_hdr = info.enable_hdr;
        self.initialized = true;
        true
    }

    pub fn destroy(&mut self) {
        self.destroy_scenes();
        self.destroy_windows();
        self.pipeline = None;
        self.initialized = false;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(win) = self.get_main_window_mut() {
            win.resize(width, height);
        }
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.resize(width, height);
        }
    }

    pub fn frame_move(&mut self, delta_time: f32) {
        self.frame_count += 1;
        self.accumulated_dt = delta_time;
        self.cumulative_time += delta_time as f64;
        self.frame_time = delta_time as f64;

        for scene in &mut self.scenes {
            scene.update(self.frame_count as u32);
        }
    }

    pub fn set_pipeline(&mut self, pipeline: RenderPipeline) {
        self.pipeline = Some(pipeline);
    }

    pub fn get_pipeline(&self) -> Option<&RenderPipeline> {
        self.pipeline.as_ref()
    }

    pub fn get_pipeline_mut(&mut self) -> Option<&mut RenderPipeline> {
        self.pipeline.as_mut()
    }

    pub fn on_global_pipeline_state_changed(&mut self) {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.on_global_pipeline_state_changed();
        }
    }

    pub fn create_window(&mut self, info: RenderWindowInfo) -> u32 {
        let id = self.next_window_id;
        self.next_window_id += 1;
        let win = RenderWindow::new(id, &info);
        self.windows.push(win);
        if self.main_window_id.is_none() {
            self.main_window_id = Some(id);
        }
        id
    }

    pub fn destroy_window(&mut self, id: u32) {
        self.windows.retain(|w| w.id != id);
        if self.main_window_id == Some(id) {
            self.main_window_id = self.windows.first().map(|w| w.id);
        }
    }

    pub fn destroy_windows(&mut self) {
        self.windows.clear();
        self.main_window_id = None;
    }

    pub fn get_window(&self, id: u32) -> Option<&RenderWindow> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_main_window(&self) -> Option<&RenderWindow> {
        self.main_window_id.and_then(|id| self.get_window(id))
    }

    pub fn get_main_window_mut(&mut self) -> Option<&mut RenderWindow> {
        if let Some(id) = self.main_window_id {
            self.windows.iter_mut().find(|w| w.id == id)
        } else {
            None
        }
    }

    pub fn create_scene(&mut self, info: RenderSceneInfo) -> usize {
        let mut scene = RenderScene::new();
        scene.initialize(info);
        self.scenes.push(scene);
        self.scenes.len() - 1
    }

    pub fn destroy_scene(&mut self, index: usize) {
        if index < self.scenes.len() {
            self.scenes[index].destroy();
            self.scenes.remove(index);
        }
    }

    pub fn destroy_scenes(&mut self) {
        for scene in &mut self.scenes {
            scene.destroy();
        }
        self.scenes.clear();
    }

    pub fn get_scene(&self, index: usize) -> Option<&RenderScene> {
        self.scenes.get(index)
    }

    pub fn get_scene_mut(&mut self, index: usize) -> Option<&mut RenderScene> {
        self.scenes.get_mut(index)
    }

    pub fn get_scene_count(&self) -> usize {
        self.scenes.len()
    }

    pub fn create_camera(&mut self, scene_index: usize, _name: &str) -> Option<usize> {
        let scene = self.scenes.get_mut(scene_index)?;
        let cam = Camera::new();
        scene.add_camera(cam);
        Some(scene.cameras.len() - 1)
    }

    pub fn create_model(&mut self, scene_index: usize) -> Option<u64> {
        let scene = self.scenes.get_mut(scene_index)?;
        let id = scene.generate_model_id();
        let model = Model::new(id);
        scene.add_model(model);
        Some(id)
    }

    pub fn create_directional_light(&mut self, scene_index: usize) -> bool {
        let scene = match self.scenes.get_mut(scene_index) {
            Some(s) => s,
            None => return false,
        };
        let light = DirectionalLight::new();
        scene.main_light = Some(light);
        true
    }

    pub fn create_spot_light(&mut self, scene_index: usize) -> Option<usize> {
        let scene = self.scenes.get_mut(scene_index)?;
        let light = SpotLight::new();
        scene.spot_lights.push(light);
        Some(scene.spot_lights.len() - 1)
    }

    pub fn create_point_light(&mut self, scene_index: usize) -> Option<usize> {
        let scene = self.scenes.get_mut(scene_index)?;
        let light = PointLight::new();
        scene.point_lights.push(light);
        Some(scene.point_lights.len() - 1)
    }

    pub fn create_sphere_light(&mut self, scene_index: usize) -> Option<usize> {
        let scene = self.scenes.get_mut(scene_index)?;
        let light = SphereLight::new();
        scene.sphere_lights.push(light);
        Some(scene.sphere_lights.len() - 1)
    }
}

impl Default for Root {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_new() {
        let root = Root::new();
        assert!(!root.initialized);
        assert_eq!(root.frame_count, 0);
        assert_eq!(root.get_scene_count(), 0);
    }

    #[test]
    fn test_root_initialize() {
        let mut root = Root::new();
        assert!(root.initialize(&RootInfo::default()));
        assert!(root.initialized);
        assert!(!root.initialize(&RootInfo::default()));
    }

    #[test]
    fn test_root_destroy() {
        let mut root = Root::new();
        root.initialize(&RootInfo::default());
        root.create_scene(RenderSceneInfo { name: "s1".to_string() });
        root.create_window(RenderWindowInfo::default());
        root.destroy();
        assert!(!root.initialized);
        assert_eq!(root.get_scene_count(), 0);
        assert!(root.get_main_window().is_none());
    }

    #[test]
    fn test_root_window_lifecycle() {
        let mut root = Root::new();
        let id = root.create_window(RenderWindowInfo::default());
        assert!(root.get_window(id).is_some());
        assert_eq!(root.get_main_window().unwrap().id, id);
        let id2 = root.create_window(RenderWindowInfo { title: "Secondary".to_string(), ..Default::default() });
        assert!(root.get_window(id2).is_some());
        root.destroy_window(id);
        assert!(root.get_window(id).is_none());
        assert_eq!(root.get_main_window().unwrap().id, id2);
    }

    #[test]
    fn test_root_scene_lifecycle() {
        let mut root = Root::new();
        let idx = root.create_scene(RenderSceneInfo { name: "main".to_string() });
        assert_eq!(root.get_scene_count(), 1);
        assert_eq!(root.get_scene(idx).unwrap().name, "main");
        root.destroy_scene(idx);
        assert_eq!(root.get_scene_count(), 0);
    }

    #[test]
    fn test_root_frame_move() {
        let mut root = Root::new();
        root.initialize(&RootInfo::default());
        root.frame_move(1.0 / 60.0);
        assert_eq!(root.frame_count, 1);
        root.frame_move(1.0 / 60.0);
        assert_eq!(root.frame_count, 2);
    }

    #[test]
    fn test_root_resize() {
        let mut root = Root::new();
        root.create_window(RenderWindowInfo { width: 800, height: 600, ..Default::default() });
        root.resize(1280, 720);
        let win = root.get_main_window().unwrap();
        assert_eq!(win.width, 1280);
        assert_eq!(win.height, 720);
    }

    #[test]
    fn test_root_create_model() {
        let mut root = Root::new();
        let idx = root.create_scene(RenderSceneInfo { name: "s".to_string() });
        let model_id = root.create_model(idx);
        assert!(model_id.is_some());
        assert_eq!(root.get_scene(idx).unwrap().models.len(), 1);
    }

    #[test]
    fn test_root_create_lights() {
        let mut root = Root::new();
        let idx = root.create_scene(RenderSceneInfo { name: "s".to_string() });
        assert!(root.create_directional_light(idx));
        assert!(root.get_scene(idx).unwrap().main_light.is_some());
        assert!(root.create_spot_light(idx).is_some());
        assert!(root.create_point_light(idx).is_some());
        assert!(root.create_sphere_light(idx).is_some());
    }

    #[test]
    fn test_root_multiple_scenes() {
        let mut root = Root::new();
        root.create_scene(RenderSceneInfo { name: "scene1".to_string() });
        root.create_scene(RenderSceneInfo { name: "scene2".to_string() });
        assert_eq!(root.get_scene_count(), 2);
        root.destroy_scenes();
        assert_eq!(root.get_scene_count(), 0);
    }
}
