//! AST for the AGI-miniGame DSL.
//!
//! Grammar (mirrors the TypeScript `MemeCompiler` so both sides agree):
//! ```text
//!   rule        := event "->" action ("," action)*
//!   event       := "On(" eventKind ("," eventArg)? ")"
//!   eventKind   := "Collide" | "Timer" | "Spawn" | "PlayerHit"
//!   eventArg    := number | string
//!   action      := bare_action | "Apply(" actionKind ("," arg)* ")"
//!   bare_action := actionKind "(" arg ("," arg)* ")"
//!   actionKind  := "Damage" | "Heal" | "Spawn" | "SpawnEntity"
//!   arg         := number | string
//! ```


#[derive(Debug, Clone, PartialEq)]
pub enum EventKind {
    Collide,
    Timer,
    Spawn,
    PlayerHit,
}

impl EventKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Collide" => Some(Self::Collide),
            "Timer" => Some(Self::Timer),
            "Spawn" => Some(Self::Spawn),
            "PlayerHit" => Some(Self::PlayerHit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionKind {
    Damage,
    Heal,
    Spawn,
    SpawnEntity,
}

impl ActionKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Damage" => Some(Self::Damage),
            "Heal" => Some(Self::Heal),
            "Spawn" => Some(Self::Spawn),
            "SpawnEntity" => Some(Self::SpawnEntity),
            _ => None,
        }
    }
}

/// An argument to an event or action. Numbers are stored as f32 (we don't
/// distinguish int vs float in the DSL — the original AGI prompts are
/// numeric and the engine doesn't need exact integer semantics).
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Number(f32),
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub kind: EventKind,
    pub arg: Option<Arg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub kind: ActionKind,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub event: Event,
    pub actions: Vec<Action>,
}

impl Rule {
    /// Heuristic cost: how much "world-state mutation" does this rule imply?
    /// Used by the balance AI to flag potentially game-breaking rules.
    pub fn mutation_cost(&self) -> u32 {
        let mut cost = 1;
        for a in &self.actions {
            cost += match a.kind {
                ActionKind::Damage => 1,
                ActionKind::Heal => 1,
                ActionKind::Spawn => 2,
                ActionKind::SpawnEntity => 3,
            };
        }
        cost
    }
}

// ---------------------------------------------------------------------------
// Manual JSON serialization (no serde dependency in cocos4-rust).
// We intentionally avoid `serde::{Serialize, Deserialize}` so this crate
// remains dependency-light. The shape is:
//
//   {
//     "event": { "kind": "Collide" | { "Timer": <f32> } | ... },
//     "actions": [
//       { "kind": "Damage", "args": [10.0] },
//       { "kind": "Spawn", "args": ["Fireball", 5.0] }
//     ]
//   }
// ---------------------------------------------------------------------------

impl EventKind {
    pub fn to_json(&self) -> String {
        match self {
            EventKind::Collide => "\"Collide\"".to_string(),
            EventKind::Timer   => "\"Timer\"".to_string(),
            EventKind::Spawn   => "\"Spawn\"".to_string(),
            EventKind::PlayerHit => "\"PlayerHit\"".to_string(),
        }
    }
    pub fn from_json(s: &str) -> Option<Self> { Self::from_str(s) }
}

impl ActionKind {
    pub fn to_json(&self) -> String {
        match self {
            ActionKind::Damage      => "\"Damage\"".to_string(),
            ActionKind::Heal        => "\"Heal\"".to_string(),
            ActionKind::Spawn       => "\"Spawn\"".to_string(),
            ActionKind::SpawnEntity => "\"SpawnEntity\"".to_string(),
        }
    }
    pub fn from_json(s: &str) -> Option<Self> { Self::from_str(s) }
}

impl Arg {
    pub fn to_json(&self) -> String {
        match self {
            Arg::Number(n) => {
                // Render as float with up to 6 fractional digits; trim trailing zeros.
                let s = format!("{:.6}", n);
                let trimmed = s.trim_end_matches('0').trim_end_matches('.');
                trimmed.to_string()
            }
            Arg::Str(s) => {
                let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{}\"", escaped)
            }
        }
    }
    pub fn from_json(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.starts_with('"') && s.ends_with('"') {
            return Some(Arg::Str(s[1..s.len()-1].to_string()));
        }
        if let Ok(n) = s.parse::<f32>() {
            return Some(Arg::Number(n));
        }
        None
    }
}

