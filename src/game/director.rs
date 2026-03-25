use std::sync::{Arc, Mutex};
use crate::base::scheduler::Scheduler;
use crate::core::scene_graph::{Scene, NodePtr};
use crate::tween::tween_system::TweenSystem;

fn update_node_components(node: &NodePtr, dt: f32) {
    if let Ok(mut n) = node.lock() {
        if !n.is_active() {
            return;
        }
        n.update_world_transform();
        let children: Vec<NodePtr> = n.get_children().to_vec();
        drop(n);
        for child in &children {
            update_node_components(child, dt);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DirectorEvent {
    Init,
    Reset,
    BeforeSceneLoading,
    BeforeSceneLaunch,
    AfterSceneLaunch,
    BeforeUpdate,
    AfterUpdate,
    BeforeDraw,
    AfterDraw,
    BeforeCommit,
    BeforeRender,
    AfterRender,
    BeforePhysics,
    AfterPhysics,
    BeginFrame,
    EndFrame,
}

type EventCallback = Box<dyn Fn() + Send + Sync>;

struct DirectorListeners {
    callbacks: Vec<(DirectorEvent, EventCallback)>,
}

impl DirectorListeners {
    fn new() -> Self {
        Self { callbacks: Vec::new() }
    }

    fn on(&mut self, event: DirectorEvent, cb: EventCallback) {
        self.callbacks.push((event, cb));
    }

    fn emit(&self, event: &DirectorEvent) {
        for (e, cb) in &self.callbacks {
            if e == event {
                cb();
            }
        }
    }

    fn off(&mut self, event: &DirectorEvent) {
        self.callbacks.retain(|(e, _)| e != event);
    }
}

pub struct Director {
    scheduler: Arc<Mutex<Scheduler>>,
    tween_system: TweenSystem,
    running_scene: Option<Scene>,
    paused: bool,
    delta_time: f32,
    total_frames: u64,
    listeners: DirectorListeners,
}

impl Director {
    pub fn new() -> Self {
        Director {
            scheduler: Arc::new(Mutex::new(Scheduler::new())),
            tween_system: TweenSystem::new(),
            running_scene: None,
            paused: false,
            delta_time: 0.0,
            total_frames: 0,
            listeners: DirectorListeners::new(),
        }
    }

    pub fn get_scheduler(&self) -> Arc<Mutex<Scheduler>> {
        Arc::clone(&self.scheduler)
    }

    pub fn get_running_scene(&self) -> Option<&Scene> {
        self.running_scene.as_ref()
    }

    pub fn run_scene(&mut self, scene: Scene) {
        if self.running_scene.is_some() {
            self.listeners.emit(&DirectorEvent::BeforeSceneLaunch);
        }
        self.running_scene = Some(scene);
        self.listeners.emit(&DirectorEvent::AfterSceneLaunch);
    }

    pub fn get_scene(&self) -> Option<&Scene> {
        self.running_scene.as_ref()
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn get_delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn get_total_frames(&self) -> u64 {
        self.total_frames
    }

    pub fn main_loop(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        self.delta_time = dt;
        self.total_frames += 1;

        self.listeners.emit(&DirectorEvent::BeginFrame);
        self.listeners.emit(&DirectorEvent::BeforeUpdate);

        if let Ok(mut sched) = self.scheduler.lock() {
            sched.update(dt);
        }

        self.update_scene_components(dt);
        self.tween_system.update(dt);

        self.listeners.emit(&DirectorEvent::AfterUpdate);
        self.listeners.emit(&DirectorEvent::BeforeRender);
        self.listeners.emit(&DirectorEvent::AfterRender);
        self.listeners.emit(&DirectorEvent::EndFrame);
    }

    fn update_scene_components(&mut self, dt: f32) {
        if let Some(scene) = &self.running_scene {
            if let Some(root) = scene.get_root() {
                update_node_components(root, dt);
            }
        }
    }

    pub fn get_tween_system(&mut self) -> &mut TweenSystem {
        &mut self.tween_system
    }

    pub fn on<F: Fn() + Send + Sync + 'static>(&mut self, event: DirectorEvent, cb: F) {
        self.listeners.on(event, Box::new(cb));
    }

    pub fn off(&mut self, event: &DirectorEvent) {
        self.listeners.off(event);
    }

    pub fn emit(&self, event: &DirectorEvent) {
        self.listeners.emit(event);
    }

    pub fn end(&mut self) {
        self.running_scene = None;
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.unschedule_all();
        }
    }
}

impl Default for Director {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_director_new() {
        let d = Director::new();
        assert!(!d.is_paused());
        assert_eq!(d.get_total_frames(), 0);
        assert!(d.get_running_scene().is_none());
    }

    #[test]
    fn test_director_pause_resume() {
        let mut d = Director::new();
        d.pause();
        assert!(d.is_paused());
        d.resume();
        assert!(!d.is_paused());
    }

    #[test]
    fn test_director_main_loop() {
        let mut d = Director::new();
        d.main_loop(0.016);
        assert_eq!(d.get_total_frames(), 1);
        assert!((d.get_delta_time() - 0.016).abs() < 1e-6);
    }

    #[test]
    fn test_director_main_loop_paused() {
        let mut d = Director::new();
        d.pause();
        d.main_loop(0.016);
        assert_eq!(d.get_total_frames(), 0);
    }

    #[test]
    fn test_director_run_scene() {
        let mut d = Director::new();
        let scene = Scene::new("TestScene");
        d.run_scene(scene);
        assert!(d.get_running_scene().is_some());
        assert_eq!(d.get_running_scene().unwrap().name, "TestScene");
    }

    #[test]
    fn test_director_event_callback() {
        let mut d = Director::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        d.on(DirectorEvent::BeginFrame, move || {
            *c.lock().unwrap() += 1;
        });
        d.main_loop(0.016);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_director_off_event() {
        let mut d = Director::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        d.on(DirectorEvent::BeginFrame, move || {
            *c.lock().unwrap() += 1;
        });
        d.off(&DirectorEvent::BeginFrame);
        d.main_loop(0.016);
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn test_director_end() {
        let mut d = Director::new();
        let scene = Scene::new("S");
        d.run_scene(scene);
        d.end();
        assert!(d.get_running_scene().is_none());
    }

    #[test]
    fn test_director_multi_frames() {
        let mut d = Director::new();
        for _ in 0..10 {
            d.main_loop(0.016);
        }
        assert_eq!(d.get_total_frames(), 10);
    }
}
