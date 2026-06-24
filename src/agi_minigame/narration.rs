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
///
/// Round 53b — the AGI-miniGame TypeScript layer's
/// `NarrationEngine.narrate` average-mood fallback now uses the
/// **same FNV-1a constants** (offset basis 2166136261, prime
/// 16777619) over the same `blueprint_id` string, so the WASM
/// and TS paths produce byte-identical sentences for the same
/// input. The TS individual-NPC path (round 33) still uses a
/// `djb2` fallback because the WASM helper doesn't model
/// individual-NPC contexts yet (round-54 follow-up).
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

    // -----------------------------------------------------------------
    // Round 125 — helper-level
    // tests for the 4
    // boundary conditions
    // (mood_branch at the
    // exact threshold) + the
    // 2 back-compat single-
    // sentence accessors
    // (`mood_4th_sentence`
    // pool[0] for non-
    // neutral + the
    // 4th-sentence-for
    // empty-pool guard) +
    // fnv1a edge cases +
    // build_sentences
    // empty-base + multiple-
    // call independence.
    //
    // The pre-round-125
    // tests covered:
    //   - base_intro
    //     sentence count = 3
    //   - mood_branch for
    //     all 4 named moods
    //   - fear-takes-
    //     priority over
    //     friendly+trust
    //   - build_sentences
    //     no-mood +
    //     neutral-mood +
    //     fear/friendly/
    //     hostile happy path
    //   - mood_4th_
    //     sentence_pool
    //     has ≥3 per active
    //     branch + neutral
    //     is empty
    //   - mood_4th_
    //     sentence_for is
    //     deterministic +
    //     variety across
    //     ids + neutral
    //     branch returns
    //     None
    //   - fnv1a stability
    //     (same → same,
    //     different →
    //     different)
    //   - mood_4th_
    //     sentence neutral
    //     returns None
    //   - cross-branch
    //     priority matches
    //     mood_palette
    //
    // Round 125 closes
    // the coverage gap
    // for:
    //   - mood_branch
    //     boundary
    //     conditions
    //     (fear=0.5,
    //     friendly=0.5,
    //     trust=0.3,
    //     friendly=-0.3 —
    //     pin the
    //     `>` vs `>=`
    //     contract)
    //   - mood_tag for
    //     all 4 branches
    //     + the catch-
    //     all `_ => "neutral"`
    //     for out-of-range
    //     branches (5, 255)
    //   - mood_4th_sentence
    //     returns Some for
    //     non-neutral
    //     branches (the
    //     back-compat
    //     pool[0] accessor)
    //   - mood_4th_sentence_for
    //     with empty
    //     blueprint_id
    //     (valid input,
    //     valid pick)
    //   - fnv1a with empty
    //     string + unicode
    //     + ASCII vs
    //     non-ASCII
    //   - build_sentences
    //     with empty base
    //     vec + any mood
    //     (returns empty
    //     or 1-elem vec
    //     depending on
    //     branch)
    //   - build_sentences
    //     called twice in
    //     a row is
    //     independent
    //     (no state leak
    //     between calls)
    //   - mood_4th_
    //     sentence_for
    //     picks a member
    //     of the pool
    //     (not arbitrary
    //     text) for each
    //     active branch
    // -----------------------------------------------------------------

    #[test]
    fn mood_branch_fear_boundary_uses_strict_greater_than_round_125() {
        // Pin the `>` (not `>=`) contract: fear=0.5 does NOT fire
        // the fear branch (only fear > 0.5 does). The pre-round-125
        // tests only covered fear=0.8.
        let at_threshold = NpcDisposition { friendly: 0.0, fear: 0.5, trust: 0.0 };
        assert_eq!(mood_branch(&at_threshold), 3, "fear=0.5 should NOT fire fear branch (uses `>`)");
        // fear=0.5001 → fires fear.
        let just_above = NpcDisposition { friendly: 0.0, fear: 0.5001, trust: 0.0 };
        assert_eq!(mood_branch(&just_above), 0, "fear=0.5001 should fire fear branch");
    }

    #[test]
    fn mood_branch_friendly_boundary_uses_strict_greater_than_round_125() {
        // Pin the `friendly > 0.5` contract: friendly=0.5 does
        // NOT fire the friendly+trust branch (would need trust
        // > 0.3 too, but the friendly gate fails first).
        let at_threshold = NpcDisposition { friendly: 0.5, fear: 0.0, trust: 0.5 };
        assert_eq!(mood_branch(&at_threshold), 3, "friendly=0.5 should NOT fire friendly+trust (uses `>`)");
        let just_above = NpcDisposition { friendly: 0.5001, fear: 0.0, trust: 0.5 };
        assert_eq!(mood_branch(&just_above), 1, "friendly=0.5001 should fire friendly+trust");
    }

    #[test]
    fn mood_branch_trust_boundary_uses_strict_greater_than_round_125() {
        // Pin the `trust > 0.3` contract: trust=0.3 does NOT
        // fire the friendly+trust branch (the trust gate fails
        // even when friendly > 0.5).
        let at_threshold = NpcDisposition { friendly: 0.7, fear: 0.0, trust: 0.3 };
        assert_eq!(mood_branch(&at_threshold), 3, "trust=0.3 should NOT fire friendly+trust (uses `>`)");
        let just_above = NpcDisposition { friendly: 0.7, fear: 0.0, trust: 0.3001 };
        assert_eq!(mood_branch(&just_above), 1, "trust=0.3001 should fire friendly+trust");
    }

    #[test]
    fn mood_branch_hostile_boundary_uses_strict_less_than_round_125() {
        // Pin the `friendly < -0.3` contract: friendly=-0.3 does
        // NOT fire the hostile branch.
        let at_threshold = NpcDisposition { friendly: -0.3, fear: 0.0, trust: 0.0 };
        assert_eq!(mood_branch(&at_threshold), 3, "friendly=-0.3 should NOT fire hostile (uses `<`)");
        let just_below = NpcDisposition { friendly: -0.3001, fear: 0.0, trust: 0.0 };
        assert_eq!(mood_branch(&just_below), 2, "friendly=-0.3001 should fire hostile");
    }

    #[test]
    fn mood_tag_returns_canonical_label_for_all_4_branches_round_125() {
        // The pre-round-125 tests only checked mood_tag
        // transitively (inside the mood_branch tests). Round 125
        // pins mood_tag directly for all 4 branches.
        assert_eq!(mood_tag(0), "fear");
        assert_eq!(mood_tag(1), "friendly");
        assert_eq!(mood_tag(2), "hostile");
        assert_eq!(mood_tag(3), "neutral");
    }

    #[test]
    fn mood_tag_out_of_range_branch_falls_back_to_neutral_round_125() {
        // The match arm `_ => "neutral"` is the catch-all for any
        // u8 value outside 0..=3 (e.g. 4, 5, 100, 255). Pin the
        // contract.
        assert_eq!(mood_tag(4), "neutral");
        assert_eq!(mood_tag(5), "neutral");
        assert_eq!(mood_tag(100), "neutral");
        assert_eq!(mood_tag(255), "neutral");
    }

    #[test]
    fn mood_4th_sentence_back_compat_returns_pool_first_round_125() {
        // The pre-round-125 test only checked the neutral
        // branch (returns None). Round 125 pins the
        // non-neutral branches: pool[0] is returned.
        assert_eq!(mood_4th_sentence(0).unwrap(), mood_4th_sentence_pool(0)[0]);
        assert_eq!(mood_4th_sentence(1).unwrap(), mood_4th_sentence_pool(1)[0]);
        assert_eq!(mood_4th_sentence(2).unwrap(), mood_4th_sentence_pool(2)[0]);
        // Neutral → None.
        assert!(mood_4th_sentence(3).is_none());
    }

    #[test]
    fn mood_4th_sentence_for_empty_blueprint_id_is_valid_round_125() {
        // Defense: a regression that early-returned on empty
        // string would silently drop the 4th sentence for
        // blueprints with no id. Empty string is a valid
        // blueprint_id (FNV-1a returns 2166136261 = the offset
        // basis; pool[2166136261 % 4] is a valid pick).
        let s = mood_4th_sentence_for(0, "");
        assert!(s.is_some());
        // Pick must be a member of the fear pool.
        let pool = mood_4th_sentence_pool(0);
        assert!(pool.contains(&s.unwrap()));
    }

    #[test]
    fn fnv1a_empty_string_returns_offset_basis_round_125() {
        // FNV-1a of "" is the 32-bit offset basis (2166136261).
        // This is the canonical "empty string" check from the
        // FNV reference test vectors.
        assert_eq!(fnv1a(""), 2166136261);
    }

    #[test]
    fn fnv1a_unicode_stable_and_distinct_round_125() {
        // Unicode input must hash deterministically and
        // produce different hashes for different strings.
        let a = fnv1a("维度_α");
        let b = fnv1a("维度_α");
        assert_eq!(a, b, "same unicode input should produce same hash");
        // Different unicode strings → different hashes (with
        // overwhelming probability).
        assert_ne!(fnv1a("α"), fnv1a("β"));
        assert_ne!(fnv1a("中"), fnv1a("文"));
    }

    #[test]
    fn build_sentences_empty_base_with_fear_mood_returns_one_4th_sentence_round_125() {
        // An empty base vec with a fear mood still appends
        // a 4th sentence (the mood branch's pool[hash % len]).
        // Total length = 1.
        let out = build_sentences(vec![], Some(&fear_mood()), "dim_x");
        assert_eq!(out.len(), 1);
        let pool = mood_4th_sentence_pool(0);
        assert!(pool.contains(&out[0].as_str()));
    }

    #[test]
    fn build_sentences_empty_base_with_neutral_mood_returns_empty_round_125() {
        // Empty base + neutral mood → no 4th sentence
        // appended → empty result.
        let out = build_sentences(vec![], Some(&neutral_mood()), "dim_x");
        assert!(out.is_empty());
    }

    #[test]
    fn build_sentences_two_sequential_calls_are_independent_round_125() {
        // Defense: a regression that cached the 4th-sentence
        // pick across calls would cause the second
        // build_sentences to ignore its base + mood inputs.
        // Pin the independence: 2 calls with different inputs
        // produce different outputs.
        let base1 = vec!["a1".to_string(), "b1".to_string()];
        let base2 = vec!["a2".to_string(), "b2".to_string()];
        let out1 = build_sentences(base1.clone(), Some(&fear_mood()), "dim_x");
        let out2 = build_sentences(base2.clone(), Some(&loved_mood()), "dim_y");
        // Each call returns its OWN base + 4th (no cross-
        // contamination).
        assert_eq!(out1[0], "a1");
        assert_eq!(out1[1], "b1");
        assert_eq!(out2[0], "a2");
        assert_eq!(out2[1], "b2");
        // Lengths are both 3 (base 2 + 1 fourth).
        assert_eq!(out1.len(), 3);
        assert_eq!(out2.len(), 3);
    }

    #[test]
    fn mood_4th_sentence_for_active_branch_picks_pool_member_round_125() {
        // Pin the contract: every active branch's
        // mood_4th_sentence_for(...) returns a member of
        // that branch's pool (not arbitrary text). The
        // pre-round-125 tests only checked the variety +
        // neutrality paths.
        for branch in 0u8..=2 {
            let pool = mood_4th_sentence_pool(branch);
            // Try 10 different blueprint ids — each must
            // return a pool member.
            for i in 0..10 {
                let id = format!("dim_test_{branch}_{i}");
                let s = mood_4th_sentence_for(branch, &id).unwrap();
                assert!(
                    pool.contains(&s),
                    "branch {branch} id {id} returned {s} which is not in the pool",
                );
            }
        }
    }

    // -----------------------------------------------------------------
    // Round 168 — FNV-1a known-vector + cross-validation tests.
    // The Rust `fnv1a` is the 32-bit cousin of the 64-bit
    // `seed_from_string` in `dsl::codegen`. They share the FNV-1a
    // algorithm (offset basis xor byte, then multiply by the prime)
    // but at different widths. These tests pin the canonical
    // 32-bit FNV-1a reference test vectors so a regression in
    // the algorithm (e.g. accidentally using the FNV-1 32-bit
    // multiply-before-xor order) fails loudly.
    //
    // Reference: http://www.isthe.com/chongo/tech/comp/fnv/
    // -----------------------------------------------------------------

    #[test]
    fn fnv1a_known_vector_a_round_168() {
        // FNV-1a 32-bit of "a" is 0xE40C292C = 3826002220.
        // This is the canonical reference vector from the FNV
        // reference test suite. A regression to FNV-1 (multiply
        // before xor) would return 0x4D2505CA.
        assert_eq!(fnv1a("a"), 0xE40C292C);
    }

    #[test]
    fn fnv1a_known_vector_foobar_round_168() {
        // FNV-1a 32-bit of "foobar" is 0xBF9CF968 = 3215606632.
        assert_eq!(fnv1a("foobar"), 0xBF9CF968);
    }

    #[test]
    fn fnv1a_known_vector_cur_chonk_round_168() {
        // FNV-1a 32-bit of "cur chonk" is 0xCCB5DB52 = 3435083602
        // (computed via the round-168 reference test). This
        // pin keeps the algorithm byte-exact — a regression
        // that swapped xor↔mul would land on a different
        // value.
        assert_eq!(fnv1a("cur chonk"), 3435083602);
    }

    #[test]
    fn fnv1a_single_byte_inputs_round_168() {
        // Pin several single-byte ASCII values to lock the
        // 32-bit FNV-1a algorithm to byte-exact outputs. A
        // regression that swapped xor↔mul or miscalculated the
        // prime would fail at least one of these. (The empty-
        // string case — `""` returning the offset basis — is
        // already covered by `fnv1a_empty_string_returns_offset
        // _basis_round_125`.)
        assert_eq!(fnv1a("\0"), 84696351, "null-byte input");
        // 'a' / 'b' / 'c' are the FNV reference test vectors
        // (pinned to the actual byte-exact outputs of the
        // round-168 implementation; same algorithm, same
        // offset basis, same prime — these constants must
        // not drift across refactors).
        assert_eq!(fnv1a("b"), 3876335077);
        assert_eq!(fnv1a("c"), 3859557458);
    }

    #[test]
    fn fnv1a_is_collision_resistant_for_similar_inputs_round_168() {
        // Two strings that differ by ONE character must produce
        // DIFFERENT hashes (avalanche property). Pin a few pairs.
        assert_ne!(fnv1a("dim_alpha"), fnv1a("dim_bravo"));
        assert_ne!(fnv1a("dim_alpha"), fnv1a("dim_alpga")); // typo swap
        assert_ne!(fnv1a("foo"), fnv1a("foo ")); // trailing space
        assert_ne!(fnv1a("foo"), fnv1a(" foo")); // leading space
    }

    // -----------------------------------------------------------------
    // Round 168 — `mood_4th_sentence_for` determinism + pool coverage.
    // The deterministic pick uses `fnv1a(blueprint_id) % pool.len()`,
    // so the SAME blueprint_id always returns the SAME pool entry.
    // A regression that added non-determinism (e.g. used a thread-
    // local rng) would break the round-72 save round-trip
    // stability (re-entering a dimension should produce the same
    // 4th sentence each time).
    // -----------------------------------------------------------------

    #[test]
    fn mood_4th_sentence_for_is_deterministic_across_calls_round_168() {
        // Same branch + same blueprint_id → same pick, every time.
        let first = mood_4th_sentence_for(0, "dim_determinism");
        for _ in 0..100 {
            assert_eq!(
                mood_4th_sentence_for(0, "dim_determinism"),
                first,
                "mood_4th_sentence_for must be deterministic for the same inputs"
            );
        }
    }

    #[test]
    fn mood_4th_sentence_for_distinct_ids_cover_distinct_picks_round_168() {
        // For a pool of size N, sampling N distinct ids must
        // cover (with high probability) most of the pool. A
        // regression that always returned pool[0] would fail
        // here (only 1 unique value across N samples).
        let pool = mood_4th_sentence_pool(1); // "friendly" pool, size 5
        let mut seen = std::collections::HashSet::new();
        for i in 0..16 {
            let id = format!("dim_distinct_{i}");
            let s = mood_4th_sentence_for(1, &id).unwrap();
            seen.insert(s);
        }
        // At least 3 distinct picks across 16 ids (collisions
        // are possible but the birthday-paradox floor is high).
        assert!(
            seen.len() >= 3,
            "expected at least 3 distinct picks across 16 ids, got {}",
            seen.len()
        );
        // Every picked value must be in the pool.
        for s in &seen {
            let s_str: &str = s.as_ref();
            assert!(
                pool.contains(&s_str),
                "picked value {s:?} is not in the pool"
            );
        }
    }

    #[test]
    fn mood_4th_sentence_for_empty_id_falls_inside_pool_round_168() {
        // Empty blueprint_id must still produce a valid pick
        // (the FNV-1a of "" is the offset basis, which is a
        // deterministic non-zero value — the mod will land on
        // some valid pool entry).
        let s = mood_4th_sentence_for(0, "").unwrap();
        let pool = mood_4th_sentence_pool(0);
        assert!(pool.contains(&s), "empty id must pick from the pool, got {s:?}");
    }
}
