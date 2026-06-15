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
}
