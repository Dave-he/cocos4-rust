//! DimensionVault — the AGI's "memory" of visited dimensions.
//!
//! Round 20 introduces [`DimensionVault`], a bounded ring of past
//! dimension visits. The vault is intentionally engine-agnostic: it
//! doesn't run dimensions, mutate world state, or schedule events.
//! Its only job is to answer three questions that the higher game
//! layer keeps asking:
//!
//! 1. *What did the player just play?* — [`DimensionVault::recent`]
//! 2. *Did they ever see this blueprint before, and if so, how did it
//!    end?* — [`DimensionVault::last_outcome_for`]
//! 3. *Given a candidate pool, which one should we run next?* —
//!    [`DimensionVault::suggest_next`]
//!
//! The vault is `Send + Sync` (no interior mutability required) so
//! the game layer can pass it across threads without wrapping.
//!
//! All public methods are deterministic given the same insertion
//! order so tests can pin behaviour.

use std::collections::{HashSet, VecDeque};

use super::ai_engine::DimensionBlueprint;

/// Outcome of a single dimension run, as reported by the runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionOutcome {
    Completed,
    Failed,
    Abandoned,
}

impl DimensionOutcome {
    /// Returns `true` when the player reached the dimension's success
    /// state. Used by [`VaultStats::completion_rate`].
    pub fn is_success(self) -> bool {
        matches!(self, DimensionOutcome::Completed)
    }
}

/// One row in the vault's ring buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct VaultEntry {
    pub blueprint_id: String,
    pub blueprint_name: String,
    pub theme_name: String,
    pub visual_style: String,
    pub difficulty: f32,
    pub outcome: DimensionOutcome,
    pub timestamp_ms: u64,
}

impl VaultEntry {
    /// Convenience constructor for tests and the game layer.
    pub fn new(blueprint: &DimensionBlueprint, outcome: DimensionOutcome, timestamp_ms: u64) -> Self {
        Self {
            blueprint_id: blueprint.id.clone(),
            blueprint_name: blueprint.name.clone(),
            theme_name: blueprint.theme.name.clone(),
            visual_style: blueprint.theme.visual_style.clone(),
            difficulty: blueprint.difficulty,
            outcome,
            timestamp_ms,
        }
    }
}

/// Aggregate stats over the entire vault. Cheap to compute: O(n) in
/// the vault length.
#[derive(Debug, Clone, PartialEq)]
pub struct VaultStats {
    pub total_visits: usize,
    pub distinct_themes: usize,
    pub distinct_blueprints: usize,
    pub completed: usize,
    pub failed: usize,
    pub abandoned: usize,
}

impl VaultStats {
    /// Fraction of visits that ended in `Completed`. Returns `0.0`
    /// for an empty vault so callers can always multiply.
    pub fn completion_rate(&self) -> f32 {
        if self.total_visits == 0 {
            0.0
        } else {
            self.completed as f32 / self.total_visits as f32
        }
    }
}

/// Bounded ring of dimension visits.
///
/// `capacity` is the hard cap. When the vault is full and a new visit
/// is recorded, the *oldest* entry is dropped — the vault remembers
/// the player's recent past, not their entire history.
#[derive(Debug)]
pub struct DimensionVault {
    capacity: usize,
    entries: VecDeque<VaultEntry>,
}

impl DimensionVault {
    /// Default capacity — 64 visits. Enough for several sessions of
    /// play without burning memory.
    pub const DEFAULT_CAPACITY: usize = 64;

