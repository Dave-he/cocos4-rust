use std::collections::HashMap;
use std::time::SystemTime;

use crate::base::value::{Value, ValueMap};

use super::economy::{Currency, CurrencyType, Inventory, Wallet};
use super::player::{PlayerProfile, PlayerProgression};
use super::gameplay::{GameplayType, GameplayState};

#[derive(Debug)]
pub struct UnifiedWorldState {
    pub player: PlayerProfile,
    pub progression: PlayerProgression,
    pub wallet: Wallet,
    pub inventory: Inventory,
    pub active_gameplay: Option<ActiveGameplayInfo>,
    pub gameplay_history: Vec<GameplayRecord>,
    pub shared_world: SharedWorld,
    pub global_data: ValueMap,
}

#[derive(Debug)]
pub struct ActiveGameplayInfo {
    pub gameplay_type: GameplayType,
    pub session_start: SystemTime,
    pub current_state: GameplayState,
}

#[derive(Debug, Clone)]
pub struct GameplayRecord {
    pub gameplay_type: GameplayType,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub score: u64,
    pub rewards_earned: Vec<RewardInfo>,
}

#[derive(Debug, Clone)]
pub struct RewardInfo {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Debug)]
pub struct SharedWorld {
    pub world_events: Vec<WorldEvent>,
    pub global_announcements: Vec<Announcement>,
    pub season_info: Option<SeasonInfo>,
    pub world_variables: ValueMap,
}

#[derive(Debug, Clone)]
pub struct WorldEvent {
    pub event_id: String,
    pub name: String,
    pub description: String,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub is_active: bool,
    pub modifiers: ValueMap,
}

#[derive(Debug, Clone)]
pub struct Announcement {
    pub id: String,
    pub title: String,
    pub content: String,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone)]
pub struct SeasonInfo {
    pub season_id: String,
    pub name: String,
    pub start_date: SystemTime,
    pub end_date: SystemTime,
    pub theme: String,
    pub bonus_multiplier: f32,
}

impl UnifiedWorldState {
    pub fn new(player: PlayerProfile) -> Self {
        Self {
            player,
            progression: PlayerProgression::new(),
            wallet: Wallet::new(),
            inventory: Inventory::new(100),
            active_gameplay: None,
            gameplay_history: Vec::new(),
            shared_world: SharedWorld::new(),
            global_data: ValueMap::new(),
        }
    }

    pub fn set_active_gameplay(&mut self, gameplay_type: GameplayType, state: GameplayState) {
        self.active_gameplay = Some(ActiveGameplayInfo {
            gameplay_type,
            session_start: SystemTime::now(),
            current_state: state,
        });
        self.progression.record_dimension_visit("current");
    }

    pub fn clear_active_gameplay(&mut self) -> Option<GameplayState> {
        self.active_gameplay.take().map(|info| info.current_state)
    }

    pub fn record_gameplay(&mut self, record: GameplayRecord) {
        self.progression.record_dimension_complete(record.score);

        for reward in &record.rewards_earned {
            if reward.item_id == "gold" {
                self.wallet.currency.add(CurrencyType::Gold, reward.quantity as u64);
            } else if reward.item_id == "gem" {
                self.wallet.currency.add(CurrencyType::Gem, reward.quantity as u64);
            } else {
                use super::economy::InventoryItem;
                let item = InventoryItem::new(&reward.item_id, &reward.item_id)
                    .with_quantity(reward.quantity);
                self.inventory.add_item(item);
            }
        }

        self.gameplay_history.push(record);
    }

    pub fn get_player_stats(&self) -> PlayerStats {
        PlayerStats {
            level: self.player.level,
            experience: self.player.experience,
            total_playtime: self.calculate_total_playtime(),
            gameplay_count: self.gameplay_history.len(),
            gold: self.wallet.get_balance(CurrencyType::Gold),
            gem: self.wallet.get_balance(CurrencyType::Gem),
        }
    }

    fn calculate_total_playtime(&self) -> u64 {
        self.gameplay_history
            .iter()
            .map(|r| {
                r.end_time
                    .duration_since(r.start_time)
                    .map(|d| d.as_secs())
                    .unwrap_or(0)
            })
            .sum()
    }

    pub fn get_global(&self, key: &str) -> Option<&Value> {
        self.global_data.get(key)
    }

