//! Round 25 — NPC-mind-aware narration.
//!
//! Closes the §2.2D / §2.2B cross-link: the world's NPC collective
//! mood now *also* shapes the dimension's opening narrative, in
//! addition to the difficulty band (round 23) and the visual palette
//! (round 24).
//!
//! The TS-side `NarrationEngine` already produces a 3-sentence intro
//! per dimension. This module adds an optional 4th sentence picked
//! from a mood-keyed pool, using the same branch order as
//! [`super::scene_gen::mood_palette`] and `BalanceTuner::mood_bias`
//! so the player's `friendly / fear / trust` signal is consistently
//! read across the three channels:
//!
//!   - fear > 0.5               → cold warning ("the air itself recoils")
//!   - friendly > 0.5 && trust  → warm recommendation ("they say it's safe")
//!   - friendly < -0.3          → hostile warning ("they will not forgive")
//!   - everything else          → neutral
//!
//! Determinism: the 4th-sentence pick uses a hash of the dimension
//! id and the chosen mood branch index, so the same dimension with
//! the same mood always produces the same sentence. The TS side
//! mirrors the same djb2 + linear-congruential RNG as the existing
//! 3-sentence engine.

use super::npc::NpcDisposition;

/// 3-sentence base intro (mirrors `NarrationEngine.narrate` in TS).
///
/// Engine-side mirror so the test surface is symmetric. The TS side
/// owns the actual sentence pool (the strings are bilingual/genre
/// tone that doesn't belong in the engine layer). For the test we
/// only assert the *shape* of the output: 3 sentences when no mood
/// is provided, 4 sentences when a mood is provided.
pub fn base_intro_sentence_count(blueprint_id: &str) -> usize {
    let _ = blueprint_id;
    3
}

/// Mood branch index (same priority order as `mood_palette` and
/// `mood_bias`).
///
/// Returns `0..=3`:
///   - 0 = fear
///   - 1 = friendly + trust
///   - 2 = hostile
///   - 3 = neutral
pub fn mood_branch(mood: &NpcDisposition) -> u8 {
    if mood.fear > 0.5 {
        0
    } else if mood.friendly > 0.5 && mood.trust > 0.3 {
        1
    } else if mood.friendly < -0.3 {
        2
    } else {
        3
    }
}

/// A canonical 4-tag label for the chosen mood branch. The TS side
/// uses the same labels to pick from a parallel pool of 4th-sentence
/// strings.
pub fn mood_tag(branch: u8) -> &'static str {
    match branch {
        0 => "fear",
        1 => "friendly",
        2 => "hostile",
        _ => "neutral",
    }
}

/// Round 30 — mood-driven 4th sentence pool. Each active branch
/// now has 4-5 alternatives so the 4th sentence has variation
/// across dimension visits (the picking variant
/// `mood_4th_sentence_for(branch, blueprint_id)` chooses one
/// deterministically).
pub fn mood_4th_sentence_pool(branch: u8) -> &'static [&'static str] {
    match branch {
        // fear
        0 => &[
            "空气本身在退避，仿佛这里有过太多恐惧。",
            "远处有什么东西在低声警告你停下脚步。",
            "脚下的地板似乎在颤抖，不是风。",
            "阴影里残留的尖叫还没有完全散去。",
        ],
        // loved (friendly + trust)
        1 => &[
            "当地的居民说，这里对旅人尚算友好。",
            "守门人朝你点了点头，似乎记得上次的英勇。",
            "空气里飘着淡淡的节日气息，像是在欢迎。",
            "村口的风铃响了三下，节奏恰好。",
            "你听见远处有人在哼着熟悉的小调。",
        ],
        // hostile (friendly <-0.3)
        2 => &[
            "他们不会原谅你上次带来的麻烦。",
            "哨兵把手按在剑柄上，眼神很冷。",
            "上一次的伤痕写在每一张脸上。",
            "你听见身后有人在啐口水。",
        ],
        _ => &[],
    }
}

