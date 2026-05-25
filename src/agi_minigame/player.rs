use std::collections::HashMap;

use crate::base::value::{Value, ValueMap};

#[derive(Debug, Clone)]
pub struct PlayerAccount {
    pub account_id: String,
    pub display_name: String,
    pub avatar: String,
    pub created_at: u64,
    pub last_login: u64,
    pub is_online: bool,
}

impl PlayerAccount {
    pub fn new(account_id: &str) -> Self {
        Self {
            account_id: account_id.to_string(),
            display_name: account_id.to_string(),
            avatar: String::new(),
            created_at: 0,
            last_login: 0,
            is_online: true,
        }
    }

    pub fn login(&mut self) {
        self.is_online = true;
    }

    pub fn logout(&mut self) {
        self.is_online = false;
    }
}

#[derive(Debug, Clone)]
pub struct PlayerProfile {
    pub account: PlayerAccount,
    pub level: u32,
    pub experience: u64,
    pub experience_to_next: u64,
    pub title: String,
    pub achievements: Vec<String>,
    pub stats: PlayerStatsMap,
    pub preferences: ValueMap,
}

#[derive(Debug, Clone)]
pub struct PlayerStatsMap {
    inner: HashMap<String, f64>,
}

impl PlayerStatsMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> f64 {
        self.inner.get(key).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, key: &str, value: f64) {
        self.inner.insert(key.to_string(), value);
    }

    pub fn add(&mut self, key: &str, delta: f64) -> f64 {
        let v = self.get(key) + delta;
        self.set(key, v);
        v
    }

    pub fn keys(&self) -> Vec<&str> {
        self.inner.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PlayerStatsMap {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerProfile {
    pub fn new(account_id: &str) -> Self {
        Self {
            account: PlayerAccount::new(account_id),
            level: 1,
            experience: 0,
            experience_to_next: 100,
            title: String::new(),
            achievements: Vec::new(),
            stats: PlayerStatsMap::new(),
            preferences: ValueMap::new(),
        }
    }

    pub fn add_experience(&mut self, amount: u64) -> u32 {
        self.experience += amount;
        let mut levels_gained = 0u32;
        while self.experience >= self.experience_to_next {
            self.experience -= self.experience_to_next;
            self.level += 1;
            levels_gained += 1;
            self.experience_to_next = Self::calc_exp_to_next(self.level);
        }
        levels_gained
    }

    fn calc_exp_to_next(level: u32) -> u64 {
        100 * (level as u64).pow(2) / 10 + 100
    }

    pub fn add_achievement(&mut self, achievement: &str) -> bool {
        if self.achievements.contains(&achievement.to_string()) {
            return false;
        }
        self.achievements.push(achievement.to_string());
        true
    }

    pub fn has_achievement(&self, achievement: &str) -> bool {
        self.achievements.contains(&achievement.to_string())
    }

    pub fn set_stat(&mut self, key: &str, value: f64) {
        self.stats.set(key, value);
    }

    pub fn get_stat(&self, key: &str) -> f64 {
        self.stats.get(key)
    }

    pub fn add_stat(&mut self, key: &str, delta: f64) -> f64 {
        self.stats.add(key, delta)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerProgression {
    pub dimensions_visited: u32,
    pub dimensions_completed: u32,
    pub total_score: u64,
    pub highest_score: u64,
    pub total_playtime_secs: u64,
    pub atom_mastery: HashMap<String, AtomMastery>,
    pub unlocked_atoms: Vec<String>,
    pub unlocked_dimensions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AtomMastery {
    pub atom_id: String,
    pub level: u32,
    pub experience: u64,
    pub best_score: u64,
    pub play_count: u32,
}

impl PlayerProgression {
    pub fn new() -> Self {
        Self {
            dimensions_visited: 0,
            dimensions_completed: 0,
            total_score: 0,
            highest_score: 0,
            total_playtime_secs: 0,
            atom_mastery: HashMap::new(),
            unlocked_atoms: Vec::new(),
            unlocked_dimensions: Vec::new(),
        }
    }

    pub fn record_dimension_visit(&mut self, dimension_id: &str) {
        self.dimensions_visited += 1;
        if !self.unlocked_dimensions.contains(&dimension_id.to_string()) {
            self.unlocked_dimensions.push(dimension_id.to_string());
        }
    }

    pub fn record_dimension_complete(&mut self, score: u64) {
        self.dimensions_completed += 1;
        self.total_score += score;
        if score > self.highest_score {
            self.highest_score = score;
        }
    }

    pub fn record_atom_play(&mut self, atom_id: &str, score: u64) {
        if !self.unlocked_atoms.contains(&atom_id.to_string()) {
            self.unlocked_atoms.push(atom_id.to_string());
        }
        let mastery = self.atom_mastery.entry(atom_id.to_string()).or_insert_with(|| {
            AtomMastery {
                atom_id: atom_id.to_string(),
                level: 0,
                experience: 0,
                best_score: 0,
                play_count: 0,
            }
        });
        mastery.play_count += 1;
        mastery.experience += score;
        if score > mastery.best_score {
            mastery.best_score = score;
        }
        mastery.level = (mastery.experience / 1000) as u32;
    }

    pub fn get_atom_mastery(&self, atom_id: &str) -> Option<&AtomMastery> {
        self.atom_mastery.get(atom_id)
    }

    pub fn is_atom_unlocked(&self, atom_id: &str) -> bool {
        self.unlocked_atoms.contains(&atom_id.to_string())
    }
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_profile_new() {
        let profile = PlayerProfile::new("player1");
        assert_eq!(profile.account.account_id, "player1");
        assert_eq!(profile.level, 1);
        assert_eq!(profile.experience, 0);
    }

    #[test]
    fn test_add_experience() {
        let mut profile = PlayerProfile::new("p1");
        let levels = profile.add_experience(100);
        assert!(levels >= 1);
        assert!(profile.level > 1);
    }

    #[test]
    fn test_achievements() {
        let mut profile = PlayerProfile::new("p1");
        assert!(profile.add_achievement("first_win"));
        assert!(!profile.add_achievement("first_win"));
        assert!(profile.has_achievement("first_win"));
    }

    #[test]
    fn test_player_stats() {
        let mut profile = PlayerProfile::new("p1");
        profile.set_stat("kills", 10.0);
        assert_eq!(profile.get_stat("kills"), 10.0);
        profile.add_stat("kills", 5.0);
        assert_eq!(profile.get_stat("kills"), 15.0);
    }

    #[test]
    fn test_progression() {
        let mut prog = PlayerProgression::new();
        prog.record_dimension_visit("dim_1");
        assert_eq!(prog.dimensions_visited, 1);
        prog.record_dimension_complete(500);
        assert_eq!(prog.total_score, 500);
        assert_eq!(prog.highest_score, 500);
        prog.record_atom_play("match3", 200);
        assert!(prog.is_atom_unlocked("match3"));
        let mastery = prog.get_atom_mastery("match3").unwrap();
        assert_eq!(mastery.play_count, 1);
        assert_eq!(mastery.best_score, 200);
    }

    #[test]
    fn test_player_account() {
        let mut account = PlayerAccount::new("acc1");
        assert!(account.is_online);
        account.logout();
        assert!(!account.is_online);
        account.login();
        assert!(account.is_online);
    }
}
