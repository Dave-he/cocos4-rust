use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::core::scene_graph::Scene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneState {
    Unloaded,
    Loading,
    Loaded,
    Active,
}

pub struct SceneEntry {
    pub name: String,
    pub state: SceneState,
    scene: Option<Scene>,
}

impl SceneEntry {
    fn new(name: &str) -> Self {
        SceneEntry {
            name: name.to_string(),
            state: SceneState::Unloaded,
            scene: None,
        }
    }
}

pub type SceneFactory = Box<dyn Fn() -> Scene + Send + Sync>;

pub struct SceneManager {
    entries: HashMap<String, SceneEntry>,
    factories: HashMap<String, SceneFactory>,
    active_scene: Option<String>,
    previous_scene: Option<String>,
    loading_scene: Option<String>,
    history: Vec<String>,
}

impl SceneManager {
    pub fn new() -> Self {
        SceneManager {
            entries: HashMap::new(),
            factories: HashMap::new(),
            active_scene: None,
            previous_scene: None,
            loading_scene: None,
            history: Vec::new(),
        }
    }

    pub fn register<F: Fn() -> Scene + Send + Sync + 'static>(&mut self, name: &str, factory: F) {
        self.factories.insert(name.to_string(), Box::new(factory));
        self.entries.insert(name.to_string(), SceneEntry::new(name));
    }

    pub fn preload(&mut self, name: &str) -> bool {
        if !self.factories.contains_key(name) {
            return false;
        }
        if let Some(entry) = self.entries.get_mut(name) {
            if entry.state == SceneState::Unloaded {
                entry.state = SceneState::Loading;
                if let Some(factory) = self.factories.get(name) {
                    let scene = factory();
                    if let Some(e) = self.entries.get_mut(name) {
                        e.scene = Some(scene);
                        e.state = SceneState::Loaded;
                    }
                }
            }
            return true;
        }
        false
    }

    pub fn load(&mut self, name: &str) -> bool {
        if !self.preload(name) {
            return false;
        }
        self.loading_scene = Some(name.to_string());
        true
    }

    pub fn run(&mut self, name: &str) -> Option<Scene> {
        if !self.entries.contains_key(name) {
            self.load(name);
        }
        if let Some(entry) = self.entries.get_mut(name) {
            if entry.state == SceneState::Unloaded {
                if let Some(factory) = self.factories.get(name) {
                    entry.scene = Some(factory());
                    entry.state = SceneState::Loaded;
                }
            }
            if entry.state == SceneState::Loaded || entry.state == SceneState::Loading {
                entry.state = SceneState::Active;
                let scene = entry.scene.take();
                if let Some(prev) = self.active_scene.take() {
                    if let Some(prev_entry) = self.entries.get_mut(&prev) {
                        if prev_entry.state == SceneState::Active {
                            prev_entry.state = SceneState::Unloaded;
                        }
                    }
                    self.previous_scene = Some(prev.clone());
                    self.history.push(prev);
                }
                self.active_scene = Some(name.to_string());
                self.loading_scene = None;
                return scene;
            }
        }
        None
    }

    pub fn run_additive(&mut self, name: &str) -> Option<Scene> {
        if !self.entries.contains_key(name) {
            return None;
        }
        if let Some(entry) = self.entries.get_mut(name) {
            if entry.state == SceneState::Unloaded {
                if let Some(factory) = self.factories.get(name) {
                    entry.scene = Some(factory());
                    entry.state = SceneState::Active;
                    return entry.scene.take();
                }
            }
        }
        None
    }

    pub fn unload(&mut self, name: &str) {
        if let Some(entry) = self.entries.get_mut(name) {
            entry.scene = None;
            entry.state = SceneState::Unloaded;
        }
        if self.active_scene.as_deref() == Some(name) {
            self.active_scene = None;
        }
    }

    pub fn get_active_scene_name(&self) -> Option<&str> {
        self.active_scene.as_deref()
    }

    pub fn get_previous_scene_name(&self) -> Option<&str> {
        self.previous_scene.as_deref()
    }

    pub fn get_loading_scene_name(&self) -> Option<&str> {
        self.loading_scene.as_deref()
    }

    pub fn get_scene_state(&self, name: &str) -> Option<SceneState> {
        self.entries.get(name).map(|e| e.state)
    }

    pub fn is_scene_loaded(&self, name: &str) -> bool {
        matches!(
            self.entries.get(name).map(|e| e.state),
            Some(SceneState::Loaded) | Some(SceneState::Active)
        )
    }

    pub fn get_registered_scenes(&self) -> Vec<&str> {
        self.entries.keys().map(|s| s.as_str()).collect()
    }

    pub fn go_back(&mut self) -> Option<String> {
        if self.history.is_empty() {
            return None;
        }
        let prev = self.history.pop()?;
        self.load(&prev.clone());
        Some(prev)
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scene(name: &str) -> Scene {
        Scene::new(name)
    }

    #[test]
    fn test_scene_manager_new() {
        let sm = SceneManager::new();
        assert!(sm.get_active_scene_name().is_none());
        assert!(sm.get_registered_scenes().is_empty());
    }

    #[test]
    fn test_scene_manager_register() {
        let mut sm = SceneManager::new();
        sm.register("Main", || make_scene("Main"));
        assert!(sm.get_registered_scenes().contains(&"Main"));
        assert_eq!(sm.get_scene_state("Main"), Some(SceneState::Unloaded));
    }

    #[test]
    fn test_scene_manager_preload() {
        let mut sm = SceneManager::new();
        sm.register("Level1", || make_scene("Level1"));
        let ok = sm.preload("Level1");
        assert!(ok);
        assert_eq!(sm.get_scene_state("Level1"), Some(SceneState::Loaded));
        assert!(sm.is_scene_loaded("Level1"));
    }

    #[test]
    fn test_scene_manager_preload_unknown() {
        let mut sm = SceneManager::new();
        let ok = sm.preload("Unknown");
        assert!(!ok);
    }

    #[test]
    fn test_scene_manager_run() {
        let mut sm = SceneManager::new();
        sm.register("Menu", || make_scene("Menu"));
        let scene = sm.run("Menu");
        assert!(scene.is_some());
        assert_eq!(scene.unwrap().name, "Menu");
        assert_eq!(sm.get_active_scene_name(), Some("Menu"));
        assert_eq!(sm.get_scene_state("Menu"), Some(SceneState::Active));
    }

    #[test]
    fn test_scene_manager_run_unregistered_auto_loads() {
        let mut sm = SceneManager::new();
        sm.register("Game", || make_scene("Game"));
        let scene = sm.run("Game");
        assert!(scene.is_some());
    }

    #[test]
    fn test_scene_manager_run_unknown() {
        let mut sm = SceneManager::new();
        let scene = sm.run("Unknown");
        assert!(scene.is_none());
    }

    #[test]
    fn test_scene_manager_switch_scenes() {
        let mut sm = SceneManager::new();
        sm.register("Menu", || make_scene("Menu"));
        sm.register("Game", || make_scene("Game"));
        sm.run("Menu");
        sm.run("Game");
        assert_eq!(sm.get_active_scene_name(), Some("Game"));
        assert_eq!(sm.get_previous_scene_name(), Some("Menu"));
        assert_eq!(sm.get_history(), &["Menu".to_string()]);
    }

    #[test]
    fn test_scene_manager_unload() {
        let mut sm = SceneManager::new();
        sm.register("Level", || make_scene("Level"));
        sm.run("Level");
        sm.unload("Level");
        assert_eq!(sm.get_scene_state("Level"), Some(SceneState::Unloaded));
        assert!(sm.get_active_scene_name().is_none());
    }

    #[test]
    fn test_scene_manager_go_back() {
        let mut sm = SceneManager::new();
        sm.register("Menu", || make_scene("Menu"));
        sm.register("Game", || make_scene("Game"));
        sm.run("Menu");
        sm.run("Game");
        let back = sm.go_back();
        assert_eq!(back, Some("Menu".to_string()));
        assert!(sm.get_history().is_empty());
    }

    #[test]
    fn test_scene_manager_go_back_empty_history() {
        let mut sm = SceneManager::new();
        let back = sm.go_back();
        assert!(back.is_none());
    }

    #[test]
    fn test_scene_manager_clear_history() {
        let mut sm = SceneManager::new();
        sm.register("A", || make_scene("A"));
        sm.register("B", || make_scene("B"));
        sm.register("C", || make_scene("C"));
        sm.run("A");
        sm.run("B");
        sm.run("C");
        assert_eq!(sm.get_history().len(), 2);
        sm.clear_history();
        assert!(sm.get_history().is_empty());
    }

    #[test]
    fn test_scene_manager_load_sets_loading() {
        let mut sm = SceneManager::new();
        sm.register("Level2", || make_scene("Level2"));
        sm.load("Level2");
        assert_eq!(sm.get_loading_scene_name(), Some("Level2"));
    }

    #[test]
    fn test_scene_manager_preload_twice_no_duplicate() {
        let mut sm = SceneManager::new();
        sm.register("Lobby", || make_scene("Lobby"));
        sm.preload("Lobby");
        sm.preload("Lobby");
        assert_eq!(sm.get_scene_state("Lobby"), Some(SceneState::Loaded));
    }
}