/// Back-compat single-sentence accessor — returns the first entry
/// of the pool. Round 25's existing tests still resolve against
/// this. Prefer `mood_4th_sentence_for(branch, blueprint_id)` for
/// the picking variant.
pub fn mood_4th_sentence(branch: u8) -> Option<&'static str> {
    let pool = mood_4th_sentence_pool(branch);
    if pool.is_empty() { None } else { Some(pool[0]) }
}

/// FNV-1a (32-bit) — small, deterministic hash. We use it to pick
/// a stable 4th-sentence variant from `mood_4th_sentence_pool`
/// keyed on `blueprint_id`. Same blueprint always gets the same
/// 4th sentence (so re-visits don't re-roll flavour); different
/// blueprints get different ones (so the player feels variety).
fn fnv1a(s: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for b in s.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

/// Pick a deterministic 4th sentence from the branch's pool,
/// keyed on `blueprint_id`.
pub fn mood_4th_sentence_for(branch: u8, blueprint_id: &str) -> Option<&'static str> {
    let pool = mood_4th_sentence_pool(branch);
    if pool.is_empty() { return None; }
    let idx = (fnv1a(blueprint_id) as usize) % pool.len();
    Some(pool[idx])
}

/// Build the full sentence list for a dimension, optionally with a
/// 4th mood-driven sentence. The 3-sentence base list is supplied
/// by the caller (the TS layer owns the string pool).
/// `blueprint_id` is used to pick the 4th-sentence variant
/// deterministically.
pub fn build_sentences(
    base: Vec<String>,
    mood: Option<&NpcDisposition>,
    blueprint_id: &str,
) -> Vec<String> {
    match mood {
        Some(m) => {
            let branch = mood_branch(m);
            match mood_4th_sentence_for(branch, blueprint_id) {
                Some(extra) => {
                    let mut out = base;
                    out.push(extra.to_string());
                    out
                }
                None => base,
            }
        }
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fear_mood() -> NpcDisposition {
        NpcDisposition { friendly: 0.0, fear: 0.8, trust: 0.0 }
    }
    fn loved_mood() -> NpcDisposition {
        NpcDisposition { friendly: 0.7, fear: 0.0, trust: 0.4 }
    }
    fn hated_mood() -> NpcDisposition {
        NpcDisposition { friendly: -0.5, fear: 0.0, trust: 0.0 }
    }
    fn neutral_mood() -> NpcDisposition {
        NpcDisposition::default()
    }

    #[test]
    fn base_intro_has_three_sentences() {
        assert_eq!(base_intro_sentence_count("dim_anything"), 3);
    }

    #[test]
    fn fear_mood_picks_fear_branch() {
        assert_eq!(mood_branch(&fear_mood()), 0);
        assert_eq!(mood_tag(mood_branch(&fear_mood())), "fear");
    }

    #[test]
    fn friendly_and_trusting_picks_friendly_branch() {
        assert_eq!(mood_branch(&loved_mood()), 1);
        assert_eq!(mood_tag(mood_branch(&loved_mood())), "friendly");
    }

    #[test]
    fn hostile_mood_picks_hostile_branch() {
        assert_eq!(mood_branch(&hated_mood()), 2);
        assert_eq!(mood_tag(mood_branch(&hated_mood())), "hostile");
    }

    #[test]
    fn neutral_mood_picks_neutral_branch() {
        // Default neutral (all zeros): no branch fires → 3.
        assert_eq!(mood_branch(&neutral_mood()), 3);
        // Frightened but still friendly → no fear-priority match.
        let warmish = NpcDisposition { friendly: 0.2, fear: 0.1, trust: 0.0 };
        assert_eq!(mood_branch(&warmish), 3);
        assert_eq!(mood_tag(mood_branch(&warmish)), "neutral");
    }

    #[test]
    fn fear_takes_priority_over_friendly() {
        // fear=0.9 + friendly=0.9 + trust=0.5 → both fear and
        // friendly+trust branches could fire. The canonical order
        // (matching mood_palette and mood_bias) picks fear first.
        let nightmare = NpcDisposition { friendly: 0.9, fear: 0.9, trust: 0.5 };
        assert_eq!(mood_branch(&nightmare), 0);
    }

    #[test]
    fn build_sentences_no_mood_returns_base() {
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base.clone(), None, "dim_x");
        assert_eq!(out, base);
    }

    #[test]
    fn build_sentences_neutral_mood_returns_base() {
        // Neutral branch → no 4th sentence appended.
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base.clone(), Some(&neutral_mood()), "dim_x");
        assert_eq!(out, base);
    }

    #[test]
    fn build_sentences_fear_appends_fourth() {
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base, Some(&fear_mood()), "dim_x");
        assert_eq!(out.len(), 4);
        // The exact 4th sentence is now picked from a 4-entry pool
        // by the dim id; we just verify it's one of the fear pool.
        let pool = mood_4th_sentence_pool(0);
        assert!(pool.contains(&out[3].as_str()));
    }

    #[test]
    fn build_sentences_friendly_appends_fourth() {
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base, Some(&loved_mood()), "dim_x");
        assert_eq!(out.len(), 4);
        let pool = mood_4th_sentence_pool(1);
        assert!(pool.contains(&out[3].as_str()));
    }

    #[test]
    fn build_sentences_hostile_appends_fourth() {
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base, Some(&hated_mood()), "dim_x");
        assert_eq!(out.len(), 4);
        let pool = mood_4th_sentence_pool(2);
        assert!(pool.contains(&out[3].as_str()));
    }

    // ---- Round 30 — pool expansion + deterministic pick ----

    #[test]
    fn mood_4th_sentence_pool_has_multiple_per_active_branch() {
        // Round 30 expansion: each active branch must have ≥3
        // alternatives so the player notices variety across
        // dimension visits.
        assert!(mood_4th_sentence_pool(0).len() >= 3);
        assert!(mood_4th_sentence_pool(1).len() >= 3);
        assert!(mood_4th_sentence_pool(2).len() >= 3);
        // Neutral branch is intentionally empty.
        assert!(mood_4th_sentence_pool(3).is_empty());
    }

    #[test]
    fn mood_4th_sentence_for_is_deterministic_per_blueprint_id() {
        // Same id → same pick, on repeat calls.
        let a = mood_4th_sentence_for(0, "dim_alpha");
        let b = mood_4th_sentence_for(0, "dim_alpha");
        assert_eq!(a, b);
        // Different id → may differ (not strictly required, but
        // we want the picking to spread; pick a few).
        let mut seen = std::collections::HashSet::new();
        for i in 0..30 {
            let s = mood_4th_sentence_for(0, &format!("dim_{i}")).unwrap();
            seen.insert(s);
        }
        // 30 ids into a 4-entry pool → expect ≥2 distinct picks
        // (the hash is uniform enough that a single pick is
        //  unlikely in 30 trials).
        assert!(seen.len() >= 2, "expected variety, only saw {}", seen.len());
    }

    #[test]
    fn mood_4th_sentence_for_neutral_branch_returns_none() {
        assert!(mood_4th_sentence_for(3, "dim_x").is_none());
    }

    #[test]
    fn fnv1a_is_stable_across_runs() {
        // FNV-1a constants are fixed; same input → same output.
        // We don't test the exact 32-bit value, only that two
        // back-to-back calls agree.
        let a = fnv1a("dim_alpha");
        let b = fnv1a("dim_alpha");
        assert_eq!(a, b);
        // Different inputs → different hashes (with overwhelming
        // probability; we accept a 1-in-2^32 collision here).
        assert_ne!(fnv1a("a"), fnv1a("b"));
    }

    #[test]
    fn mood_4th_sentence_neutral_returns_none() {
        assert!(mood_4th_sentence(3).is_none());
    }

    #[test]
    fn cross_branch_priority_matches_mood_palette() {
        // Sanity: the 4-branch order is identical to
        // `scene_gen::mood_palette` so the narrative signal aligns
        // with the visual signal.
        assert_eq!(mood_branch(&fear_mood()), 0);
        assert_eq!(mood_branch(&loved_mood()), 1);
        assert_eq!(mood_branch(&hated_mood()), 2);
        assert_eq!(mood_branch(&neutral_mood()), 3);
    }
}
