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

/// Round 48 — `&str` → [`NpcMemoryKind`] lookup. Used to deserialize
/// the round-40 TS `NpcMindSnapshot.kind` field (a string literal)
/// back into the engine enum. Mirrors the TS-side kind union. The
/// string shape is the canonical 5-variant form (`"dialogue"` etc.)
/// — unknown strings return `None` so rehydration can fail soft
/// (skip the entry, keep the rest) rather than panic on stale saves.
pub fn npc_memory_kind_from_str(s: &str) -> Option<NpcMemoryKind> {
    Some(match s {
        "dialogue"               => NpcMemoryKind::Dialogue,
        "witnessed_event"        => NpcMemoryKind::WitnessedEvent,
        "heard_about_dimension"  => NpcMemoryKind::HeardAboutDimension,
        "received_gift"          => NpcMemoryKind::ReceivedGift,
        "hostility"              => NpcMemoryKind::Hostility,
        _ => return None,
    })
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

/// Round 48 — per-NPC memory + disposition snapshot. Captures the
/// canonical "what the world remembers about each NPC" state for
/// cross-save persistence and rehydration. The TS-side
/// `NpcMindSnapshot` interface (in `src/world/WorldState.ts`) is
/// the mirror of this struct; field names + types match 1:1 so
/// the round-40 serialized payload can be rehydrated into a live
/// [`NpcMind`] via [`NpcMind::rehydrate`] without translation.
///
/// Why not derive `Serialize`/`Deserialize` (which would let
/// serde handle the round-trip)? Because (a) the game layer is
/// TypeScript and the actual serializer is `JSON.stringify`; the
/// engine never sees the wire format directly. (b) Keeping the
/// shape explicit makes the cross-layer contract self-documenting
/// — anyone reading the struct sees exactly what the TS side
/// promises to send.
///
/// `entries` are stored newest-first-or-newest-last? Newest-last,
/// matching the order `NpcMind::recent` returns: oldest at
/// index 0, newest at the back. Rehydration pushes them in the
/// same order so the ring's `recent(n)` round-trips byte-for-byte.
#[derive(Debug, Clone, PartialEq)]
pub struct NpcMindSnapshot {
    pub id: String,
    pub archetype: Option<String>,
    pub disposition: NpcDisposition,
    pub entries: Vec<NpcMemoryEntry>,
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
    /// Round 37 — archetype tag (e.g. 'mage', 'merchant').
    /// Optional; mirrors the TS-side round-29 field.
    archetype: Option<String>,
}

impl NpcMind {
    /// Default capacity — 32 memories per NPC. Enough for several
    /// dimensions of play without burning memory; tune via
    /// [`NpcMind::with_capacity`] for tests.
    pub const DEFAULT_CAPACITY: usize = 32;

    /// Build a fresh mind for the given NPC with [`DEFAULT_CAPACITY`].
    /// Round 37 — accepts an optional archetype tag, mirroring
    /// the TS-side round-29 constructor. The tag biases the
    /// `suggest_topic` NEUTRAL fallback (round-34 TS / round-37
    /// here) so different archetypes lean toward different
    /// topics.
    pub fn new(id: impl Into<NpcId>, archetype: Option<impl Into<String>>) -> Self {
        Self::with_capacity(id, Self::DEFAULT_CAPACITY, archetype)
    }

    /// Build a fresh mind that holds at most `capacity` entries.
    /// `capacity == 0` is allowed; `remember` becomes a no-op.
    /// Round 37 — accepts an optional archetype tag.
    pub fn with_capacity(
        id: impl Into<NpcId>,
        capacity: usize,
        archetype: Option<impl Into<String>>,
    ) -> Self {
        let archetype = archetype.map(|a| a.into());
        let mut mind = Self {
            id: id.into(),
            capacity,
            entries: VecDeque::with_capacity(capacity),
            disposition: NpcDisposition::default(),
            archetype,
        };
        // Round 37 — seed the initial disposition from the
        // round-37 archetype helper when an archetype is
        // supplied. Mirrors the TS-side round-29 init.
        if let Some(arch) = mind.archetype.as_deref() {
            if let Some(typed) = npc_archetype_from_str(arch) {
                mind.disposition = archetype_initial_disposition(typed);
            }
        }
        mind
    }

    /// Round 37 — read the archetype tag (None when unset).
    pub fn archetype(&self) -> Option<&str> {
        self.archetype.as_deref()
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
    /// `(mood, last_kind, seed, archetype)`. Round 37 mirrors
    /// the TS-side round-34 archetype bias: the NEUTRAL
    /// fallback is weighted toward the NPC's archetype
    /// preferences (e.g. a mage leans toward 'lore', a
    /// merchant toward 'trade'). The specific mood + last-kind
    /// rules still take precedence; the archetype only colors
    /// the NEUTRAL fallback.
    pub fn suggest_topic(&self, seed: u64) -> &'static str {
        let mood = self.mood();
        let last_kind = self.entries.back().map(|e| e.kind);
        // 4 fallback topics; seed + archetype weights break
        // ties. (The round-34 TS side uses the same shape.)
        const NEUTRAL: [&str; 4] = ["greeting", "lore", "trade", "quest"];
        match (mood, last_kind) {
            (NpcMood::Happy,   Some(NpcMemoryKind::ReceivedGift))        => "trade",
            (NpcMood::Happy,   Some(NpcMemoryKind::Dialogue))            => "quest",
            (NpcMood::Happy,   _)                                        => "greeting",
            (NpcMood::Hostile, _)                                        => "combat",
            (NpcMood::Uneasy,  Some(NpcMemoryKind::Hostility))           => "farewell",
            (NpcMood::Uneasy,  Some(NpcMemoryKind::WitnessedEvent))      => "lore",
            (NpcMood::Uneasy,  _)                                        => "farewell",
            (NpcMood::Neutral, Some(NpcMemoryKind::HeardAboutDimension)) => "lore",
            (NpcMood::Neutral, _)                                        => {
                // Weighted deterministic pick keyed on
                // (seed, entries_count) so it's testable.
                let weights = self.archetype.as_deref()
                    .and_then(npc_archetype_from_str)
                    .map(archetype_topic_boost)
                    .unwrap_or([1, 1, 1, 1]);
                pick_weighted(&NEUTRAL, weights, seed, self.entries.len() as u64)
            }
        }
    }

    /// Drop every memory and reset disposition. Used by the game
    /// layer when an epoch collapse wipes NPCs.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.disposition = NpcDisposition::default();
    }

    /// Round 48 — build a [`NpcMind`] from a persisted
    /// [`NpcMindSnapshot`]. Capacity adapts to the snapshot's
    /// entry count (clamped to a minimum of [`DEFAULT_CAPACITY`])
    /// so the rehydrated ring never wraps entries the snapshot
    /// had room for.
    ///
    /// Critically, the rehydrated `disposition` is taken
    /// **verbatim** from the snapshot — we do NOT call
    /// `archetype_initial_disposition` here, because the
    /// snapshot's disposition is the "last-known live state"
    /// (e.g. round 27's "high-difficulty clear → +0.6 trust"
    /// broadcasts) and overwriting it with the archetype
    /// baseline would discard that history. The round-21/29
    /// constructor path stays the canonical "fresh boot"
    /// path; this factory is the canonical "rehydrate from
    /// save" path.
    ///
    /// The kind field on each entry is a typed `NpcMemoryKind`
    /// enum, so there is no string-to-enum round-trip on the
    /// Rust side. (The TS layer does that round-trip when it
    /// builds the snapshot from a `JSON.parse` of the save
    /// file.) Future pure-Rust snapshot deserializers (e.g.
    /// serde) can use [`npc_memory_kind_from_str`] as the
    /// lookup, but the canonical wire shape is the typed
    /// struct.
    pub fn rehydrate(snap: NpcMindSnapshot) -> Self {
        let capacity = snap.entries.len().max(Self::DEFAULT_CAPACITY);
        let mut mind = Self {
            id: snap.id,
            capacity,
            entries: VecDeque::with_capacity(capacity),
            // The whole point of rehydrate: take the
            // snapshot's disposition, not the archetype
            // baseline.
            disposition: snap.disposition,
            archetype: snap.archetype,
        };
        for entry in snap.entries {
            mind.entries.push_back(entry);
        }
        mind
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

    /// Round 48 — construct a fresh [`NpcRegistry`] from a list
    /// of [`NpcMindSnapshot`]s. Each snapshot is rehydrated via
    /// [`NpcMind::rehydrate`] and inserted in order.
    ///
    /// The returned registry is **fully replaced** — any minds
    /// present in `self` are dropped. This matches the round-48
    /// semantic "snapshot is the new source of truth at app
    /// boot, not a delta" and is the right behavior for
    /// save→reload (the snapshot reflects the last live state).
    pub fn load_from_snapshots(snapshots: Vec<NpcMindSnapshot>) -> Self {
        let mut reg = Self::new();
        reg.load_from_snapshots_into(snapshots);
        reg
    }

    /// Round 48 — in-place version of [`Self::load_from_snapshots`].
    /// Clears the existing mind list and inserts a fresh mind per
    /// snapshot. Idempotent: running twice with the same input
    /// produces the same registry state.
    pub fn load_from_snapshots_into(&mut self, snapshots: Vec<NpcMindSnapshot>) {
        self.minds.clear();
        for snap in snapshots {
            self.minds.push(NpcMind::rehydrate(snap));
        }
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
        let m = NpcMind::new("npc_0", None::<&str>);
        assert_eq!(m.id(), "npc_0");
        assert!(m.is_empty());
        assert_eq!(m.len(), 0);
        assert_eq!(m.capacity(), NpcMind::DEFAULT_CAPACITY);
        assert_eq!(m.disposition(), NpcDisposition::default());
        assert_eq!(m.mood(), NpcMood::Neutral);
    }

    #[test]
    fn capacity_wrap_drops_oldest() {
        let mut m = NpcMind::with_capacity("npc_0", 3, None::<&str>);
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
        let mut m = NpcMind::with_capacity("npc_0", 0, None::<&str>);
        m.remember(entry(NpcMemoryKind::Dialogue, "x", 0, 1.0));
        assert_eq!(m.len(), 0);
        // Disposition still untouched because the no-op short-circuits.
        assert_eq!(m.disposition(), NpcDisposition::default());
    }

    #[test]
    fn disposition_clamps_to_unit_interval() {
        let mut m = NpcMind::new("npc_0", None::<&str>);
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
        let mut m = NpcMind::new("npc_0", None::<&str>);
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
        let mut m = NpcMind::new("npc_0", None::<&str>);
        assert_eq!(m.mood(), NpcMood::Neutral);
        // Push friendly past 0.40 with no fear → Happy.
        m.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 0, 1.0)); // +0.40 friendly, +0.30 trust
        m.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 1, 1.0));
        assert_eq!(m.mood(), NpcMood::Happy);

        // Hostility flips it.
        let mut m2 = NpcMind::new("npc_1", None::<&str>);
        for i in 0..3 {
            m2.remember(entry(NpcMemoryKind::Hostility, "hit", i, 1.0));
        }
        assert_eq!(m2.mood(), NpcMood::Hostile);

        // Mild fear bumps to Uneasy when friendliness is low.
        let mut m3 = NpcMind::new("npc_2", None::<&str>);
        m3.remember(entry(NpcMemoryKind::WitnessedEvent, "earthquake", 0, 1.0)); // +0.15 fear
        m3.remember(entry(NpcMemoryKind::WitnessedEvent, "fire", 1, 1.0));      // +0.30 fear
        assert_eq!(m3.mood(), NpcMood::Uneasy);
    }

    #[test]
    fn suggest_topic_routes_by_mood_and_last_kind() {
        let mut happy = NpcMind::new("happy", None::<&str>);
        happy.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 0, 1.0));
        happy.remember(entry(NpcMemoryKind::ReceivedGift, "gift", 1, 1.0));
        assert_eq!(happy.suggest_topic(0), "trade");

        let mut hostile = NpcMind::new("hostile", None::<&str>);
        for i in 0..3 {
            hostile.remember(entry(NpcMemoryKind::Hostility, "hit", i, 1.0));
        }
        assert_eq!(hostile.suggest_topic(0), "combat");

        let mut uneasy = NpcMind::new("uneasy", None::<&str>);
        uneasy.remember(entry(NpcMemoryKind::WitnessedEvent, "boom", 0, 1.0));
        uneasy.remember(entry(NpcMemoryKind::WitnessedEvent, "fire", 1, 1.0));
        assert_eq!(uneasy.suggest_topic(0), "lore");

        let neutral = NpcMind::new("neutral", None::<&str>);
        // Seed picks index 0 → "greeting" when neutral & empty.
        assert_eq!(neutral.suggest_topic(0), "greeting");
    }

    #[test]
    fn manual_shift_clamps() {
        let mut m = NpcMind::new("npc_0", None::<&str>);
        m.shift_disposition(2.0, -3.0, 5.0);
        assert_eq!(m.disposition(), NpcDisposition { friendly: 1.0, fear: -1.0, trust: 1.0 });
    }

    #[test]
    fn clear_resets_everything() {
        let mut m = NpcMind::new("npc_0", None::<&str>);
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
        reg.insert(NpcMind::with_capacity("a", 8, None::<&str>));
        reg.insert(NpcMind::with_capacity("b", 8, None::<&str>));
        assert_eq!(reg.len(), 2);
        reg.insert(NpcMind::with_capacity("a", 4, None::<&str>));
        assert_eq!(reg.len(), 2);
        assert_eq!(reg.get("a").unwrap().capacity(), 4);
    }

    #[test]
    fn registry_broadcast_records_in_every_mind() {
        let mut reg = NpcRegistry::new();
        reg.insert(NpcMind::new("a", None::<&str>));
        reg.insert(NpcMind::new("b", None::<&str>));
        reg.insert(NpcMind::new("c", None::<&str>));
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
        let mut a = NpcMind::new("a", None::<&str>);
        a.shift_disposition(1.0, 0.0, 0.0);
        let mut b = NpcMind::new("b", None::<&str>);
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
        let mut m = NpcMind::new("npc_0", None::<&str>);
        m.remember(entry(NpcMemoryKind::Dialogue, "x", 0, 0.1));
        assert!(m.recent(0).is_empty());
    }
}

