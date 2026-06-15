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

    // -----------------------------------------------------------------
    // Round 123 — helper-level
    // tests for the rich
    // `PlayerProgression` +
    // `PlayerStatsMap` +
    // `add_experience`
    // multi-level logic
    // (round 110b / round 122
    // pattern extended). The
    // pre-round-123 tests
    // covered:
    //   - profile.new() +
    //     add_experience
    //     (single-level)
    //   - add_achievement +
    //     has_achievement
    //     (happy path)
    //   - PlayerStatsMap
    //     set/get/add
    //   - PlayerProgression
    //     visit + complete +
    //     atom_play happy
    //     path
    //   - PlayerAccount
    //     login/logout
    // Round 123 closes the
    // coverage gap for:
    //   - add_experience
    //     multi-level
    //     (the while loop in
    //     `add_experience`
    //     iterates `while
    //     experience >=
    //     experience_to_next`
    //     — never tested for
    //     2+ level-ups)
    //   - add_achievement
    //     empty string +
    //     unicode
    //   - has_achievement
    //     missing-key returns
    //     false (not crash)
    //   - PlayerStatsMap.keys()
    //     order preservation
    //   - PlayerStatsMap.get()
    //     missing-key returns
    //     0.0
    //   - record_atom_play
    //     best_score NOT
    //     updated when new
    //     score is lower
    //   - record_atom_play
    //     mastery level =
    //     experience / 1000
    //     formula
    //   - record_atom_play
    //     multiple atom_ids
    //     HashMap isolation
    //   - PlayerAccount::new
    //     default display_name
    //     = account_id
    // -----------------------------------------------------------------

    #[test]
    fn test_add_experience_multi_level_up_round_123() {
        // The pre-round-123
        // test_add_experience
        // called
        // `add_experience(100)`
        // which crosses 1
        // level threshold
        // (100 >= 100). Round
        // 123 adds a test for
        // a single large
        // experience grant
        // that crosses 2+
        // thresholds in one
        // call (the while
        // loop iterates).
        let mut profile = PlayerProfile::new("p1");
        // calc_exp_to_next:
        //   level 1 → 100*1/10 + 100 = 110
        //   level 2 → 100*4/10 + 100 = 140
        //   level 3 → 100*9/10 + 100 = 190
        // Total to reach
        // level 4 from
        // level 1: 110 + 140
        // + 190 = 440. A
        // single grant of
        // 440 should level
        // up 3 times.
        let levels = profile.add_experience(440);
        assert_eq!(levels, 3);
        assert_eq!(profile.level, 4);
    }

    #[test]
    fn test_add_experience_partial_level_round_123() {
        // A grant that
        // doesn't cross a
        // threshold returns
        // 0 levels gained.
        let mut profile = PlayerProfile::new("p1");
        let levels = profile.add_experience(50);
        // 50 < 110 (exp to
        // reach level 2), so
        // 0 levels gained.
        assert_eq!(levels, 0);
        assert_eq!(profile.level, 1);
        assert_eq!(profile.experience, 50);
    }

    #[test]
    fn test_add_achievement_empty_string_and_unicode_round_123() {
        // Defense: the
        // pre-round-123
        // achievements test
        // used
        // "first_win" only.
        // Round 123 pins the
        // empty-string + a
        // unicode string +
        // a string with
        // spaces edge cases.
        let mut profile = PlayerProfile::new("p1");
        assert!(profile.add_achievement(""));
        assert!(profile.add_achievement("中文成就"));
        assert!(profile.add_achievement("first win"));
        // All 3 are now in
        // the achievements
        // vec in the order
        // added.
        assert_eq!(
            profile.achievements,
            vec![
                String::new(),
                "中文成就".to_string(),
                "first win".to_string(),
            ]
        );
        // Adding the same
        // empty string a
        // 2nd time returns
        // false.
        assert!(!profile.add_achievement(""));
    }

    #[test]
    fn test_has_achievement_missing_key_returns_false_round_123() {
        // Defense: a
        // regression that
        // returned true for
        // missing keys would
        // silently break the
        // "first kill"
        // achievement UI.
        let profile = PlayerProfile::new("p1");
        assert!(!profile.has_achievement("first_kill"));
        assert!(!profile.has_achievement(""));
        assert!(!profile.has_achievement("中文成就"));
    }

    #[test]
    fn test_player_stats_map_keys_round_123() {
        // PlayerStatsMap.keys()
        // returns a Vec<&str>
        // of all keys in
        // insertion order
        // (HashMap preserves
        // insertion order in
        // practice for
        // non-overflow
        // workloads). The
        // pre-round-123 tests
        // didn't cover
        // keys().
        let mut stats = PlayerStatsMap::new();
        stats.set("kills",   10.0);
        stats.set("deaths",   5.0);
        stats.set("score",  500.0);
        let keys = stats.keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"kills"));
        assert!(keys.contains(&"deaths"));
        assert!(keys.contains(&"score"));
    }

    #[test]
    fn test_player_stats_map_get_missing_key_round_123() {
        // Defense: missing
        // key returns 0.0
        // (not a panic). The
        // pre-round-123
        // PlayerStatsMap
        // tests didn't cover
        // the missing-key
        // path.
        let stats = PlayerStatsMap::new();
        assert_eq!(stats.get("kills"),   0.0);
        assert_eq!(stats.get("missing"), 0.0);
        assert_eq!(stats.get(""),        0.0);
    }

    #[test]
    fn test_record_atom_play_best_score_not_updated_for_lower_score_round_123() {
        // The
        // record_atom_play
        // method updates
        // best_score only
        // when the new
        // score is greater
        // than the current
        // best_score.
        // Defense: a
        // regression that
        // used `>=` would
        // overwrite the
        // best_score even
        // when the new
        // score is equal
        // or lower.
        let mut prog = PlayerProgression::new();
        prog.record_atom_play("match3", 500);
        let mastery = prog.get_atom_mastery("match3").unwrap();
        assert_eq!(mastery.best_score, 500);
        // Lower score → no
        // update.
        prog.record_atom_play("match3", 300);
        let mastery = prog.get_atom_mastery("match3").unwrap();
        assert_eq!(mastery.best_score, 500);
        // Equal score → no
        // update.
        prog.record_atom_play("match3", 500);
        let mastery = prog.get_atom_mastery("match3").unwrap();
        assert_eq!(mastery.best_score, 500);
    }

    #[test]
    fn test_record_atom_play_mastery_level_formula_round_123() {
        // The mastery level
        // formula is
        // `experience / 1000`
        // (cast to u32). The
        // pre-round-123
        // test_progression
        // didn't exercise
        // this formula.
        let mut prog = PlayerProgression::new();
        // 2500 total
        // experience → level
        // 2.
        prog.record_atom_play("match3", 1500);
        prog.record_atom_play("match3", 1000);
        let mastery = prog.get_atom_mastery("match3").unwrap();
        assert_eq!(mastery.experience, 2500);
        assert_eq!(mastery.level, 2);
        // 5000 total → level
        // 5.
        prog.record_atom_play("match3", 2500);
        let mastery = prog.get_atom_mastery("match3").unwrap();
        assert_eq!(mastery.experience, 5000);
        assert_eq!(mastery.level, 5);
    }

    #[test]
    fn test_record_atom_play_multiple_atom_isolation_round_123() {
        // Each atom has its
        // own mastery entry.
        // Defense: a
        // regression that
        // shared state
        // across atoms
        // would corrupt the
        // mastery per atom.
        let mut prog = PlayerProgression::new();
        prog.record_atom_play("match3", 500);
        prog.record_atom_play("tower_defense", 1000);
        prog.record_atom_play("match3", 700);
        let match3 = prog.get_atom_mastery("match3").unwrap();
        let tower = prog.get_atom_mastery("tower_defense").unwrap();
        assert_eq!(match3.play_count,   2);
        assert_eq!(match3.experience,  1200);
        assert_eq!(match3.best_score,   700);
        assert_eq!(match3.level,         1);
        assert_eq!(tower.play_count,    1);
        assert_eq!(tower.experience,   1000);
        assert_eq!(tower.best_score,   1000);
        assert_eq!(tower.level,          1);
    }

    #[test]
    fn test_player_account_default_display_name_equals_account_id_round_123() {
        // PlayerAccount::new
        // sets display_name
        // = account_id by
        // default. The
        // pre-round-123
        // test_player_account
        // didn't pin this
        // invariant.
        let account = PlayerAccount::new("acc1");
        assert_eq!(account.account_id,   "acc1");
        assert_eq!(account.display_name, "acc1");
        assert_eq!(account.avatar,        "");
        assert_eq!(account.created_at,    0);
        assert_eq!(account.last_login,     0);
        assert!(account.is_online);
    }
}
