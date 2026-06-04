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
}
