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
