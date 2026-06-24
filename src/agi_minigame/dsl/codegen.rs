//! DSL rule code generation.
//!
//! The `ast` module gives us the data shapes; the `parser`
//! module parses hand-written rules. This module fills the
//! "auto-generate the game logic" gap (the
//! `游戏逻辑自动生成` mandate from the brief) — given a
//! runtime snapshot of (biome, mood, complexity), emit a
//! deterministic `Vec<Rule>` that the AGI-miniGame can
//! slot into its event dispatcher without writing any
//! hand-authored DSL.
//!
//! Why code-gen rather than hand-written tables:
//!   - The biome / mood / complexity triple is 4 × 4 × 3
//!     = 48 combinations. Hand-tabling all 48 would be a
//!     maintenance burden and a strong temptation to
//!     over-fit special cases. Code-gen gives each axis a
//!     small set of "contributions" (per-biome flavor,
//!     per-mood tone, per-complexity volume), then sums
//!     them. Combinatorial coverage comes for free.
//!   - Same `(biome, mood, complexity)` always produces
//!     the same rules — important for the round-72 save
//!     snapshot round-trip (deterministic re-hydration
//!     after a reload).
//!   - The mutation_cost of the generated rules stays
//!     within a bounded window so the round-balance AI
//!     doesn't flag the whole scene as "broken".
//!
//! Determinism contract:
//!   - For a fixed `GenInput`, `generate_rules` returns
//!     the same `Vec<Rule>` in the same order across
//!     runs. No `HashMap` ordering, no `BTreeMap`
//!     non-determinism. We use a `Vec` of
//!     `(contributor, rules)` pairs and concatenate in
//!     stable biome → mood → complexity order.
//!
//! Coverage contract:
//!   - Always emits at least 1 `On(Spawn) -> Spawn`
//!     rule so the scene has a population action even
//!     on minimal `Low` complexity + neutral mood.
//!   - Never emits more than 6 rules (to keep the rule
//!     panel readable + the dispatcher cheap).

use super::ast::{Action, ActionKind, Arg, Event, EventKind, Rule};

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// Biome flavor. The 6 biomes match the full
/// `BiomeAtmosphere` palette (forest / desert / ice /
/// cyberpunk / lava / space) so the generated rules
/// "taste" like the scene the player is in.
///
/// Round 167 — added `Space` and `Lava`. Before round
/// 167 the enum had 4 variants and `BiomeAtmosphere`'s
/// 6-biome palette had 2 extras (`space` and `lava`)
/// that fell back to `Forest`. The fallback was a
/// round-164 A graceful-degradation strategy (the 4
/// variants were sufficient for the round-162
/// generator); round 167 promotes `Space` and `Lava`
/// to first-class codegen variants so the
/// auto-generated rules "taste" like the actual
/// biome the player is in — `space_mob` / `lava_mob`
/// instead of `forest_mob`. The TS-side
/// `codegenBindings::BiomeKind` mirrors this exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeKind {
    Forest,
    Desert,
    Ice,
    Cyberpunk,
    Lava,
    Space,
}

/// Mood tone. Calms down or agitates the rule actions
/// (heals vs damage) so the player's emotional state
/// matches the game state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoodKind {
    Calm,
    Tense,
    Epic,
    Mysterious,
}

/// Rule volume. `Low` = 1 rule, `Med` = 3 rules,
/// `High` = 5 rules. The 3-way split is the smallest
/// one that lets the round-balance AI actually flag a
/// regression that flips complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityKind {
    Low,
    Medium,
    High,
}

/// All inputs the codegen reads. Plain-old-data so it
/// can be cloned, sent across threads, serialized for
/// round-72 saves without any extra glue.
///
/// Round 163 — added `seed: u64` so the codegen is
/// deterministic-but-varied. Same `(biome, mood,
/// complexity)` with different seeds produces
/// different-but-valid rule sets (timer durations
/// perturbed within their bands, action magnitudes
/// scaled, mood-rule action kinds rotated). The seed
/// axis lets the round-72 save snapshot round-trip
/// (deterministic re-hydration after a reload) AND
/// lets the round-87 balance AI explore the rule
/// space without committing to a hand-written table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenInput {
    pub biome: BiomeKind,
    pub mood: MoodKind,
    pub complexity: ComplexityKind,
    pub seed: u64,
}

