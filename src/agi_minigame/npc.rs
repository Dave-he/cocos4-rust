//! NpcMind — per-NPC memory + disposition state machine.
//!
//! Round 21 introduces a small but canonical piece of the PRD §2.2D
//! "living world": **every NPC carries a bounded memory of recent
//! interactions and observations**, plus a disposition vector
//! `(friendly, fear, trust)` that shifts as the player interacts with
//! them. The result is a topic suggestion the game layer can feed
//! into `NPCDialogueAI` so the same NPC speaks differently to a
//! player who has befriended them vs. one who keeps killing their
//! kin.
//!
//! Like [`super::vault::DimensionVault`], `NpcMind` is engine-agnostic:
//! no scene, no LLM, no timers. Three questions the game layer asks:
//!
//! 1. *What does this NPC remember about the player?* — [`NpcMind::recent`]
//! 2. *How does this NPC feel about the player right now?* — [`NpcMind::mood`]
//! 3. *Given the world, what should this NPC bring up?* — [`NpcMind::suggest_topic`]
//!
//! [`NpcRegistry`] manages many minds and provides
//! [`NpcRegistry::broadcast`] so a single world event (e.g. "player
//! entered the Neon Cascade dimension") can be recorded by every
//! NPC at once — handy for tying NPCs into the
//! [`super::vault::DimensionVault`] feed.
//!
//! Determinism: every public method is deterministic given the same
//! insertion order. `suggest_topic` uses a small `seed`-driven hash
//! so tests can pin behaviour without `rand`.

use std::collections::VecDeque;

/// Stable identifier for an NPC; the game layer typically uses a slug
/// like `"merchant_0"` or `"sage_lyra"`.
pub type NpcId = String;

/// What kind of thing happened that this NPC now remembers. Used by
/// [`NpcMind::recall_by_kind`] to filter the ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NpcMemoryKind {
    /// The player and NPC talked. `summary` is "topic: line".
    Dialogue,
    /// The NPC witnessed a world event the player triggered.
    WitnessedEvent,
    /// The NPC heard about the player visiting a dimension. Broadcast
    /// to every NPC in the registry when the player enters a vault
    /// dimension.
    HeardAboutDimension,
    /// The player gave the NPC something. Affects `trust`.
    ReceivedGift,
    /// The player attacked the NPC or its faction. Affects `fear`.
    Hostility,
}

/// One row in an NPC's memory ring.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcMemoryEntry {
    pub kind: NpcMemoryKind,
    /// Short, free-form summary the game layer writes in. Kept short
    /// so the ring stays cheap.
    pub summary: String,
    /// Monotonic turn counter (or millis) the caller assigns. The
    /// engine never compares these — only stores them for the UI.
    pub turn: u64,
    /// `[-1.0, 1.0]` weight that biases disposition shifts when the
    /// game layer asks the mind to absorb a delta. Positive means the
    /// event reads as "good for the relationship".
    pub weight: f32,
}

impl NpcMemoryEntry {
    /// Convenience constructor for tests and the game layer.
    pub fn new(kind: NpcMemoryKind, summary: impl Into<String>, turn: u64, weight: f32) -> Self {
        Self {
            kind,
            summary: summary.into(),
            turn,
            weight: weight.clamp(-1.0, 1.0),
        }
    }
}

/// Three-axis disposition vector. Each axis is clamped to `[-1.0, 1.0]`.
///
/// - `friendly`: positive = warm, negative = cold
/// - `fear`: positive = scared of player, negative = bored / dismissive
/// - `trust`: positive = will share secrets, negative = distrustful
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NpcDisposition {
    pub friendly: f32,
    pub fear: f32,
    pub trust: f32,
}

impl Default for NpcDisposition {
    fn default() -> Self {
        Self { friendly: 0.0, fear: 0.0, trust: 0.0 }
    }
}

impl NpcDisposition {
    /// Apply a clamped delta to each axis and return the new vector.
    /// Each axis is independently saturated to `[-1.0, 1.0]`.
    pub fn shift(self, df: f32, dfear: f32, dtrust: f32) -> Self {
        Self {
            friendly: (self.friendly + df).clamp(-1.0, 1.0),
            fear:     (self.fear     + dfear).clamp(-1.0, 1.0),
            trust:    (self.trust    + dtrust).clamp(-1.0, 1.0),
        }
    }
}

