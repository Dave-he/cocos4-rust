use std::collections::HashMap;

use crate::base::value::{Value, ValueMap};

use super::world_state::UnifiedWorldState;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplayType {
    Match3,
    TowerDefense,
    Card,
    TurnCombat,
    Parkour,
    Puzzle,
    Shooting,
    Synthesis,
    Simulation,
    Composite(Vec<GameplayType>),
    Custom(String),
}

impl GameplayType {
    pub fn name(&self) -> &str {
        match self {
            GameplayType::Match3 => "match3",
            GameplayType::TowerDefense => "tower_defense",
            GameplayType::Card => "card",
            GameplayType::TurnCombat => "turn_combat",
            GameplayType::Parkour => "parkour",
            GameplayType::Puzzle => "puzzle",
            GameplayType::Shooting => "shooting",
            GameplayType::Synthesis => "synthesis",
            GameplayType::Simulation => "simulation",
            GameplayType::Composite(types) => "composite",
            GameplayType::Custom(name) => name,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "match3" => GameplayType::Match3,
            "tower_defense" => GameplayType::TowerDefense,
            "card" => GameplayType::Card,
            "turn_combat" => GameplayType::TurnCombat,
            "parkour" => GameplayType::Parkour,
            "puzzle" => GameplayType::Puzzle,
            "shooting" => GameplayType::Shooting,
            "synthesis" => GameplayType::Synthesis,
            "simulation" => GameplayType::Simulation,
            other => GameplayType::Custom(other.to_string()),
        }
    }

    pub fn all_types() -> Vec<GameplayType> {
        vec![
            GameplayType::Match3,
            GameplayType::TowerDefense,
            GameplayType::Card,
            GameplayType::TurnCombat,
            GameplayType::Parkour,
            GameplayType::Puzzle,
            GameplayType::Shooting,
            GameplayType::Synthesis,
            GameplayType::Simulation,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct GameplayState {
    pub data: ValueMap,
    pub timestamp: u64,
    pub score: u64,
    pub is_active: bool,
}

impl GameplayState {
    pub fn new() -> Self {
        Self {
            data: ValueMap::new(),
            timestamp: 0,
            score: 0,
            is_active: true,
        }
    }

    pub fn with_score(mut self, score: u64) -> Self {
        self.score = score;
        self
    }

    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.data.get(key) {
            Some(Value::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i32> {
        match self.data.get(key) {
            Some(Value::Integer(n)) => Some(*n),
            _ => None,
        }
    }

    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.data.get(key) {
            Some(Value::Float(n)) => Some(*n),
            _ => None,
        }
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_string(), value);
    }
}

impl Default for GameplayState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum GameplayEvent {
    GameStart,
    GameEnd { score: u64 },
    PlayerAction { action: String, params: ValueMap },
    RewardEarned { item_id: String, quantity: u32 },
    StateChanged { key: String },
    Custom { event_type: String, data: ValueMap },
}

pub trait GameplayModule: Send + Sync {
    fn module_type(&self) -> GameplayType;
    fn name(&self) -> &str;
    fn version(&self) -> u32 {
        1
    }

    fn on_init(&mut self, world_state: &mut UnifiedWorldState);
    fn on_enter(&mut self, world_state: &mut UnifiedWorldState);
    fn on_update(&mut self, dt: f32, world_state: &mut UnifiedWorldState);
    fn on_exit(&mut self, world_state: &mut UnifiedWorldState);
    fn on_destroy(&mut self);

    fn save_state(&self) -> GameplayState;
    fn load_state(&mut self, state: GameplayState);

    fn handle_event(&mut self, event: GameplayEvent, world_state: &mut UnifiedWorldState);
    fn handle_input(&mut self, input: &str, world_state: &mut UnifiedWorldState);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gameplay_type_names() {
        assert_eq!(GameplayType::Match3.name(), "match3");
        assert_eq!(GameplayType::TowerDefense.name(), "tower_defense");
        assert_eq!(GameplayType::Custom("custom1".to_string()).name(), "custom1");
    }

    #[test]
    fn test_gameplay_type_from_name() {
        assert_eq!(GameplayType::from_name("match3"), GameplayType::Match3);
        assert_eq!(
            GameplayType::from_name("my_game"),
            GameplayType::Custom("my_game".to_string())
        );
    }

    #[test]
    fn test_gameplay_state() {
        let state = GameplayState::new()
            .with_score(500)
            .with_timestamp(1000);
        assert_eq!(state.score, 500);
        assert_eq!(state.timestamp, 1000);
    }

    #[test]
    fn test_gameplay_state_data() {
        let mut state = GameplayState::new();
        state.set("level", Value::Integer(5));
        state.set("name", Value::String("test".to_string()));
        assert_eq!(state.get_int("level"), Some(5));
        assert_eq!(state.get_str("name"), Some("test"));
    }

    #[test]
    fn test_all_types() {
        let types = GameplayType::all_types();
        assert_eq!(types.len(), 9);
    }
}
