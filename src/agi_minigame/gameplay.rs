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

    // -----------------------------------------------------------------
    // Round 122 — helper-level
    // tests for the 8 named
    // `GameplayType` variants
    // (round 110b pattern
    // extended). The pre-
    // round-122 tests covered
    // 3 of 11 variants in
    // `GameplayType::name()`
    // + 1 named variant in
    // `GameplayType::from_name()`
    // + the count + builder +
    // get_int/get_str helpers.
    // Round 122 closes the
    // coverage gap for:
    //   - 8 named variants
    //     (Card / TurnCombat /
    //     Parkour / Puzzle /
    //     Shooting / Synthesis
    //     / Simulation /
    //     Composite) in name()
    //   - 8 named variants in
    //     from_name() — the
    //     round-110b round-trip
    //     contract
    //     (from_name(X.name())
    //     == X) for each named
    //     variant
    //   - all_types() order
    //     (the 9 named
    //     variants in the
    //     canonical order
    //     used by SceneManager
    //     PORTAL_PALETTE)
    //   - get_float() (the
    //     missing typed getter)
    //   - missing-key None
    //     returns for get_int
    //     / get_str / get_float
    //   - GameplayModule trait
    //     default `version()`
    //     returns 1
    // -----------------------------------------------------------------

    #[test]
    fn test_gameplay_type_name_all_8_named_variants_round_122() {
        // Round 110b covered
        // Match3 / TowerDefense
        // + Custom fallback;
        // round 122 closes the
        // gap for the
        // remaining 8 named
        // variants (Card /
        // TurnCombat / Parkour
        // / Puzzle / Shooting
        // / Synthesis /
        // Simulation /
        // Composite).
        assert_eq!(GameplayType::Card.name(),         "card");
        assert_eq!(GameplayType::TurnCombat.name(),   "turn_combat");
        assert_eq!(GameplayType::Parkour.name(),      "parkour");
        assert_eq!(GameplayType::Puzzle.name(),       "puzzle");
        assert_eq!(GameplayType::Shooting.name(),     "shooting");
        assert_eq!(GameplayType::Synthesis.name(),    "synthesis");
        assert_eq!(GameplayType::Simulation.name(),   "simulation");
        // Composite variant:
        // name() returns the
        // constant "composite"
        // regardless of the
        // inner vec. (Defense:
        // a regression that
        // returned a serialized
        // form of the inner
        // vec would silently
        // change the WASM
        // bridge JSON contract.)
        assert_eq!(
            GameplayType::Composite(vec![GameplayType::Match3, GameplayType::Card]).name(),
            "composite"
        );
    }

    #[test]
    fn test_gameplay_type_from_name_all_9_named_variants_round_122() {
        // Round-trip contract
        // for the 9 named
        // variants (round 110b
        // pattern extended):
        // `from_name(X.name())
        // == X` for each named
        // variant. Defense: a
        // typo in either name()
        // or from_name() would
        // silently desync the
        // WASM bridge.
        let named: &[(GameplayType, &str)] = &[
            (GameplayType::Match3,       "match3"),
            (GameplayType::TowerDefense, "tower_defense"),
            (GameplayType::Card,         "card"),
            (GameplayType::TurnCombat,   "turn_combat"),
            (GameplayType::Parkour,      "parkour"),
            (GameplayType::Puzzle,       "puzzle"),
            (GameplayType::Shooting,     "shooting"),
            (GameplayType::Synthesis,    "synthesis"),
            (GameplayType::Simulation,   "simulation"),
        ];
        for (variant, expected_name) in named {
            assert_eq!(&variant.name(), expected_name);
            assert_eq!(&GameplayType::from_name(expected_name), variant);
        }
    }

    #[test]
    fn test_gameplay_type_from_name_custom_fallback_round_122() {
        // Round 110b covered
        // "my_game" → Custom;
        // round 122 pins the
        // fallback for an
        // empty string + a
        // unicode string + a
        // string with spaces
        // (all should become
        // Custom variants with
        // the raw string
        // preserved).
        assert_eq!(
            GameplayType::from_name(""),
            GameplayType::Custom(String::new())
        );
        assert_eq!(
            GameplayType::from_name("中文类型"),
            GameplayType::Custom("中文类型".to_string())
        );
        assert_eq!(
            GameplayType::from_name("my game"),
            GameplayType::Custom("my game".to_string())
        );
    }

    #[test]
    fn test_all_types_in_canonical_order_round_122() {
        // The 9 named
        // GameplayType
        // variants in the
        // canonical display
        // order (matches the
        // PORTAL_ATOMS QWERTY
        // ordering 1..8 in the
        // TS KeyboardShortcuts
        // + SceneManager
        // PORTAL_PALETTE).
        // Simulation was added
        // after the original
        // 8 and is the 9th
        // entry.
        let types = GameplayType::all_types();
        assert_eq!(
            types,
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
        );
    }

    #[test]
    fn test_gameplay_state_get_float_round_122() {
        // The pre-round-122
        // test_gameplay_state_data
        // covered get_int +
        // get_str but missed
        // get_float (the 3rd
        // typed getter on
        // GameplayState).
        // Round 122 closes the
        // gap.
        let mut state = GameplayState::new();
        state.set("multiplier", Value::Float(1.5));
        state.set("score",      Value::Float(0.0));
        assert_eq!(state.get_float("multiplier"), Some(1.5));
        assert_eq!(state.get_float("score"),      Some(0.0));
    }

    #[test]
    fn test_gameplay_state_get_returns_none_for_missing_keys_round_122() {
        // Round 110b pattern:
        // typed getters must
        // return None for a
        // missing key (not
        // crash). The pre-
        // round-122 tests
        // didn't cover the
        // missing-key path.
        let state = GameplayState::new();
        assert_eq!(state.get_int("missing"),   None);
        assert_eq!(state.get_str("missing"),   None);
        assert_eq!(state.get_float("missing"), None);
    }

    #[test]
    fn test_gameplay_state_get_returns_none_for_wrong_value_type_round_122() {
        // Typed getters must
        // also return None
        // when the key exists
        // but holds a
        // different Value
        // variant. Defense:
        // a regression that
        // used
        // `unwrap_or_default`
        // would silently
        // return 0 for an
        // Integer-keyed-by-
        // Float-key access
        // pattern.
        let mut state = GameplayState::new();
        state.set("name", Value::String("hello".to_string()));
        // get_int("name") →
        // None (key exists,
        // wrong type).
        assert_eq!(state.get_int("name"), None);
        // get_float("name") →
        // None (key exists,
        // wrong type).
        assert_eq!(state.get_float("name"), None);
    }

    #[test]
    fn test_gameplay_module_trait_default_version_round_122() {
        // The GameplayModule
        // trait declares
        // `fn version(&self) ->
        // u32 { 1 }` — the
        // default impl returns
        // 1. A regression that
        // removed the default
        // would break every
        // implementor. Round
        // 122 pins the default
        // via a test
        // GameplayModule
        // struct.
        struct Stub;
        impl GameplayModule for Stub {
            fn module_type(&self) -> GameplayType { GameplayType::Match3 }
            fn name(&self) -> &str { "stub" }
            fn on_init(&mut self, _ws: &mut UnifiedWorldState) {}
            fn on_enter(&mut self, _ws: &mut UnifiedWorldState) {}
            fn on_update(&mut self, _dt: f32, _ws: &mut UnifiedWorldState) {}
            fn on_exit(&mut self, _ws: &mut UnifiedWorldState) {}
            fn on_destroy(&mut self) {}
            fn save_state(&self) -> GameplayState { GameplayState::new() }
            fn load_state(&mut self, _state: GameplayState) {}
            fn handle_event(&mut self, _e: GameplayEvent, _ws: &mut UnifiedWorldState) {}
            fn handle_input(&mut self, _i: &str, _ws: &mut UnifiedWorldState) {}
        }
        let stub = Stub;
        assert_eq!(stub.version(), 1);
        assert_eq!(stub.name(), "stub");
        assert_eq!(stub.module_type(), GameplayType::Match3);
    }
}