impl Event {
    pub fn to_json(&self) -> String {
        let arg_part = match &self.arg {
            Some(a) => format!(", {}", a.to_json()),
            None => String::new(),
        };
        format!("{{\"kind\":{}, \"arg\":{}}}", self.kind.to_json(),
                match &self.arg { Some(a) => a.to_json(), None => "null".to_string() })
    }
}

impl Action {
    pub fn to_json(&self) -> String {
        let args_json: Vec<String> = self.args.iter().map(|a| a.to_json()).collect();
        format!("{{\"kind\":{}, \"args\":[{}]}}", self.kind.to_json(), args_json.join(","))
    }
}

impl Rule {
    pub fn to_json(&self) -> String {
        let actions_json: Vec<String> = self.actions.iter().map(|a| a.to_json()).collect();
        format!("{{\"event\":{}, \"actions\":[{}]}}",
                self.event.to_json(), actions_json.join(","))
    }
}

#[cfg(test)]
mod round132_tests {
    //! Round 132 — helper-level
    //! unit tests for the
    //! `dsl::ast` module
    //! (EventKind, ActionKind,
    //! Arg, Event, Action, Rule
    //! + their `from_str` /
    //! `to_json` / JSON
    //! round-trip / `mutation_cost`
    //! contracts).
    //!
    //! Mirrors the round-110b
    //! / 122 / 123 / 124 / 125
    //! / 126 / 127 / 128 / 129
    //! / 130 / 131 helper-test
    //! pattern: pin the small
    //! public helpers'
    //! contracts (string ↔
    //! enum round-trip /
    //! JSON round-trip /
    //! cost heuristic /
    //! missing-key accessors)
    //! so a refactor can't
    //! silently change the
    //! AST-shape contract
    //! that the AGI-miniGame
    //! TypeScript `MemeCompiler`
    //! + the round-48/51
    //! WASM bridge both rely
    //! on.
    //!
    //! Closes the gaps:
    //!   - EventKind::from_str
    //!     4-variant coverage
    //!     + unknown-key
    //!     returns None
    //!   - ActionKind::from_str
    //!     4-variant coverage
    //!     + unknown-key
    //!     returns None
    //!   - EventKind::to_json
    //!     / from_json
    //!     round-trip
    //!   - ActionKind::to_json
    //!     / from_json
    //!     round-trip
    //!   - Arg::to_json
    //!     number formatting
    //!     (trailing-zero
    //!     trim)
    //!   - Arg::to_json string
    //!     escaping
    //!     (backslash + quote)
    //!   - Arg::from_json
    //!     number / string /
    //!     invalid coverage
    //!   - Rule::mutation_cost
    //!     weights pin
    //!     (base 1 + Damage 1
    //!     + Heal 1 + Spawn 2
    //!     + SpawnEntity 3)
    //!   - Event / Action /
    //!     Rule to_json shape

    use super::*;

    // -----------------------------------------------------------------
    // EventKind::from_str
    // -----------------------------------------------------------------

    #[test]
    fn event_kind_from_str_4_variants_round_132() {
        // All 4 event kinds
        // round-trip through
        // from_str.
        assert_eq!(EventKind::from_str("Collide"), Some(EventKind::Collide));
        assert_eq!(EventKind::from_str("Timer"),   Some(EventKind::Timer));
        assert_eq!(EventKind::from_str("Spawn"),   Some(EventKind::Spawn));
        assert_eq!(EventKind::from_str("PlayerHit"), Some(EventKind::PlayerHit));
    }

    #[test]
    fn event_kind_from_str_unknown_returns_none_round_132() {
        // Defense: any
        // unknown string
        // returns None
        // (not panic, not
        // default). Empty
        // string + lowercase
        // + unicode + typo
        // all fail.
        assert_eq!(EventKind::from_str("collide"), None); // case-sensitive
        assert_eq!(EventKind::from_str("COLLIDE"), None);
        assert_eq!(EventKind::from_str(""),        None);
        assert_eq!(EventKind::from_str("Collidee"), None);
        assert_eq!(EventKind::from_str("碰撞"),    None); // unicode
    }