    pub fn set_global(&mut self, key: &str, value: Value) {
        self.global_data.insert(key.to_string(), value);
    }
}

impl SharedWorld {
    pub fn new() -> Self {
        Self {
            world_events: Vec::new(),
            global_announcements: Vec::new(),
            season_info: None,
            world_variables: ValueMap::new(),
        }
    }

    pub fn add_event(&mut self, event: WorldEvent) {
        self.world_events.push(event);
    }

    pub fn get_active_events(&self) -> Vec<&WorldEvent> {
        self.world_events
            .iter()
            .filter(|e| e.is_active)
            .collect()
    }

    pub fn remove_event(&mut self, event_id: &str) {
        self.world_events.retain(|e| e.event_id != event_id);
    }

    pub fn set_variable(&mut self, key: &str, value: Value) {
        self.world_variables.insert(key.to_string(), value);
    }

    pub fn get_variable(&self, key: &str) -> Option<&Value> {
        self.world_variables.get(key)
    }

    pub fn add_announcement(&mut self, announcement: Announcement) {
        self.global_announcements.push(announcement);
    }
}

impl Default for SharedWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub level: u32,
    pub experience: u64,
    pub total_playtime: u64,
    pub gameplay_count: usize,
    pub gold: u64,
    pub gem: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_world_state_new() {
        let ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        assert_eq!(ws.player.account.account_id, "p1");
        assert!(ws.active_gameplay.is_none());
        assert!(ws.gameplay_history.is_empty());
    }

    #[test]
    fn test_set_clear_active_gameplay() {
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        ws.set_active_gameplay(GameplayType::Match3, GameplayState::new());
        assert!(ws.active_gameplay.is_some());

        let state = ws.clear_active_gameplay();
        assert!(state.is_some());
        assert!(ws.active_gameplay.is_none());
    }

    #[test]
    fn test_record_gameplay_with_rewards() {
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        let now = SystemTime::now();
        let record = GameplayRecord {
            gameplay_type: GameplayType::Match3,
            start_time: now,
            end_time: now,
            score: 500,
            rewards_earned: vec![
                RewardInfo { item_id: "gold".to_string(), quantity: 100 },
                RewardInfo { item_id: "gem".to_string(), quantity: 5 },
            ],
        };
        ws.record_gameplay(record);
        assert_eq!(ws.progression.total_score, 500);
        assert_eq!(ws.wallet.get_balance(CurrencyType::Gold), 100);
        assert_eq!(ws.wallet.get_balance(CurrencyType::Gem), 5);
    }

    #[test]
    fn test_shared_world() {
        let mut sw = SharedWorld::new();
        sw.add_event(WorldEvent {
            event_id: "e1".to_string(),
            name: "Festival".to_string(),
            description: "Test event".to_string(),
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            is_active: true,
            modifiers: ValueMap::new(),
        });
        assert_eq!(sw.get_active_events().len(), 1);
        sw.remove_event("e1");
        assert_eq!(sw.get_active_events().len(), 0);
    }

    #[test]
    fn test_global_data() {
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        ws.set_global("difficulty", Value::Float(1.5));
        assert!(ws.get_global("difficulty").is_some());
    }

    #[test]
    fn test_player_stats() {
        let ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        let stats = ws.get_player_stats();
        assert_eq!(stats.level, 1);
        assert_eq!(stats.gameplay_count, 0);
    }

    // -----------------------------------------------------------------
    // Round 124 — helper-level
    // tests for the
    // `record_gameplay` else
    // branch (non-currency
    // rewards) + the
    // `calculate_total_playtime`
    // private helper +
    // SharedWorld edge cases
    // (round 110b / round 122 /
    // round 123 pattern
    // extended). The pre-
    // round-124 tests covered:
    //   - new() + active
    //     gameplay set/clear
    //   - record_gameplay
    //     happy path (gold +
    //     gem rewards)
    //   - SharedWorld
    //     add_event +
    //     remove_event +
    //     get_active_events
    //     happy path
    //   - global_data
    //     set + get
    //   - get_player_stats
    //     default state
    // Round 124 closes the
    // coverage gap for:
    //   - record_gameplay
    //     non-currency rewards
    //     (the else branch —
    //     adds to inventory)
    //   - record_gameplay
    //     empty rewards vec
    //   - record_gameplay
    //     multiple records
    //     accumulating score
    //   - calculate_total_playtime
    //     multi-record sum
    //     (the private helper
    //     tested via
    //     `get_player_stats`)
    //   - SharedWorld
    //     get_active_events
    //     filters out
    //     non-active events
    //   - SharedWorld
    //     remove_event with
    //     non-existent id
    //     (no-op, not panic)
    //   - SharedWorld
    //     get_variable
    //     missing-key returns
    //     None
    //   - SharedWorld
    //     set_variable +
    //     get_variable
    //     round-trip
    //   - UnifiedWorldState
    //     get_global
    //     missing-key returns
    //     None
    // -----------------------------------------------------------------

    #[test]
    fn test_record_gameplay_non_currency_reward_adds_to_inventory_round_124() {
        // The pre-round-124
        // test_record_gameplay_with_rewards
        // only tested gold + gem
        // (both currency rewards).
        // The else branch
        // (non-currency rewards)
        // adds an `InventoryItem`
        // to the inventory.
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        let now = SystemTime::now();
        let record = GameplayRecord {
            gameplay_type: GameplayType::Puzzle,
            start_time: now,
            end_time: now,
            score: 250,
            rewards_earned: vec![
                RewardInfo { item_id: "scroll".to_string(), quantity: 3 },
                RewardInfo { item_id: "potion".to_string(), quantity: 1 },
            ],
        };
        ws.record_gameplay(record);
        // Non-currency rewards
        // land in the inventory
        // (not the wallet).
        assert_eq!(ws.wallet.get_balance(CurrencyType::Gold), 0);
        assert_eq!(ws.wallet.get_balance(CurrencyType::Gem), 0);
        assert_eq!(ws.inventory.get_item("scroll").unwrap().quantity, 3);
        assert_eq!(ws.inventory.get_item("potion").unwrap().quantity, 1);
    }

    #[test]
    fn test_record_gameplay_empty_rewards_vec_round_124() {
        // A record with no
        // rewards just adds to
        // the gameplay_history
        // and accumulates score.
        // Defense: a regression
        // that early-returned
        // on empty rewards
        // would silently drop
        // the score.
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        let now = SystemTime::now();
        let record = GameplayRecord {
            gameplay_type: GameplayType::Match3,
            start_time: now,
            end_time: now,
            score: 100,
            rewards_earned: vec![],
        };
        ws.record_gameplay(record);
        assert_eq!(ws.progression.total_score, 100);
        assert_eq!(ws.gameplay_history.len(), 1);
    }

    #[test]
    fn test_record_gameplay_accumulates_score_across_multiple_records_round_124() {
        // 3 records → total_score
        // is the sum of all 3
        // record scores.
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        let now = SystemTime::now();
        for score in [100u64, 250, 75] {
            ws.record_gameplay(GameplayRecord {
                gameplay_type: GameplayType::Card,
                start_time: now,
                end_time: now,
                score,
                rewards_earned: vec![],
            });
        }
        assert_eq!(ws.progression.total_score, 425);
        assert_eq!(ws.gameplay_history.len(), 3);
    }

    #[test]
    fn test_calculate_total_playtime_multi_record_sum_round_124() {
        // The private
        // `calculate_total_playtime`
        // helper sums each
        // record's end_time -
        // start_time. The
        // pre-round-124
        // test_record_gameplay
        // used start_time ==
        // end_time so total
        // playtime was always
        // 0. Round 124 pins
        // the multi-record
        // sum via a constructed
        // history of 3 records
        // with different
        // durations.
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        // Add 3 records with
        // different durations
        // (10s + 20s + 30s =
        // 60s total).
        let base = SystemTime::now();
        for (offset_secs, duration_secs) in [(0u64, 10u64), (60, 20), (120, 30)] {
            let start = base + std::time::Duration::from_secs(offset_secs);
            let end = start + std::time::Duration::from_secs(duration_secs);
            ws.record_gameplay(GameplayRecord {
                gameplay_type: GameplayType::Match3,
                start_time: start,
                end_time: end,
                score: 100,
                rewards_earned: vec![],
            });
        }
        // total_playtime =
        // 10 + 20 + 30 = 60s.
        let stats = ws.get_player_stats();
        assert_eq!(stats.total_playtime, 60);
        assert_eq!(stats.gameplay_count, 3);
    }

    #[test]
    fn test_get_active_events_filters_out_inactive_round_124() {
        // The pre-round-124
        // test_shared_world only
        // added 1 active event.
        // Round 124 pins the
        // filter behavior: 3
        // events added (2 active
        // + 1 inactive) → only
        // the 2 active events
        // are returned.
        let mut sw = SharedWorld::new();
        let now = SystemTime::now();
        for (i, is_active) in [(1u32, true), (2, true), (3, false)] {
            sw.add_event(WorldEvent {
                event_id: format!("e{i}"),
                name: format!("Event {i}"),
                description: String::new(),
                start_time: now,
                end_time: now,
                is_active,
                modifiers: ValueMap::new(),
            });
        }
        let active = sw.get_active_events();
        assert_eq!(active.len(), 2);
        // The 2 active event_ids
        // are e1 + e2 (e3 is
        // filtered out).
        let active_ids: Vec<&str> = active.iter().map(|e| e.event_id.as_str()).collect();
        assert!(active_ids.contains(&"e1"));
        assert!(active_ids.contains(&"e2"));
        assert!(!active_ids.contains(&"e3"));
    }

    #[test]
    fn test_remove_event_non_existent_id_is_no_op_round_124() {
        // Defense: a regression
        // that used
        // `swap_remove` would
        // silently corrupt the
        // vec order. The
        // `retain` impl is
        // order-preserving.
        let mut sw = SharedWorld::new();
        let now = SystemTime::now();
        sw.add_event(WorldEvent {
            event_id: "e1".to_string(),
            name: "Festival".to_string(),
            description: String::new(),
            start_time: now,
            end_time: now,
            is_active: true,
            modifiers: ValueMap::new(),
        });
        // Remove a non-existent
        // id — the existing
        // event is preserved.
        sw.remove_event("nonexistent");
        assert_eq!(sw.world_events.len(), 1);
        assert_eq!(sw.world_events[0].event_id, "e1");
    }

    #[test]
    fn test_shared_world_get_variable_missing_key_returns_none_round_124() {
        // Defense: missing
        // key returns None
        // (not crash, not
        // default). The
        // pre-round-124
        // SharedWorld tests
        // didn't cover
        // get_variable.
        let sw = SharedWorld::new();
        assert!(sw.get_variable("missing").is_none());
        assert!(sw.get_variable("").is_none());
    }

    #[test]
    fn test_shared_world_set_variable_get_variable_round_trip_round_124() {
        // Round-trip: a value
        // set via
        // `set_variable` must
        // be retrievable via
        // `get_variable`.
        let mut sw = SharedWorld::new();
        sw.set_variable("difficulty", Value::Float(2.0));
        sw.set_variable("mode", Value::String("hard".to_string()));
        match sw.get_variable("difficulty") {
            Some(Value::Float(f)) => assert_eq!(*f, 2.0),
            _ => panic!("expected Float(2.0)"),
        }
        match sw.get_variable("mode") {
            Some(Value::String(s)) => assert_eq!(s, "hard"),
            _ => panic!("expected String(\"hard\")"),
        }
    }

    #[test]
    fn test_unified_world_state_get_global_missing_key_returns_none_round_124() {
        // The pre-round-124
        // test_global_data only
        // tested the happy
        // path. Round 124 pins
        // the missing-key
        // contract.
        let ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        assert!(ws.get_global("missing").is_none());
        assert!(ws.get_global("").is_none());
    }

    #[test]
    fn test_active_gameplay_info_field_access_round_124() {
        // After set_active_gameplay,
        // the
        // ActiveGameplayInfo
        // struct must have
        // the correct
        // gameplay_type +
        // session_start is
        // recent + current_state
        // matches the input.
        let mut ws = UnifiedWorldState::new(PlayerProfile::new("p1"));
        let mut state = GameplayState::new();
        state.set("level", Value::Integer(3));
        let before = SystemTime::now();
        ws.set_active_gameplay(GameplayType::TowerDefense, state);
        let after = SystemTime::now();
        let info = ws.active_gameplay.as_ref().unwrap();
        assert_eq!(info.gameplay_type, GameplayType::TowerDefense);
        assert_eq!(info.current_state.get_int("level"), Some(3));
        // session_start is
        // recent (between
        // before + after).
        assert!(info.session_start >= before);
        assert!(info.session_start <= after);
        // set_active_gameplay
        // also calls
        // record_dimension_visit
        // — dimensions_visited
        // is 1.
        assert_eq!(ws.progression.dimensions_visited, 1);
    }
}