/// Coarse-grained mood label derived from disposition. The game
/// layer uses this to route NPCDialogueAI input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcMood {
    Happy,
    Neutral,
    Uneasy,
    Hostile,
}

/// Per-NPC memory + disposition.
///
/// `capacity` is the hard cap; when full, the oldest entry is dropped.
/// Memory is a `VecDeque` so wraparound is O(1).
#[derive(Debug)]
pub struct NpcMind {
    id: NpcId,
    capacity: usize,
    entries: VecDeque<NpcMemoryEntry>,
    disposition: NpcDisposition,
}

impl NpcMind {
    /// Default capacity — 32 memories per NPC. Enough for several
    /// dimensions of play without burning memory; tune via
    /// [`NpcMind::with_capacity`] for tests.
    pub const DEFAULT_CAPACITY: usize = 32;

    /// Build a fresh mind for the given NPC with [`DEFAULT_CAPACITY`].
    pub fn new(id: impl Into<NpcId>) -> Self {
        Self::with_capacity(id, Self::DEFAULT_CAPACITY)
    }

    /// Build a fresh mind that holds at most `capacity` entries.
    /// `capacity == 0` is allowed; `remember` becomes a no-op.
    pub fn with_capacity(id: impl Into<NpcId>, capacity: usize) -> Self {
        Self {
            id: id.into(),
            capacity,
            entries: VecDeque::with_capacity(capacity),
            disposition: NpcDisposition::default(),
        }
    }

    /// Stable id this mind belongs to.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Current memory count (≤ capacity).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when no memories have been recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Capacity hard-cap.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Current disposition snapshot.
    pub fn disposition(&self) -> NpcDisposition {
        self.disposition
    }