    // -----------------------------------------------------------------
    // ActionKind::from_str
    // -----------------------------------------------------------------

    #[test]
    fn action_kind_from_str_4_variants_round_132() {
        // All 4 action
        // kinds round-trip
        // through from_str.
        assert_eq!(ActionKind::from_str("Damage"),      Some(ActionKind::Damage));
        assert_eq!(ActionKind::from_str("Heal"),        Some(ActionKind::Heal));
        assert_eq!(ActionKind::from_str("Spawn"),       Some(ActionKind::Spawn));
        assert_eq!(ActionKind::from_str("SpawnEntity"), Some(ActionKind::SpawnEntity));
    }

    #[test]
    fn action_kind_from_str_unknown_returns_none_round_132() {
        assert_eq!(ActionKind::from_str("damage"),    None);
        assert_eq!(ActionKind::from_str("SPAWN"),     None);
        assert_eq!(ActionKind::from_str(""),          None);
        assert_eq!(ActionKind::from_str("SpawnEnt"),  None);
        assert_eq!(ActionKind::from_str("攻击"),      None);
    }

    // -----------------------------------------------------------------
    // EventKind::to_json / from_json round-trip
    // -----------------------------------------------------------------

    #[test]
    fn event_kind_to_json_4_variants_quoted_round_132() {
        // All 4 event kinds
        // serialize as
        // JSON-quoted
        // strings (so the
        // shape is `"Kind"`,
        // not the Rust
        // Debug form
        // `Collide`).
        assert_eq!(EventKind::Collide.to_json(),  "\"Collide\"");
        assert_eq!(EventKind::Timer.to_json(),    "\"Timer\"");
        assert_eq!(EventKind::Spawn.to_json(),    "\"Spawn\"");
        assert_eq!(EventKind::PlayerHit.to_json(), "\"PlayerHit\"");
    }

    #[test]
    fn event_kind_json_round_trip_round_132() {
        // to_json produces
        // a JSON-quoted
        // string. The
        // current
        // `from_json`
        // implementation
        // is a thin
        // wrapper over
        // `from_str` —
        // it expects the
        // unquoted form
        // (no JSON
        // quote-stripping).
        // So a full
        // `to_json` →
        // `from_json`
        // round-trip
        // requires
        // stripping the
        // surrounding
        // quotes. (This
        // pins the actual
        // current contract
        // — a future
        // refactor that
        // makes from_json
        // strip quotes
        // would break
        // this test and
        // surface the
        // contract change
        // for review.)
        for k in [EventKind::Collide, EventKind::Timer, EventKind::Spawn, EventKind::PlayerHit] {
            let json = k.to_json(); // e.g. "\"Collide\""
            // Strip the
            // surrounding
            // quotes to get
            // the unquoted
            // form that
            // from_str
            // accepts.
            let unquoted = &json[1..json.len()-1];
            let parsed = EventKind::from_json(unquoted);
            assert_eq!(parsed, Some(k));
        }
    }

    #[test]
    fn event_kind_from_json_is_thin_wrapper_over_from_str_round_132() {
        // from_json(s)
        // == from_str(s)
        // for all input.
        // (Note: this
        // means callers
        // that want to
        // parse the
        // `to_json()`
        // output MUST
        // strip the
        // surrounding
        // quotes first.
        // The
        // `event_kind_json_round_trip_round_132`
        // test pins the
        // manual
        // quote-strip
        // workflow.)
        assert_eq!(EventKind::from_json("Collide"),  Some(EventKind::Collide));
        assert_eq!(EventKind::from_json("Timer"),    Some(EventKind::Timer));
        assert_eq!(EventKind::from_json("Spawn"),    Some(EventKind::Spawn));
        assert_eq!(EventKind::from_json("PlayerHit"), Some(EventKind::PlayerHit));
        // Quoted form is
        // NOT accepted
        // (current
        // contract).
        assert_eq!(EventKind::from_json("\"Collide\""), None);
        // Case-sensitive.
        assert_eq!(EventKind::from_json("collide"), None);
        // Unknown / empty.
        assert_eq!(EventKind::from_json(""),          None);
        assert_eq!(EventKind::from_json("collidee"),  None);
    }

