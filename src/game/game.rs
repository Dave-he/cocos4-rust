use std::sync::{Arc, Mutex};
use crate::game::director::Director;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    Hide,
    Show,
    LowMemory,
    GameInited,
    EngineInited,
    RendererInited,
    Restart,
    Pause,
    Resume,
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub frame_rate: u32,
    pub show_fps: bool,
    pub debug_mode: u32,
    pub render_mode: u8,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            frame_rate: 60,
            show_fps: false,
            debug_mode: 0,
            render_mode: 0,
        }
    }
}

type GameEventCallback = Box<dyn Fn(&GameEvent) + Send + Sync>;

pub struct Game {
    config: GameConfig,
    director: Arc<Mutex<Director>>,
    inited: bool,
    paused: bool,
    listeners: Vec<(GameEvent, GameEventCallback)>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            config: GameConfig::default(),
            director: Arc::new(Mutex::new(Director::new())),
            inited: false,
            paused: false,
            listeners: Vec::new(),
        }
    }

    pub fn with_config(config: GameConfig) -> Self {
        Game {
            config,
            director: Arc::new(Mutex::new(Director::new())),
            inited: false,
            paused: false,
            listeners: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        if self.inited {
            return;
        }
        self.inited = true;
        self.emit_event(&GameEvent::EngineInited);
        self.emit_event(&GameEvent::GameInited);
    }

    pub fn is_inited(&self) -> bool {
        self.inited
    }

    pub fn get_director(&self) -> Arc<Mutex<Director>> {
        Arc::clone(&self.director)
    }

    pub fn get_config(&self) -> &GameConfig {
        &self.config
    }

    pub fn set_frame_rate(&mut self, rate: u32) {
        self.config.frame_rate = rate;
    }

    pub fn get_frame_rate(&self) -> u32 {
        self.config.frame_rate
    }

    pub fn pause(&mut self) {
        if !self.paused {
            self.paused = true;
            if let Ok(mut d) = self.director.lock() {
                d.pause();
            }
            self.emit_event(&GameEvent::Pause);
        }
    }

    pub fn resume(&mut self) {
        if self.paused {
            self.paused = false;
            if let Ok(mut d) = self.director.lock() {
                d.resume();
            }
            self.emit_event(&GameEvent::Resume);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn step(&mut self, dt: f32) {
        if !self.inited {
            return;
        }
        if let Ok(mut d) = self.director.lock() {
            d.main_loop(dt);
        }
    }

    pub fn restart(&mut self) {
        self.inited = false;
        self.emit_event(&GameEvent::Restart);
        self.init();
    }

    pub fn on<F: Fn(&GameEvent) + Send + Sync + 'static>(&mut self, event: GameEvent, cb: F) {
        self.listeners.push((event, Box::new(cb)));
    }

    pub fn off(&mut self, event: &GameEvent) {
        self.listeners.retain(|(e, _)| e != event);
    }

    fn emit_event(&self, event: &GameEvent) {
        for (e, cb) in &self.listeners {
            if e == event {
                cb(event);
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_new() {
        let g = Game::new();
        assert!(!g.is_inited());
        assert!(!g.is_paused());
        assert_eq!(g.get_frame_rate(), 60);
    }

    #[test]
    fn test_game_init() {
        let mut g = Game::new();
        g.init();
        assert!(g.is_inited());
    }

    #[test]
    fn test_game_init_twice() {
        let mut g = Game::new();
        g.init();
        g.init();
        assert!(g.is_inited());
    }

    #[test]
    fn test_game_pause_resume() {
        let mut g = Game::new();
        g.init();
        g.pause();
        assert!(g.is_paused());
        g.resume();
        assert!(!g.is_paused());
    }

    #[test]
    fn test_game_step() {
        let mut g = Game::new();
        g.init();
        g.step(0.016);
        let d = g.get_director();
        assert_eq!(d.lock().unwrap().get_total_frames(), 1);
    }

    #[test]
    fn test_game_step_before_init() {
        let mut g = Game::new();
        g.step(0.016);
        let d = g.get_director();
        assert_eq!(d.lock().unwrap().get_total_frames(), 0);
    }

    #[test]
    fn test_game_event_callback() {
        let mut g = Game::new();
        let fired = Arc::new(Mutex::new(false));
        let f = Arc::clone(&fired);
        g.on(GameEvent::GameInited, move |_| {
            *f.lock().unwrap() = true;
        });
        g.init();
        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn test_game_off_event() {
        let mut g = Game::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        g.on(GameEvent::GameInited, move |_| {
            *c.lock().unwrap() += 1;
        });
        g.off(&GameEvent::GameInited);
        g.init();
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn test_game_set_frame_rate() {
        let mut g = Game::new();
        g.set_frame_rate(30);
        assert_eq!(g.get_frame_rate(), 30);
    }

    #[test]
    fn test_game_restart() {
        let mut g = Game::new();
        g.init();
        g.step(0.016);
        g.restart();
        assert!(g.is_inited());
    }

    #[test]
    fn test_game_config_default() {
        let cfg = GameConfig::default();
        assert_eq!(cfg.frame_rate, 60);
        assert!(!cfg.show_fps);
    }

    #[test]
    fn test_game_with_config() {
        let cfg = GameConfig {
            frame_rate: 30,
            show_fps: true,
            debug_mode: 1,
            render_mode: 2,
        };
        let g = Game::with_config(cfg);
        assert_eq!(g.get_frame_rate(), 30);
        assert!(g.get_config().show_fps);
    }
}
