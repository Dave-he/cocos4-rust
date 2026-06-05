//! Round 23 — Reflexive scene generation.
//!
//! Closes the round-22 reflexive loop: the world's NPC collective
//! mood should *actually* shape the next dimension's generation
//! parameters, not just be observable in the HUD.
//!
//! What this module does
//! ---------------------
//! Given a `GenerationHint` (the caller's knobs) and the collective
//! `NpcDisposition` (typically from
//! `NpcRegistry::average_disposition`), [`build_generation_config_with_mood`]
//! returns a [`GenerationConfig`] whose:
//!
//!   1. `difficulty_range` has been nudged by the mood:
//!        - `fear > 0.5` → upper bound `-0.05` (the world already
//!          feels scary — don't pile on)
//!        - `friendly > 0.5 && trust > 0.3` → lower bound `+0.05`
//!          (raise the stakes; the player is doing well)
//!        - `friendly < -0.3` → lower bound `-0.05` (player is
//!          hated — a difficulty bump won't fix social rot, ease up)
//!      All bounds are then clamped to `[0.1, 1.0]` and the
//!      invariant `lo ≤ hi` is preserved.
//!
//!   2. `preferred_types` is reordered so the mood-relevant atoms
//!      come first. Each branch above promotes one of two candidate
//!      atoms; the seed picks deterministically. When multiple
//!      branches fire, all promoted atoms appear before the
//!      un-promoted base pool, with no duplicates.
//!
//!   3. `excluded_types` is filled from `recent_loss_count` (mirror
//!      of the TS `GameplayCombinerAI`): `>= 3` recent losses
//!      excludes `Shooting`.
//!
//! When `mood` is the default neutral disposition, the result is
//! the plain "mood-less" base — the reflexive loop adds information
//! when there *is* information, never noise.

use rand::seq::SliceRandom;
use rand::SeedableRng;

use super::ai_engine::GenerationConfig;
use super::gameplay::GameplayType;
use super::npc::NpcDisposition;

/// Caller-provided knobs the scene generator does *not* infer.
/// Mirrors the `base` argument that `GameplayCombinerAI.toGenerationConfig`
/// takes on the TS side.
#[derive(Debug, Clone)]
pub struct GenerationHint {
    pub min_atoms: usize,
    pub max_atoms: usize,
    pub reward_multiplier: f32,
    pub base_difficulty_range: (f32, f32),
}

impl Default for GenerationHint {
    fn default() -> Self {
        Self {
            min_atoms: 2,
            max_atoms: 4,
            reward_multiplier: 1.0,
            base_difficulty_range: (0.3, 0.8),
        }
    }
}

/// Build a `GenerationConfig` whose `difficulty_range` and
/// `preferred_types` reflect the collective NPC mood.
///
/// `seed` makes the per-branch atom pick deterministic — the same
/// `(level, losses, mood, hint, seed)` always produces the same
/// output.
pub fn build_generation_config_with_mood(
    player_level: u32,
    recent_loss_count: u32,
    mood: &NpcDisposition,
    hint: GenerationHint,
    seed: u64,
) -> GenerationConfig {
    // 1. Difficulty bounds = base hint, nudged by mood.
    let (base_lo, base_hi) = hint.base_difficulty_range;
    let mut lo = base_lo;
    let mut hi = base_hi;
    if mood.fear > 0.5 {
        hi -= 0.05;
    }
    if mood.friendly > 0.5 && mood.trust > 0.3 {
        lo += 0.05;
    }
    if mood.friendly < -0.3 {
        lo -= 0.05;
    }
    lo = lo.clamp(0.1, 1.0);
    hi = hi.clamp(0.1, 1.0);
    if lo > hi {
        lo = hi;
    }

    // 2. Preferred types: stage pool + mood-promoted atoms to the front.
    let base_pool = default_preferred_pool(player_level);
    let promoted = mood_promoted_atoms(mood, seed);
    let preferred_types = merge_with_promoted(&base_pool, &promoted);

    // 3. Excluded types: mirror TS GameplayCombinerAI's "≥3 losses → drop shooting".
    let mut excluded_types: Vec<GameplayType> = Vec::new();
    if recent_loss_count >= 3 {
        excluded_types.push(GameplayType::Shooting);
    }

    GenerationConfig {
        min_atoms: hint.min_atoms,
        max_atoms: hint.max_atoms.max(hint.min_atoms),
        difficulty_range: (lo, hi),
        allow_composite: true,
        seed: Some(seed),
        player_level,
        preferred_types,
        excluded_types,
        reward_multiplier: hint.reward_multiplier,
    }
}

