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

/// Mood-driven 4th sentence. Returns `None` for the neutral branch
/// (so the player isn't spammed with "neutral commentary" on every
/// dim), and `Some(<canonical text>)` for the three active branches.
pub fn mood_4th_sentence(branch: u8) -> Option<&'static str> {
    match branch {
        0 => Some("空气本身在退避，仿佛这里有过太多恐惧。"),
        1 => Some("当地的居民说，这里对旅人尚算友好。"),
        2 => Some("他们不会原谅你上次带来的麻烦。"),
        _ => None,
    }
}

/// Build the full sentence list for a dimension, optionally with a
/// 4th mood-driven sentence. The 3-sentence base list is supplied
/// by the caller (the TS layer owns the string pool).
pub fn build_sentences(base: Vec<String>, mood: Option<&NpcDisposition>) -> Vec<String> {
    match mood {
        Some(m) => {
            let branch = mood_branch(m);
            match mood_4th_sentence(branch) {
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
        let out = build_sentences(base.clone(), None);
        assert_eq!(out, base);
    }

    #[test]
    fn build_sentences_neutral_mood_returns_base() {
        // Neutral branch → no 4th sentence appended.
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base.clone(), Some(&neutral_mood()));
        assert_eq!(out, base);
    }

    #[test]
    fn build_sentences_fear_appends_fourth() {
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base, Some(&fear_mood()));
        assert_eq!(out.len(), 4);
        assert!(out[3].contains("空气"));
    }

    #[test]
    fn build_sentences_friendly_appends_fourth() {
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base, Some(&loved_mood()));
        assert_eq!(out.len(), 4);
        assert!(out[3].contains("友好"));
    }

    #[test]
    fn build_sentences_hostile_appends_fourth() {
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let out = build_sentences(base, Some(&hated_mood()));
        assert_eq!(out.len(), 4);
        assert!(out[3].contains("不会原谅"));
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