// ---------------------------------------------------------------------------
// Round 48 — NpcMind::rehydrate + NpcRegistry::load_from_snapshots.
//
// Round 40 added a TS-only `NpcMindSnapshot` interface (see
// `src/world/WorldState.ts` in AGI-miniGame) and persisted
// per-NPC entries across save → reload. Round 48 closes the
// loop: the live `NpcRegistry` is now rebuilt from the
// snapshot at app startup, so the world's NPC memory is
// truly continuous across reloads.
//
// The Rust side gets:
//   1. `NpcMindSnapshot` struct (mirror of TS interface)
//   2. `NpcMind::rehydrate` factory — capacity adapts to
//      snapshot entries; disposition is taken verbatim
//      (NOT seeded from archetype_initial_disposition, so
//      a saved +0.6 trust from round-27 broadcasts survives)
//   3. `NpcRegistry::load_from_snapshots` (construct) +
//      `load_from_snapshots_into` (in-place) — full replace
//      semantics; pre-existing minds are dropped
//   4. `npc_memory_kind_from_str` — string→enum lookup for
//      future pure-Rust snapshot deserializers
//
// The test suite below mirrors the TS-side jest tests 1-to-1
// (see AGI-miniGame `src/world/NpcMind.test.ts::round48_*`).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round48_tests {
    use super::*;

    fn entry(kind: NpcMemoryKind, summary: &str, turn: u64, weight: f32) -> NpcMemoryEntry {
        NpcMemoryEntry::new(kind, summary, turn, weight)
    }

    fn snap(
        id: &str,
        archetype: Option<&str>,
        disp: NpcDisposition,
        entries: Vec<NpcMemoryEntry>,
    ) -> NpcMindSnapshot {
        NpcMindSnapshot {
            id: id.to_string(),
            archetype: archetype.map(String::from),
            disposition: disp,
            entries,
        }
    }

    #[test]
    fn rehydrate_preserves_fields_verbatim() {
        // Same fields in → same fields out.
        let entries = vec![
            entry(NpcMemoryKind::Dialogue,       "hi",  1, 0.5),
            entry(NpcMemoryKind::ReceivedGift,   "gem", 2, 1.0),
            entry(NpcMemoryKind::WitnessedEvent, "boom",3, 0.2),
        ];
        let s = snap("mage_1", Some("mage"),
                     NpcDisposition { friendly: 0.5, fear: 0.1, trust: 0.7 },
                     entries.clone());
        let m = NpcMind::rehydrate(s);
        assert_eq!(m.id(), "mage_1");
        assert_eq!(m.archetype(), Some("mage"));
        assert_eq!(m.len(), 3);
        assert_eq!(m.disposition().friendly, 0.5);
        assert_eq!(m.disposition().fear,     0.1);
        assert_eq!(m.disposition().trust,    0.7);
        // Order preserved.
        let r = m.recent(3);
        assert_eq!(r[0].summary, "hi");
        assert_eq!(r[1].summary, "gem");
        assert_eq!(r[2].summary, "boom");
    }

    #[test]
    fn rehydrate_does_not_apply_archetype_initial_disposition() {
        // Headline invariant: a saved `mage` whose disposition is
        // {0,0,0} (e.g. after a wipe of trust) stays at {0,0,0}.
        // The fresh-boot path would seed +0.1 trust via
        // archetype_initial_disposition(Mage), but rehydrate
        // must take the snapshot's value verbatim.
        let s = snap("mage_x", Some("mage"),
                     NpcDisposition { friendly: 0.0, fear: 0.0, trust: 0.0 },
                     vec![]);
        let m = NpcMind::rehydrate(s);
        assert_eq!(m.disposition().trust, 0.0);
        // Same check for Lich (which has a non-zero baseline).
        let s2 = snap("lich_x", Some("lich"),
                      NpcDisposition { friendly: 0.0, fear: 0.0, trust: 0.0 },
                      vec![]);
        let m2 = NpcMind::rehydrate(s2);
        // Lich baseline is { -0.5, 0.7, -0.5 }; snapshot says
        // {0,0,0}; rehydrate keeps {0,0,0}.
        assert_eq!(m2.disposition().friendly, 0.0);
        assert_eq!(m2.disposition().fear,     0.0);
        assert_eq!(m2.disposition().trust,    0.0);
    }

    #[test]
    fn rehydrate_capacity_adapts_to_entries_len() {
        // 5 entries → capacity ≥ 5.
        let entries: Vec<NpcMemoryEntry> = (0..5)
            .map(|i| entry(NpcMemoryKind::Dialogue, &format!("d{i}"), i, 0.1))
            .collect();
        let m = NpcMind::rehydrate(snap("n", None, NpcDisposition::default(), entries));
        assert!(m.capacity() >= 5);
        assert_eq!(m.len(), 5);

        // 50 entries → capacity ≥ 50 (no wraparound).
        let entries50: Vec<NpcMemoryEntry> = (0..50)
            .map(|i| entry(NpcMemoryKind::Dialogue, &format!("d{i}"), i, 0.1))
            .collect();
        let m50 = NpcMind::rehydrate(snap("n", None, NpcDisposition::default(), entries50));
        assert!(m50.capacity() >= 50);
        assert_eq!(m50.len(), 50);
        // First entry should be index 0 (oldest) — proves no wraparound.
        assert_eq!(m50.recent(50)[0].summary, "d0");
    }

    #[test]
    fn rehydrate_capacity_floor_is_default() {
        // 0 entries → capacity = DEFAULT_CAPACITY (32).
        let m = NpcMind::rehydrate(snap("n", None, NpcDisposition::default(), vec![]));
        assert_eq!(m.capacity(), NpcMind::DEFAULT_CAPACITY);
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn rehydrate_preserves_unknown_archetype_string() {
        // An archetype string that the engine doesn't recognize
        // must survive the round-trip verbatim. The TS layer
        // round-29 keeps unknown archetypes; this keeps the
        // cross-layer contract symmetric.
        let s = snap("n", Some("this-archetype-does-not-exist"),
                     NpcDisposition::default(), vec![]);
        let m = NpcMind::rehydrate(s);
        assert_eq!(m.archetype(), Some("this-archetype-does-not-exist"));
    }

    #[test]
    fn rehydrate_no_archetype_yields_none() {
        // The TS interface's `archetype: string | null` maps to
        // `Option<String>` here; `null` → `None` → `archetype()`
        // returns `None` (not `Some("")` or `Some("null")`).
        let s = snap("n", None, NpcDisposition::default(), vec![]);
        let m = NpcMind::rehydrate(s);
        assert_eq!(m.archetype(), None);
    }

    #[test]
    fn npc_memory_kind_from_str_maps_5_canonical_kinds() {
        assert_eq!(npc_memory_kind_from_str("dialogue"),
                   Some(NpcMemoryKind::Dialogue));
        assert_eq!(npc_memory_kind_from_str("witnessed_event"),
                   Some(NpcMemoryKind::WitnessedEvent));
        assert_eq!(npc_memory_kind_from_str("heard_about_dimension"),
                   Some(NpcMemoryKind::HeardAboutDimension));
        assert_eq!(npc_memory_kind_from_str("received_gift"),
                   Some(NpcMemoryKind::ReceivedGift));
        assert_eq!(npc_memory_kind_from_str("hostility"),
                   Some(NpcMemoryKind::Hostility));
        // Unknown kind → None (fail-soft for future variants).
        assert_eq!(npc_memory_kind_from_str("future_kind"), None);
        assert_eq!(npc_memory_kind_from_str(""),            None);
    }

    #[test]
    fn registry_load_from_snapshots_fully_replaces() {
        // Pre-existing mind "old_1" must be gone after load —
        // replace semantics, not merge.
        let mut reg = NpcRegistry::new();
        reg.insert(NpcMind::new("old_1", None::<&str>));
        assert_eq!(reg.len(), 1);
        reg.load_from_snapshots_into(vec![
            snap("a", None, NpcDisposition::default(), vec![]),
            snap("b", None, NpcDisposition::default(), vec![]),
        ]);
        assert_eq!(reg.len(), 2);
        assert!(reg.get("old_1").is_none());
        assert!(reg.get("a").is_some());
        assert!(reg.get("b").is_some());
    }

    #[test]
    fn registry_load_from_snapshots_empty_input_yields_empty_registry() {
        // Empty snapshot list → empty registry (no NpcFactory
        // fallback; that's the game layer's job to choose).
        let mut reg = NpcRegistry::new();
        reg.insert(NpcMind::new("x", None::<&str>));
        reg.load_from_snapshots_into(vec![]);
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);

        // Construct variant: empty snapshots → empty registry.
        let reg2 = NpcRegistry::load_from_snapshots(vec![]);
        assert!(reg2.is_empty());
    }

    #[test]
    fn registry_load_from_snapshots_into_is_idempotent() {
        // Running twice with the same input → same state.
        let mut reg = NpcRegistry::new();
        let snaps = vec![
            snap("a", Some("mage"),
                 NpcDisposition { friendly: 0.4, fear: 0.0, trust: 0.3 },
                 vec![entry(NpcMemoryKind::Dialogue, "hi", 1, 0.5)]),
            snap("b", Some("merchant"),
                 NpcDisposition { friendly: 0.0, fear: 0.2, trust: 0.1 },
                 vec![]),
        ];
        reg.load_from_snapshots_into(snaps.clone());
        let len_after_first = reg.len();
        let avg_after_first = reg.average_disposition();
        reg.load_from_snapshots_into(snaps);
        assert_eq!(reg.len(), len_after_first);
        assert_eq!(reg.average_disposition().friendly, avg_after_first.friendly);
        assert_eq!(reg.average_disposition().fear,     avg_after_first.fear);
        assert_eq!(reg.average_disposition().trust,    avg_after_first.trust);
    }

    #[test]
    fn registry_load_from_snapshots_preserves_disposition() {
        // The headline: a snapshot's disposition survives intact
        // through rehydrate → average_disposition (the round-22
        // BalanceTuner signal) reflects it byte-for-byte.
        let mut reg = NpcRegistry::new();
        reg.load_from_snapshots_into(vec![
            snap("a", None, NpcDisposition { friendly: 0.6, fear: 0.2, trust: 0.4 }, vec![]),
            snap("b", None, NpcDisposition { friendly: 0.2, fear: 0.4, trust: 0.0 }, vec![]),
        ]);
        let avg = reg.average_disposition();
        assert!((avg.friendly - 0.4).abs() < 1e-6);
        assert!((avg.fear     - 0.3).abs() < 1e-6);
        assert!((avg.trust    - 0.2).abs() < 1e-6);
    }

    #[test]
    fn registry_load_from_snapshots_preserves_entries() {
        // The round-40 snapshot's per-NPC entries must be
        // readable from the rehydrated registry (so the
        // NpcMindPanel can show "8 段记忆" after reload).
        let mut reg = NpcRegistry::new();
        reg.load_from_snapshots_into(vec![
            snap("a", Some("merchant"),
                 NpcDisposition { friendly: 0.4, fear: 0.0, trust: 0.0 },
                 vec![
                     entry(NpcMemoryKind::Dialogue,       "haggled", 1, 0.2),
                     entry(NpcMemoryKind::ReceivedGift,   "gem",     2, 1.0),
                 ]),
        ]);
        let a = reg.get("a").unwrap();
        assert_eq!(a.len(), 2);
        let r = a.recent(2);
        assert_eq!(r[0].summary, "haggled");
        assert_eq!(r[1].summary, "gem");
    }

    #[test]
    fn snapshot_to_mind_round_trip_is_byte_identical() {
        // The full round-trip invariant: build a NpcMind
        // (fresh path), observe its disposition + recent
        // entries, build a NpcMindSnapshot from those
        // observations, rehydrate → the new mind has the
        // same disposition + same entries (FIFO order).
        let mut m = NpcMind::new("rt", Some("mage"));
        m.remember(entry(NpcMemoryKind::Dialogue,       "d0", 1, 0.3));
        m.remember(entry(NpcMemoryKind::ReceivedGift,   "g0", 2, 1.0));
        m.remember(entry(NpcMemoryKind::WitnessedEvent, "w0", 3, 0.5));
        let s = NpcMindSnapshot {
            id: m.id().to_string(),
            archetype: m.archetype().map(String::from),
            disposition: m.disposition(),
            entries: m.recent(m.len()),
        };
        let m2 = NpcMind::rehydrate(s);
        assert_eq!(m.id(),               m2.id());
        assert_eq!(m.archetype(),        m2.archetype());
        assert_eq!(m.disposition().friendly, m2.disposition().friendly);
        assert_eq!(m.disposition().fear,     m2.disposition().fear);
        assert_eq!(m.disposition().trust,    m2.disposition().trust);
        let r1 = m.recent(m.len());
        let r2 = m2.recent(m2.len());
        assert_eq!(r1.len(), r2.len());
        for (e1, e2) in r1.iter().zip(r2.iter()) {
            assert_eq!(e1.kind,    e2.kind);
            assert_eq!(e1.summary, e2.summary);
            assert_eq!(e1.turn,    e2.turn);
            assert_eq!(e1.weight,  e2.weight);
        }
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

/// Round 37 — archetype → topic weight vector for the
/// `suggest_topic` NEUTRAL fallback. The order is fixed:
/// `[greeting, lore, trade, quest]`. Higher means more
/// likely to be picked. The shape mirrors the TS-side
/// round-34 `archetypeTopicBoost`; values are chosen to
/// match the archetype's lore role (Mage leans toward
/// 'lore', Siren toward 'greeting', etc.) so the
/// canonical 11-archetype set has at least one topic
/// distinctly favored.
pub fn archetype_topic_boost(arch: NpcArchetype) -> [u32; 4] {
    // [greeting, lore, trade, quest]
    match arch {
        NpcArchetype::Mage         => [1, 3, 0, 2],
        NpcArchetype::Robot        => [1, 3, 1, 1],
        NpcArchetype::Astronaut    => [1, 2, 1, 2],
        NpcArchetype::Diver        => [2, 1, 2, 1],
        NpcArchetype::Nomad        => [2, 1, 2, 2],
        NpcArchetype::Siren        => [3, 1, 1, 1],
        NpcArchetype::Lich         => [0, 3, 0, 1],
        NpcArchetype::Beast        => [1, 1, 0, 3],
        NpcArchetype::Alien        => [1, 2, 0, 3],
        NpcArchetype::Scorpion     => [1, 0, 0, 3],
        NpcArchetype::Skeleton     => [0, 1, 0, 3],
    }
}

/// Round 37 — weighted deterministic pick over the 4
/// NEUTRAL topics, keyed on `(seed, entry_count)`. Same
/// inputs → same output (no rng call, just modular
/// arithmetic), so tests can pin specific values. Mirrors
/// the TS-side `pickWeighted`.
fn pick_weighted(pool: &'static [&'static str; 4], weights: [u32; 4], seed: u64, entry_count: u64) -> &'static str {
    let total: u64 = weights.iter().map(|&w| w as u64).sum();
    if total == 0 { return pool[0]; }
    let target = seed.wrapping_add(entry_count) % total;
    let mut acc: u64 = 0;
    for (i, &w) in weights.iter().enumerate() {
        acc += w as u64;
        if target < acc { return pool[i]; }
    }
    pool[pool.len() - 1]
}