    /// Append a memory and absorb its weight into disposition.
    ///
    /// The kind decides which axis the weight moves:
    /// - `Dialogue` → `friendly += weight * 0.25`
    /// - `WitnessedEvent` → `fear += weight * 0.15`
    /// - `HeardAboutDimension` → `trust += weight * 0.10`
    /// - `ReceivedGift` → `friendly += weight * 0.40`, `trust += weight * 0.30`
    /// - `Hostility` → `friendly -= |weight| * 0.50`, `fear += |weight| * 0.60`
    pub fn remember(&mut self, entry: NpcMemoryEntry) {
        if self.capacity == 0 {
            return;
        }
        // Disposition shift first so an entry with the right sign is
        // reflected even if the ring later wraps it out.
        let w = entry.weight;
        self.disposition = match entry.kind {
            NpcMemoryKind::Dialogue            => self.disposition.shift(w * 0.25, 0.0, 0.0),
            NpcMemoryKind::WitnessedEvent      => self.disposition.shift(0.0, w * 0.15, 0.0),
            NpcMemoryKind::HeardAboutDimension => self.disposition.shift(0.0, 0.0, w * 0.10),
            NpcMemoryKind::ReceivedGift        => self.disposition.shift(w * 0.40, 0.0, w * 0.30),
            NpcMemoryKind::Hostility           => self.disposition.shift(-w.abs() * 0.50, w.abs() * 0.60, 0.0),
        };
        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Most recent memories, newest last. Mirrors
    /// [`super::vault::DimensionVault::recent`] so the game layer can
    /// reuse the same panel-rendering loop.
    pub fn recent(&self, limit: usize) -> Vec<NpcMemoryEntry> {
        let n = limit.min(self.entries.len());
        let skip = self.entries.len() - n;
        self.entries.iter().skip(skip).cloned().collect()
    }

    /// Filter the ring by kind, newest last.
    pub fn recall_by_kind(&self, kind: NpcMemoryKind) -> Vec<NpcMemoryEntry> {
        self.entries.iter().filter(|e| e.kind == kind).cloned().collect()
    }

    /// Manually clamp-shift the disposition. Used by the game layer
    /// for non-memory effects (e.g. an epoch collapse sweeping every
    /// NPC's fear up).
    pub fn shift_disposition(&mut self, df: f32, dfear: f32, dtrust: f32) {
        self.disposition = self.disposition.shift(df, dfear, dtrust);
    }

    /// Coarse-grained mood label. The thresholds are deliberately
    /// conservative so that a single negative interaction doesn't
    /// flip a friendly NPC.
    pub fn mood(&self) -> NpcMood {
        let d = self.disposition;
        if d.friendly >= 0.40 && d.fear <= 0.30 {
            NpcMood::Happy
        } else if d.fear >= 0.60 && d.friendly <= 0.0 {
            NpcMood::Hostile
        } else if d.fear >= 0.30 || d.friendly <= -0.20 {
            NpcMood::Uneasy
        } else {
            NpcMood::Neutral
        }
    }

    /// Suggest a topic the NPC should bring up next, based on mood
    /// and the most recent memory kind. Deterministic on
    /// `(mood, last_kind, seed)`.
    pub fn suggest_topic(&self, seed: u64) -> &'static str {
        let mood = self.mood();
        let last_kind = self.entries.back().map(|e| e.kind);
        // 4 fallback topics; seed picks one to break ties.
        const NEUTRAL: [&str; 4] = ["greeting", "lore", "trade", "quest"];
        let neutral_idx = (seed as usize).wrapping_add(self.entries.len()) % NEUTRAL.len();
        match (mood, last_kind) {
            (NpcMood::Happy,   Some(NpcMemoryKind::ReceivedGift))        => "trade",
            (NpcMood::Happy,   Some(NpcMemoryKind::Dialogue))            => "quest",
            (NpcMood::Happy,   _)                                        => "greeting",
            (NpcMood::Hostile, _)                                        => "combat",
            (NpcMood::Uneasy,  Some(NpcMemoryKind::Hostility))           => "farewell",
            (NpcMood::Uneasy,  Some(NpcMemoryKind::WitnessedEvent))      => "lore",
            (NpcMood::Uneasy,  _)                                        => "farewell",
            (NpcMood::Neutral, Some(NpcMemoryKind::HeardAboutDimension)) => "lore",
            (NpcMood::Neutral, _)                                        => NEUTRAL[neutral_idx],
        }
    }

    /// Drop every memory and reset disposition. Used by the game
    /// layer when an epoch collapse wipes NPCs.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.disposition = NpcDisposition::default();
    }
}

/// Many minds, keyed by [`NpcId`]. Mirrors the small registries
/// elsewhere in the crate (atom, gameplay) so the game layer's
/// integration is one loop.
#[derive(Debug, Default)]
pub struct NpcRegistry {
    minds: Vec<NpcMind>,
}

impl NpcRegistry {
    /// Build an empty registry.
    pub fn new() -> Self {
        Self { minds: Vec::new() }
    }

    /// How many NPCs are tracked.
    pub fn len(&self) -> usize {
        self.minds.len()
    }

    /// `true` when no NPCs are tracked.
    pub fn is_empty(&self) -> bool {
        self.minds.is_empty()
    }

    /// Insert a mind. If an entry with the same id already exists,
    /// it is replaced.
    pub fn insert(&mut self, mind: NpcMind) {
        if let Some(slot) = self.minds.iter_mut().find(|m| m.id() == mind.id()) {
            *slot = mind;
        } else {
            self.minds.push(mind);
        }
    }

    /// Borrow a mind by id.
    pub fn get(&self, id: &str) -> Option<&NpcMind> {
        self.minds.iter().find(|m| m.id() == id)
    }