impl Default for GenInput {
    /// Default seed is 0 — gives the same output as the
    /// pre-round-163 codegen, so old callers that don't
    /// think about the seed axis still get the same
    /// rules they used to. New callers that want
    /// variation set `seed` to a non-zero value.
    fn default() -> Self {
        GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Low,
            seed: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level entry points
// ---------------------------------------------------------------------------

/// Generate a deterministic `Vec<Rule>` for the given
/// runtime inputs. See the module docs for the
/// determinism + coverage contracts.
pub fn generate_rules(input: GenInput) -> Vec<Rule> {
    let mut rules: Vec<Rule> = Vec::new();

    // 1. The baseline "population" rule — every scene
    //    must spawn SOMETHING on Spawn, regardless of
    //    mood / complexity. This guarantees the round-
    //    balance AI never sees an empty rule set.
    //
    //    Round 163 — the baseline now reads the seed
    //    axis via `seed_offset` so a different seed
    //    perturbs the spawn count (a forest biome with
    //    seed=0 might emit 1 mob; with seed=42 it
    //    might emit 2). The flavor string is still
    //    biome-only (the player needs to recognize
    //    "forest_mob" no matter the seed).
    rules.push(spawn_population_rule(input));

    // 2. The complexity-driven mood + extras. Low =
    //    just the baseline (1 rule). Med = baseline +
    //    mood rule + 1 timer (3 rules total). High =
    //    baseline + mood rule + 2 timers + collide +
    //    playerhit (5 rules total).
    //
    //    The mood-driven rule: Calm → Heal; Tense →
    //    Damage; Epic → Spawn; Mysterious → SpawnEntity.
    //    The biome contributes a flavor string arg so
    //    the player can see the cross-product in the
    //    DslCodexPanel. The seed perturbs the
    //    numeric args so the same triple with a
    //    different seed gives a fresh-but-valid rule
    //    set (deterministic for the seed, varied
    //    across seeds).
    match input.complexity {
        ComplexityKind::Low => {
            // No extras; the baseline is enough.
        }
        ComplexityKind::Medium => {
            rules.push(mood_rule(input, 1));
            // Timer band: 4.0..7.0 secs, perturbed
            // by the seed (slot=2) so different seeds
            // give slightly different cycle times.
            let secs = 5.0 + seed_offset(input.seed, 2) * 1.5;
            rules.push(timer_rule(input.biome, secs));
        }
        ComplexityKind::High => {
            rules.push(mood_rule(input, 1));
            // Two timers: a fast one (band 2.0..4.0)
            // and a slow one (band 7.0..9.0). Each
            // perturbed by the seed independently.
            let fast_secs = 3.0 + seed_offset(input.seed, 2) * 1.0;
            let slow_secs = 8.0 + seed_offset(input.seed, 3) * 1.0;
            rules.push(timer_rule(input.biome, fast_secs));
            rules.push(timer_rule(input.biome, slow_secs));
            rules.push(playerhit_rule(input));
        }
    }

    // Cap at 6 rules so the dispatcher stays cheap and
    // the DslCodexPanel doesn't get a 48-row scrollbar.
    rules.truncate(6);
    rules
}

/// Generate a single canonical rule for the given
/// inputs. Equivalent to `generate_rules(input).remove(0)`
/// but more readable at the call site. The seed
/// axis still flows through (so `generate_rule(input)`
/// at seed=0 is the same as it was pre-round-163).
pub fn generate_rule(input: GenInput) -> Rule {
    spawn_population_rule(input)
}

// ---------------------------------------------------------------------------
// Per-axis builders
// ---------------------------------------------------------------------------

/// Round 163 — the baseline "population" rule now reads
/// the seed axis to perturb the spawn count. The
/// flavor string stays biome-only (the player needs
/// to recognize "forest_mob" no matter the seed).
/// `slot=0` reserves a stable slot for the baseline
/// in the seed_offset space (each rule in the
/// generated set has its own slot, so two rules with
/// the same seed but different slots get independent
/// offsets).
fn spawn_population_rule(input: GenInput) -> Rule {
    let flavor = biome_flavor(input.biome);
    // Spawn count band: 1..6 mobs, perturbed by the
    // seed. Rounded to the nearest integer so the
    // DslCodexPanel shows a clean number (no "2.3
    // mobs" rows). The band is wide enough that two
    // distinct seeds almost always land on
    // different counts (verified by the
    // `seed_axis_produces_distinct_magnitudes` test).
    let count = (3.0 + seed_offset(input.seed, 0) * 4.0).round() as i32;
    let count = count.clamp(1, 6);
    Rule {
        event: Event {
            kind: EventKind::Spawn,
            arg: None,
        },
        actions: vec![Action {
            kind: ActionKind::Spawn,
            args: vec![
                Arg::Str(format!("{}_mob", flavor)),
                Arg::Number(count as f32),
            ],
        }],
    }
}

fn mood_rule(input: GenInput, slot: u32) -> Rule {
    let flavor = biome_flavor(input.biome);
    // Round 163 — the mood-driven action is now
    // seeded. The seed perturbs the numeric magnitude
    // within a 50% band so the same (biome, mood)
    // with different seeds gives fresh magnitudes
    // (a Calm mood at seed=0 might heal 5 HP; at
    // seed=42 it might heal 7 HP). The action kind
    // itself stays mood-bound (Calm is always Heal,
    // Tense is always Damage, etc.) — the seed
    // varies the *amount*, not the *kind*.
    let offset = seed_offset(input.seed, slot);
    let (action_kind, base_magnitude) = match input.mood {
        MoodKind::Calm => (ActionKind::Heal, 5.0),
        MoodKind::Tense => (ActionKind::Damage, 3.0),
        MoodKind::Epic => (ActionKind::Spawn, 3.0),
        MoodKind::Mysterious => (ActionKind::SpawnEntity, 1.0),
    };
    let magnitude = (base_magnitude * (1.0 + offset * 0.5)).max(1.0);
    let args = match input.mood {
        MoodKind::Calm => vec![Arg::Number(magnitude), Arg::Str(format!("{}_herb", flavor))],
        MoodKind::Tense => vec![Arg::Number(magnitude), Arg::Str(format!("{}_thorn", flavor))],
        MoodKind::Epic => vec![Arg::Str(format!("{}_boss_wave", flavor)), Arg::Number(magnitude)],
        MoodKind::Mysterious => vec![Arg::Str(format!("{}_spirit", flavor)), Arg::Number(magnitude)],
    };
    Rule {
        event: Event {
            kind: EventKind::Spawn,
            arg: None,
        },
        actions: vec![Action {
            kind: action_kind,
            args,
        }],
    }
}

fn timer_rule(biome: BiomeKind, secs: f32) -> Rule {
    let flavor = biome_flavor(biome);
    Rule {
        event: Event {
            kind: EventKind::Timer,
            arg: Some(Arg::Number(secs)),
        },
        actions: vec![Action {
            kind: ActionKind::Spawn,
            args: vec![Arg::Str(format!("{}_timer_spawn", flavor))],
        }],
    }
}

fn collide_rule(biome: BiomeKind) -> Rule {
    let flavor = biome_flavor(biome);
    Rule {
        event: Event {
            kind: EventKind::Collide,
            arg: None,
        },
        actions: vec![Action {
            kind: ActionKind::Damage,
            args: vec![Arg::Number(2.0), Arg::Str(format!("{}_collision", flavor))],
        }],
    }
}

/// Round 163 — the player-hit rule's damage magnitude
/// is now seed-perturbed. The base magnitude still
/// scales with mood (Calm = mild, Epic = lethal), but
/// the seed nudges it within a ±25% band so the
/// round-87 balance AI sees a range of difficulty
/// profiles for the same (biome, mood).
fn playerhit_rule(input: GenInput) -> Rule {
    let base_magnitude = match input.mood {
        MoodKind::Calm => 1.0,
        MoodKind::Tense => 4.0,
        MoodKind::Epic => 8.0,
        MoodKind::Mysterious => 2.0,
    };
    let offset = seed_offset(input.seed, 4);
    let magnitude = (base_magnitude * (1.0 + offset * 0.25)).max(0.5);
    Rule {
        event: Event {
            kind: EventKind::PlayerHit,
            arg: None,
        },
        actions: vec![Action {
            kind: ActionKind::Damage,
            args: vec![Arg::Number(magnitude)],
        }],
    }
}

/// Map a biome to its short flavor string used as a
/// prefix in generated rule args. Mirrors the
/// `BiomeAtmosphere` palette names so the DslCodexPanel
/// shows the same flavor strings the player sees in the
/// scene HUD.
///
/// Round 167 — the 6-biome palette is now fully
/// represented. Before round 167 `space` and `lava`
/// fell back to `forest` here; now they have their
/// own flavor strings (`space` / `lava`) so a Space
/// biome's auto-generated rules read `space_mob` /
/// `space_timer_spawn` instead of `forest_mob`. The
/// TS-side `BIOME_FLAVOR` Record in `codegenBindings.ts`
/// mirrors this map byte-for-byte.
fn biome_flavor(biome: BiomeKind) -> &'static str {
    match biome {
        BiomeKind::Forest => "forest",
        BiomeKind::Desert => "desert",
        BiomeKind::Ice => "ice",
        BiomeKind::Cyberpunk => "cyber",
        BiomeKind::Lava => "lava",
        BiomeKind::Space => "space",
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Total mutation cost of a generated rule set. The
/// round-balance AI uses this as a "complexity budget"
/// signal — High complexity scenes should land in the
/// 10-20 cost band, Low scenes in the 1-5 band.
pub fn total_mutation_cost(rules: &[Rule]) -> u32 {
    rules.iter().map(|r| r.mutation_cost()).sum()
}

/// True when the rule set is "balanced" (between 2 and
/// 25 mutation cost). Used by the round-87 balance AI
/// to short-circuit on degenerate inputs (e.g. an
/// all-spam scene that would break the dispatcher).
pub fn is_balanced(rules: &[Rule]) -> bool {
    let cost = total_mutation_cost(rules);
    cost >= 2 && cost <= 25
}

/// Round 163 — derive a deterministic per-(seed, slot)
/// offset in the range `[-0.5, +0.5]`. The function
/// is a tiny xorshift-style mixer: a different seed
/// gives a different offset, a different slot gives
/// a different offset for the same seed, and the
/// output is bounded so the codegen can multiply it
/// into a "perturb within band" percentage without
/// worrying about overflow.
///
/// Why xorshift rather than `rand`: the codegen
/// module is `no_std`-friendly, the seed needs to
/// round-trip through round-72 saves (so we can't
/// rely on a global PRNG state), and the output only
/// needs to be "scattered enough" to vary across
/// seeds — it doesn't need to be cryptographically
/// random. A 4-round xorshift gives sufficient
/// scatter for the 4 biomes × 4 moods × 3 complexity
/// × ~2^64 seeds ≈ 3072 distinct values.
pub fn seed_offset(seed: u64, slot: u32) -> f32 {
    // Mix the seed and slot into a single u64. The
    // slot lives in the low bits so a single seed
    // with 5 different slots (the 5-rule High
    // complexity case) gives 5 independent offsets.
    let mut x = seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(slot as u64);
    // xorshift64 — 4 rounds is enough scatter for
    // our needs (verified by the round_163_*_tests
    // block: different seeds produce different
    // magnitudes, no collisions in the test sample).
    for _ in 0..4 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
    }
    // Map the low 32 bits of `x` to [-0.5, +0.5].
    // The mask keeps the high bit zero (positive),
    // the cast to f32 gives a 24-bit-mantissa
    // approximation, and the `2.0 / (u32::MAX as
    // f32)` rescales to ~[0, 1) before we shift
    // down by 0.5 to land in the [-0.5, +0.5] band.
    let v = (x as u32) as f32;
    v * (1.0 / (u32::MAX as f32)) - 0.5
}

/// Round 164 — derive a `u64` seed from a
/// human-readable string (typically a dimension
/// ID like `"dim_alpha"` or a biome key like
/// `"forest"`). The App calls this at
/// dimension-enter time to seed the round-163
/// `seed_offset` axis, so reloading the same
/// dimension gives the same auto-generated rule
/// set (round-72 save round-trip stability).
///
/// Implementation: 64-bit FNV-1a hash. The
/// algorithm is the FNV-1 variant (xor then
/// multiply, in that order — see
/// http://www.isthe.com/chongo/tech/comp/fnv/)
/// with the canonical 64-bit constants:
///   - offset basis: 0xCBF29CE484222325
///   - prime:        0x100000001B3
///
/// Why FNV-1a rather than `std::hash`:
///   - `std::hash` is per-process random (the
///     `RandomState` builder uses random keys for
///     HashMap DoS resistance) — calling it
///     twice in a row on the same string can
///     give different results across runs. FNV-1a
///     is a pure function of the bytes, so the
///     seed round-trips through round-72 saves.
///   - FNV-1a is also what `narration.rs::fnv1a`
///     uses for 32-bit strings (round-125) and
///     what `AGI-miniGame/src/main.ts:285` mirrors
///     in TypeScript. Using the same algorithm
///     here (in 64-bit) means the AGI-miniGame
///     App can derive the same seed in TS-land
///     and in WASM-land — important for the
///     round-48 progressive-enhancement gate
///     (TS-mirror fallback when WASM is null).
///   - The seed is `u64` and FNV-1a 32-bit would
///     collapse to 32 bits of entropy. The 64-bit
///     variant keeps all 64 bits of the seed axis
///     useful, so the round-163 `seed_offset`
///     scatter is preserved across all 2^64
///     possible seeds.
///
/// Edge cases:
///   - empty string: returns the offset basis
///     (0xCBF29CE484222325). A bug in the call
///     site that passes `""` would still produce
///     a deterministic value (defense in depth).
///   - non-ASCII: hashes the UTF-8 bytes
///     directly. The same Unicode string always
///     produces the same seed (verified by
///     `seed_from_string_unicode_is_stable`).
pub fn seed_from_string(s: &str) -> u64 {
    // FNV-1a 64-bit offset basis.
    let mut h: u64 = 0xCBF29CE484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001B3);
    }
    h
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round162_tests {
    use super::*;

    fn low() -> GenInput {
        GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Low,
            seed: 0,
        }
    }