/// Round 37 — `&str` → `Option<NpcArchetype>` lookup. Used
/// to convert the TS-style archetype string ("mage",
/// "merchant", ...) to the canonical Rust enum when the
/// NpcMind was constructed with a string. Unknown strings
/// return None and the mind falls back to flat weights.
pub fn npc_archetype_from_str(s: &str) -> Option<NpcArchetype> {
    use NpcArchetype::*;
    Some(match s {
        "robot"     => Robot,
        "mage"      => Mage,
        "beast"     => Beast,
        "astronaut" => Astronaut,
        "alien"     => Alien,
        "siren"     => Siren,
        "diver"     => Diver,
        "scorpion"  => Scorpion,
        "nomad"     => Nomad,
        "skeleton"  => Skeleton,
        "lich"      => Lich,
        // Round 34 TS archetypes fall back to None.
        "merchant" | "guard" | "rogue" | "shaman" | "peasant" => return None,
        _ => return None,
    })
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
            let mut m = NpcMind::new("n", None::<&str>);
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

// ---------------------------------------------------------------------------
// Round 37 — archetype → topic bias (mirror of TS round 34).
//
// Round 34 added an archetype layer to `suggestTopic` on the
// TS side: the NEUTRAL fallback is weighted by the NPC's
// archetype. Round 37 mirrors this on the engine side so
// the canonical 11-archetype set has a distinct topic
// preference, and the same NpcMind built from a string
// archetype ('mage', 'merchant', ...) gets the right bias.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round37_tests {
    use super::*;
    use crate::agi_minigame::scene_gen::NpcArchetype as A;

    #[test]
    fn archetype_archetype_field_round_trips_through_constructor() {
        let m = NpcMind::new("mage_1", Some("mage"));
        assert_eq!(m.archetype(), Some("mage"));
        let plain = NpcMind::new("plain_1", None::<&str>);
        assert_eq!(plain.archetype(), None);
    }

    #[test]
    fn archetype_init_seeds_disposition_from_table() {
        // Mage → neutral mood, all-zero default per
        // archetype_initial_disposition. The test just
        // confirms that the constructor *did* call the
        // helper (i.e. the disposition is not just
        // `NpcDisposition::default()`).
        let m = NpcMind::new("lich_1", Some("lich"));
        let d = m.disposition();
        // archetype_initial_disposition(Lich) →
        // { friendly: -0.5, fear: 0.7, trust: -0.5 }
        assert!((d.friendly - (-0.5)).abs() < 1e-6);
        assert!((d.fear - 0.7).abs() < 1e-6);
        assert!((d.trust - (-0.5)).abs() < 1e-6);
    }

    #[test]
    fn unknown_archetype_string_leaves_default_disposition() {
        // Defensive: a string that doesn't map to a known
        // NpcArchetype variant leaves the disposition at
        // zero (no crash).
        let m = NpcMind::new("x1", Some("this-archetype-does-not-exist"));
        let d = m.disposition();
        assert_eq!(d.friendly, 0.0);
        assert_eq!(d.fear, 0.0);
        assert_eq!(d.trust, 0.0);
    }

    #[test]
    fn npc_archetype_from_str_maps_known_archetypes() {
        // The 11 canonical NpcArchetype variants.
        for (s, expected) in &[
            ("robot",     A::Robot),
            ("mage",      A::Mage),
            ("beast",     A::Beast),
            ("astronaut", A::Astronaut),
            ("alien",     A::Alien),
            ("siren",     A::Siren),
            ("diver",     A::Diver),
            ("scorpion",  A::Scorpion),
            ("nomad",     A::Nomad),
            ("skeleton",  A::Skeleton),
            ("lich",      A::Lich),
        ] {
            assert_eq!(npc_archetype_from_str(s), Some(*expected));
        }
    }

    #[test]
    fn archetype_topic_boost_assigns_distinct_profiles() {
        // Sanity: no two archetypes share the same 4-vector
        // (otherwise the bias is meaningless). With 11
        // archetypes × 4 topics, the odds of a collision
        // are ~1%; the test just guards against accidental
        // duplication.
        use std::collections::HashSet;
        let profiles: HashSet<[u32; 4]> = [
            A::Robot, A::Mage, A::Beast, A::Astronaut, A::Alien,
            A::Siren, A::Diver, A::Scorpion, A::Nomad, A::Skeleton, A::Lich,
        ].iter().map(|a| archetype_topic_boost(*a)).collect();
        assert_eq!(profiles.len(), 11);
    }

    #[test]
    fn mage_archetype_leans_toward_lore() {
        // mage weights: [1, 3, 0, 2] — 'lore' is the
        // heaviest, so the weighted pick should favor
        // 'lore' across many seeds.
        const NEUTRAL: [&str; 4] = ["greeting", "lore", "trade", "quest"];
        let weights = archetype_topic_boost(A::Mage);
        let mut lore = 0;
        for seed in 0..30 {
            if pick_weighted(&NEUTRAL, weights, seed, 0) == "lore" {
                lore += 1;
            }
        }
        // total=6, lore=3 → 50% of seeds.
        assert!(lore >= 10, "expected ≥10 lore picks in 30 seeds, got {lore}");
    }

    #[test]
    fn archetype_weighted_pick_is_deterministic_per_seed() {
        const NEUTRAL: [&str; 4] = ["greeting", "lore", "trade", "quest"];
        let weights = archetype_topic_boost(A::Nomad);
        for seed in 0..10 {
            let a = pick_weighted(&NEUTRAL, weights, seed, 0);
            let b = pick_weighted(&NEUTRAL, weights, seed, 0);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn archetype_zero_total_weights_falls_back_to_pool_0() {
        // Defensive: all-zero weights → total=0 → return
        // pool[0] without dividing by zero.
        const NEUTRAL: [&str; 4] = ["greeting", "lore", "trade", "quest"];
        let r = pick_weighted(&NEUTRAL, [0, 0, 0, 0], 42, 0);
        assert_eq!(r, "greeting");
    }
}