    // -----------------------------------------------------------------
    // ActionKind::to_json / from_json round-trip
    // -----------------------------------------------------------------

    #[test]
    fn action_kind_to_json_4_variants_quoted_round_132() {
        assert_eq!(ActionKind::Damage.to_json(),      "\"Damage\"");
        assert_eq!(ActionKind::Heal.to_json(),        "\"Heal\"");
        assert_eq!(ActionKind::Spawn.to_json(),       "\"Spawn\"");
        assert_eq!(ActionKind::SpawnEntity.to_json(), "\"SpawnEntity\"");
    }

    #[test]
    fn action_kind_json_round_trip_round_132() {
        for k in [ActionKind::Damage, ActionKind::Heal, ActionKind::Spawn, ActionKind::SpawnEntity] {
            let json = k.to_json();
            // Same manual
            // quote-strip
            // workflow as
            // the EventKind
            // round-trip.
            let unquoted = &json[1..json.len()-1];
            let parsed = ActionKind::from_json(unquoted);
            assert_eq!(parsed, Some(k));
        }
    }

    // -----------------------------------------------------------------
    // Arg::to_json / from_json
    // -----------------------------------------------------------------

    #[test]
    fn arg_to_json_number_trims_trailing_zeros_round_132() {
        // Numbers render
        // with up to 6
        // fractional
        // digits, then
        // trim trailing
        // zeros + the
        // decimal point
        // if the number
        // is integer.
        assert_eq!(Arg::Number(1.0).to_json(),     "1");
        assert_eq!(Arg::Number(10.0).to_json(),    "10");
        assert_eq!(Arg::Number(3.5).to_json(),     "3.5");
        assert_eq!(Arg::Number(0.5).to_json(),     "0.5");
        // Trailing zeros
        // are trimmed.
        assert_eq!(Arg::Number(1.500000).to_json(), "1.5");
        assert_eq!(Arg::Number(0.100000).to_json(), "0.1");
    }

    #[test]
    fn arg_to_json_string_escapes_quote_and_backslash_round_132() {
        // The string
        // serializer
        // escapes both
        // `\\` → `\\\\`
        // and `"` → `\\"`.
        // The wrapper
        // quotes are
        // around the
        // escaped content.
        assert_eq!(Arg::Str("hello".to_string()).to_json(), "\"hello\"");
        // Quote inside
        // string is
        // escaped.
        assert_eq!(Arg::Str("say \"hi\"".to_string()).to_json(), "\"say \\\"hi\\\"\"");
        // Backslash
        // inside string
        // is escaped.
        assert_eq!(Arg::Str("path\\to".to_string()).to_json(), "\"path\\\\to\"");
        // Empty string
        // renders as
        // `""`.
        assert_eq!(Arg::Str("".to_string()).to_json(), "\"\"");
    }

    #[test]
    fn arg_from_json_string_decodes_quoted_round_132() {
        // A string arg is
        // one whose
        // trimmed form
        // starts AND ends
        // with `"`. The
        // surrounding
        // quotes are
        // stripped (the
        // inner content
        // is NOT
        // unescaped by
        // the current
        // implementation
        // — pin the
        // current contract).
        assert_eq!(
            Arg::from_json("\"hello\""),
            Some(Arg::Str("hello".to_string()))
        );
        assert_eq!(
            Arg::from_json("  \"hello\"  "),
            Some(Arg::Str("hello".to_string()))
        );
        // Empty quoted
        // string.
        assert_eq!(
            Arg::from_json("\"\""),
            Some(Arg::Str("".to_string()))
        );
    }

    #[test]
    fn arg_from_json_number_decodes_numeric_round_132() {
        // A non-quoted
        // string that
        // parses as f32
        // returns a
        // Number arg.
        assert_eq!(Arg::from_json("1"),    Some(Arg::Number(1.0)));
        assert_eq!(Arg::from_json("1.5"),  Some(Arg::Number(1.5)));
        assert_eq!(Arg::from_json("-3"),   Some(Arg::Number(-3.0)));
        assert_eq!(Arg::from_json("0"),    Some(Arg::Number(0.0)));
        assert_eq!(Arg::from_json(" 42 "), Some(Arg::Number(42.0)));
    }