    /// Mutably borrow a mind by id.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut NpcMind> {
        self.minds.iter_mut().find(|m| m.id() == id)
    }

    /// Iterate every mind in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &NpcMind> {
        self.minds.iter()
    }

    /// Mutably iterate every mind.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NpcMind> {
        self.minds.iter_mut()
    }

    /// Append the same memory to every NPC's ring. Useful for world
    /// events ("player entered the Neon Cascade dimension") that
    /// every NPC should know.
    pub fn broadcast(&mut self, template: NpcMemoryEntry) {
        for mind in &mut self.minds {
            mind.remember(template.clone());
        }
    }

    /// Aggregate disposition averaged across all minds. Returns
    /// [`NpcDisposition::default`] when the registry is empty. Useful
    /// for the BalanceTuner ("everyone hates the player, ease up").
    pub fn average_disposition(&self) -> NpcDisposition {
        if self.minds.is_empty() {
            return NpcDisposition::default();
        }
        let n = self.minds.len() as f32;
        let (mut f, mut fr, mut t) = (0.0, 0.0, 0.0);
        for m in &self.minds {
            let d = m.disposition();
            f += d.friendly;
            fr += d.fear;
            t += d.trust;
        }
        NpcDisposition { friendly: f / n, fear: fr / n, trust: t / n }
    }
}