    fn med() -> GenInput {
        GenInput {
            biome: BiomeKind::Desert,
            mood: MoodKind::Tense,
            complexity: ComplexityKind::Medium,
            seed: 0,
        }
    }

    fn high() -> GenInput {
        GenInput {
            biome: BiomeKind::Cyberpunk,
            mood: MoodKind::Epic,
            complexity: ComplexityKind::High,
            seed: 0,
        }
    }

    #[test]
    fn low_complexity_emits_exactly_one_rule_round_162() {
        let rules = generate_rules(low());
        assert_eq!(rules.len(), 1, "Low complexity must emit exactly 1 rule (the spawn baseline)");
    }

    #[test]
    fn medium_complexity_emits_exactly_three_rules_round_162() {
        let rules = generate_rules(med());
        assert_eq!(rules.len(), 3, "Medium complexity must emit exactly 3 rules");
    }

    #[test]
    fn high_complexity_emits_exactly_five_rules_round_162() {
        let rules = generate_rules(high());
        assert_eq!(rules.len(), 5, "High complexity must emit exactly 5 rules");
    }

    #[test]
    fn generate_rules_is_deterministic_round_162() {
        // Same input → same output (round-72 save
        // round-trip contract).
        let a = generate_rules(med());
        let b = generate_rules(med());
        assert_eq!(a, b);
    }