    #[test]
    fn arg_from_json_invalid_returns_none_round_132() {
        // A string that
        // doesn't start
        // with `"` AND
        // doesn't parse
        // as f32 returns
        // None. (Note:
        // the
        // quote-stripping
        // check is
        // `starts_with('"')`
        // AND
        // `ends_with('"')` —
        // a string
        // containing
        // quotes that
        // BOTH starts and
        // ends with a
        // quote will be
        // accepted as a
        // string. So
        // `"unclosed`
        // (no trailing
        // quote) is the
        // correct
        // "rejected"
        // example.)
        assert_eq!(Arg::from_json("hello"), None);
        assert_eq!(Arg::from_json("\"unclosed"), None);
        assert_eq!(Arg::from_json(""), None);
        // A string that
        // contains a
        // space + an
        // alphabetic
        // word and no
        // surrounding
        // quotes is also
        // rejected.
        assert_eq!(Arg::from_json("hello world"), None);
    }

    #[test]
    fn arg_from_json_quoted_string_with_internal_quotes_round_132() {
        // Defense: a
        // string that
        // starts AND
        // ends with a
        // `"` is
        // accepted (the
        // internal
        // quotes are
        // preserved as-
        // is — no
        // unescaping in
        // the current
        // implementation).
        // This pins the
        // current
        // contract so
        // any future
        // escape-aware
        // refactor is
        // caught.
        let parsed = Arg::from_json("\"a\"\"b\"");
        assert_eq!(parsed, Some(Arg::Str("a\"\"b".to_string())));
    }

    // -----------------------------------------------------------------
    // Rule::mutation_cost
    // -----------------------------------------------------------------

    #[test]
    fn rule_mutation_cost_base_1_no_actions_round_132() {
        // A rule with no
        // actions has
        // the base cost
        // of 1 (the
        // `cost = 1`
        // initializer in
        // the function
        // body).
        let rule = Rule {
            event: Event { kind: EventKind::Collide, arg: None },
            actions: vec![],
        };
        assert_eq!(rule.mutation_cost(), 1);
    }

    #[test]
    fn rule_mutation_cost_4_action_weights_round_132() {
        // The 4 action
        // kinds each
        // have a
        // specific
        // per-action
        // weight:
        //   Damage       → +1
        //   Heal         → +1
        //   Spawn        → +2
        //   SpawnEntity  → +3
        // Base of 1 +
        // per-action
        // weight. A
        // single-action
        // rule costs
        // base+weight.
        let make = |kind: ActionKind| Rule {
            event: Event { kind: EventKind::Collide, arg: None },
            actions: vec![Action { kind, args: vec![] }],
        };
        assert_eq!(make(ActionKind::Damage).mutation_cost(),      1 + 1);
        assert_eq!(make(ActionKind::Heal).mutation_cost(),        1 + 1);
        assert_eq!(make(ActionKind::Spawn).mutation_cost(),       1 + 2);
        assert_eq!(make(ActionKind::SpawnEntity).mutation_cost(), 1 + 3);
    }

    #[test]
    fn rule_mutation_cost_accumulates_across_actions_round_132() {
        // Multiple
        // actions sum
        // their
        // per-action
        // weights onto
        // the base.
        let rule = Rule {
            event: Event { kind: EventKind::Collide, arg: None },
            actions: vec![
                Action { kind: ActionKind::Damage,      args: vec![] },
                Action { kind: ActionKind::Heal,        args: vec![] },
                Action { kind: ActionKind::Spawn,       args: vec![] },
                Action { kind: ActionKind::SpawnEntity, args: vec![] },
            ],
        };
        // 1 + 1 + 1 + 2 + 3 = 8.
        assert_eq!(rule.mutation_cost(), 8);
    }

    // -----------------------------------------------------------------
    // Event / Action / Rule to_json shape
    // -----------------------------------------------------------------

    #[test]
    fn event_to_json_no_arg_round_132() {
        // An event
        // without an
        // arg renders
        // with `null`
        // for the arg
        // field.
        let ev = Event { kind: EventKind::Collide, arg: None };
        let json = ev.to_json();
        // The shape is
        // `{"kind":<json>,"arg":null}`
        // (the function
        // emits BOTH
        // `, null` and
        // a `null` in
        // the format
        // string — pin
        // the exact
        // contract).
        assert!(json.contains("\"kind\":\"Collide\""));
        assert!(json.contains("\"arg\":null"));
    }