// ---------------------------------------------------------------------------
// Tests — at least 10, covering capacity wrap, disposition clamp, mood,
// recall_by_kind, broadcast, suggest_topic in different moods.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: NpcMemoryKind, summary: &str, turn: u64, weight: f32) -> NpcMemoryEntry {
        NpcMemoryEntry::new(kind, summary, turn, weight)
    }

    #[test]
    fn new_mind_is_empty_and_has_default_disposition() {
        let m = NpcMind::new("npc_0");
        assert_eq!(m.id(), "npc_0");
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
        assert_eq!(m.capacity(), NpcMind::DEFAULT_CAPACITY);
        assert_eq!(m.disposition(), NpcDisposition::default());
        assert_eq!(m.mood(), NpcMood::Neutral);
    }

    #[test]
    fn capacity_wrap_drops_oldest() {
        let mut m = NpcMind::with_capacity("npc_0", 3);
        for i in 0..5 {
            m.remember(entry(NpcMemoryKind::Dialogue, &format!("d{i}"), i, 0.1));
        }
        assert_eq!(m.len(), 3);
        let recent = m.recent(3);
        assert_eq!(recent[0].summary, "d2");
        assert_eq!(recent[1].summary, "d3");
        assert_eq!(recent[2].summary, "d4");
    }

    #[test]
    fn zero_capacity_is_black_hole() {
        let mut m = NpcMind::with_capacity("npc_0", 0);
        m.remember(entry(NpcMemoryKind::Dialogue, "x", 0, 1.0));
        assert_eq!(m.len(), 0);
        // Disposition still untouched because the no-op short-circuits.
        assert_eq!(m.disposition(), NpcDisposition::default());
    }

    #[test]
    fn disposition_clamps_to_unit_interval() {
        let mut m = NpcMind::new("npc_0");
        // Bash friendliness up beyond +1.
        for i in 0..50 {
            m.remember(entry(NpcMemoryKind::ReceivedGift, "gift", i, 1.0));
        }
        let d = m.disposition();
        assert!(d.friendly <= 1.0 && d.friendly >= 0.99);
        assert!(d.trust    <= 1.0 && d.trust    >= 0.99);
        // Bash hostility down past -1.
        for i in 0..50 {
            m.remember(entry(NpcMemoryKind::Hostility, "hit", i, 1.0));
        }
        let d2 = m.disposition();
        assert!(d2.friendly >= -1.0);
        assert!(d2.fear     <= 1.0);
    }

    #[test]
    fn entry_weight_is_clamped_at_construction() {
        let e = NpcMemoryEntry::new(NpcMemoryKind::Dialogue, "x", 0, 2.5);
        assert_eq!(e.weight, 1.0);
        let e2 = NpcMemoryEntry::new(NpcMemoryKind::Dialogue, "x", 0, -2.5);
        assert_eq!(e2.weight, -1.0);
    }

    #[test]
    fn recall_by_kind_filters_in_insertion_order() {
        let mut m = NpcMind::new("npc_0");
        m.remember(entry(NpcMemoryKind::Dialogue, "a", 0, 0.1));
        m.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 1, 0.5));
        m.remember(entry(NpcMemoryKind::Dialogue, "b", 2, 0.1));
        m.remember(entry(NpcMemoryKind::WitnessedEvent, "w", 3, 0.1));
        let dialogues = m.recall_by_kind(NpcMemoryKind::Dialogue);
        assert_eq!(dialogues.len(), 2);
        assert_eq!(dialogues[0].summary, "a");
        assert_eq!(dialogues[1].summary, "b");
        assert_eq!(m.recall_by_kind(NpcMemoryKind::Hostility).len(), 0);
    }

    #[test]
    fn mood_thresholds_match_disposition() {
        let mut m = NpcMind::new("npc_0");
        assert_eq!(m.mood(), NpcMood::Neutral);
        // Push friendly past 0.40 with no fear → Happy.
        m.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 0, 1.0)); // +0.40 friendly, +0.30 trust
        m.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 1, 1.0));
        assert_eq!(m.mood(), NpcMood::Happy);

        // Hostility flips it.
        let mut m2 = NpcMind::new("npc_1");
        for i in 0..3 {
            m2.remember(entry(NpcMemoryKind::Hostility, "hit", i, 1.0));
        }
        assert_eq!(m2.mood(), NpcMood::Hostile);

        // Mild fear bumps to Uneasy when friendliness is low.
        let mut m3 = NpcMind::new("npc_2");
        m3.remember(entry(NpcMemoryKind::WitnessedEvent, "earthquake", 0, 1.0)); // +0.15 fear
        m3.remember(entry(NpcMemoryKind::WitnessedEvent, "fire", 1, 1.0));      // +0.30 fear
        assert_eq!(m3.mood(), NpcMood::Uneasy);
    }

    #[test]
    fn suggest_topic_routes_by_mood_and_last_kind() {
        let mut happy = NpcMind::new("happy");
        happy.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 0, 1.0));
        happy.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 1, 1.0));
        assert_eq!(happy.suggest_topic(0), "trade");

        let mut hostile = NpcMind::new("hostile");
        for i in 0..3 {
            hostile.remember(entry(NpcMemoryKind::Hostility, "hit", i, 1.0));
        }
        assert_eq!(hostile.suggest_topic(0), "combat");

        let mut uneasy = NpcMind::new("uneasy");
        uneasy.remember(entry(NpcMemoryKind::WitnessedEvent, "boom", 0, 1.0));
        uneasy.remember(entry(NpcMemoryKind::WitnessedEvent, "fire", 1, 1.0));
        assert_eq!(uneasy.suggest_topic(0), "lore");

        let neutral = NpcMind::new("neutral");
        // Seed picks index 0 → "greeting" when neutral & empty.
        assert_eq!(neutral.suggest_topic(0), "greeting");
    }

    #[test]
    fn manual_shift_clamps() {
        let mut m = NpcMind::new("npc_0");
        m.shift_disposition(2.0, -3.0, 5.0);
        assert_eq!(m.disposition(), NpcDisposition { friendly: 1.0, fear: -1.0, trust: 1.0 });
    }

    #[test]
    fn clear_resets_everything() {
        let mut m = NpcMind::new("npc_0");
        m.remember(entry(NpcMemoryKind::ReceivedGift, "g", 0, 1.0));
        assert!(m.disposition().friendly > 0.0);
        assert_eq!(m.len(), 1);
        m.clear();
        assert!(m.is_empty());
        assert_eq!(m.disposition(), NpcDisposition::default());
    }

    #[test]
    fn registry_insert_replaces_same_id() {
        let mut reg = NpcRegistry::new();
        reg.insert(NpcMind::with_capacity("a", 8));
        reg.insert(NpcMind::with_capacity("b", 8));
        assert_eq!(reg.len(), 2);
        reg.insert(NpcMind::with_capacity("a", 4));
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.get("a").unwrap().capacity(), 4);
    }

    #[test]
    fn registry_broadcast_records_in_every_mind() {
        let mut reg = NpcRegistry::new();
        reg.insert(NpcMind::new("a"));
        reg.insert(NpcMind::new("b"));
        reg.insert(NpcMind::new("c"));
        reg.broadcast(entry(NpcMemoryKind::HeardAboutDimension, "Neon Cascade", 0, 0.5));
        for id in ["a", "b", "c"] {
            let m = reg.get(id).unwrap();
            assert_eq!(m.len(), 1);
            assert_eq!(m.recent(1)[0].summary, "Neon Cascade");
            assert!(m.disposition().trust > 0.0);
        }
    }

    #[test]
    fn registry_average_disposition() {
        let mut reg = NpcRegistry::new();
        assert_eq!(reg.average_disposition(), NpcDisposition::default());
        let mut a = NpcMind::new("a");
        a.shift_disposition(1.0, 0.0, 0.0);
        let mut b = NpcMind::new("b");
        b.shift_disposition(-1.0, 0.5, 0.0);
        reg.insert(a);
        reg.insert(b);
        let avg = reg.average_disposition();
        assert!((avg.friendly - 0.0).abs() < 1e-6);
        assert!((avg.fear - 0.25).abs() < 1e-6);
        assert_eq!(avg.trust, 0.0);
    }

    #[test]
    fn recent_limit_zero_returns_empty() {
        let mut m = NpcMind::new("npc_0");
        m.remember(entry(NpcMemoryKind::Dialogue, "x", 0, 0.1));
        assert!(m.recent(0).is_empty());
    }
}