/// Atom ids that the mood promotes to the front of the preferred
/// list. Two candidates per branch — the seed picks one of them
/// deterministically. Multiple branches can fire; the union of
/// picks is returned in branch order.
pub fn mood_promoted_atoms(mood: &NpcDisposition, seed: u64) -> Vec<GameplayType> {
    let branches: Vec<Vec<GameplayType>> = [
        (mood.fear > 0.5).then(|| vec![GameplayType::Parkour, GameplayType::Puzzle]),
        (mood.friendly > 0.5 && mood.trust > 0.3)
            .then(|| vec![GameplayType::Match3, GameplayType::Synthesis]),
        (mood.friendly < -0.3)
            .then(|| vec![GameplayType::TowerDefense, GameplayType::TurnCombat]),
    ]
    .into_iter()
    .flatten()
    .collect();

    if branches.is_empty() {
        return Vec::new();
    }

    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut promoted: Vec<GameplayType> = Vec::with_capacity(branches.len());
    for branch in &branches {
        let pick = branch.choose(&mut rng).expect("non-empty branch").clone();
        promoted.push(pick);
    }
    promoted
}

/// Default preferred_types pool (mirror of `GameplayCombinerAI`'s
/// stage tables in TS).
fn default_preferred_pool(player_level: u32) -> Vec<GameplayType> {
    if player_level <= 4 {
        vec![GameplayType::Parkour, GameplayType::Synthesis, GameplayType::Match3]
    } else if player_level <= 14 {
        vec![
            GameplayType::TowerDefense,
            GameplayType::Card,
            GameplayType::Puzzle,
            GameplayType::Synthesis,
        ]
    } else {
        vec![
            GameplayType::TurnCombat,
            GameplayType::Synthesis,
            GameplayType::Shooting,
            GameplayType::Card,
            GameplayType::TowerDefense,
        ]
    }
}