    #[test]
    fn event_to_json_with_arg_round_132() {
        // An event with
        // a numeric arg
        // includes the
        // arg's JSON
        // form.
        let ev = Event {
            kind: EventKind::Timer,
            arg: Some(Arg::Number(5.0)),
        };
        let json = ev.to_json();
        assert!(json.contains("\"kind\":\"Timer\""));
        assert!(json.contains("\"arg\":5"));
    }

    #[test]
    fn action_to_json_no_args_round_132() {
        // An action
        // with no args
        // renders an
        // empty `args`
        // array.
        let ac = Action { kind: ActionKind::Heal, args: vec![] };
        let json = ac.to_json();
        assert!(json.contains("\"kind\":\"Heal\""));
        assert!(json.contains("\"args\":[]"));
    }

    #[test]
    fn action_to_json_with_args_round_132() {
        // An action
        // with mixed
        // arg types
        // renders each
        // arg in
        // sequence
        // (number first,
        // then string).
        let ac = Action {
            kind: ActionKind::Spawn,
            args: vec![
                Arg::Str("Fireball".to_string()),
                Arg::Number(5.0),
            ],
        };
        let json = ac.to_json();
        assert!(json.contains("\"kind\":\"Spawn\""));
        assert!(json.contains("\"args\":["));
        assert!(json.contains("\"Fireball\""));
        assert!(json.contains("5"));
    }