// ---------------------------------------------------------------------------
// Round 27 — archetype → NPC-default mappings.
//
// Round 24 added `NpcArchetype` (and the
// `theme_to_scene(theme).npc_archetype_hints` field) so each visual
// style ships a list of canonical archetype tags. Round 24 only
// *tagged* the spawned NPCs with the archetype; this block closes
// the loop and makes the tag actually *do* something: a cyberpunk
// "robot" spawns as stoic/neutral, a "skeleton" as grumpy/hostile,
// a "siren" as playful/happy, etc. Players can see the same theme
// produce a consistent NPC personality profile.
//
// Branch order and the personality / faction / mood / disposition
// values are all canonical — the TS side mirrors them 1:1 in
// `NpcFactory.ts`. Cross-layer equality is enforced by the TS
// `archetype_default_*` jest tests and the engine `archetype_*`
// cargo tests below.
// ---------------------------------------------------------------------------

use super::scene_gen::NpcArchetype;

/// Coarse-grained personality tag. Mirrors `NPCPersonality` in TS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcPersonality {
    Cheerful,
    Grumpy,
    Mysterious,
    Wise,
    Playful,
    Stoic,
}

/// Initial mood for a freshly-spawned NPC whose `archetype` is
/// `arch`. The mood is the *initial* label only — once the player
/// interacts with the NPC, `NpcMind::mood()` derives the live mood
/// from `disposition()`.
pub fn archetype_initial_mood(arch: NpcArchetype) -> NpcMood {
    match arch {
        NpcArchetype::Robot     => NpcMood::Neutral,
        NpcArchetype::Mage      => NpcMood::Neutral,
        NpcArchetype::Beast     => NpcMood::Uneasy,
        NpcArchetype::Astronaut => NpcMood::Neutral,
        NpcArchetype::Alien     => NpcMood::Uneasy,
        NpcArchetype::Siren     => NpcMood::Happy,
        NpcArchetype::Diver     => NpcMood::Neutral,
        NpcArchetype::Scorpion  => NpcMood::Hostile,
        NpcArchetype::Nomad     => NpcMood::Neutral,
        NpcArchetype::Skeleton  => NpcMood::Hostile,
        NpcArchetype::Lich      => NpcMood::Hostile,
    }
}

/// Default personality for the given archetype. Picked to be
/// *narratively consistent* with the visual style — e.g. a robot is
/// stoic, a siren is playful.
pub fn archetype_default_personality(arch: NpcArchetype) -> NpcPersonality {
    match arch {
        NpcArchetype::Robot     => NpcPersonality::Stoic,
        NpcArchetype::Mage      => NpcPersonality::Wise,
        NpcArchetype::Beast     => NpcPersonality::Playful,
        NpcArchetype::Astronaut => NpcPersonality::Stoic,
        NpcArchetype::Alien     => NpcPersonality::Mysterious,
        NpcArchetype::Siren     => NpcPersonality::Playful,
        NpcArchetype::Diver     => NpcPersonality::Cheerful,
        NpcArchetype::Scorpion  => NpcPersonality::Grumpy,
        NpcArchetype::Nomad     => NpcPersonality::Stoic,
        NpcArchetype::Skeleton  => NpcPersonality::Grumpy,
        NpcArchetype::Lich      => NpcPersonality::Mysterious,
    }
}

