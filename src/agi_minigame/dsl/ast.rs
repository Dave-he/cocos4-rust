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