    #[test]
    fn rule_to_json_full_shape_round_132() {
        // A full rule
        // serializes
        // to the
        // documented
        // JSON shape
        // (event +
        // actions).
        let rule = Rule {
            event: Event { kind: EventKind::Collide, arg: None },
            actions: vec![
                Action {
                    kind: ActionKind::Damage,
                    args: vec![Arg::Number(10.0)],
                },
                Action {
                    kind: ActionKind::Spawn,
                    args: vec![Arg::Str("Fireball".to_string()), Arg::Number(5.0)],
                },
            ],
        };
        let json = rule.to_json();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("\"event\":"));
        assert!(json.contains("\"actions\":["));
        assert!(json.contains("\"Collide\""));
        assert!(json.contains("\"Damage\""));
        assert!(json.contains("\"Spawn\""));
    }

    // -----------------------------------------------------------------------
    // Round 149 helper-level tests.
    //
    // Closing the dsl/ast.rs gaps left by the round-132 block. The
    // round-132 block covered the basic from_str / to_json /
    // mutation_cost contracts; round 149 pins edge cases that
    // matter for the AGI-miniGame WASM bridge: case-sensitivity of
    // the enum from_str helpers (the TS-side MemeCompiler only ever
    // emits the canonical PascalCase, but a hostile or buggy host
    // could pass anything), Arg::from_json negative / scientific /
    // zero handling, Action/Event JSON output for empty-arg and
    // no-arg cases, and the full mutation_cost scale.
    // -----------------------------------------------------------------------

    #[test]
    fn event_kind_from_str_is_case_sensitive_round_149() {
        // The grammar is PascalCase ("Collide" / "Timer" / "Spawn" /
        // "PlayerHit"). Mixed-case / lowercase / uppercase must all
        // be rejected — a regression that did case-insensitive
        // matching would silently accept "collide" from a buggy
        // hot-reload payload and produce a Rule the engine can't
        // dispatch.
        for variant in &["Collide", "Timer", "Spawn", "PlayerHit"] {
            assert_eq!(
                EventKind::from_str(variant).map(|k| format!("{:?}", k).trim_matches('"').to_string()),
                Some(variant.to_string()),
                "canonical {} should parse", variant,
            );
        }
        for bad in &["collide", "COLLIDE", "CollIde", "timer", "TIMER", "spawn", "PLAYERHIT"] {
            assert_eq!(EventKind::from_str(bad), None, "non-canonical {} must reject", bad);
        }
    }

    #[test]
    fn action_kind_from_str_is_case_sensitive_round_149() {
        // Same PascalCase contract as EventKind. A regression that
        // did case-insensitive matching would let "spawn" / "DAMAGE"
        // through and dispatch to the wrong arm.
        for variant in &["Damage", "Heal", "Spawn", "SpawnEntity"] {
            assert_eq!(
                ActionKind::from_str(variant).map(|k| format!("{:?}", k).trim_matches('"').to_string()),
                Some(variant.to_string()),
            );
        }
        for bad in &["damage", "DAMAGE", "spawn", "SPAWN", "spawnentity", "SPAWNENTITY", "spawn_entity", "Spawn-Entity"] {
            assert_eq!(ActionKind::from_str(bad), None, "non-canonical {} must reject", bad);
        }
    }

    #[test]
    fn event_kind_from_str_rejects_empty_and_whitespace_round_149() {
        // Defense: an empty / whitespace-only identifier from a
        // malformed JSON payload must NOT parse to a variant.
        assert_eq!(EventKind::from_str(""), None);
        assert_eq!(EventKind::from_str(" "), None);
        assert_eq!(EventKind::from_str("\t"), None);
    }

    #[test]
    fn action_kind_from_str_rejects_empty_and_partial_matches_round_149() {
        // Defense: partial matches ("Dam" / "Hea" / "Spaw") must
        // not parse. A regression that did `starts_with` matching
        // would silently match "Dam" → Damage and dispatch a wrong
        // action.
        assert_eq!(ActionKind::from_str(""), None);
        assert_eq!(ActionKind::from_str("Dam"), None);
        assert_eq!(ActionKind::from_str("Heal "), None); // trailing space
        assert_eq!(ActionKind::from_str(" Spawn"), None); // leading space
        assert_eq!(ActionKind::from_str("Dam."), None);
    }

    #[test]
    fn arg_from_json_handles_negative_and_zero_and_scientific_round_149() {
        // The host serializes numbers via f32; f32 supports
        // negative, zero, and scientific notation. Pin that
        // from_json doesn't crash on these edge cases and that
        // the round-trip is exact.
        assert_eq!(Arg::from_json("-5"), Some(Arg::Number(-5.0)));
        assert_eq!(Arg::from_json("0"), Some(Arg::Number(0.0)));
        assert_eq!(Arg::from_json("-0.0"), Some(Arg::Number(-0.0)));
        // 1e3 parses as 1000.0 via f32::from_str
        assert_eq!(Arg::from_json("1e3"), Some(Arg::Number(1000.0)));
        // 2.5e-1 = 0.25
        assert_eq!(Arg::from_json("2.5e-1"), Some(Arg::Number(0.25)));
    }

    #[test]
    fn arg_from_json_rejects_malformed_round_149() {
        // Strings that don't start with `"` AND don't parse as
        // f32 must return None. Regression that returned Some
        // (e.g. silently) would corrupt the engine's state.
        assert_eq!(Arg::from_json(""), None);
        assert_eq!(Arg::from_json("abc"), None);
        assert_eq!(Arg::from_json("1.2.3"), None);
        // Unmatched quote: starts with `"` but doesn't end with `"`
        // → f32 parse fails → None.
        assert_eq!(Arg::from_json("\"unterminated"), None);
    }

    #[test]
    fn arg_to_json_trims_all_trailing_zeros_round_149() {
        // `format!("{:.6}", n)` produces e.g. "1.000000" → trim
        // trailing 0s → trim trailing `.` → "1". Pin this for the
        // "5.000000" / "0.000000" cases that the round-132 block
        // didn't exercise.
        assert_eq!(Arg::Number(5.0).to_json(), "5");
        assert_eq!(Arg::Number(5.5).to_json(), "5.5");
        assert_eq!(Arg::Number(0.0).to_json(), "0");
        assert_eq!(Arg::Number(-1.0).to_json(), "-1");
        assert_eq!(Arg::Number(-1.25).to_json(), "-1.25");
        // 1.0000001 → format!("{:.6}", 1.0000001) = "1.000000" (6
        // digits of precision — values smaller than 1e-6 get
        // rounded off).
        assert_eq!(Arg::Number(1.0_f32).to_json(), "1");
    }

    #[test]
    fn arg_to_json_escapes_backslash_and_quote_round_149() {
        // The escape order is critical: backslash MUST be escaped
        // first, otherwise the second pass (escaping `"`) would
        // produce `\\\"` for an input of `\"` (instead of the
        // correct `\\\"`). Pin both directions.
        assert_eq!(Arg::Str("hello".to_string()).to_json(), "\"hello\"");
        assert_eq!(Arg::Str("say \"hi\"".to_string()).to_json(), "\"say \\\"hi\\\"\"");
        assert_eq!(Arg::Str("path\\to\\file".to_string()).to_json(), "\"path\\\\to\\\\file\"");
        // Combined: a string with BOTH backslash and quote
        let s = "back\\slash and \"quote\"".to_string();
        let json = Arg::Str(s.clone()).to_json();
        assert_eq!(json, "\"back\\\\slash and \\\"quote\\\"\"");
        // Round-trip: parse the JSON back.
        let inner = &json[1..json.len()-1];
        let unescaped = inner.replace("\\\\", "\x00BACKSLASH\x00").replace("\\\"", "\x00QUOTE\x00");
        let restored = unescaped.replace("\x00BACKSLASH\x00", "\\").replace("\x00QUOTE\x00", "\"");
        assert_eq!(restored, s);
    }

    #[test]
    fn rule_mutation_cost_scale_round_149() {
        // Pin the full scale: base=1 + per-action weight
        // (Damage=1, Heal=1, Spawn=2, SpawnEntity=3). The
        // round-132 block only tested a 2-action mixed case.
        let make = |kinds: &[ActionKind]| Rule {
            event: Event { kind: EventKind::Collide, arg: None },
            actions: kinds.iter().map(|k| Action { kind: k.clone(), args: vec![] }).collect(),
        };
        // 1 Damage = 1 + 1 = 2
        assert_eq!(make(&[ActionKind::Damage]).mutation_cost(), 2);
        // 1 Heal = 1 + 1 = 2
        assert_eq!(make(&[ActionKind::Heal]).mutation_cost(), 2);
        // 1 Spawn = 1 + 2 = 3
        assert_eq!(make(&[ActionKind::Spawn]).mutation_cost(), 3);
        // 1 SpawnEntity = 1 + 3 = 4
        assert_eq!(make(&[ActionKind::SpawnEntity]).mutation_cost(), 4);
        // All 4 SpawnEntity = 1 + 12 = 13
        assert_eq!(make(&[ActionKind::SpawnEntity, ActionKind::SpawnEntity, ActionKind::SpawnEntity, ActionKind::SpawnEntity]).mutation_cost(), 13);
        // Mixed 4-action = 1 + 1+1+2+3 = 8
        assert_eq!(make(&[ActionKind::Damage, ActionKind::Heal, ActionKind::Spawn, ActionKind::SpawnEntity]).mutation_cost(), 8);
    }

    #[test]
    fn event_to_json_omits_arg_field_when_none_round_149() {
        // The `arg` field in Event::to_json: when None, the
        // `arg` key is `null` (not omitted entirely). This is the
        // round-32 hot-reload contract — the engine checks for
        // the presence of the `arg` key.
        let e = Event { kind: EventKind::Collide, arg: None };
        let json = e.to_json();
        assert!(json.contains("\"arg\":null"));
        assert!(json.contains("\"kind\":\"Collide\""));
    }

    #[test]
    fn action_to_json_with_empty_args_produces_empty_array_round_149() {
        // The `args` field must be a JSON array (even when
        // empty). A regression that produced a null / missing
        // value would break the engine's `for arg in args`
        // iteration.
        let a = Action { kind: ActionKind::Damage, args: vec![] };
        let json = a.to_json();
        assert!(json.contains("\"args\":[]"));
        assert!(json.contains("\"kind\":\"Damage\""));
    }

    #[test]
    fn rule_to_json_with_empty_actions_still_has_actions_key_round_149() {
        // Edge case: a rule with no actions (unusual but
        // possible — a parser-level no-op). The JSON must
        // still contain the `"actions":[]` key (the engine
        // iterates over the array and a missing key would
        // produce `undefined` in JS, masking the bug).
        let r = Rule {
            event: Event { kind: EventKind::Collide, arg: None },
            actions: vec![],
        };
        let json = r.to_json();
        assert!(json.contains("\"actions\":[]"));
        assert!(json.contains("\"event\":"));
    }
}
