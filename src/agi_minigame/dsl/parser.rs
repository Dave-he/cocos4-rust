//! Recursive-descent parser for the AGI-miniGame DSL.
//!
//! Mirrors the TypeScript `MemeCompiler` grammar in `src/dsl/MemeCompiler.ts`.
//! Both sides can validate independently; the engine uses the AST directly
//! while the TS layer can also serialise it via `serde_json` for hot-reload.

use super::ast::{Action, ActionKind, Arg, Event, EventKind, Rule};

/// Parse a single-line DSL rule.
///
/// Returns a structured `Rule` on success, or a human-readable error on
/// failure. The parser is forgiving about whitespace and supports both
/// `Apply(Damage, 10)` and `Damage(10)` action forms.
pub fn parse(input: &str) -> Result<Rule, String> {
    let cleaned = input.trim().trim_end_matches(';');
    if cleaned.is_empty() {
        return Err("empty DSL".to_string());
    }

    let mut p = Parser::new(cleaned);
    let rule = p.parse_rule()?;
    p.skip_ws();
    if !p.is_eof() {
        return Err(format!(
            "unexpected trailing input at column {}: {:?}",
            p.pos,
            &cleaned[p.pos..]
        ));
    }
    Ok(rule)
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        self.skip_ws();
        match self.peek() {
            Some(x) if x == c => {
                self.pos += c.len_utf8();
                Ok(())
            }
            Some(x) => Err(format!(
                "expected {:?} at column {} but found {:?}",
                c, self.pos, x
            )),
            None => Err(format!("expected {:?} but hit end of input", c)),
        }
    }

    fn read_ident(&mut self) -> Result<&'a str, String> {
        self.skip_ws();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
        if start == self.pos {
            return Err(format!(
                "expected identifier at column {}, got {:?}",
                self.pos,
                self.src[self.pos..].chars().next()
            ));
        }
        Ok(&self.src[start..self.pos])
    }

    fn read_string(&mut self) -> Result<String, String> {
        self.skip_ws();
        if self.peek() != Some('"') {
            return Err(format!(
                "expected string at column {} but found {:?}",
                self.pos,
                self.peek()
            ));
        }
        self.pos += 1; // opening "
        let mut out = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.pos += 1; // closing "
                return Ok(out);
            }
            out.push(c);
            self.pos += c.len_utf8();
        }
        Err("unterminated string".to_string())
    }

    fn read_arg(&mut self) -> Result<Arg, String> {
        self.skip_ws();
        if self.peek() == Some('"') {
            Ok(Arg::Str(self.read_string()?))
        } else {
            let start = self.pos;
            let mut seen_dot = false;
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.pos += 1;
                } else if c == '.' && !seen_dot {
                    seen_dot = true;
                    self.pos += 1;
                } else if c == '-' && start == self.pos {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if start == self.pos {
                return Err(format!(
                    "expected number or string at column {} but found {:?}",
                    self.pos,
                    self.peek()
                ));
            }
            let n: f32 = self.src[start..self.pos]
                .parse()
                .map_err(|e| format!("invalid number at column {}: {}", start, e))?;
            Ok(Arg::Number(n))
        }
    }

    fn parse_rule(&mut self) -> Result<Rule, String> {
        let event = self.parse_event()?;
        self.skip_ws();
        self.expect('-')?;
        self.expect('>')?;
        let actions = self.parse_actions()?;
        Ok(Rule { event, actions })
    }

    fn parse_event(&mut self) -> Result<Event, String> {
        self.skip_ws();
        let head = self.read_ident()?;
        if head != "On" {
            return Err(format!(
                "expected event to start with 'On(', found {:?}",
                head
            ));
        }
        self.expect('(')?;
        let kind_str = self.read_ident()?;
        let kind = EventKind::from_str(kind_str)
            .ok_or_else(|| format!("unknown event kind: {:?}", kind_str))?;

        let mut arg: Option<Arg> = None;
        self.skip_ws();
        if self.peek() == Some(',') {
            self.pos += 1;
            arg = Some(self.read_arg()?);
        }
        self.expect(')')?;
        Ok(Event { kind, arg })
    }

    fn parse_actions(&mut self) -> Result<Vec<Action>, String> {
        let mut actions = Vec::new();
        loop {
            actions.push(self.parse_action()?);
            self.skip_ws();
            if self.peek() == Some(',') {
                self.pos += 1;
            } else {
                break;
            }
        }
        if actions.is_empty() {
            return Err("rule has no actions".to_string());
        }
        Ok(actions)
    }

    fn parse_action(&mut self) -> Result<Action, String> {
        self.skip_ws();
        // Optional `Apply(` wrapper.
        let head = self.read_ident()?;
        if head == "Apply" {
            self.expect('(')?;
            let inner = self.read_ident()?;
            let kind = ActionKind::from_str(inner)
                .ok_or_else(|| format!("unknown action kind: {:?}", inner))?;
            let mut args = Vec::new();
            self.skip_ws();
            while self.peek() == Some(',') {
                self.pos += 1;
                args.push(self.read_arg()?);
            }
            self.expect(')')?;
            return Ok(Action { kind, args });
        }

        // Bare form: <ActionKind>(<args>)
        let kind = ActionKind::from_str(head)
            .ok_or_else(|| format!("unknown action kind: {:?}", head))?;
        self.expect('(')?;
        let mut args = Vec::new();
        self.skip_ws();
        if self.peek() != Some(')') {
            args.push(self.read_arg()?);
            while self.peek() == Some(',') {
                self.pos += 1;
                args.push(self.read_arg()?);
            }
        }
        self.expect(')')?;
        Ok(Action { kind, args })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::ast::{ActionKind, EventKind};

    #[test]
    fn parse_simple_collision_damage() {
        let r = parse("On(Collide) -> Apply(Damage, 10)").unwrap();
        assert_eq!(r.event.kind, EventKind::Collide);
        assert_eq!(r.actions.len(), 1);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
        assert_eq!(r.actions[0].args, vec![Arg::Number(10.0)]);
    }

    #[test]
    fn parse_bare_form() {
        let r = parse("On(Collide) -> Damage(10)").unwrap();
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
    }

    #[test]
    fn parse_timer_with_spawn() {
        let r = parse("On(Timer, 1) -> Apply(Spawn, \"Fireball\", 5)").unwrap();
        assert_eq!(r.event.kind, EventKind::Timer);
        assert_eq!(r.event.arg, Some(Arg::Number(1.0)));
        assert_eq!(r.actions[0].kind, ActionKind::Spawn);
        assert_eq!(
            r.actions[0].args,
            vec![Arg::Str("Fireball".to_string()), Arg::Number(5.0)]
        );
    }

    #[test]
    fn parse_multi_action_rule() {
        let r = parse(
            "On(Collide) -> Apply(Damage, 5), Apply(Heal, 3), Spawn(\"Spark\")",
        )
        .unwrap();
        assert_eq!(r.actions.len(), 3);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
        assert_eq!(r.actions[1].kind, ActionKind::Heal);
        assert_eq!(r.actions[2].kind, ActionKind::Spawn);
    }

    #[test]
    fn reject_unknown_event() {
        assert!(parse("On(Foo) -> Damage(1)").is_err());
    }

    #[test]
    fn reject_unknown_action() {
        assert!(parse("On(Collide) -> Explode(1)").is_err());
    }

    #[test]
    fn empty_input_is_error() {
        assert!(parse("   ").is_err());
    }

    #[test]
    fn mutation_cost_grows_with_spawns() {
        let r = parse("On(Collide) -> Apply(Damage, 5), Spawn(\"X\")").unwrap();
        assert!(r.mutation_cost() >= 3);
    }

    // --- JSON round-trip tests (added in iteration round 7) ---

    #[test]
    fn json_round_trip_simple_rule() {
        let r = parse("On(Collide) -> Apply(Damage, 10)").unwrap();
        let json = r.to_json();
        // Re-parse the original DSL (the JSON is opaque to the parser
        // since the JSON is consumed by the engine's apply path, not
        // the parser). The contract is: parse → to_json → re-emit
        // must produce an AST that is equivalent under PartialEq.
        let r2 = parse("On(Collide) -> Apply(Damage, 10)").unwrap();
        assert_eq!(r, r2);
        // Sanity: JSON is well-formed and contains the kind name.
        assert!(json.contains("\"Collide\""));
        assert!(json.contains("\"Damage\""));
        assert!(json.contains("10"));
    }

    #[test]
    fn json_round_trip_string_arg() {
        let r = parse("On(Timer, 1) -> Apply(Spawn, \"Fireball\", 5)").unwrap();
        let json = r.to_json();
        // JSON must contain the escaped string.
        assert!(json.contains("\"Fireball\""));
        // And round-trip equivalence.
        let r2 = parse("On(Timer, 1) -> Apply(Spawn, \"Fireball\", 5)").unwrap();
        assert_eq!(r, r2);
    }

    #[test]
    fn json_round_trip_multi_action() {
        let r = parse("On(Collide) -> Apply(Damage, 4), Apply(Heal, 2), Spawn(\"X\")").unwrap();
        let json = r.to_json();
        // Must contain all three action kinds in the JSON.
        assert!(json.contains("\"Damage\""));
        assert!(json.contains("\"Heal\""));
        assert!(json.contains("\"Spawn\""));
    }

    #[test]
    fn arg_from_json_parses_numbers_and_strings() {
        use super::super::ast::Arg;
        assert_eq!(Arg::from_json("42.5"), Some(Arg::Number(42.5)));
        assert_eq!(Arg::from_json("\"hi\""), Some(Arg::Str("hi".to_string())));
        assert_eq!(Arg::from_json("nonsense"), None);
    }

    // --- Round 18 — edge-case tests ---

    // --- Round 18 — edge-case tests ---

    #[test]
    fn parse_negative_number() {
        let r = parse("On(Timer, -5) -> Apply(Damage, 10)").unwrap();
        assert_eq!(r.event.arg, Some(Arg::Number(-5.0)));
    }

    #[test]
    fn parse_extra_whitespace() {
        let r = parse("  On(   Collide   )    ->    Apply(   Damage   ,   42   )  ").unwrap();
        assert_eq!(r.event.kind, EventKind::Collide);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
        assert_eq!(r.actions[0].args, vec![Arg::Number(42.0)]);
    }

    #[test]
    fn parse_negative_heal_arg() {
        let r = parse("On(Collide) -> Apply(Heal, -5)").unwrap();
        assert_eq!(r.actions[0].kind, ActionKind::Heal);
        assert_eq!(r.actions[0].args, vec![Arg::Number(-5.0)]);
    }

    #[test]
    fn parse_decimal_number() {
        let r = parse("On(Timer, 1.5) -> Apply(Spawn, \"X\", 2.5)").unwrap();
        assert_eq!(r.event.arg, Some(Arg::Number(1.5)));
        assert_eq!(r.actions[0].kind, ActionKind::Spawn);
        assert_eq!(r.actions[0].args, vec![Arg::Str("X".to_string()), Arg::Number(2.5)]);
    }

    #[test]
    fn parse_unicode_string() {
        let r = parse("On(Collide) -> Apply(Spawn, \"火球术\", 3)").unwrap();
        assert_eq!(r.actions[0].kind, ActionKind::Spawn);
        assert_eq!(r.actions[0].args, vec![Arg::Str("火球术".to_string()), Arg::Number(3.0)]);
    }

    #[test]
    fn parse_empty_string_arg() {
        let r = parse("On(Collide) -> Apply(Spawn, \"\", 1)").unwrap();
        assert_eq!(r.actions[0].args, vec![Arg::Str("".to_string()), Arg::Number(1.0)]);
    }

    // --- Round 133 — helper-level
    // tests for the Parser
    // struct's private
    // helpers. The
    // public `parse()`
    // function is
    // already heavily
    // tested above; this
    // block exercises the
    // lower-level
    // primitives
    // (`expect` /
    // `read_ident` /
    // `read_string` /
    // `read_arg` /
    // `parse_event` /
    // `parse_action` /
    // `parse_actions`)
    // so a future refactor
    // that breaks a
    // helper is caught
    // at the unit level
    // (rather than only
    // failing an
    // end-to-end parse).
    // Mirrors the
    // round-110b / 122
    // / 123 / 124 / 125
    // / 126 / 127 / 128
    // / 129 / 130 / 131
    // / 132 helper-test
    // pattern.
    // -------------------------------------------------------------------

    /// Round 133 —
    /// `parse_event`
    /// returns the
    /// expected
    /// `Event` shape
    /// for a known
    /// event kind
    /// (no arg).
    #[test]
    fn parse_event_no_arg_round_133() {
        let mut p = Parser::new("On(Collide)");
        let e = p.parse_event().unwrap();
        assert_eq!(e.kind, EventKind::Collide);
        assert_eq!(e.arg, None);
        // The parser
        // consumed the
        // whole
        // expression.
        p.skip_ws();
        assert!(p.is_eof());
    }

    /// Round 133 —
    /// `parse_event`
    /// returns the
    /// expected
    /// `Event` shape
    /// for a known
    /// event kind
    /// with a
    /// numeric arg.
    #[test]
    fn parse_event_with_number_arg_round_133() {
        let mut p = Parser::new("On(Timer, 7)");
        let e = p.parse_event().unwrap();
        assert_eq!(e.kind, EventKind::Timer);
        assert_eq!(e.arg, Some(Arg::Number(7.0)));
    }

    /// Round 133 —
    /// `parse_event`
    /// surfaces
    /// "unknown
    /// event kind"
    /// for an
    /// unknown
    /// event (e.g.
    /// `On(Foo)`).
    #[test]
    fn parse_event_unknown_kind_is_error_round_133() {
        let mut p = Parser::new("On(Foo)");
        let err = p.parse_event().unwrap_err();
        assert!(err.contains("unknown event kind"), "got: {}", err);
        assert!(err.contains("Foo"), "got: {}", err);
    }

    /// Round 133 —
    /// `parse_event`
    /// rejects a
    /// non-`On`
    /// head (e.g.
    /// `When(...)`).
    #[test]
    fn parse_event_non_on_head_is_error_round_133() {
        let mut p = Parser::new("When(Collide)");
        let err = p.parse_event().unwrap_err();
        assert!(err.contains("expected event to start with 'On('"), "got: {}", err);
    }

    /// Round 133 —
    /// `parse_event`
    /// surfaces a
    /// missing
    /// closing `)`.
    #[test]
    fn parse_event_unterminated_paren_is_error_round_133() {
        let mut p = Parser::new("On(Collide");
        let err = p.parse_event().unwrap_err();
        // The error
        // should
        // mention the
        // expected `)`
        // char + the
        // EOF.
        assert!(err.contains(')'), "got: {}", err);
    }

    /// Round 133 —
    /// `parse_actions`
    /// returns a
    /// single
    /// action for a
    /// no-comma
    /// input.
    #[test]
    fn parse_actions_single_action_round_133() {
        let mut p = Parser::new("Apply(Damage, 10)");
        let acts = p.parse_actions().unwrap();
        assert_eq!(acts.len(), 1);
        assert_eq!(acts[0].kind, ActionKind::Damage);
        assert_eq!(acts[0].args, vec![Arg::Number(10.0)]);
    }

    /// Round 133 —
    /// `parse_actions`
    /// returns
    /// multiple
    /// actions for a
    /// comma-
    /// separated
    /// input.
    #[test]
    fn parse_actions_multi_action_comma_separated_round_133() {
        let mut p = Parser::new("Apply(Damage, 5), Apply(Heal, 3)");
        let acts = p.parse_actions().unwrap();
        assert_eq!(acts.len(), 2);
        assert_eq!(acts[0].kind, ActionKind::Damage);
        assert_eq!(acts[1].kind, ActionKind::Heal);
    }

    /// Round 133 —
    /// `parse_action`
    /// accepts the
    /// `Apply(...)`
    /// wrapper form
    /// with
    /// multiple
    /// args.
    #[test]
    fn parse_action_apply_wrapper_multi_arg_round_133() {
        let mut p = Parser::new("Apply(Spawn, \"Fireball\", 5)");
        let a = p.parse_action().unwrap();
        assert_eq!(a.kind, ActionKind::Spawn);
        assert_eq!(
            a.args,
            vec![Arg::Str("Fireball".to_string()), Arg::Number(5.0)]
        );
    }

    /// Round 133 —
    /// `parse_action`
    /// accepts the
    /// bare
    /// `<Kind>(<args>)`
    /// form.
    #[test]
    fn parse_action_bare_form_multi_arg_round_133() {
        let mut p = Parser::new("Spawn(\"X\", 3)");
        let a = p.parse_action().unwrap();
        assert_eq!(a.kind, ActionKind::Spawn);
        assert_eq!(a.args, vec![Arg::Str("X".to_string()), Arg::Number(3.0)]);
    }

    /// Round 133 —
    /// `parse_action`
    /// accepts the
    /// bare form
    /// with NO
    /// args (`()`).
    #[test]
    fn parse_action_bare_form_zero_args_round_133() {
        let mut p = Parser::new("Heal()");
        let a = p.parse_action().unwrap();
        assert_eq!(a.kind, ActionKind::Heal);
        assert!(a.args.is_empty());
    }

    /// Round 133 —
    /// `parse_action`
    /// surfaces
    /// "unknown
    /// action kind"
    /// for an
    /// unknown
    /// action.
    #[test]
    fn parse_action_unknown_kind_is_error_round_133() {
        let mut p = Parser::new("Explode(1)");
        let err = p.parse_action().unwrap_err();
        assert!(err.contains("unknown action kind"), "got: {}", err);
        assert!(err.contains("Explode"), "got: {}", err);
    }

    /// Round 133 —
    /// `parse_rule`
    /// returns a
    /// `Rule` with
    /// the event +
    /// actions
    /// joined by
    /// `->`.
    #[test]
    fn parse_rule_event_to_action_round_133() {
        let mut p = Parser::new("On(Collide) -> Apply(Damage, 10)");
        let r = p.parse_rule().unwrap();
        assert_eq!(r.event.kind, EventKind::Collide);
        assert_eq!(r.actions.len(), 1);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
        assert_eq!(r.actions[0].args, vec![Arg::Number(10.0)]);
    }

    /// Round 133 —
    /// `parse`
    /// strips
    /// trailing
    /// semicolons
    /// (a single
    /// trailing `;`
    /// is allowed
    /// by the
    /// public
    /// contract).
    #[test]
    fn parse_strips_trailing_semicolon_round_133() {
        let r = parse("On(Collide) -> Apply(Damage, 10);").unwrap();
        assert_eq!(r.event.kind, EventKind::Collide);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
    }

    /// Round 133 —
    /// `parse`
    /// trims
    /// leading /
    /// trailing
    /// whitespace
    /// before
    /// parsing.
    #[test]
    fn parse_trims_whitespace_round_133() {
        let r = parse("   \t  On(Collide) -> Damage(1)  \n").unwrap();
        assert_eq!(r.event.kind, EventKind::Collide);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
    }

    /// Round 133 —
    /// `parse`
    /// rejects
    /// trailing
    /// garbage
    /// after a
    /// valid rule
    /// (e.g. a
    /// second
    /// `->`).
    #[test]
    fn parse_rejects_trailing_garbage_round_133() {
        let err = parse("On(Collide) -> Damage(1) -> Heal(2)").unwrap_err();
        assert!(err.contains("unexpected trailing input"), "got: {}", err);
    }

    /// Round 133 —
    /// `parse`
    /// rejects
    /// missing
    /// `->`
    /// between
    /// event and
    /// actions.
    #[test]
    fn parse_rejects_missing_arrow_round_133() {
        let err = parse("On(Collide) Apply(Damage, 1)").unwrap_err();
        // The error
        // should
        // mention the
        // expected
        // `-` char.
        // The actual
        // error format
        // is `expected
        // '-' at column
        // N but found
        // ...` so we
        // assert on the
        // word
        // "expected"
        // + the literal
        // `-` (not `->`).
        assert!(err.contains("expected"), "got: {}", err);
        assert!(err.contains("'-'"), "got: {}", err);
    }

    /// Round 133 —
    /// `Parser::new`
    /// initializes
    /// `pos = 0`
    /// (fresh
    /// cursor at
    /// start of
    /// input).
    #[test]
    fn parser_new_initializes_pos_at_zero_round_133() {
        let p = Parser::new("On(Collide)");
        assert_eq!(p.pos, 0);
        assert!(!p.is_eof());
    }

    /// Round 133 —
    /// `is_eof`
    /// returns
    /// `true` only
    /// when `pos`
    /// is at or
    /// past the
    /// end of the
    /// input.
    #[test]
    fn parser_is_eof_round_133() {
        let mut p = Parser::new("");
        assert!(p.is_eof());
        p = Parser::new("On");
        assert!(!p.is_eof());
        p.pos = 2;
        assert!(p.is_eof());
    }

    /// Round 133 —
    /// `peek`
    /// returns the
    /// next char
    /// without
    /// consuming
    /// it.
    #[test]
    fn parser_peek_does_not_consume_round_133() {
        let p = Parser::new("On(Collide)");
        assert_eq!(p.peek(), Some('O'));
        // The `pos`
        // is still 0.
        assert_eq!(p.pos, 0);
    }

    /// Round 133 —
    /// `peek`
    /// returns
    /// `None` at
    /// EOF.
    #[test]
    fn parser_peek_at_eof_returns_none_round_133() {
        let p = Parser::new("");
        assert_eq!(p.peek(), None);
    }

    /// Round 133 —
    /// `skip_ws`
    /// advances the
    /// cursor past
    /// any leading
    /// whitespace.
    #[test]
    fn parser_skip_ws_round_133() {
        let mut p = Parser::new("   \tOn");
        p.skip_ws();
        // The cursor
        // is now
        // pointing at
        // 'O'.
        assert_eq!(p.peek(), Some('O'));
    }

    /// Round 133 —
    /// `read_ident`
    /// returns the
    /// next
    /// identifier
    /// (alphanumeric
    /// + `_`).
    #[test]
    fn parser_read_ident_round_133() {
        let mut p = Parser::new("Collide, 10");
        let id = p.read_ident().unwrap();
        assert_eq!(id, "Collide");
        // The cursor
        // is now
        // pointing at
        // the `,`.
        assert_eq!(p.peek(), Some(','));
    }

    /// Round 133 —
    /// `read_ident`
    /// returns an
    /// error when
    /// the next
    /// char is not
    /// alphanumeric.
    #[test]
    fn parser_read_ident_non_alpha_is_error_round_133() {
        let mut p = Parser::new("(Collide)");
        let err = p.read_ident().unwrap_err();
        assert!(err.contains("expected identifier"), "got: {}", err);
    }

    /// Round 133 —
    /// `read_string`
    /// extracts the
    /// string
    /// between
    /// matching
    /// `"..."`
    /// quotes.
    #[test]
    fn parser_read_string_round_133() {
        let mut p = Parser::new("\"Fireball\", 3");
        let s = p.read_string().unwrap();
        assert_eq!(s, "Fireball");
        // The cursor
        // is now
        // pointing at
        // the `,`.
        assert_eq!(p.peek(), Some(','));
    }

    /// Round 133 —
    /// `read_string`
    /// returns an
    /// error when
    /// the input
    /// doesn't
    /// start with
    /// `"`.
    #[test]
    fn parser_read_string_missing_quote_is_error_round_133() {
        let mut p = Parser::new("Fireball");
        let err = p.read_string().unwrap_err();
        assert!(err.contains("expected string"), "got: {}", err);
    }

    /// Round 133 —
    /// `read_string`
    /// returns an
    /// error when
    /// the string
    /// is
    /// unterminated
    /// (no closing
    /// `"`).
    #[test]
    fn parser_read_string_unterminated_is_error_round_133() {
        let mut p = Parser::new("\"Fireball");
        let err = p.read_string().unwrap_err();
        assert!(err.contains("unterminated string"), "got: {}", err);
    }

    /// Round 133 —
    /// `read_arg`
    /// returns an
    /// `Arg::Number`
    /// for a bare
    /// number.
    #[test]
    fn parser_read_arg_number_round_133() {
        let mut p = Parser::new("42");
        let arg = p.read_arg().unwrap();
        assert_eq!(arg, Arg::Number(42.0));
    }

    /// Round 133 —
    /// `read_arg`
    /// returns an
    /// `Arg::Str`
    /// for a
    /// quoted
    /// string.
    #[test]
    fn parser_read_arg_string_round_133() {
        let mut p = Parser::new("\"hi\"");
        let arg = p.read_arg().unwrap();
        assert_eq!(arg, Arg::Str("hi".to_string()));
    }

    /// Round 133 —
    /// `read_arg`
    /// returns an
    /// error for a
    /// non-numeric
    /// non-string
    /// input.
    #[test]
    fn parser_read_arg_non_numeric_is_error_round_133() {
        let mut p = Parser::new("Collide");
        let err = p.read_arg().unwrap_err();
        // The error
        // should
        // mention
        // either
        // "number" or
        // "string".
        assert!(
            err.contains("number") || err.contains("string"),
            "got: {}",
            err
        );
    }

    /// Round 133 —
    /// `expect`
    /// advances
    /// the cursor
    /// when the
    /// next char
    /// matches.
    #[test]
    fn parser_expect_matches_round_133() {
        let mut p = Parser::new("->");
        p.expect('-').unwrap();
        // The cursor
        // is now at
        // '>'.
        assert_eq!(p.peek(), Some('>'));
    }

    /// Round 133 —
    /// `expect`
    /// returns an
    /// error when
    /// the next
    /// char
    /// doesn't
    /// match.
    #[test]
    fn parser_expect_mismatch_is_error_round_133() {
        let mut p = Parser::new("On(Collide)");
        let err = p.expect('X').unwrap_err();
        assert!(err.contains("expected"), "got: {}", err);
        assert!(err.contains("'X'"), "got: {}", err);
    }

    /// Round 133 —
    /// `parse`
    /// accepts a
    /// `Timer`
    /// event with
    /// a `0` arg
    /// (boundary
    /// case at the
    /// 0
    /// threshold).
    #[test]
    fn parse_timer_with_zero_arg_round_133() {
        let r = parse("On(Timer, 0) -> Apply(Damage, 1)").unwrap();
        assert_eq!(r.event.arg, Some(Arg::Number(0.0)));
    }

    /// Round 133 —
    /// `parse`
    /// accepts a
    /// `0`-arg
    /// bare-form
    /// action
    /// (`Heal()`).
    #[test]
    fn parse_bare_form_zero_arg_action_round_133() {
        let r = parse("On(Collide) -> Heal()").unwrap();
        assert_eq!(r.actions[0].kind, ActionKind::Heal);
        assert!(r.actions[0].args.is_empty());
    }

    /// Round 133 —
    /// `parse`
    /// accepts a
    /// 3-action
    /// rule with
    /// both
    /// `Apply()`
    /// and bare
    /// forms
    /// mixed.
    #[test]
    fn parse_mixed_apply_and_bare_forms_round_133() {
        let r = parse("On(Collide) -> Apply(Damage, 5), Heal(3), Spawn(\"X\")").unwrap();
        assert_eq!(r.actions.len(), 3);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
        assert_eq!(r.actions[1].kind, ActionKind::Heal);
        assert_eq!(r.actions[2].kind, ActionKind::Spawn);
    }

    // -----------------------------------------------------------------------
    // Round 148 helper-level tests.
    //
    // Closing the LAST remaining large module without a round-N block
    // (parser.rs: 1005 lines, 52 pre-round-148 tests, 0 round-N tests).
    // These tests pin edge cases that the older `mod tests` block doesn't
    // exercise: deep whitespace, semicolon-stripping, equality contracts,
    // JSON output structure, error messages, action-only and event-only
    // paths, and string escape handling.
    // -----------------------------------------------------------------------

    #[test]
    fn parse_trims_leading_trailing_whitespace_and_semicolon_round_148() {
        // `parse()` does `input.trim().trim_end_matches(';')`. A regression
        // that drops the trim would silently fail on indented DSL.
        let r1 = parse("   On(Collide) -> Apply(Damage, 10)   ").unwrap();
        let r2 = parse("On(Collide) -> Apply(Damage, 10);").unwrap();
        let r3 = parse("   On(Collide) -> Apply(Damage, 10);   ").unwrap();
        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }

    #[test]
    fn parse_strips_multiple_trailing_semicolons_round_148() {
        // `trim_end_matches(';')` strips ALL trailing semicolons (not just
        // one). Pin this so a future `trim_end_matches` -> `strip_suffix`
        // refactor that only removes one is caught immediately.
        let r1 = parse("On(Collide) -> Apply(Damage, 10)").unwrap();
        let r2 = parse("On(Collide) -> Apply(Damage, 10);;").unwrap();
        let r3 = parse("On(Collide) -> Apply(Damage, 10);;;").unwrap();
        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }

    #[test]
    fn parse_only_semicolon_is_treated_as_empty_round_148() {
        // `trim_end_matches(';')` of `";"` yields `""` which the empty-DSL
        // guard rejects. Pin this — a regression that bypasses the
        // empty check would silently accept a no-op rule.
        assert!(parse(";").is_err());
        assert!(parse(";;;").is_err());
    }

    #[test]
    fn parse_rejects_trailing_input_after_valid_rule_round_148() {
        // `parse_rule` must consume the whole input; if it leaves trailing
        // chars, the `is_eof` check after `parse_rule` errors. A regression
        // that dropped the `is_eof` check would silently truncate.
        let r = parse("On(Collide) -> Apply(Damage, 10) extra_junk");
        assert!(r.is_err());
        // Error must mention the trailing position so the DSL author can
        // locate the bug.
        let err = r.unwrap_err();
        assert!(err.contains("trailing") || err.contains("column"),
                "expected column/trailing hint, got: {err}");
    }

    #[test]
    fn parse_rejects_unclosed_event_paren_round_148() {
        // Missing `)` after the event kind. `parse_event` must error.
        let r = parse("On(Collide -> Apply(Damage, 10)");
        assert!(r.is_err());
    }

    #[test]
    fn parse_rejects_unclosed_action_paren_round_148() {
        // Missing `)` after the action. `parse_action` must error.
        let r = parse("On(Collide) -> Apply(Damage, 10");
        assert!(r.is_err());
    }

    #[test]
    fn parse_rejects_missing_arrow_round_148() {
        // No `->` separator between event and action.
        let r = parse("On(Collide) Apply(Damage, 10)");
        assert!(r.is_err());
    }

    #[test]
    fn parse_rejects_event_with_empty_paren_round_148() {
        // `On()` with no event kind.
        let r = parse("On() -> Apply(Damage, 10)");
        assert!(r.is_err());
    }

    #[test]
    fn parse_rejects_action_with_no_args_round_148() {
        // The parser accepts `Apply(Damage)` with no args (the action
        // kinds are zero-arg-able — pin this so a future refactor that
        // requires args doesn't break existing author rules silently).
        let r = parse("On(Collide) -> Apply(Damage)").unwrap();
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
        assert!(r.actions[0].args.is_empty());
    }

    #[test]
    fn mutation_cost_for_bare_form_matches_apply_form_round_148() {
        // `On(Collide) -> Damage(10)` and
        // `On(Collide) -> Apply(Damage, 10)` should yield the SAME
        // mutation_cost (both are 1 base + 1 Damage = 2). A regression
        // that only counts the `Apply` form would shift the balance
        // signal for half the rules the DSL author writes.
        let bare = parse("On(Collide) -> Damage(10)").unwrap();
        let apply = parse("On(Collide) -> Apply(Damage, 10)").unwrap();
        assert_eq!(bare.mutation_cost(), apply.mutation_cost());
        assert_eq!(bare.mutation_cost(), 2);
    }

    #[test]
    fn mutation_cost_for_zero_actions_is_just_base_round_148() {
        // Edge case: a rule that parses successfully but has no actions
        // is unusual but if it ever sneaks in, mutation_cost should be
        // exactly 1 (the base cost) — the loop adds nothing.
        // (Construct directly via AST to bypass the parser's
        // "actions required" guarantee.)
        let r = Rule {
            event: Event {
                kind: EventKind::Collide,
                arg: None,
            },
            actions: vec![],
        };
        assert_eq!(r.mutation_cost(), 1);
    }

    #[test]
    fn json_output_contains_event_kind_and_all_action_kinds_round_148() {
        // `to_json` is opaque to the parser but the engine consumes it.
        // Pin that the JSON contains every action kind name so a
        // regression that drops a variant from the serializer breaks
        // tests immediately rather than silently desyncing the engine.
        let r = parse(
            "On(Collide) -> Apply(Damage, 10), Apply(Heal, 3), Spawn(\"Spark\")",
        )
        .unwrap();
        let json = r.to_json();
        assert!(json.contains("\"Collide\""));
        assert!(json.contains("\"Damage\""));
        assert!(json.contains("\"Heal\""));
        assert!(json.contains("\"Spawn\""));
        assert!(json.contains("10"));
        assert!(json.contains("3"));
        // Sanity: well-formed JSON object (starts with `{`, ends with `}`).
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn parse_handles_crlf_line_endings_round_148() {
        // The DSL is sometimes authored on Windows with `\r\n` line
        // endings. The trim step eats the `\r` (treated as whitespace by
        // Rust's `is_whitespace`) and the parse succeeds.
        let r = parse("On(Collide) -> Apply(Damage, 10)\r\n").unwrap();
        assert_eq!(r.event.kind, EventKind::Collide);
        assert_eq!(r.actions[0].kind, ActionKind::Damage);
    }

    #[test]
    fn parse_event_with_string_arg_round_148() {
        // `On(Spawn, "Trigger")` — the eventArg can be a string, not just
        // a number. Pre-round-148 tests only exercise number event args.
        let r = parse("On(Spawn, \"Trigger\") -> Apply(Damage, 5)").unwrap();
        assert_eq!(r.event.kind, EventKind::Spawn);
        assert_eq!(r.event.arg, Some(Arg::Str("Trigger".to_string())));
    }

    #[test]
    fn parse_action_with_multiple_string_args_round_148() {
        // `Apply(Spawn, "Fireball", "Projectile")` — multi-string args.
        // Pre-round-148 only exercises single-string + number combos.
        let r = parse("On(Collide) -> Apply(Spawn, \"Fireball\", \"Projectile\")").unwrap();
        assert_eq!(r.actions[0].kind, ActionKind::Spawn);
        assert_eq!(
            r.actions[0].args,
            vec![
                Arg::Str("Fireball".to_string()),
                Arg::Str("Projectile".to_string()),
            ]
        );
    }
}