    /// Build an empty vault with [`DEFAULT_CAPACITY`].
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Build an empty vault that holds at most `capacity` entries.
    /// `capacity == 0` is allowed and behaves as a black hole
    /// (`record` is a no-op, `recent` always returns empty).
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        }
    }

    /// Maximum number of entries the vault can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of entries currently in the vault.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when the vault has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Append a visit. Drops the oldest entry if the vault is full.
    pub fn record(
        &mut self,
        blueprint: &DimensionBlueprint,
        outcome: DimensionOutcome,
        timestamp_ms: u64,
    ) {
        if self.capacity == 0 {
            return;
        }
        let entry = VaultEntry::new(blueprint, outcome, timestamp_ms);
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Most recent visits, newest last (chronological order of the
    /// last `limit` entries). `limit == 0` returns an empty vec;
    /// `limit >= len()` returns everything in insertion order.
    pub fn recent(&self, limit: usize) -> Vec<VaultEntry> {
        let n = limit.min(self.entries.len());
        let skip = self.entries.len() - n;
        self.entries.iter().skip(skip).cloned().collect()
    }

    /// Most recent visit for the given blueprint id, if any.
    pub fn last_outcome_for(&self, blueprint_id: &str) -> Option<DimensionOutcome> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.blueprint_id == blueprint_id)
            .map(|e| e.outcome)
    }

    /// Last `n` themes (most recent first). Used by
    /// [`DimensionVault::suggest_next`] but exposed for the game
    /// layer's UI ("recent worlds: …").
    pub fn recent_themes(&self, n: usize) -> Vec<String> {
        self.recent(n)
            .into_iter()
            .rev()
            .map(|e| e.theme_name)
            .collect()
    }

    /// Pick a blueprint the player has not seen in the last
    /// `avoid_window` visits. Falls back to a deterministic
    /// `seed`-driven pick when every blueprint is in the window.
    ///
    /// Returns `None` only when `candidates` is empty.
    ///
    /// The function is `O(candidates × avoid_window)`, which is
    /// fine for the 6–12 candidate pools the game layer passes in.
    pub fn suggest_next(
        &self,
        candidates: &[DimensionBlueprint],
        avoid_window: usize,
        seed: u64,
    ) -> Option<usize> {
        if candidates.is_empty() {
            return None;
        }

        // Bind the recent slice to a local so the &str borrow lives
        // long enough for the filter closure below.
        let recent = self.recent(avoid_window);
        let recent_ids: HashSet<&str> = recent
            .iter()
            .map(|e| e.blueprint_id.as_str())
            .collect();

        // Pass 1: any candidate whose id is not in the recent window.
        let fresh: Vec<usize> = (0..candidates.len())
            .filter(|i| !recent_ids.contains(candidates[*i].id.as_str()))
            .collect();

        if !fresh.is_empty() {
            // Deterministic pick from the fresh pool: the first id
            // whose hash(seed) lands in the smallest bucket. This
            // keeps the choice stable across calls for the same
            // (vault, candidates, seed) tuple.
            return Some(fresh[seed as usize % fresh.len()]);
        }

        // Pass 2: every candidate was seen recently. Pick the one
        // that was seen *longest ago* (largest reverse-iteration
        // position, since rev() goes newest → oldest). Ties broken
        // by the seed (smallest index wins).
        let mut ranked: Vec<(usize, usize)> = candidates
            .iter()
            .enumerate()
            .map(|(i, bp)| {
                let position = self
                    .entries
                    .iter()
                    .rev()
                    .position(|e| e.blueprint_id == bp.id)
                    .unwrap_or(usize::MAX);
                (i, position)
            })
            .collect();
        // Sort: largest position first (oldest recent visit wins),
        // ties broken by smallest index for determinism.
        ranked.sort_by(|(ai, ap), (bi, bp)| ap.cmp(bp).then(ai.cmp(bi)));
        ranked.reverse();
        let (chosen_idx, chosen_pos) = ranked[seed as usize % ranked.len()];
        if chosen_pos == usize::MAX {
            // No candidate has ever been visited. Fall back to a
            // straight seed pick.
            Some(seed as usize % candidates.len())
        } else {
            Some(chosen_idx)
        }
    }

    /// Aggregate stats over every entry in the vault.
    pub fn stats(&self) -> VaultStats {
        let mut distinct_themes = HashSet::new();
        let mut distinct_blueprints = HashSet::new();
        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut abandoned = 0usize;

        for e in &self.entries {
            distinct_themes.insert(e.theme_name.clone());
            distinct_blueprints.insert(e.blueprint_id.clone());
            match e.outcome {
                DimensionOutcome::Completed => completed += 1,
                DimensionOutcome::Failed => failed += 1,
                DimensionOutcome::Abandoned => abandoned += 1,
            }
        }

        VaultStats {
            total_visits: self.entries.len(),
            distinct_themes: distinct_themes.len(),
            distinct_blueprints: distinct_blueprints.len(),
            completed,
            failed,
            abandoned,
        }
    }

    /// Clear every entry. The capacity is preserved.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for DimensionVault {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Round 20 — unit tests for DimensionVault.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round20_tests {
    use super::*;
    use crate::agi_minigame::ai_engine::DimensionTheme;

    fn make_blueprint(id: &str, theme: &str, atoms: Vec<&str>) -> DimensionBlueprint {
        DimensionBlueprint {
            id: id.to_string(),
            name: format!("{id} name"),
            description: format!("{id} desc"),
            atom_ids: atoms.into_iter().map(String::from).collect(),
            atom_weights: std::collections::HashMap::new(),
            difficulty: 0.5,
            rules: Vec::<crate::agi_minigame::ai_engine::GeneratedRule>::new(),
            rewards: Vec::new(),
            theme: DimensionTheme {
                name: theme.to_string(),
                visual_style: format!("{theme}-style"),
                music_mood: "neutral".to_string(),
                color_palette: vec!["#000".to_string()],
            },
            time_limit_secs: Some(60),
            objectives: Vec::new(),
        }
    }

    #[test]
    fn new_vault_is_empty() {
        let v = DimensionVault::new();
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
        assert_eq!(v.capacity(), DimensionVault::DEFAULT_CAPACITY);
        assert!(v.recent(5).is_empty());
        assert_eq!(v.stats().total_visits, 0);
        assert_eq!(v.stats().completion_rate(), 0.0);
    }

    #[test]
    fn record_then_recent_returns_visits_newest_last() {
        let mut v = DimensionVault::with_capacity(4);
        let a = make_blueprint("a", "t1", vec!["match3"]);
        let b = make_blueprint("b", "t2", vec!["tower_defense"]);
        v.record(&a, DimensionOutcome::Completed, 100);
        v.record(&b, DimensionOutcome::Failed, 200);
        let recent = v.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].blueprint_id, "a");
        assert_eq!(recent[1].blueprint_id, "b");
    }

    #[test]
    fn ring_drops_oldest_when_full() {
        let mut v = DimensionVault::with_capacity(2);
        let a = make_blueprint("a", "t", vec!["match3"]);
        let b = make_blueprint("b", "t", vec!["match3"]);
        let c = make_blueprint("c", "t", vec!["match3"]);
        v.record(&a, DimensionOutcome::Completed, 1);
        v.record(&b, DimensionOutcome::Completed, 2);
        v.record(&c, DimensionOutcome::Completed, 3);
        assert_eq!(v.len(), 2);
        let recent = v.recent(10);
        assert_eq!(recent[0].blueprint_id, "b");
        assert_eq!(recent[1].blueprint_id, "c");
    }

    #[test]
    fn capacity_zero_is_a_black_hole() {
        let mut v = DimensionVault::with_capacity(0);
        let a = make_blueprint("a", "t", vec!["match3"]);
        v.record(&a, DimensionOutcome::Completed, 1);
        assert!(v.is_empty());
        assert!(v.recent(1).is_empty());
    }

    #[test]
    fn last_outcome_for_returns_most_recent_match() {
        let mut v = DimensionVault::new();
        let a = make_blueprint("a", "t", vec!["match3"]);
        v.record(&a, DimensionOutcome::Failed, 1);
        v.record(&a, DimensionOutcome::Completed, 2);
        assert_eq!(v.last_outcome_for("a"), Some(DimensionOutcome::Completed));
        assert_eq!(v.last_outcome_for("nope"), None);
    }

    #[test]
    fn stats_counts_distinct_themes_and_outcomes() {
        let mut v = DimensionVault::new();
        let a = make_blueprint("a", "ice", vec!["match3"]);
        let b = make_blueprint("b", "fire", vec!["parkour"]);
        let c = make_blueprint("a", "ice", vec!["match3"]); // re-use a/ice
        v.record(&a, DimensionOutcome::Completed, 1);
        v.record(&b, DimensionOutcome::Failed, 2);
        v.record(&c, DimensionOutcome::Abandoned, 3);
        let s = v.stats();
        assert_eq!(s.total_visits, 3);
        assert_eq!(s.distinct_blueprints, 2);
        assert_eq!(s.distinct_themes, 2);
        assert_eq!(s.completed, 1);
        assert_eq!(s.failed, 1);
        assert_eq!(s.abandoned, 1);
        assert!((s.completion_rate() - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn suggest_next_returns_none_for_empty_candidates() {
        let v = DimensionVault::new();
        assert_eq!(v.suggest_next(&[], 2, 0), None);
    }

    #[test]
    fn suggest_next_picks_a_fresh_blueprint() {
        let mut v = DimensionVault::new();
        let a = make_blueprint("a", "t", vec!["match3"]);
        v.record(&a, DimensionOutcome::Completed, 1);
        // "a" is in the recent window; "b" and "c" are not. The
        // suggestion should land on either 0 (b) or 1 (c).
        let pool = vec![
            make_blueprint("a", "t", vec!["match3"]),
            make_blueprint("b", "t", vec!["match3"]),
            make_blueprint("c", "t", vec!["match3"]),
        ];
        let pick = v.suggest_next(&pool, 1, 0).unwrap();
        assert!(pick == 1 || pick == 2, "expected fresh pick, got {pick}");
    }

    #[test]
    fn suggest_next_picks_least_recent_when_all_seen() {
        let mut v = DimensionVault::with_capacity(4);
        let a = make_blueprint("a", "t", vec!["match3"]);
        let b = make_blueprint("b", "t", vec!["match3"]);
        v.record(&a, DimensionOutcome::Completed, 1);
        v.record(&b, DimensionOutcome::Completed, 2);
        v.record(&a, DimensionOutcome::Completed, 3);
        // recent window of 4 covers all entries, so a and b are both
        // "seen". The oldest among them is "b" (timestamp 2), which
        // should be chosen (index 1).
        let pool = vec![a, b];
        let pick = v.suggest_next(&pool, 4, 0).unwrap();
        assert_eq!(pick, 1);
    }

    #[test]
    fn suggest_next_handles_completely_unseen_candidates() {
        let v = DimensionVault::new();
        let pool = vec![
            make_blueprint("a", "t", vec!["match3"]),
            make_blueprint("b", "t", vec!["match3"]),
        ];
        let pick = v.suggest_next(&pool, 4, 0).unwrap();
        assert!(pick < 2);
    }

    #[test]
    fn clear_resets_entries_but_keeps_capacity() {
        let mut v = DimensionVault::with_capacity(8);
        let a = make_blueprint("a", "t", vec!["match3"]);
        v.record(&a, DimensionOutcome::Completed, 1);
        v.clear();
        assert!(v.is_empty());
        assert_eq!(v.capacity(), 8);
    }

    #[test]
    fn recent_themes_returns_recent_theme_names() {
        let mut v = DimensionVault::new();
        v.record(&make_blueprint("a", "ice", vec!["match3"]), DimensionOutcome::Completed, 1);
        v.record(&make_blueprint("b", "fire", vec!["parkour"]), DimensionOutcome::Failed, 2);
        v.record(&make_blueprint("c", "ice", vec!["match3"]), DimensionOutcome::Completed, 3);
        let themes = v.recent_themes(2);
        // Most recent first.
        assert_eq!(themes, vec!["ice".to_string(), "fire".to_string()]);
    }

    #[test]
    fn outcome_helper_is_success() {
        assert!(DimensionOutcome::Completed.is_success());
        assert!(!DimensionOutcome::Failed.is_success());
        assert!(!DimensionOutcome::Abandoned.is_success());
    }
}

// ---------------------------------------------------------------------------
// Round 129 — vault.rs helper-level unit tests.
// Mirrors the round-110b / 122 / 123 / 124 / 125 / 126 / 127 / 128
// pattern: pin behavior of the small public helpers
// (`DEFAULT_CAPACITY`, `len`, `is_empty`, `capacity`,
// `recent` edge cases, `recent_themes` ordering, `suggest_next`
// determinism, `VaultEntry::new` field copy, `VaultStats::completion_rate`)
// so refactors can't silently change the contract.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round129_tests {
    use super::*;
    use crate::agi_minigame::ai_engine::DimensionTheme;

    fn make_blueprint(id: &str, theme: &str) -> DimensionBlueprint {
        DimensionBlueprint {
            id: id.to_string(),
            name: format!("{id} name"),
            description: format!("{id} desc"),
            atom_ids: vec!["match3".to_string()],
            atom_weights: std::collections::HashMap::new(),
            difficulty: 0.5,
            rules: Vec::<crate::agi_minigame::ai_engine::GeneratedRule>::new(),
            rewards: Vec::new(),
            theme: DimensionTheme {
                name: theme.to_string(),
                visual_style: format!("{theme}-style"),
                music_mood: "neutral".to_string(),
                color_palette: vec!["#000".to_string()],
            },
            time_limit_secs: Some(60),
            objectives: Vec::new(),
        }
    }

    #[test]
    fn default_capacity_is_64() {
        // The DEFAULT_CAPACITY constant is part of the
        // public API — pinned here so a refactor that
        // changes it to (say) 128 doesn't silently
        // double the memory footprint of every App.
        assert_eq!(DimensionVault::DEFAULT_CAPACITY, 64);
        let v = DimensionVault::new();
        assert_eq!(v.capacity(), 64);
    }

    #[test]
    fn with_capacity_honors_non_default_value() {
        // A non-default capacity (e.g. 5 for short
        // test runs) should be reflected by `capacity()`
        // and should NOT trigger ring drop until 6
        // entries are inserted.
        let mut v = DimensionVault::with_capacity(5);
        assert_eq!(v.capacity(), 5);
        for i in 0..5 {
            v.record(&make_blueprint(&format!("d{i}"), "t"), DimensionOutcome::Completed, i as u64);
        }
        assert_eq!(v.len(), 5);
        // The 6th insert triggers the ring drop.
        v.record(&make_blueprint("d5", "t"), DimensionOutcome::Completed, 5);
        assert_eq!(v.len(), 5);
        assert_eq!(v.recent(10).first().unwrap().blueprint_id, "d1");
    }

    #[test]
    fn recent_zero_returns_empty() {
        // A `limit == 0` request should yield an empty
        // vec — the doc says so but the implementation
        // must be pinned.
        let mut v = DimensionVault::new();
        v.record(&make_blueprint("a", "t"), DimensionOutcome::Completed, 1);
        assert!(v.recent(0).is_empty());
    }

    #[test]
    fn recent_with_limit_greater_than_len_returns_everything_in_chronological_order() {
        // `recent(100)` on a 3-entry vault should give
        // back exactly 3 entries in insertion order
        // (oldest first, newest last).
        let mut v = DimensionVault::with_capacity(8);
        v.record(&make_blueprint("a", "t"), DimensionOutcome::Completed, 1);
        v.record(&make_blueprint("b", "t"), DimensionOutcome::Completed, 2);
        v.record(&make_blueprint("c", "t"), DimensionOutcome::Completed, 3);
        let recent = v.recent(100);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].blueprint_id, "a");
        assert_eq!(recent[1].blueprint_id, "b");
        assert_eq!(recent[2].blueprint_id, "c");
    }

    #[test]
    fn recent_themes_zero_returns_empty() {
        // Mirrors `recent(0)`.
        let mut v = DimensionVault::new();
        v.record(&make_blueprint("a", "ice"), DimensionOutcome::Completed, 1);
        assert!(v.recent_themes(0).is_empty());
    }

    #[test]
    fn recent_themes_returns_most_recent_first_and_caps_at_n() {
        // 5 entries (themes: ice, fire, ice, wind, fire).
        // recent_themes(3) should yield [fire, wind, ice]
        // (the 3 most recent themes, in newest-first order).
        let mut v = DimensionVault::with_capacity(8);
        v.record(&make_blueprint("a", "ice"),  DimensionOutcome::Completed, 1);
        v.record(&make_blueprint("b", "fire"), DimensionOutcome::Completed, 2);
        v.record(&make_blueprint("c", "ice"),  DimensionOutcome::Completed, 3);
        v.record(&make_blueprint("d", "wind"), DimensionOutcome::Completed, 4);
        v.record(&make_blueprint("e", "fire"), DimensionOutcome::Completed, 5);
        let themes = v.recent_themes(3);
        assert_eq!(themes, vec!["fire".to_string(), "wind".to_string(), "ice".to_string()]);
    }

    #[test]
    fn suggest_next_seed_changes_deterministic_pick_from_fresh_pool() {
        // When a fresh pool has 2+ candidates, the
        // suggested pick should be a deterministic
        // function of `seed` (`fresh[seed as usize % fresh.len()]`)
        // so a regression that swaps to RNG or
        // hash-based picking fails this test.
        let mut v = DimensionVault::with_capacity(4);
        let a = make_blueprint("a", "t");
        v.record(&a, DimensionOutcome::Completed, 1);
        let pool = vec![
            make_blueprint("a", "t"), // seen — not fresh
            make_blueprint("b", "t"), // fresh
            make_blueprint("c", "t"), // fresh
        ];
        // With seed=0, pick = fresh[0 % 2] = 1 (b).
        // With seed=1, pick = fresh[1 % 2] = 2 (c).
        assert_eq!(v.suggest_next(&pool, 1, 0), Some(1));
        assert_eq!(v.suggest_next(&pool, 1, 1), Some(2));
        // With seed=2, pick = fresh[2 % 2] = 0 (b).
        assert_eq!(v.suggest_next(&pool, 1, 2), Some(1));
    }

    #[test]
    fn last_outcome_for_after_clear_returns_none() {
        // `clear()` wipes the entries; `last_outcome_for`
        // for any id (including a previously-inserted
        // one) should now return None.
        let mut v = DimensionVault::new();
        let a = make_blueprint("a", "t");
        v.record(&a, DimensionOutcome::Failed, 1);
        assert_eq!(v.last_outcome_for("a"), Some(DimensionOutcome::Failed));
        v.clear();
        assert_eq!(v.last_outcome_for("a"), None);
        assert!(v.is_empty());
    }

    #[test]
    fn vault_entry_new_copies_blueprint_fields() {
        // `VaultEntry::new` must copy all 6 source
        // fields (id, name, theme.name, theme.visual_style,
        // difficulty) — a regression that forgot any
        // would show up here.
        let bp = DimensionBlueprint {
            id: "bp1".to_string(),
            name: "BP Name".to_string(),
            description: "ignored".to_string(),
            atom_ids: vec!["a1".to_string()],
            atom_weights: std::collections::HashMap::new(),
            difficulty: 0.73,
            rules: Vec::new(),
            rewards: Vec::new(),
            theme: DimensionTheme {
                name: "lava".to_string(),
                visual_style: "lava-style".to_string(),
                music_mood: "epic".to_string(),
                color_palette: vec!["#f00".to_string()],
            },
            time_limit_secs: Some(120),
            objectives: Vec::new(),
        };
        let entry = VaultEntry::new(&bp, DimensionOutcome::Completed, 999);
        assert_eq!(entry.blueprint_id, "bp1");
        assert_eq!(entry.blueprint_name, "BP Name");
        assert_eq!(entry.theme_name, "lava");
        assert_eq!(entry.visual_style, "lava-style");
        assert!((entry.difficulty - 0.73).abs() < 1e-6);
        assert_eq!(entry.outcome, DimensionOutcome::Completed);
        assert_eq!(entry.timestamp_ms, 999);
    }

    #[test]
    fn completion_rate_is_one_for_all_completed_vault() {
        // Pinned the upper boundary (was only the 0.0
        // boundary tested before).
        let mut v = DimensionVault::with_capacity(4);
        for i in 0..3 {
            v.record(
                &make_blueprint(&format!("d{i}"), "t"),
                DimensionOutcome::Completed,
                i as u64,
            );
        }
        let s = v.stats();
        assert_eq!(s.completion_rate(), 1.0);
        assert_eq!(s.completed, 3);
        assert_eq!(s.failed, 0);
        assert_eq!(s.abandoned, 0);
    }

    #[test]
    fn stats_distinct_themes_counts_unique_names() {
        // 4 entries with 3 distinct themes → distinct_themes == 3.
        let mut v = DimensionVault::with_capacity(8);
        v.record(&make_blueprint("a", "ice"),  DimensionOutcome::Completed, 1);
        v.record(&make_blueprint("b", "fire"), DimensionOutcome::Failed, 2);
        v.record(&make_blueprint("c", "ice"),  DimensionOutcome::Completed, 3);
        v.record(&make_blueprint("d", "wind"), DimensionOutcome::Completed, 4);
        let s = v.stats();
        assert_eq!(s.total_visits, 4);
        assert_eq!(s.distinct_themes, 3);
        assert_eq!(s.distinct_blueprints, 4);
    }
}
