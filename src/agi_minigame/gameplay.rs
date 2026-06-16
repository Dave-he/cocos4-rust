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

    // -----------------------------------------------------------------
    // Round 144 — helper-level
    // tests for the
    // round-N 扩展
    // pattern
    // (round 110b /
    // 122 / 130 /
    // 142 / 143).
    // The pre-
    // round-144
    // `mod tests`
    // block (above)
    // covers the
    // basics: 11
    // GameplayType
    // variants in
    // name() /
    // from_name(),
    // builder
    // (with_score +
    // with_timestamp),
    // typed getters
    // (get_int /
    // get_str /
    // get_float +
    // missing-key +
    // wrong-type
    // None
    // returns),
    // all_types()
    // canonical
    // order, and
    // GameplayModule
    // trait default
    // `version()`.
    //
    // Round 144
    // closes the
    // remaining
    // small gaps:
    //   - GameplayState::set
    //     overwriting
    //     existing
    //     keys (a
    //     regression
    //     that used
    //     `entry().or_insert()`
    //     would
    //     silently
    //     keep the
    //     old value)
    //   - GameplayState::is_active
    //     default true
    //     (only
    //     implicitly
    //     tested via
    //     `new()`)
    //   - builder
    //     chain
    //     order
    //     independence
    //     (with_score
    //     →
    //     with_timestamp
    //     == with_timestamp
    //     →
    //     with_score)
    //   - Composite(Vec)
    //     equality +
    //     hash (the
    //     derive
    //     contract
    //     matters
    //     because
    //     Composite
    //     keys are
    //     used in
    //     HashMap
    //     lookups
    //     in
    //     SceneManager
    //     PORTAL_PALETTE)
    //   - Custom(String)
    //     round-trip
    //     preserves
    //     the raw
    //     string
    //     (covers
    //     empty +
    //     unicode +
    //     special
    //     characters)
    //   - all_types()
    //     returns no
    //     duplicates
    //     (the
    //     canonical
    //     order test
    //     pins the
    //     contents
    //     but not
    //     uniqueness)
    //   - all_types()
    //     does NOT
    //     include
    //     Composite
    //     or Custom
    //     (those are
    //     user-
    //     supplied
    //     variants;
    //     all_types()
    //     is the
    //     "named 9"
    //     set)
    //   - GameplayType
    //     Hash +
    //     Eq
    //     contract:
    //     two
    //     equal
    //     variants
    //     hash
    //     to the
    //     same
    //     value
    //     (defense
    //     against
    //     a future
    //     regression
    //     that
    //     removes
    //     the
    //     derive)
    //   - GameplayEvent
    //     variant
    //     count +
    //     payload
    //     field
    //     names
    //     (pins the
    //     WASM
    //     bridge
    //     JSON
    //     contract)
    //   - GameplayState
    //     builder
    //     returns
    //     Self
    //     (chainable)
    // -----------------------------------------------------------------

    #[test]
    fn test_gameplay_state_set_overwrites_existing_key_round_144() {
        // GameplayState::set
        // inserts into
        // the underlying
        // ValueMap. A
        // regression that
        // used
        // `entry().or_insert()`
        // would silently
        // keep the OLD
        // value when set
        // is called a 2nd
        // time with the
        // same key. Round
        // 144 pins the
        // overwrite
        // contract.
        let mut state = GameplayState::new();
        state.set("level", Value::Integer(1));
        state.set("level", Value::Integer(99));
        assert_eq!(state.get_int("level"), Some(99));
        // Also verify a
        // different
        // Value variant
        // overwrites the
        // previous one
        // (Integer → String).
        state.set("level", Value::String("hi".to_string()));
        assert_eq!(state.get_str("level"), Some("hi"));
        assert_eq!(state.get_int("level"), None);
    }

    #[test]
    fn test_gameplay_state_is_active_default_true_round_144() {
        // The
        // `is_active`
        // field is
        // set to
        // `true` in
        // `new()` and
        // not
        // exposed via
        // a builder
        // method. The
        // pre-round-
        // 144 tests
        // never
        // asserted
        // this
        // directly.
        // Round 144
        // pins the
        // default to
        // true (a
        // regression
        // that
        // changed it
        // to false
        // would break
        // the
        // GameplayState
        // consumers
        // that gate
        // on
        // `is_active`).
        let state = GameplayState::new();
        assert_eq!(state.is_active, true);
        // Default
        // impl also
        // returns
        // is_active
        // = true.
        let default_state: GameplayState = Default::default();
        assert_eq!(default_state.is_active, true);
    }

    #[test]
    fn test_gameplay_state_builder_order_independent_round_144() {
        // The
        // `with_score`
        // and
        // `with_timestamp`
        // builders are
        // both
        // chainable +
        // commutative
        // (the order
        // of the
        // chain
        // doesn't
        // matter).
        // Round 144
        // pins this
        // so a future
        // refactor
        // that makes
        // them
        // order-
        // dependent
        // fails the
        // test.
        let a = GameplayState::new()
            .with_score(500)
            .with_timestamp(1000);
        let b = GameplayState::new()
            .with_timestamp(1000)
            .with_score(500);
        assert_eq!(a.score, b.score);
        assert_eq!(a.timestamp, b.timestamp);
        assert_eq!(a.score, 500);
        assert_eq!(a.timestamp, 1000);
    }

    #[test]
    fn test_gameplay_type_composite_equality_and_hash_round_144() {
        // `Composite(Vec<GameplayType>)`
        // derives
        // PartialEq +
        // Eq + Hash.
        // Two Composites
        // are equal iff
        // their inner
        // Vecs are
        // element-wise
        // equal (Vec's
        // PartialEq is
        // element-wise +
        // order-
        // sensitive).
        // This contract
        // matters because
        // Composite is
        // used as a
        // HashMap key in
        // SceneManager
        // PORTAL_PALETTE.
        let a = GameplayType::Composite(vec![GameplayType::Match3, GameplayType::Card]);
        let b = GameplayType::Composite(vec![GameplayType::Match3, GameplayType::Card]);
        let c = GameplayType::Composite(vec![GameplayType::Card, GameplayType::Match3]); // reversed
        assert_eq!(a, b);
        assert_ne!(a, c); // Vec order matters
        // Hash contract: equal values hash to the same value.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
        // And: Composite can be used as a HashMap key (this is
        // the actual host use case).
        let mut map: std::collections::HashMap<GameplayType, &str> = std::collections::HashMap::new();
        map.insert(a.clone(), "portal_a");
        assert_eq!(map.get(&b).copied(), Some("portal_a"));
    }

    #[test]
    fn test_gameplay_type_custom_round_trip_preserves_raw_string_round_144() {
        // `Custom(String)` is the catch-all for unrecognized
        // names. The round-trip `from_name(X.name()) == X`
        // must hold for the raw string contents (no trimming,
        // no case folding, no escaping). Round 144 pins this
        // for edge-case strings: empty + unicode + special chars.
        let edge_cases = [
            "",                          // empty
            "中文类型",                   // CJK
            "with spaces inside",        // spaces
            "tab\there",                 // tab
            "new\nline",                 // newline
            "quote\"inside",             // quote
            "back\\slash",               // backslash
        ];
        for raw in edge_cases {
            let v = GameplayType::from_name(raw);
            assert_eq!(v, GameplayType::Custom(raw.to_string()));
            // name() returns the raw string verbatim.
            assert_eq!(v.name(), raw);
        }
    }

    #[test]
    fn test_all_types_returns_no_duplicates_round_144() {
        // The
        // canonical
        // 9-element
        // set
        // returned
        // by
        // `all_types()`
        // must be
        // unique
        // (no
        // duplicate
        // variant).
        // A
        // regression
        // that
        // accidentally
        // included
        // a
        // variant
        // twice
        // would
        // not be
        // caught
        // by
        // the
        // round-122
        // order
        // test
        // (which
        // only
        // asserts
        // equality
        // with a
        // fixed
        // list).
        let types = GameplayType::all_types();
        let unique: std::collections::HashSet<_> = types.iter().collect();
        assert_eq!(unique.len(), types.len());
        assert_eq!(types.len(), 9);
        // And: Composite
        // / Custom are
        // NOT in the
        // named 9 (they
        // are user-
        // supplied
        // variants).
        assert!(!types.contains(&GameplayType::Composite(vec![])));
        assert!(!types.contains(&GameplayType::Custom("foo".to_string())));
    }

    #[test]
    fn test_gameplay_type_hash_eq_contract_round_144() {
        // All
        // `GameplayType`
        // variants
        // derive
        // Hash +
        // Eq.
        // The
        // contract:
        // `a == b`
        // implies
        // `hash(a)
        // == hash(b)`.
        // Round
        // 144
        // pins
        // this
        // for
        // each
        // of
        // the
        // 9
        // named
        // variants
        // (defense
        // against
        // a future
        // regression
        // that
        // removes
        // the
        // derive
        // — e.g.
        // replacing
        // it with
        // a manual
        // impl).
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let named: Vec<GameplayType> = vec![
            GameplayType::Match3,
            GameplayType::TowerDefense,
            GameplayType::Card,
            GameplayType::TurnCombat,
            GameplayType::Parkour,
            GameplayType::Puzzle,
            GameplayType::Shooting,
            GameplayType::Synthesis,
            GameplayType::Simulation,
        ];
        for v in &named {
            let mut h = DefaultHasher::new();
            v.hash(&mut h);
            // Hash should be deterministic across calls.
            let mut h2 = DefaultHasher::new();
            v.hash(&mut h2);
            assert_eq!(h.finish(), h2.finish());
        }
    }

    #[test]
    fn test_gameplay_event_variant_count_round_144() {
        // The
        // `GameplayEvent`
        // enum has
        // 6
        // variants.
        // Round
        // 144
        // pins
        // the
        // count
        // (a
        // regression
        // that
        // added
        // or
        // removed
        // a
        // variant
        // would
        // silently
        // change
        // the
        // WASM
        // bridge
        // JSON
        // contract
        // for
        // `handle_event`).
        // (Compile-
        // time
        // check
        // via
        // a
        // match
        // that
        // covers
        // all
        // variants
        // — adding
        // a
        // new
        // variant
        // without
        // updating
        // this
        // test
        // triggers
        // a
        // non-
        // exhaustive
        // match
        // error.)
        let events = vec![
            GameplayEvent::GameStart,
            GameplayEvent::GameEnd { score: 100 },
            GameplayEvent::PlayerAction {
                action: "jump".to_string(),
                params: ValueMap::new(),
            },
            GameplayEvent::RewardEarned {
                item_id: "gold".to_string(),
                quantity: 5,
            },
            GameplayEvent::StateChanged {
                key: "level".to_string(),
            },
            GameplayEvent::Custom {
                event_type: "custom_event".to_string(),
                data: ValueMap::new(),
            },
        ];
        assert_eq!(events.len(), 6);
        // Pin the discriminant count by mapping each variant
        // to a sentinel and counting the unique sentinels.
        let mut fingerprints = std::collections::HashSet::new();
        for ev in &events {
            // Use a simple structural fingerprint
            // (no Hash derive on GameplayEvent; the enum is
            // Debug + Clone but not Hash, so we map each
            // variant to a unique &str tag).
            let tag = match ev {
                GameplayEvent::GameStart => "GameStart",
                GameplayEvent::GameEnd { .. } => "GameEnd",
                GameplayEvent::PlayerAction { .. } => "PlayerAction",
                GameplayEvent::RewardEarned { .. } => "RewardEarned",
                GameplayEvent::StateChanged { .. } => "StateChanged",
                GameplayEvent::Custom { .. } => "Custom",
            };
            fingerprints.insert(tag);
        }
        assert_eq!(fingerprints.len(), 6);
    }

    #[test]
    fn test_gameplay_state_builder_returns_self_round_144() {
        // The
        // `with_*`
        // builders
        // return
        // `Self`
        // (not
        // `&mut
        // Self`).
        // This
        // is
        // what
        // makes
        // them
        // chainable
        // in
        // builder
        // style.
        // Round
        // 144
        // pins
        // the
        // return
        // type
        // by
        // chaining
        // 3
        // calls
        // and
        // verifying
        // the
        // final
        // value
        // reflects
        // all
        // three.
        // (A
        // regression
        // that
        // returned
        // `&mut
        // Self`
        // would
        // still
        // type-
        // check
        // but
        // would
        // not
        // support
        // the
        // fluent
        // style
        // used
        // in
        // host
        // code.)
        let state = GameplayState::new()
            .with_score(42)
            .with_timestamp(1234)
            .with_score(99); // last-wins
        assert_eq!(state.score, 99);
        assert_eq!(state.timestamp, 1234);
    }
}