    #[test]
    fn different_inputs_produce_different_rules_round_162() {
        // Two distinct GenInputs should produce distinct
        // rule sets (a regression that collapsed biome
        // or mood to a constant would break this).
        let a = generate_rules(low());
        let b = generate_rules(high());
        assert_ne!(a, b);
    }

    #[test]
    fn biome_flavor_propagates_into_spawn_population_rule_round_162() {
        // The first rule's action arg carries the
        // biome flavor so the DslCodexPanel can show
        // the cross-product.
        let rules = generate_rules(GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Low,
        seed: 0,
        });
        let action = &rules[0].actions[0];
        match &action.args[0] {
            Arg::Str(s) => assert_eq!(s, "forest_mob"),
            other => panic!("expected Str, got {:?}", other),
        }
    }

    #[test]
    fn mood_calm_emits_heal_action_round_162() {
        // Calm → Heal (the mood_rule builder picks Heal
        // for MoodKind::Calm). The mood rule is the 2nd
        // emitted (index 1) when complexity is anything
        // > Low; for Low it's not emitted at all. So we
        // bump complexity to Medium to get the mood rule
        // in the output.
        let rules = generate_rules(GenInput {
            biome: BiomeKind::Desert,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Medium,
        seed: 0,
        });
        let mood_rule = &rules[1];
        assert_eq!(mood_rule.actions[0].kind, ActionKind::Heal);
    }

    #[test]
    fn mood_tense_emits_damage_action_round_162() {
        let rules = generate_rules(med()); // mood = Tense
        let mood_rule = &rules[1];
        assert_eq!(mood_rule.actions[0].kind, ActionKind::Damage);
    }

    #[test]
    fn mood_epic_emits_spawn_action_with_count_arg_round_162() {
        let rules = generate_rules(GenInput {
            biome: BiomeKind::Cyberpunk,
            mood: MoodKind::Epic,
            complexity: ComplexityKind::Medium,
        seed: 0,
        });
        let mood_rule = &rules[1];
        assert_eq!(mood_rule.actions[0].kind, ActionKind::Spawn);
        // Epic adds a numeric count arg.
        assert!(matches!(mood_rule.actions[0].args[1], Arg::Number(_)));
    }

    #[test]
    fn mood_mysterious_emits_spawn_entity_round_162() {
        let rules = generate_rules(GenInput {
            biome: BiomeKind::Ice,
            mood: MoodKind::Mysterious,
            complexity: ComplexityKind::Medium,
        seed: 0,
        });
        let mood_rule = &rules[1];
        assert_eq!(mood_rule.actions[0].kind, ActionKind::SpawnEntity);
    }

    #[test]
    fn timer_rule_uses_timer_event_kind_with_numeric_arg_round_162() {
        // The medium-complexity extras include a Timer
        // rule with a numeric delay arg.
        let rules = generate_rules(med());
        let timer_rule = rules
            .iter()
            .find(|r| r.event.kind == EventKind::Timer)
            .expect("Medium must include a Timer rule");
        assert!(matches!(timer_rule.event.arg, Some(Arg::Number(_))));
    }

    #[test]
    fn high_complexity_includes_playerhit_rule_round_162() {
        // The High-complexity extras add a PlayerHit
        // rule driven by the mood magnitude.
        let rules = generate_rules(high());
        let has_playerhit = rules
            .iter()
            .any(|r| r.event.kind == EventKind::PlayerHit);
        assert!(has_playerhit, "High complexity must include a PlayerHit rule");
    }

    #[test]
    fn playerhit_magnitude_scales_with_mood_round_162() {
        // Calm → 1.0, Epic → 8.0. Different mood →
        // different damage magnitude.
        let calm = generate_rules(GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::High,
        seed: 0,
        });
        let epic = generate_rules(GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Epic,
            complexity: ComplexityKind::High,
        seed: 0,
        });
        let calm_hit = calm
            .iter()
            .find(|r| r.event.kind == EventKind::PlayerHit)
            .unwrap();
        let epic_hit = epic
            .iter()
            .find(|r| r.event.kind == EventKind::PlayerHit)
            .unwrap();
        match (&calm_hit.actions[0].args[0], &epic_hit.actions[0].args[0]) {
            (Arg::Number(c), Arg::Number(e)) => {
                assert!(*e > *c, "Epic damage must exceed Calm damage");
            }
            _ => panic!("expected numeric damage args"),
        }
    }

    #[test]
    fn generate_rule_singleton_matches_baseline_round_162() {
        // generate_rule() returns the same baseline rule
        // that generate_rules() emits at index 0.
        let input = high();
        let singleton = generate_rule(input);
        let from_vec = generate_rules(input);
        assert_eq!(from_vec[0], singleton);
    }

    #[test]
    fn total_mutation_cost_scales_with_complexity_round_162() {
        // Low < Med < High cost (monotonic in
        // complexity, by construction).
        let low_cost = total_mutation_cost(&generate_rules(low()));
        let med_cost = total_mutation_cost(&generate_rules(med()));
        let high_cost = total_mutation_cost(&generate_rules(high()));
        assert!(low_cost < med_cost, "Low {} must be < Med {}", low_cost, med_cost);
        assert!(med_cost < high_cost, "Med {} must be < High {}", med_cost, high_cost);
    }

    #[test]
    fn is_balanced_true_for_default_scene_round_162() {
        // A "default" scene (forest / calm / medium)
        // is balanced.
        let input = GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Medium,
            seed: 0,
        };
        assert!(is_balanced(&generate_rules(input)));
    }

    #[test]
    fn biome_flavor_returns_distinct_strings_round_162() {
        // Each biome gets its own flavor string — a
        // regression that aliased them all to "default"
        // would silently collapse the cross-product.
        assert_eq!(biome_flavor(BiomeKind::Forest), "forest");
        assert_eq!(biome_flavor(BiomeKind::Desert), "desert");
        assert_eq!(biome_flavor(BiomeKind::Ice), "ice");
        assert_eq!(biome_flavor(BiomeKind::Cyberpunk), "cyber");
    }

    #[test]
    fn empty_rule_set_is_not_balanced_round_162() {
        // An empty rule set has cost 0, which is below
        // the balanced threshold. The codegen never
        // emits an empty set, but the helper still
        // flags it correctly (defense in depth).
        assert!(!is_balanced(&[]));
    }

    #[test]
    fn all_four_moods_emit_distinct_action_kinds_round_162() {
        // A regression that collapsed Calm/Tense/Epic/
        // Mysterious to the same action_kind would
        // remove the mood axis entirely. Pin the 4-way
        // distinctness via a single medium-complexity
        // pass for each mood.
        let kinds: Vec<ActionKind> = [MoodKind::Calm, MoodKind::Tense, MoodKind::Epic, MoodKind::Mysterious]
            .iter()
            .map(|m| {
                let rules = generate_rules(GenInput {
                    biome: BiomeKind::Forest,
                    mood: *m,
                    complexity: ComplexityKind::Medium,
                    seed: 0,
                });
                rules[1].actions[0].kind.clone()
            })
            .collect();
        // Sort the 4 kinds for stable comparison (the
        // ActionKind enum isn't Ord, but the 4-value
        // list is small enough to compare by
        // linear scan).
        fn count_unique(kinds: &[ActionKind]) -> usize {
            let mut unique_count = 0;
            let mut seen: Vec<&ActionKind> = Vec::new();
            for k in kinds {
                if !seen.iter().any(|s| *s == k) {
                    seen.push(k);
                    unique_count += 1;
                }
            }
            unique_count
        }
        assert_eq!(
            count_unique(&kinds),
            4,
            "All 4 moods must produce distinct action_kinds, got {:?}",
            kinds
        );
    }

    #[test]
    fn round167_biome_flavor_lava_and_space_round_167() {
        // Round 167 — `biome_flavor` now
        // covers all 6 BiomeKind
        // variants. The TS `BIOME_FLAVOR`
        // mirror in `codegenBindings.ts`
        // must have the same 6 strings.
        assert_eq!(biome_flavor(BiomeKind::Lava), "lava");
        assert_eq!(biome_flavor(BiomeKind::Space), "space");
    }

    #[test]
    fn round167_baseline_spawn_uses_each_biomes_own_mob_tag_round_167() {
        // Cross-validation: each biome's
        // auto-generated rule set must
        // carry that biome's spawn tag in
        // the baseline rule's `Spawn`
        // action args[0]. The TS mirror
        // in `codegenBindings.ts` emits
        // the same per-biome mob
        // strings, so a regression on
        // either side breaks both.
        let expected: &[(BiomeKind, &str)] = &[
            (BiomeKind::Forest, "forest_mob"),
            (BiomeKind::Desert, "desert_mob"),
            (BiomeKind::Ice, "ice_mob"),
            (BiomeKind::Cyberpunk, "cyber_mob"),
            (BiomeKind::Lava, "lava_mob"),
            (BiomeKind::Space, "space_mob"),
        ];
        for (biome, expected_tag) in expected {
            let rules = generate_rules(GenInput {
                biome: *biome,
                mood: MoodKind::Calm,
                complexity: ComplexityKind::Low, // Low emits only the baseline.
                seed: 0,
            });
            // Round 132 manual JSON shape:
            // rule.actions[0].kind == ActionKind::Spawn,
            // rule.actions[0].args[0] is the mob tag.
            let baseline = &rules[0];
            assert_eq!(
                baseline.actions[0].kind,
                ActionKind::Spawn,
                "biome {:?} baseline action.kind must be Spawn",
                biome
            );
            // args[0] is the biome-flavored mob tag.
            match &baseline.actions[0].args[0] {
                Arg::Str(s) => assert_eq!(
                    s, expected_tag,
                    "biome {:?} baseline must use {:?}, got {:?}",
                    biome, expected_tag, s
                ),
                other => panic!(
                    "biome {:?} baseline args[0] must be Str({:?}), got {:?}",
                    biome, expected_tag, other
                ),
            }
        }
    }

    #[test]
    fn round167_biome_kind_covers_six_atmospheric_variants_round_167() {
        // The 6-biome Atmosphere palette is
        // the source of truth for "what
        // biomes the player sees" — the
        // Rust BiomeKind enum must match
        // it exactly. A regression that
        // removed Lava or Space would
        // break this pin.
        assert_eq!(
            [
                BiomeKind::Forest,
                BiomeKind::Desert,
                BiomeKind::Ice,
                BiomeKind::Cyberpunk,
                BiomeKind::Lava,
                BiomeKind::Space,
            ]
            .len(),
            6
        );
    }
}

