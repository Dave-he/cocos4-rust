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
}
