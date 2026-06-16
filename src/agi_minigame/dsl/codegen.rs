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

/// Biome flavor. The 4 biomes match `BiomeAtmosphere`
/// (forest / desert / ice / cyberpunk) so the generated
/// rules "taste" like the scene the player is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeKind {
    Forest,
    Desert,
    Ice,
    Cyberpunk,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenInput {
    pub biome: BiomeKind,
    pub mood: MoodKind,
    pub complexity: ComplexityKind,
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
    rules.push(spawn_population_rule(input.biome));

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
    //    DslCodexPanel.
    match input.complexity {
        ComplexityKind::Low => {
            // No extras; the baseline is enough.
        }
        ComplexityKind::Medium => {
            rules.push(mood_rule(input));
            rules.push(timer_rule(input.biome, 5.0));
        }
        ComplexityKind::High => {
            rules.push(mood_rule(input));
            rules.push(timer_rule(input.biome, 3.0));
            rules.push(timer_rule(input.biome, 8.0));
            rules.push(playerhit_rule(input.mood));
        }
    }

    // Cap at 6 rules so the dispatcher stays cheap and
    // the DslCodexPanel doesn't get a 48-row scrollbar.
    rules.truncate(6);
    rules
}

/// Generate a single canonical rule for the given
/// inputs. Equivalent to `generate_rules(input).remove(0)`
/// but more readable at the call site.
pub fn generate_rule(input: GenInput) -> Rule {
    spawn_population_rule(input.biome)
}

// ---------------------------------------------------------------------------
// Per-axis builders
// ---------------------------------------------------------------------------

fn spawn_population_rule(biome: BiomeKind) -> Rule {
    let flavor = biome_flavor(biome);
    Rule {
        event: Event {
            kind: EventKind::Spawn,
            arg: None,
        },
        actions: vec![Action {
            kind: ActionKind::Spawn,
            args: vec![Arg::Str(format!("{}_mob", flavor))],
        }],
    }
}

fn mood_rule(input: GenInput) -> Rule {
    let flavor = biome_flavor(input.biome);
    let (action_kind, args) = match input.mood {
        MoodKind::Calm => (
            ActionKind::Heal,
            vec![Arg::Number(5.0), Arg::Str(format!("{}_herb", flavor))],
        ),
        MoodKind::Tense => (
            ActionKind::Damage,
            vec![Arg::Number(3.0), Arg::Str(format!("{}_thorn", flavor))],
        ),
        MoodKind::Epic => (
            ActionKind::Spawn,
            vec![Arg::Str(format!("{}_boss_wave", flavor)), Arg::Number(3.0)],
        ),
        MoodKind::Mysterious => (
            ActionKind::SpawnEntity,
            vec![Arg::Str(format!("{}_spirit", flavor)), Arg::Number(1.0)],
        ),
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

fn playerhit_rule(mood: MoodKind) -> Rule {
    let magnitude = match mood {
        MoodKind::Calm => 1.0,
        MoodKind::Tense => 4.0,
        MoodKind::Epic => 8.0,
        MoodKind::Mysterious => 2.0,
    };
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
fn biome_flavor(biome: BiomeKind) -> &'static str {
    match biome {
        BiomeKind::Forest => "forest",
        BiomeKind::Desert => "desert",
        BiomeKind::Ice => "ice",
        BiomeKind::Cyberpunk => "cyber",
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
        }
    }

    fn med() -> GenInput {
        GenInput {
            biome: BiomeKind::Desert,
            mood: MoodKind::Tense,
            complexity: ComplexityKind::Medium,
        }
    }

    fn high() -> GenInput {
        GenInput {
            biome: BiomeKind::Cyberpunk,
            mood: MoodKind::Epic,
            complexity: ComplexityKind::High,
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
        });
        let epic = generate_rules(GenInput {
            biome: BiomeKind::Forest,
            mood: MoodKind::Epic,
            complexity: ComplexityKind::High,
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
}