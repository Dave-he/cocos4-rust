/****************************************************************************
Rust port of Cocos Creator Application Manager
Original C++ version Copyright (c) 2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

pub mod root;

use crate::base::RefCounted;
use crate::base::RefCountedImpl;

pub trait BaseApplication: RefCounted {
    fn get_id(&self) -> u64;
    fn run(&mut self);
    fn get_title(&self) -> String;
}

pub trait ApplicationManager: RefCounted {
    fn get_instance(&self) -> Option<Box<dyn BaseApplication>>;
}

pub trait PlatformInterface: RefCounted {
    fn create_window(&mut self, id: u32, title: &str, width: i32, height: i32);
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
    pub target_fps: u32,
    pub show_fps: bool,
    pub debug_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            title: String::from("Cocos Game"),
            width: 960,
            height: 640,
            fullscreen: false,
            target_fps: 60,
            show_fps: false,
            debug_mode: false,
        }
    }
}

pub struct GameTime {
    pub total_time: f64,
    pub delta_time: f32,
    pub frame_count: u64,
    pub fps: f32,
    frame_time_acc: f32,
    fps_frame_count: u32,
}

impl GameTime {
    pub fn new() -> Self {
        GameTime {
            total_time: 0.0,
            delta_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            frame_time_acc: 0.0,
            fps_frame_count: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.delta_time = dt;
        self.total_time += dt as f64;
        self.frame_count += 1;

        self.frame_time_acc += dt;
        self.fps_frame_count += 1;
        if self.frame_time_acc >= 1.0 {
            self.fps = self.fps_frame_count as f32 / self.frame_time_acc;
            self.frame_time_acc = 0.0;
            self.fps_frame_count = 0;
        }
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Init,
    Running,
    Paused,
    Stopped,
}

pub type UpdateCallback = Box<dyn Fn(f32) + Send + Sync>;
pub type LifecycleCallback = Box<dyn Fn() + Send + Sync>;

pub struct GameLoop {
    pub config: AppConfig,
    pub time: GameTime,
    pub state: AppState,
    on_update: Vec<UpdateCallback>,
    on_start: Vec<LifecycleCallback>,
    on_pause: Vec<LifecycleCallback>,
    on_resume: Vec<LifecycleCallback>,
    on_stop: Vec<LifecycleCallback>,
}

impl GameLoop {
    pub fn new(config: AppConfig) -> Self {
        GameLoop {
            config,
            time: GameTime::new(),
            state: AppState::Init,
            on_update: Vec::new(),
            on_start: Vec::new(),
            on_pause: Vec::new(),
            on_resume: Vec::new(),
            on_stop: Vec::new(),
        }
    }

    pub fn add_on_update<F>(&mut self, callback: F)
    where
        F: Fn(f32) + Send + Sync + 'static,
    {
        self.on_update.push(Box::new(callback));
    }

    pub fn add_on_start<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_start.push(Box::new(callback));
    }

    pub fn add_on_pause<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_pause.push(Box::new(callback));
    }

    pub fn add_on_resume<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_resume.push(Box::new(callback));
    }

    pub fn add_on_stop<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_stop.push(Box::new(callback));
    }

    pub fn start(&mut self) {
        if self.state == AppState::Init || self.state == AppState::Stopped {
            self.state = AppState::Running;
            for cb in &self.on_start {
                cb();
            }
        }
    }

    pub fn pause(&mut self) {
        if self.state == AppState::Running {
            self.state = AppState::Paused;
            for cb in &self.on_pause {
                cb();
            }
        }
    }

    pub fn resume(&mut self) {
        if self.state == AppState::Paused {
            self.state = AppState::Running;
            for cb in &self.on_resume {
                cb();
            }
        }
    }

    pub fn stop(&mut self) {
        if self.state != AppState::Stopped {
            self.state = AppState::Stopped;
            for cb in &self.on_stop {
                cb();
            }
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.state != AppState::Running {
            return;
        }
        self.time.update(dt);
        for cb in &self.on_update {
            cb(dt);
        }
    }

    pub fn is_running(&self) -> bool {
        self.state == AppState::Running
    }

    pub fn is_paused(&self) -> bool {
        self.state == AppState::Paused
    }

    pub fn get_fps(&self) -> f32 {
        self.time.fps
    }

    pub fn get_total_time(&self) -> f64 {
        self.time.total_time
    }

    pub fn get_frame_count(&self) -> u64 {
        self.time.frame_count
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new(AppConfig::default())
    }
}

pub struct AppManager {
    pub game_loop: GameLoop,
    ref_count: RefCountedImpl,
}

impl std::fmt::Debug for AppManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppManager")
            .field("state", &self.game_loop.state)
            .field("frame_count", &self.game_loop.time.frame_count)
            .finish()
    }
}

impl AppManager {
    pub fn new(config: AppConfig) -> Self {
        AppManager {
            game_loop: GameLoop::new(config),
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn start(&mut self) {
        self.game_loop.start();
    }

    pub fn pause(&mut self) {
        self.game_loop.pause();
    }

    pub fn resume(&mut self) {
        self.game_loop.resume();
    }

    pub fn stop(&mut self) {
        self.game_loop.stop();
    }

    pub fn tick(&mut self, dt: f32) {
        self.game_loop.tick(dt);
    }
}

impl RefCounted for AppManager {
    fn add_ref(&self) {
        self.ref_count.add_ref();
    }

    fn release(&self) {
        self.ref_count.release();
    }

    fn get_ref_count(&self) -> u32 {
        self.ref_count.get_ref_count()
    }

    fn is_last_reference(&self) -> bool {
        self.ref_count.is_last_reference()
    }
}

impl Default for AppManager {
    fn default() -> Self {
        Self::new(AppConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.title, "Cocos Game");
        assert_eq!(config.width, 960);
        assert_eq!(config.height, 640);
        assert_eq!(config.target_fps, 60);
        assert!(!config.fullscreen);
    }

    #[test]
    fn test_game_time_update() {
        let mut time = GameTime::new();
        time.update(0.016);
        assert_eq!(time.frame_count, 1);
        assert!((time.total_time - 0.016).abs() < 1e-6);
        assert!((time.delta_time - 0.016).abs() < 1e-6);
    }

    #[test]
    fn test_game_time_fps() {
        let mut time = GameTime::new();
        for _ in 0..61 {
            time.update(1.0 / 60.0);
        }
        assert!(time.fps > 0.0);
    }

    #[test]
    fn test_game_loop_start_stop() {
        let mut game_loop = GameLoop::new(AppConfig::default());
        assert_eq!(game_loop.state, AppState::Init);
        game_loop.start();
        assert!(game_loop.is_running());
        game_loop.stop();
        assert_eq!(game_loop.state, AppState::Stopped);
    }

    #[test]
    fn test_game_loop_pause_resume() {
        let mut game_loop = GameLoop::new(AppConfig::default());
        game_loop.start();
        game_loop.pause();
        assert!(game_loop.is_paused());
        game_loop.resume();
        assert!(game_loop.is_running());
    }

    #[test]
    fn test_game_loop_tick() {
        let mut game_loop = GameLoop::new(AppConfig::default());
        game_loop.start();
        let counter = Arc::new(Mutex::new(0u32));
        let counter_clone = Arc::clone(&counter);
        game_loop.add_on_update(move |_dt| {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
        });
        game_loop.tick(0.016);
        game_loop.tick(0.016);
        assert_eq!(*counter.lock().unwrap(), 2);
    }

    #[test]
    fn test_game_loop_tick_paused_no_update() {
        let mut game_loop = GameLoop::new(AppConfig::default());
        game_loop.start();
        let counter = Arc::new(Mutex::new(0u32));
        let counter_clone = Arc::clone(&counter);
        game_loop.add_on_update(move |_dt| {
            let mut c = counter_clone.lock().unwrap();
            *c += 1;
        });
        game_loop.pause();
        game_loop.tick(0.016);
        assert_eq!(*counter.lock().unwrap(), 0);
    }

    #[test]
    fn test_game_loop_lifecycle_callbacks() {
        let mut game_loop = GameLoop::new(AppConfig::default());
        let started = Arc::new(Mutex::new(false));
        let paused = Arc::new(Mutex::new(false));
        let resumed = Arc::new(Mutex::new(false));
        let stopped = Arc::new(Mutex::new(false));

        let s = Arc::clone(&started);
        game_loop.add_on_start(move || { *s.lock().unwrap() = true; });

        let p = Arc::clone(&paused);
        game_loop.add_on_pause(move || { *p.lock().unwrap() = true; });

        let r = Arc::clone(&resumed);
        game_loop.add_on_resume(move || { *r.lock().unwrap() = true; });

        let st = Arc::clone(&stopped);
        game_loop.add_on_stop(move || { *st.lock().unwrap() = true; });

        game_loop.start();
        assert!(*started.lock().unwrap());

        game_loop.pause();
        assert!(*paused.lock().unwrap());

        game_loop.resume();
        assert!(*resumed.lock().unwrap());

        game_loop.stop();
        assert!(*stopped.lock().unwrap());
    }

    #[test]
    fn test_app_manager_ref_count() {
        let mgr = AppManager::new(AppConfig::default());
        mgr.add_ref();
        assert_eq!(mgr.get_ref_count(), 2);
        mgr.release();
        assert_eq!(mgr.get_ref_count(), 1);
        assert!(mgr.is_last_reference());
    }
}