/// Merge promoted atoms to the front of the base pool, preserving
/// the base order for the rest and removing duplicates.
fn merge_with_promoted(
    base: &[GameplayType],
    promoted: &[GameplayType],
) -> Vec<GameplayType> {
    let mut seen: Vec<GameplayType> = Vec::with_capacity(base.len() + promoted.len());
    for p in promoted {
        if !seen.contains(p) {
            seen.push(p.clone());
        }
    }
    for b in base {
        if !seen.contains(b) {
            seen.push(b.clone());
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neutral() -> NpcDisposition {
        NpcDisposition::default()
    }

    #[test]
    fn neutral_mood_preserves_base_hint_range() {
        // No mood nudges → difficulty_range must equal the base hint.
        let cfg = build_generation_config_with_mood(
            5, 0, &neutral(), GenerationHint::default(), 42,
        );
        assert!((cfg.difficulty_range.0 - 0.3).abs() < 1e-5);
        assert!((cfg.difficulty_range.1 - 0.8).abs() < 1e-5);
    }

    #[test]
    fn high_fear_lowers_upper_below_base() {
        // fear > 0.5 → upper -= 0.05. Lower untouched.
        let fear = NpcDisposition { friendly: 0.0, fear: 0.8, trust: 0.0 };
        let cfg = build_generation_config_with_mood(
            5, 0, &fear, GenerationHint::default(), 7,
        );
        assert!(cfg.difficulty_range.1 < 0.80,
                "expected upper < 0.80, got {}", cfg.difficulty_range.1);
        // f32 tolerance (`0.80 - 0.05` is `0.75` exactly).
        assert!((cfg.difficulty_range.0 - 0.30).abs() < 1e-5);
        assert!((cfg.difficulty_range.1 - 0.75).abs() < 1e-5);
    }

    #[test]
    fn friendly_and_trusting_raises_lower_above_base() {
        // friendly > 0.5 && trust > 0.3 → lower += 0.05. Upper untouched.
        let loved = NpcDisposition { friendly: 0.7, fear: 0.0, trust: 0.4 };
        let cfg = build_generation_config_with_mood(
            5, 0, &loved, GenerationHint::default(), 13,
        );
        assert!(cfg.difficulty_range.0 > 0.30);
        // Use tolerance for f32 arithmetic (`0.30 + 0.05` is `0.35000002`).
        assert!((cfg.difficulty_range.0 - 0.35).abs() < 1e-5,
                "expected lower ~0.35, got {}", cfg.difficulty_range.0);
        assert!((cfg.difficulty_range.1 - 0.80).abs() < 1e-5);
    }

    #[test]
    fn hated_lowers_lower_below_base() {
        // friendly < -0.3 → lower -= 0.05. Upper untouched.
        let hated = NpcDisposition { friendly: -0.5, fear: 0.0, trust: 0.0 };
        let cfg = build_generation_config_with_mood(
            5, 0, &hated, GenerationHint::default(), 21,
        );
        assert!(cfg.difficulty_range.0 < 0.30);
        assert!((cfg.difficulty_range.0 - 0.25).abs() < 1e-5);
        assert!((cfg.difficulty_range.1 - 0.80).abs() < 1e-5);
    }

    #[test]
    fn stacked_moods_clamp_to_unit_range() {
        // All three branches fire (fear + friendly<-0.3). Both lo and hi
        // adjustments are still in [-0.05, +0.05] and clamp cleanly.
        let nightmare = NpcDisposition { friendly: -1.0, fear: 1.0, trust: -1.0 };
        let cfg = build_generation_config_with_mood(
            20, 0, &nightmare, GenerationHint::default(), 99,
        );
        assert!(cfg.difficulty_range.0 >= 0.1);
        assert!(cfg.difficulty_range.1 <= 1.0);
        assert!(cfg.difficulty_range.0 <= cfg.difficulty_range.1);
    }

    #[test]
    fn seed_determinism_same_input_same_output() {
        // Same (level, losses, mood, hint, seed) must produce identical
        // output. Field-level comparison; no serde needed.
        let fear = NpcDisposition { friendly: 0.0, fear: 0.8, trust: 0.0 };
        let a = build_generation_config_with_mood(
            5, 0, &fear, GenerationHint::default(), 42,
        );
        let b = build_generation_config_with_mood(
            5, 0, &fear, GenerationHint::default(), 42,
        );
        assert_eq!(a.difficulty_range, b.difficulty_range);
        assert_eq!(a.preferred_types, b.preferred_types);
        assert_eq!(a.excluded_types, b.excluded_types);
        assert_eq!(a.player_level, b.player_level);
        assert_eq!(a.min_atoms, b.min_atoms);
        assert_eq!(a.max_atoms, b.max_atoms);
    }

    #[test]
    fn neutral_mood_does_not_promote() {
        // No mood branches fire → head of preferred_types is whatever
        // the base pool starts with for level 5 (TowerDefense).
        let cfg = build_generation_config_with_mood(
            5, 0, &neutral(), GenerationHint::default(), 42,
        );
        let base_head = default_preferred_pool(5)[0].clone();
        assert_eq!(cfg.preferred_types[0], base_head);
    }

    #[test]
    fn extreme_fear_caps_difficulty_upper() {
        // fear=1.0 (saturated) and high level — upper must still be ≤ 1.0.
        let fear = NpcDisposition { friendly: 0.0, fear: 1.0, trust: 0.0 };
        let cfg = build_generation_config_with_mood(
            20, 0, &fear, GenerationHint::default(), 1,
        );
        assert!(cfg.difficulty_range.1 <= 1.0);
        assert!(cfg.difficulty_range.0 >= 0.1);
    }

    #[test]
    fn preferred_types_are_deduped_across_promoted_and_base() {
        // For 50 different seeds, preferred_types must contain no
        // duplicates regardless of which atom each branch picks.
        let loved = NpcDisposition { friendly: 0.7, fear: 0.0, trust: 0.4 };
        for seed in 0..50 {
            let cfg = build_generation_config_with_mood(
                5, 0, &loved, GenerationHint::default(), seed,
            );
            let mut seen = std::collections::HashSet::new();
            for t in &cfg.preferred_types {
                let key = format!("{:?}", t);
                assert!(seen.insert(key.clone()),
                        "dup at seed {}: {:?} (type={:?})",
                        seed, cfg.preferred_types, t);
            }
        }
    }

    #[test]
    fn default_hint_fields() {
        // Pin the defaults so the TS layer has a stable contract.
        let h = GenerationHint::default();
        assert_eq!(h.min_atoms, 2);
        assert_eq!(h.max_atoms, 4);
        assert_eq!(h.reward_multiplier, 1.0);
        assert_eq!(h.base_difficulty_range, (0.3, 0.8));
    }

    #[test]
    fn excluded_types_drops_shooting_after_three_losses() {
        // Mirror TS: recent_loss_count >= 3 → Shooting excluded.
        let cfg = build_generation_config_with_mood(
            5, 3, &neutral(), GenerationHint::default(), 0,
        );
        assert!(cfg.excluded_types.contains(&GameplayType::Shooting));
    }

    #[test]
    fn excluded_types_empty_below_three_losses() {
        // Below 3 losses → no exclusions, even with neutral mood.
        let cfg = build_generation_config_with_mood(
            5, 2, &neutral(), GenerationHint::default(), 0,
        );
        assert!(cfg.excluded_types.is_empty());
    }
}