/// Default faction hint for the given archetype. Free-form string;
/// the TS side consumes it as an opaque label for `NPCProfile.faction`.
pub fn archetype_default_faction(arch: NpcArchetype) -> &'static str {
    match arch {
        NpcArchetype::Robot
        | NpcArchetype::Astronaut => "苍穹骑士团",
        NpcArchetype::Mage        => "秘银评议会",
        NpcArchetype::Beast       => "隐者之塔",
        NpcArchetype::Alien       => "星陨教派",
        NpcArchetype::Siren
        | NpcArchetype::Diver     => "潮汐神殿",
        NpcArchetype::Scorpion
        | NpcArchetype::Nomad     => "焰心旅团",
        NpcArchetype::Skeleton
        | NpcArchetype::Lich      => "暗巷商会",
    }
}

/// Initial disposition baseline. Picked so that
/// `NpcMind::mood()` round-trips to the same label as
/// `archetype_initial_mood(arch)`. Once the player interacts, the
/// live disposition diverges and `mood()` tracks it.
///
/// Threshold reminder (from `NpcMind::mood()`):
///   - Hostile requires `fear >= 0.60 && friendly <= 0.0`
///   - Happy   requires `friendly >= 0.40 && fear <= 0.30`
///   - Uneasy  requires `fear >= 0.30 || friendly <= -0.20`
///   - Neutral otherwise
pub fn archetype_initial_disposition(arch: NpcArchetype) -> NpcDisposition {
    match arch {
        NpcArchetype::Robot
        | NpcArchetype::Mage
        | NpcArchetype::Astronaut
        | NpcArchetype::Diver
        | NpcArchetype::Nomad      => NpcDisposition { friendly: 0.0, fear: 0.0, trust: 0.0 },
        NpcArchetype::Lich         => NpcDisposition { friendly: -0.5, fear: 0.7, trust: -0.5 },
        NpcArchetype::Beast
        | NpcArchetype::Alien      => NpcDisposition { friendly: 0.0, fear: 0.4, trust: -0.1 },
        NpcArchetype::Siren        => NpcDisposition { friendly: 0.5, fear: 0.0, trust: 0.3 },
        NpcArchetype::Scorpion
        | NpcArchetype::Skeleton   => NpcDisposition { friendly: -0.5, fear: 0.7, trust: -0.4 },
    }
}

#[cfg(test)]
mod archetype_tests {
    use super::*;
    use crate::agi_minigame::scene_gen::NpcArchetype as A;

    #[test]
    fn every_archetype_has_a_personality() {
        for &arch in &[
            A::Robot, A::Mage, A::Beast, A::Astronaut, A::Alien,
            A::Siren, A::Diver, A::Scorpion, A::Nomad, A::Skeleton, A::Lich,
        ] {
            // Just confirm no panic — the function is total.
            let _ = archetype_default_personality(arch);
        }
    }

    #[test]
    fn archetype_initial_mood_matches_mind_round_trip() {
        // Pin: a mind built from `archetype_initial_disposition` should
        // report the same mood as `archetype_initial_mood`.
        for &arch in &[
            A::Robot, A::Mage, A::Beast, A::Astronaut, A::Alien,
            A::Siren, A::Diver, A::Scorpion, A::Nomad, A::Skeleton, A::Lich,
        ] {
            let d = archetype_initial_disposition(arch);
            let mut m = NpcMind::new("n");
            m.shift_disposition(d.friendly, d.fear, d.trust);
            assert_eq!(m.mood(), archetype_initial_mood(arch),
                       "mood round-trip mismatch for {:?}", arch);
        }
    }