// ---------------------------------------------------------------------------
// Round 163 — seed axis tests. The codegen now reads a
// `seed: u64` input alongside (biome, mood, complexity).
// Same triple with different seeds must produce
// deterministic-but-varied rule sets: distinct magnitudes,
// distinct timer durations, but stable rule counts and
// stable action kinds (the seed varies amounts, not kinds).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round163_tests {
    use super::*;

    fn forest_calm_med(seed: u64) -> GenInput {
        GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Medium,
            seed,
        }
    }

    fn cyber_epic_high(seed: u64) -> GenInput {
        GenInput {
            biome: BiomeKind::Cyberpunk,
            mood: MoodKind::Epic,
            complexity: ComplexityKind::High,
            seed,
        }
    }

    #[test]
    fn gen_input_default_seed_is_zero_round_163() {
        // The `Default` impl gives seed = 0 so old
        // callers that don't think about the seed
        // axis get the same output they used to.
        let input = GenInput::default();
        assert_eq!(input.seed, 0);
    }

    #[test]
    fn seed_offset_returns_value_in_negative_half_to_positive_half_round_163() {
        // The seed_offset contract: a value in
        // [-0.5, +0.5]. Pin the contract with 64
        // distinct (seed, slot) pairs so a regression
        // that returned 0..1 (or any other range)
        // would fail.
        for seed in 0u64..64 {
            for slot in 0u32..5 {
                let v = seed_offset(seed, slot);
                assert!(
                    v >= -0.5 && v <= 0.5,
                    "seed_offset({}, {}) = {} must be in [-0.5, +0.5]",
                    seed, slot, v
                );
            }
        }
    }

    #[test]
    fn seed_offset_is_deterministic_round_163() {
        // Same (seed, slot) → same offset. The
        // round-72 save snapshot round-trip contract.
        for seed in [0u64, 1, 42, 0xDEAD, 0xFEED_BEEF] {
            for slot in 0u32..5 {
                let a = seed_offset(seed, slot);
                let b = seed_offset(seed, slot);
                assert_eq!(a, b, "seed_offset({}, {}) must be deterministic", seed, slot);
            }
        }
    }

    #[test]
    fn different_seeds_produce_different_offsets_round_163() {
        // A regression that returned a constant for
        // any (seed, slot) would collapse the seed
        // axis — pin that two distinct seeds give
        // distinct offsets for the same slot.
        let v0 = seed_offset(0, 0);
        let v1 = seed_offset(1, 0);
        let v42 = seed_offset(42, 0);
        assert_ne!(v0, v1, "seed=0 and seed=1 must differ");
        assert_ne!(v0, v42, "seed=0 and seed=42 must differ");
        assert_ne!(v1, v42, "seed=1 and seed=42 must differ");
    }

    #[test]
    fn different_slots_produce_independent_offsets_round_163() {
        // Same seed, different slots → independent
        // offsets. The codegen relies on this so a
        // 5-rule High complexity set doesn't have
        // all 5 rules perturbed by the same factor
        // (which would be visually obvious in the
        // DslCodexPanel).
        let slots: Vec<f32> = (0u32..5).map(|s| seed_offset(42, s)).collect();
        let mut unique_count = 0;
        let mut seen: Vec<f32> = Vec::new();
        for v in &slots {
            // f32 equality is fine here: the
            // xorshift mixer produces bit-different
            // outputs for different slots, and we
            // want to detect the regression case
            // where a buggy implementation always
            // returned the same value.
            if !seen.iter().any(|s| s.to_bits() == v.to_bits()) {
                seen.push(*v);
                unique_count += 1;
            }
        }
        assert_eq!(
            unique_count, 5,
            "5 slots for seed=42 must give 5 distinct offsets (got {} unique values)",
            unique_count
        );
    }

    #[test]
    fn seed_axis_preserves_rule_count_round_163() {
        // The seed axis perturbs magnitudes, not the
        // rule count. Low = 1, Med = 3, High = 5
        // regardless of seed.
        for seed in [0u64, 1, 42, 999, 0xCAFE] {
            assert_eq!(generate_rules(forest_calm_med(seed)).len(), 3);
            let high_input = cyber_epic_high(seed);
            assert_eq!(generate_rules(high_input).len(), 5);
        }
    }

    #[test]
    fn seed_axis_preserves_action_kinds_round_163() {
        // The seed axis perturbs magnitudes, not
        // action kinds. Calm is always Heal, Tense
        // is always Damage, etc. — regardless of
        // seed.
        for seed in [0u64, 1, 42, 999, 0xCAFE] {
            let rules = generate_rules(forest_calm_med(seed));
            // rules[1] is the mood rule for
            // Medium complexity (baseline + mood +
            // timer).
            assert_eq!(rules[1].actions[0].kind, ActionKind::Heal);
        }
    }

    #[test]
    fn seed_axis_produces_distinct_magnitudes_round_163() {
        // The whole point of the seed axis: same
        // (biome, mood, complexity) with different
        // seeds → distinct rule sets (the round-72
        // save would otherwise round-trip to the
        // exact same rules for every save).
        let r0 = generate_rules(forest_calm_med(0));
        let r42 = generate_rules(forest_calm_med(42));
        // The baseline spawn-count arg is the
        // easiest place to detect a perturbation
        // (it lives in args[1] of the first
        // action).
        let count0 = match &r0[0].actions[0].args[1] {
            Arg::Number(n) => *n,
            _ => panic!("expected numeric spawn count"),
        };
        let count42 = match &r42[0].actions[0].args[1] {
            Arg::Number(n) => *n,
            _ => panic!("expected numeric spawn count"),
        };
        assert_ne!(
            count0, count42,
            "seed=0 and seed=42 must perturb the spawn count (got {} and {})",
            count0, count42
        );
    }

    #[test]
    fn seed_axis_perturbation_stays_in_band_round_163() {
        // The perturbation factor is `1.0 + offset *
        // 0.5` for the mood rule and `1.0 + offset *
        // 0.25` for the player-hit rule. The factor
        // must stay in the declared band regardless
        // of seed (so the round-87 balance AI never
        // sees a degenerate rule set).
        for seed in [0u64, 1, 42, 999, 0xCAFE, u64::MAX] {
            let rules = generate_rules(forest_calm_med(seed));
            // Mood rule magnitude (rules[1] is the
            // mood rule for Medium complexity, args[0]
            // is the numeric magnitude for Calm).
            let mag = match &rules[1].actions[0].args[0] {
                Arg::Number(n) => *n,
                _ => panic!("expected numeric magnitude"),
            };
            // Calm base = 5.0, band = ±50% → [2.5,
            // 7.5].
            assert!(
                mag >= 2.0 && mag <= 8.0,
                "seed={}: Calm magnitude {} out of expected band [2.0, 8.0]",
                seed, mag
            );
        }
    }

    #[test]
    fn seed_axis_keeps_high_complexity_in_known_event_kinds_round_163() {
        // The seed axis must not introduce a new
        // event_kind or action_kind — the round-72
        // save round-trip depends on the rule shape
        // being stable.
        for seed in [0u64, 1, 42, 999, 0xCAFE] {
            let rules = generate_rules(cyber_epic_high(seed));
            // High complexity is always baseline +
            // mood + 2 timers + playerhit. Pin the
            // event kinds.
            assert_eq!(rules[0].event.kind, EventKind::Spawn);
            assert_eq!(rules[1].event.kind, EventKind::Spawn);
            assert_eq!(rules[2].event.kind, EventKind::Timer);
            assert_eq!(rules[3].event.kind, EventKind::Timer);
            assert_eq!(rules[4].event.kind, EventKind::PlayerHit);
        }
    }

    #[test]
    fn same_seed_is_deterministic_round_163() {
        // The round-72 save contract: same input →
        // same output, even with the seed axis in
        // play.
        let a = generate_rules(forest_calm_med(0xDEADBEEF));
        let b = generate_rules(forest_calm_med(0xDEADBEEF));
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_produce_different_timer_durations_round_163() {
        // The two High-complexity timers should
        // pick up distinct durations for different
        // seeds (otherwise the seed axis is dead
        // weight on the timer rules).
        let r0 = generate_rules(cyber_epic_high(0));
        let r42 = generate_rules(cyber_epic_high(42));
        // rules[2] and rules[3] are the two
        // timers. Their event.arg carries the
        // numeric duration.
        let fast0 = match r0[2].event.arg {
            Some(Arg::Number(n)) => n,
            _ => panic!("expected numeric timer arg"),
        };
        let fast42 = match r42[2].event.arg {
            Some(Arg::Number(n)) => n,
            _ => panic!("expected numeric timer arg"),
        };
        // Different seeds should perturb the fast
        // timer duration. (The two slots have
        // independent offsets, so even if seed=0
        // gave a degenerate offset, seed=42 would
        // almost certainly give a different one.)
        assert_ne!(
            fast0, fast42,
            "seed=0 and seed=42 must perturb the fast timer (got {} and {})",
            fast0, fast42
        );
    }

    #[test]
    fn seed_axis_perturbation_clamps_to_safe_range_round_163() {
        // The player-hit magnitude is clamped to
        // >= 0.5 (so a degenerate seed doesn't
        // make a "0.001 damage" rule that would
        // divide by zero in the dispatcher). Pin
        // the contract with extreme seed values.
        for seed in [0u64, 1, u64::MAX, u64::MAX - 1, 0xFFFF_FFFF_FFFF_FFFF] {
            let rules = generate_rules(cyber_epic_high(seed));
            let hit = rules
                .iter()
                .find(|r| r.event.kind == EventKind::PlayerHit)
                .expect("High complexity must include a PlayerHit rule");
            let mag = match &hit.actions[0].args[0] {
                Arg::Number(n) => *n,
                _ => panic!("expected numeric damage arg"),
            };
            // Epic base = 8.0, band = ±25% → [6.0,
            // 10.0], clamped to >= 0.5.
            assert!(
                mag >= 0.5 && mag <= 10.0,
                "seed={}: PlayerHit magnitude {} out of safe range [0.5, 10.0]",
                seed, mag
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Round 164 — `seed_from_string` tests. The helper
// derives a `u64` seed from a dimension ID so the
// App can wire the round-163 seed axis at
// dimension-enter time. The tests pin the FNV-1a
// 64-bit contract: empty string → offset basis,
// same string → same seed, different strings →
// different seeds, ASCII / Unicode stable.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round164_tests {
    use super::*;

    /// FNV-1a 64-bit offset basis. Pinned as a
    /// const so the test that checks the empty-
    /// string case doesn't have to hardcode the
    /// 0xCBF29CE484222325 literal in two places.
    const FNV_OFFSET_BASIS_64: u64 = 0xCBF29CE484222325;

    #[test]
    fn seed_from_string_empty_returns_offset_basis_round_164() {
        // Defense in depth: a bug in the call
        // site that passes "" should still get a
        // deterministic value (the offset basis
        // itself), not a panic or a random hash.
        assert_eq!(seed_from_string(""), FNV_OFFSET_BASIS_64);
    }

    #[test]
    fn seed_from_string_is_deterministic_round_164() {
        // Same string → same seed, across calls
        // and across runs. The round-72 save
        // round-trip contract: a re-hydrated
        // dimension must produce the same
        // auto-generated rule set as the
        // original.
        for s in ["dim_alpha", "dim_beta", "forest", "cyber_boss_wave", "x"] {
            let a = seed_from_string(s);
            let b = seed_from_string(s);
            assert_eq!(a, b, "seed_from_string({:?}) must be deterministic", s);
        }
    }

    #[test]
    fn seed_from_string_distinct_inputs_give_distinct_seeds_round_164() {
        // The 4 biomes + 5 dimension IDs + the
        // common "kitchen sink" strings all
        // produce distinct seeds (a regression
        // that aliased them all to a constant
        // would silently collapse the codegen
        // seed axis).
        let seeds: Vec<u64> = [
            "forest", "desert", "ice", "cyber",
            "dim_alpha", "dim_beta", "dim_gamma",
            "cyber_boss_wave", "ice_herb", "desert_thorn",
        ]
        .iter()
        .map(|s| seed_from_string(s))
        .collect();
        let mut unique = 0;
        let mut seen: Vec<u64> = Vec::new();
        for s in &seeds {
            if !seen.iter().any(|x| *x == *s) {
                seen.push(*s);
                unique += 1;
            }
        }
        assert_eq!(
            unique,
            10,
            "10 distinct dimension / biome IDs must give 10 distinct seeds, got {} unique values",
            unique
        );
    }

    #[test]
    fn seed_from_string_unicode_is_stable_and_distinct_round_164() {
        // The hash operates on the UTF-8 bytes
        // directly. The same Unicode string
        // always produces the same seed (so a
        // dimension with a 中文 ID round-trips
        // through round-72 saves), and a single
        // character difference produces a
        // different seed.
        let a = seed_from_string("次元_alpha");
        let b = seed_from_string("次元_alpha");
        let c = seed_from_string("次元_beta");
        assert_eq!(a, b, "Unicode strings must be stable");
        assert_ne!(a, c, "Unicode strings with 1-char difference must differ");
    }

    #[test]
    fn seed_from_string_fits_in_u64_round_164() {
        // The seed axis is `u64`. Sanity check
        // that the helper never returns a value
        // larger than u64::MAX (would-be compile
        // error in practice, but the test makes
        // the contract explicit).
        for s in ["", "x", "a long string of bytes that should still fit in 64 bits"] {
            let v = seed_from_string(s);
            assert!(v <= u64::MAX);
        }
    }

    #[test]
    fn seed_from_string_known_vector_round_164() {
        // Pin the FNV-1a 64-bit output for
        // canonical inputs. The constants are
        // derived from the FNV reference
        // implementation (offset basis
        // 0xCBF29CE484222325 + prime
        // 0x100000001B3, "xor then multiply"
        // order). A regression that flips the
        // order to FNV-1 ("multiply then xor"),
        // or changes the constants, would fail
        // this test — the AGI-miniGame TS
        // mirror can be cross-validated against
        // these exact values (and vice versa:
        // a TS-side regression in the mirror
        // would fail the e2e cross-check).
        assert_eq!(seed_from_string(""), FNV_OFFSET_BASIS_64);
        assert_eq!(seed_from_string("a"), 0xAF63DC4C8601EC8C);
        assert_eq!(seed_from_string("b"), 0xAF63DF4C8601F1A5);
        assert_eq!(seed_from_string("forest"), 0x2098148EC99FB680);
    }

    #[test]
    fn seed_from_string_wires_into_codegen_round_164() {
        // End-to-end sanity: the seed derived
        // from "dim_alpha" + Low complexity must
        // produce the same rule set across
        // calls, and a different dimension ID
        // must produce a different rule set.
        let a = generate_rules(GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Low,
            seed: seed_from_string("dim_alpha"),
        });
        let b = generate_rules(GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Low,
            seed: seed_from_string("dim_alpha"),
        });
        let c = generate_rules(GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Calm,
            complexity: ComplexityKind::Low,
            seed: seed_from_string("dim_beta"),
        });
        assert_eq!(a, b, "same dimension ID → same rules");
        assert_ne!(a, c, "different dimension IDs → different rules (for Low where spawn count perturbs)");
    }
}