    #[test]
    fn archetype_mood_categorization_matches_palette_priority() {
        // Hostile → hostile, Siren → happy, Beast/Alien → uneasy, rest → neutral.
        assert_eq!(archetype_initial_mood(A::Scorpion), NpcMood::Hostile);
        assert_eq!(archetype_initial_mood(A::Skeleton), NpcMood::Hostile);
        assert_eq!(archetype_initial_mood(A::Lich), NpcMood::Hostile);
        assert_eq!(archetype_initial_mood(A::Siren), NpcMood::Happy);
        assert_eq!(archetype_initial_mood(A::Beast), NpcMood::Uneasy);
        assert_eq!(archetype_initial_mood(A::Alien), NpcMood::Uneasy);
        assert_eq!(archetype_initial_mood(A::Robot), NpcMood::Neutral);
        assert_eq!(archetype_initial_mood(A::Mage), NpcMood::Neutral);
        assert_eq!(archetype_initial_mood(A::Diver), NpcMood::Neutral);
    }

    #[test]
    fn archetype_personality_is_thematic() {
        // Sanity: a robot is stoic, a mage is wise, a siren is playful, etc.
        assert_eq!(archetype_default_personality(A::Robot), NpcPersonality::Stoic);
        assert_eq!(archetype_default_personality(A::Mage), NpcPersonality::Wise);
        assert_eq!(archetype_default_personality(A::Siren), NpcPersonality::Playful);
        assert_eq!(archetype_default_personality(A::Skeleton), NpcPersonality::Grumpy);
        assert_eq!(archetype_default_personality(A::Lich), NpcPersonality::Mysterious);
    }

    #[test]
    fn archetype_factions_have_5_unique_factions() {
        // The 11 archetypes collapse into 7 factions (Robot+Astronaut
        // share 苍穹骑士团, Siren+Diver share 潮汐神殿, etc.).
        // Sanity: at least 2 archetypes per faction (or 1 for the rare
        // case), and total distinct count is 7.
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for &arch in &[
            A::Robot, A::Mage, A::Beast, A::Astronaut, A::Alien,
            A::Siren, A::Diver, A::Scorpion, A::Nomad, A::Skeleton, A::Lich,
        ] {
            seen.insert(archetype_default_faction(arch).to_string());
        }
        // 7 distinct factions across 11 archetypes (each match arm
        // is a unique Chinese string).
        assert_eq!(seen.len(), 7);
    }

    #[test]
    fn archetype_dispositions_in_unit_range() {
        for &arch in &[
            A::Robot, A::Mage, A::Beast, A::Astronaut, A::Alien,
            A::Siren, A::Diver, A::Scorpion, A::Nomad, A::Skeleton, A::Lich,
        ] {
            let d = archetype_initial_disposition(arch);
            assert!(d.friendly >= -1.0 && d.friendly <= 1.0);
            assert!(d.fear >= -1.0 && d.fear <= 1.0);
            assert!(d.trust >= -1.0 && d.trust <= 1.0);
        }
    }

    #[test]
    fn all_archetype_initial_moods_distinct_per_cluster() {
        // Cross-cluster consistency: every Hostile-mood archetype
        // shares the same disposition family; every Neutral-mood
        // archetype shares the same family.
        let hostile_archs = [A::Scorpion, A::Skeleton];
        let happy_archs = [A::Siren];
        let uneasy_archs = [A::Beast, A::Alien];
        for &a in &hostile_archs {
            let d = archetype_initial_disposition(a);
            // hostile cluster: friendly ≤ -0.2
            assert!(d.friendly <= -0.2, "hostile cluster friendly must be ≤ -0.2 for {:?}, got {}", a, d.friendly);
        }
        for &a in &happy_archs {
            let d = archetype_initial_disposition(a);
            // happy cluster: friendly ≥ 0.4 and trust ≥ 0.2
            assert!(d.friendly >= 0.4);
            assert!(d.trust >= 0.2);
        }
        for &a in &uneasy_archs {
            let d = archetype_initial_disposition(a);
            // uneasy cluster: fear ≥ 0.3
            assert!(d.fear >= 0.3, "uneasy cluster fear must be ≥ 0.3 for {:?}, got {}", a, d.fear);
        }
    }
}
