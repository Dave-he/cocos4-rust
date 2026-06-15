use std::any::Any;

use crate::base::value::{Value, ValueMap};

use super::super::atom::{Atom, AtomContext, AtomId, AtomPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardRarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub cost: u32,
    pub card_type: CardType,
    pub rarity: CardRarity,
    pub effects: Vec<CardEffect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardType {
    Attack,
    Defense,
    Spell,
    Summon,
}

#[derive(Debug, Clone)]
pub struct CardEffect {
    pub effect_type: EffectType,
    pub value: i32,
    pub target: TargetType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Damage,
    Heal,
    Shield,
    Draw,
    Buff,
    Debuff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetType {
    Self_,
    SingleEnemy,
    AllEnemies,
    SingleAlly,
    AllAllies,
}

impl Card {
    pub fn new(id: &str, name: &str, cost: u32, card_type: CardType, rarity: CardRarity) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            cost,
            card_type,
            rarity,
            effects: Vec::new(),
        }
    }

    pub fn with_effect(mut self, effect_type: EffectType, value: i32, target: TargetType) -> Self {
        self.effects.push(CardEffect { effect_type, value, target });
        self
    }
}

#[derive(Debug, Clone)]
pub struct CardInstance {
    pub card: Card,
    pub instance_id: String,
    pub is_upgraded: bool,
}

impl CardInstance {
    pub fn new(card: Card, instance_id: &str) -> Self {
        Self {
            card,
            instance_id: instance_id.to_string(),
            is_upgraded: false,
        }
    }

    pub fn upgrade(&mut self) {
        self.is_upgraded = true;
        for effect in &mut self.card.effects {
            effect.value = (effect.value as f32 * 1.5) as i32;
        }
    }
}

pub struct CardAtom {
    phase: AtomPhase,
    deck: Vec<CardInstance>,
    hand: Vec<CardInstance>,
    discard: Vec<CardInstance>,
    max_hand_size: usize,
    energy: u32,
    max_energy: u32,
    energy_regen: u32,
    score: u64,
    cards_played: u32,
}

impl CardAtom {
    pub fn new(max_hand_size: usize, max_energy: u32) -> Self {
        Self {
            phase: AtomPhase::Uninitialized,
            deck: Vec::new(),
            hand: Vec::new(),
            discard: Vec::new(),
            max_hand_size,
            energy: max_energy,
            max_energy,
            energy_regen: 3,
            score: 0,
            cards_played: 0,
        }
    }

    pub fn add_card_to_deck(&mut self, card: Card) {
        let id = format!("ci_{}", self.deck.len() + self.hand.len() + self.discard.len());
        self.deck.push(CardInstance::new(card, &id));
    }

    pub fn generate_starter_deck(&mut self) {
        let starters = vec![
            Card::new("strike", "打击", 1, CardType::Attack, CardRarity::Common)
                .with_effect(EffectType::Damage, 6, TargetType::SingleEnemy),
            Card::new("defend", "防御", 1, CardType::Defense, CardRarity::Common)
                .with_effect(EffectType::Shield, 5, TargetType::Self_),
            Card::new("heal", "治疗", 2, CardType::Spell, CardRarity::Common)
                .with_effect(EffectType::Heal, 8, TargetType::Self_),
            Card::new("fireball", "火球", 3, CardType::Spell, CardRarity::Rare)
                .with_effect(EffectType::Damage, 15, TargetType::AllEnemies),
            Card::new("draw", "抽牌", 1, CardType::Spell, CardRarity::Common)
                .with_effect(EffectType::Draw, 2, TargetType::Self_),
        ];

        for card in starters {
            for _ in 0..2 {
                self.add_card_to_deck(card.clone());
            }
        }
    }

    pub fn shuffle_deck(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for i in (1..self.deck.len()).rev() {
            let j = rng.gen_range(0..=i);
            self.deck.swap(i, j);
        }
    }

    pub fn draw_cards(&mut self, count: usize) -> usize {
        let mut drawn = 0;
        for _ in 0..count {
            if self.hand.len() >= self.max_hand_size {
                break;
            }
            if self.deck.is_empty() {
                self.refill_deck();
            }
            if let Some(card) = self.deck.pop() {
                self.hand.push(card);
                drawn += 1;
            }
        }
        drawn
    }

    fn refill_deck(&mut self) {
        self.deck.append(&mut self.discard);
        self.shuffle_deck();
    }

    pub fn play_card(&mut self, hand_index: usize) -> Option<&Card> {
        if hand_index >= self.hand.len() {
            return None;
        }

        let card_instance = &self.hand[hand_index];
        if card_instance.card.cost > self.energy {
            return None;
        }

        self.energy -= card_instance.card.cost;
        self.cards_played += 1;
        self.score += card_instance.card.cost as u64 * 10;

        let played = self.hand.remove(hand_index);
        self.discard.push(played);
        Some(&self.hand.last().map(|ci| &ci.card).unwrap_or(&self.discard.last().unwrap().card))
    }

    pub fn start_turn(&mut self) {
        self.energy = (self.energy + self.energy_regen).min(self.max_energy);
        self.draw_cards(5);
    }

    pub fn end_turn(&mut self) {
        while let Some(card) = self.hand.pop() {
            self.discard.push(card);
        }
    }

    pub fn get_energy(&self) -> u32 { self.energy }
    pub fn get_max_energy(&self) -> u32 { self.max_energy }
    pub fn get_hand_size(&self) -> usize { self.hand.len() }
    pub fn get_deck_size(&self) -> usize { self.deck.len() }
    pub fn get_discard_size(&self) -> usize { self.discard.len() }
    pub fn get_score(&self) -> u64 { self.score }
}

impl Atom for CardAtom {
    fn atom_id(&self) -> AtomId { "card".to_string() }
    fn atom_name(&self) -> &str { "卡牌" }

    fn on_init(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Initialized; }

    fn on_enter(&mut self, _ctx: &mut AtomContext) {
        self.deck.clear();
        self.hand.clear();
        self.discard.clear();
        self.energy = self.max_energy;
        self.score = 0;
        self.cards_played = 0;
        self.generate_starter_deck();
        self.shuffle_deck();
        self.start_turn();
        self.phase = AtomPhase::Running;
    }

    fn on_update(&mut self, _ctx: &mut AtomContext) {}

    fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
    fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
    fn on_exit(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Completed; }
    fn on_destroy(&mut self) { self.phase = AtomPhase::Uninitialized; }

    fn save_state(&self) -> ValueMap {
        let mut map = ValueMap::new();
        map.insert("energy".to_string(), Value::Integer(self.energy as i32));
        map.insert("score".to_string(), Value::Integer(self.score as i32));
        map.insert("cards_played".to_string(), Value::Integer(self.cards_played as i32));
        map.insert("deck_size".to_string(), Value::Integer(self.deck.len() as i32));
        map.insert("hand_size".to_string(), Value::Integer(self.hand.len() as i32));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Integer(n)) = state.get("energy") { self.energy = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("score") { self.score = *n as u64; }
        if let Some(Value::Integer(n)) = state.get("cards_played") { self.cards_played = *n as u32; }
    }

    fn handle_event(&mut self, event: &str, data: &ValueMap, _ctx: &mut AtomContext) {
        match event {
            "play_card" => {
                if let Some(Value::Integer(idx)) = data.get("hand_index") {
                    self.play_card(*idx as usize);
                }
            }
            "end_turn" => {
                self.end_turn();
                self.start_turn();
            }
            "draw" => {
                let count = data.get("count").and_then(|v| if let Value::Integer(n) = v { Some(*n as usize) } else { None }).unwrap_or(1);
                self.draw_cards(count);
            }
            _ => {}
        }
    }

    fn current_phase(&self) -> AtomPhase { self.phase }
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws)
    }

    #[test]
    fn test_card_atom_init() {
        let mut atom = CardAtom::new(10, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        assert!(atom.get_hand_size() > 0);
        assert!(atom.get_deck_size() > 0);
    }

    #[test]
    fn test_draw_cards() {
        let mut atom = CardAtom::new(10, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        let initial_hand = atom.get_hand_size();
        atom.draw_cards(2);
        assert!(atom.get_hand_size() >= initial_hand);
    }

    #[test]
    fn test_play_card() {
        let mut atom = CardAtom::new(10, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        let hand_size = atom.get_hand_size();
        let initial_energy = atom.get_energy();

        if hand_size > 0 {
            let card = &atom.hand[0].card;
            let cost = card.cost;
            if cost <= initial_energy {
                atom.play_card(0);
                assert_eq!(atom.get_energy(), initial_energy - cost);
            }
        }
    }

    #[test]
    fn test_turn_cycle() {
        let mut atom = CardAtom::new(10, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        atom.end_turn();
        assert_eq!(atom.get_hand_size(), 0);

        atom.start_turn();
        assert!(atom.get_hand_size() > 0);
    }

    #[test]
    fn test_starter_deck() {
        let mut atom = CardAtom::new(10, 10);
        atom.generate_starter_deck();
        assert!(atom.deck.len() >= 10);
    }

    #[test]
    fn test_card_upgrade() {
        let card = Card::new("strike", "打击", 1, CardType::Attack, CardRarity::Common)
            .with_effect(EffectType::Damage, 6, TargetType::SingleEnemy);
        let mut ci = CardInstance::new(card, "ci_0");
        assert!(!ci.is_upgraded);
        ci.upgrade();
        assert!(ci.is_upgraded);
        assert!(ci.card.effects[0].value > 6);
    }
}

// ---------------------------------------------------------------------------
// Round 136 — helper-level
// tests for the lower-level
// `CardRarity` /
// `CardType` /
// `EffectType` /
// `TargetType` /
// `Card` / `CardEffect`
// / `CardInstance` /
// `CardAtom` primitives.
// The high-level `Atom`
// lifecycle is already
// exercised by the
// existing `tests` mod;
// this block adds
// focused unit tests
// for the free-standing
// helpers + `CardAtom`
// accessors + the
// upgrade 1.5x formula
// + deck / hand /
// discard mechanics
// so a future refactor
// that breaks a
// primitive is caught
// at the unit level.
//
// Mirrors the
// round-110b / 122
// / 123 / 124 / 125
// / 126 / 127 / 128
// / 129 / 130 / 131
// / 132 / 133 / 134
// / 135 helper-test
// pattern.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round136_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    /// Round 136 — local
    /// mirror of the
    /// `tests` mod
    /// `make_ctx`
    /// helper (the
    /// private
    /// original is
    /// not visible
    /// from a sibling
    /// mod).
    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws)
    }

    /// Round 136 — helper
    /// to spin up an
    /// initialized +
    /// entered `CardAtom`
    /// (post-`on_enter`:
    /// starter deck
    /// generated +
    /// shuffled + first
    /// turn started).
    fn make_running_atom() -> CardAtom {
        let mut atom = CardAtom::new(10, 10);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        atom
    }

    // --- CardRarity / CardType / EffectType / TargetType ---

    /// Round 136 —
    /// `CardRarity`
    /// has 4 variants
    /// (Common / Rare /
    /// Epic / Legendary).
    #[test]
    fn card_rarity_has_4_variants_round_136() {
        let v = [
            CardRarity::Common,
            CardRarity::Rare,
            CardRarity::Epic,
            CardRarity::Legendary,
        ];
        for &x in &v { assert_eq!(x, x); }
        // Distinct
        // variants
        // are
        // not
        // equal.
        assert_ne!(CardRarity::Common, CardRarity::Legendary);
        assert_ne!(CardRarity::Rare,   CardRarity::Epic);
    }

    /// Round 136 —
    /// `CardType` has
    /// 4 variants
    /// (Attack / Defense
    /// / Spell / Summon).
    #[test]
    fn card_type_has_4_variants_round_136() {
        let v = [
            CardType::Attack,
            CardType::Defense,
            CardType::Spell,
            CardType::Summon,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(CardType::Attack, CardType::Defense);
        assert_ne!(CardType::Spell,  CardType::Summon);
    }

    /// Round 136 —
    /// `EffectType`
    /// has 6 variants
    /// (Damage / Heal /
    /// Shield / Draw /
    /// Buff / Debuff).
    #[test]
    fn effect_type_has_6_variants_round_136() {
        let v = [
            EffectType::Damage,
            EffectType::Heal,
            EffectType::Shield,
            EffectType::Draw,
            EffectType::Buff,
            EffectType::Debuff,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(EffectType::Buff,   EffectType::Debuff);
        assert_ne!(EffectType::Damage, EffectType::Draw);
    }

    /// Round 136 —
    /// `TargetType`
    /// has 5 variants
    /// (Self_ /
    /// SingleEnemy /
    /// AllEnemies /
    /// SingleAlly /
    /// AllAllies).
    #[test]
    fn target_type_has_5_variants_round_136() {
        let v = [
            TargetType::Self_,
            TargetType::SingleEnemy,
            TargetType::AllEnemies,
            TargetType::SingleAlly,
            TargetType::AllAllies,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(TargetType::Self_, TargetType::AllEnemies);
        assert_ne!(TargetType::SingleEnemy, TargetType::AllAllies);
    }

    // --- Card / CardEffect / CardInstance ---

    /// Round 136 —
    /// `Card::new`
    /// stores the
    /// constructor
    /// args verbatim
    /// + starts with
    /// an empty
    /// `effects` vec.
    #[test]
    fn card_new_stores_fields_and_empty_effects_round_136() {
        let c = Card::new("strike", "打击", 1, CardType::Attack, CardRarity::Common);
        assert_eq!(c.id, "strike");
        assert_eq!(c.name, "打击");
        assert_eq!(c.cost, 1);
        assert_eq!(c.card_type, CardType::Attack);
        assert_eq!(c.rarity, CardRarity::Common);
        assert!(c.effects.is_empty());
    }

    /// Round 136 —
    /// `Card::with_effect`
    /// appends a
    /// `CardEffect`
    /// to the
    /// `effects` vec
    /// and returns
    /// `self` for
    /// builder
    /// chaining.
    #[test]
    fn card_with_effect_appends_to_effects_round_136() {
        let c = Card::new("strike", "打击", 1, CardType::Attack, CardRarity::Common)
            .with_effect(EffectType::Damage, 6, TargetType::SingleEnemy)
            .with_effect(EffectType::Heal,   2, TargetType::Self_);
        assert_eq!(c.effects.len(), 2);
        assert_eq!(c.effects[0].effect_type, EffectType::Damage);
        assert_eq!(c.effects[0].value, 6);
        assert_eq!(c.effects[1].effect_type, EffectType::Heal);
        assert_eq!(c.effects[1].value, 2);
    }

    /// Round 136 —
    /// `CardInstance::new`
    /// stores the
    /// underlying
    /// card +
    /// instance_id
    /// verbatim +
    /// defaults
    /// `is_upgraded`
    /// to `false`.
    #[test]
    fn card_instance_new_defaults_is_upgraded_false_round_136() {
        let c = Card::new("a", "A", 1, CardType::Attack, CardRarity::Common);
        let ci = CardInstance::new(c, "ci_0");
        assert_eq!(ci.instance_id, "ci_0");
        assert!(!ci.is_upgraded);
    }

    /// Round 136 —
    /// `CardInstance::upgrade`
    /// multiplies
    /// each effect's
    /// value by 1.5
    /// (rounded via
    /// i32 cast).
    #[test]
    fn card_instance_upgrade_multiplies_value_by_1_5_round_136() {
        // 6 × 1.5 = 9.0 → 9.
        let c = Card::new("strike", "打击", 1, CardType::Attack, CardRarity::Common)
            .with_effect(EffectType::Damage, 6, TargetType::SingleEnemy);
        let mut ci = CardInstance::new(c, "ci_0");
        ci.upgrade();
        assert!(ci.is_upgraded);
        assert_eq!(ci.card.effects[0].value, 9);
    }

    /// Round 136 —
    /// `CardInstance::upgrade`
    /// is a one-way
    /// toggle (calling
    /// it twice still
    /// leaves
    /// `is_upgraded`
    /// = `true`, and
    /// applies the 1.5x
    /// multiplier
    /// again — so a
    /// second call
    /// multiplies the
    /// already-upgraded
    /// value by 1.5
    /// again).
    #[test]
    fn card_instance_upgrade_is_one_way_with_recurring_1_5x_round_136() {
        let c = Card::new("a", "A", 1, CardType::Attack, CardRarity::Common)
            .with_effect(EffectType::Damage, 6, TargetType::SingleEnemy);
        let mut ci = CardInstance::new(c, "ci_0");
        // First
        // upgrade:
        // 6 → 9.
        ci.upgrade();
        assert_eq!(ci.card.effects[0].value, 9);
        // Second
        // upgrade:
        // 9 → 13
        // (9 × 1.5 = 13.5
        // → 13).
        ci.upgrade();
        assert_eq!(ci.card.effects[0].value, 13);
    }

    // --- CardAtom accessors ---

    /// Round 136 —
    /// `CardAtom::new`
    /// initializes
    /// `energy` to
    /// `max_energy`
    /// (full) and
    /// `score` /
    /// `cards_played`
    /// to 0.
    #[test]
    fn card_atom_new_initializes_energy_to_max_round_136() {
        let atom = CardAtom::new(7, 5);
        assert_eq!(atom.get_energy(), 5);
        assert_eq!(atom.get_max_energy(), 5);
        assert_eq!(atom.get_hand_size(), 0);
        assert_eq!(atom.get_deck_size(), 0);
        assert_eq!(atom.get_discard_size(), 0);
        assert_eq!(atom.get_score(), 0);
    }

    /// Round 136 —
    /// `add_card_to_deck`
    /// appends a
    /// fresh
    /// `CardInstance`
    /// to the deck
    /// (the deck
    /// grows by 1).
    #[test]
    fn card_atom_add_card_to_deck_appends_round_136() {
        let mut atom = CardAtom::new(10, 10);
        let before = atom.get_deck_size();
        atom.add_card_to_deck(Card::new("a", "A", 1, CardType::Attack, CardRarity::Common));
        assert_eq!(atom.get_deck_size(), before + 1);
    }

    /// Round 136 —
    /// `add_card_to_deck`
    /// generates a
    /// unique
    /// `instance_id`
    /// each time
    /// (e.g.
    /// `ci_0`,
    /// `ci_1`).
    #[test]
    fn card_atom_add_card_to_deck_generates_unique_ids_round_136() {
        let mut atom = CardAtom::new(10, 10);
        atom.add_card_to_deck(Card::new("a", "A", 1, CardType::Attack, CardRarity::Common));
        atom.add_card_to_deck(Card::new("b", "B", 1, CardType::Attack, CardRarity::Common));
        assert_eq!(atom.deck[0].instance_id, "ci_0");
        assert_eq!(atom.deck[1].instance_id, "ci_1");
    }

    /// Round 136 —
    /// `generate_starter_deck`
    /// produces at
    /// least 10
    /// cards (5
    /// starters × 2
    /// copies each).
    #[test]
    fn card_atom_generate_starter_deck_produces_10_round_136() {
        let mut atom = CardAtom::new(10, 10);
        atom.generate_starter_deck();
        assert_eq!(atom.get_deck_size(), 10);
    }

    /// Round 136 —
    /// `draw_cards`
    /// is capped at
    /// `max_hand_size`
    /// (asking for
    /// 100 cards
    /// only fills
    /// the hand up
    /// to the max).
    #[test]
    fn card_atom_draw_cards_capped_at_max_hand_size_round_136() {
        let mut atom = CardAtom::new(3, 10);
        // Seed
        // the
        // deck
        // so
        // we
        // can
        // hit
        // the
        // cap.
        for _ in 0..20 {
            atom.add_card_to_deck(Card::new("a", "A", 1, CardType::Attack, CardRarity::Common));
        }
        atom.draw_cards(100);
        // Hand
        // is
        // capped
        // at
        // 3
        // (max_hand_size).
        assert_eq!(atom.get_hand_size(), 3);
    }

    /// Round 136 —
    /// `draw_cards`
    /// returns the
    /// count of
    /// cards actually
    /// drawn (≤
    /// the requested
    /// count, capped
    /// at
    /// `max_hand_size`).
    #[test]
    fn card_atom_draw_cards_returns_actual_count_round_136() {
        let mut atom = CardAtom::new(3, 10);
        for _ in 0..20 {
            atom.add_card_to_deck(Card::new("a", "A", 1, CardType::Attack, CardRarity::Common));
        }
        let drawn = atom.draw_cards(100);
        // 3 cards drawn (capped).
        assert_eq!(drawn, 3);
    }

    /// Round 136 —
    /// `play_card` with
    /// an out-of-bounds
    /// `hand_index`
    /// returns `None`
    /// (no state
    /// change).
    #[test]
    fn card_atom_play_card_out_of_bounds_returns_none_round_136() {
        let mut atom = make_running_atom();
        let energy_before = atom.get_energy();
        let result = atom.play_card(999);
        assert!(result.is_none());
        assert_eq!(atom.get_energy(), energy_before);
    }

    /// Round 136 —
    /// `play_card` on
    /// a card with
    /// cost > current
    /// energy returns
    /// `None` (no
    /// state change,
    /// no energy
    /// spent).
    #[test]
    fn card_atom_play_card_too_expensive_returns_none_round_136() {
        let mut atom = CardAtom::new(10, 1);
        // Add
        // an
        // expensive
        // card
        // to
        // the
        // deck.
        atom.add_card_to_deck(Card::new("costly", "expensive", 5, CardType::Spell, CardRarity::Epic));
        // Hand-fill
        // via
        // draw.
        atom.draw_cards(10);
        let energy_before = atom.get_energy();
        // Find
        // the
        // expensive
        // card
        // in
        // the
        // hand
        // (it
        // might
        // be
        // mixed
        // with
        // cheap
        // cards
        // from
        // shuffle).
        // Simpler:
        // set
        // energy
        // to
        // 0
        // and
        // verify
        // no
        // card
        // is
        // playable.
        atom.energy = 0;
        let result = atom.play_card(0);
        assert!(result.is_none());
        assert_eq!(atom.get_energy(), 0);
        // Restore
        // for
        // teardown.
        let _ = energy_before;
    }

    /// Round 136 —
    /// `play_card` on
    /// an affordable
    /// card decrements
    /// energy by the
    /// card's cost.
    #[test]
    fn card_atom_play_card_decrements_energy_by_cost_round_136() {
        let mut atom = CardAtom::new(10, 10);
        atom.add_card_to_deck(Card::new("strike", "打击", 1, CardType::Attack, CardRarity::Common));
        atom.draw_cards(1);
        let energy_before = atom.get_energy();
        let hand_size_before = atom.get_hand_size();
        atom.play_card(0);
        assert_eq!(atom.get_energy(), energy_before - 1);
        // Hand
        // shrank
        // by
        // 1,
        // discard
        // grew
        // by
        // 1.
        assert_eq!(atom.get_hand_size(), hand_size_before - 1);
        assert_eq!(atom.get_discard_size(), 1);
    }

    /// Round 136 —
    /// `play_card`
    /// increments
    /// `score` by
    /// `cost × 10`.
    #[test]
    fn card_atom_play_card_increments_score_by_cost_times_10_round_136() {
        let mut atom = CardAtom::new(10, 10);
        atom.add_card_to_deck(Card::new("strike", "打击", 3, CardType::Attack, CardRarity::Common));
        atom.draw_cards(1);
        atom.play_card(0);
        // 3 × 10 = 30.
        assert_eq!(atom.get_score(), 30);
    }

    /// Round 136 —
    /// `start_turn`
    /// regenerates
    /// energy by
    /// `energy_regen`
    /// (capped at
    /// `max_energy`)
    /// and draws 5
    /// cards.
    #[test]
    fn card_atom_start_turn_regenerates_and_draws_5_round_136() {
        let mut atom = CardAtom::new(10, 10);
        // Seed
        // the
        // deck.
        for _ in 0..20 {
            atom.add_card_to_deck(Card::new("a", "A", 1, CardType::Attack, CardRarity::Common));
        }
        // Drain
        // energy.
        atom.energy = 0;
        atom.start_turn();
        // Energy
        // is
        // (0 +
        // 3).min(10)
        // = 3
        // (the
        // default
        // energy_regen).
        assert_eq!(atom.get_energy(), 3);
        // Hand
        // grew
        // by
        // 5
        // (started
        // empty,
        // drew
        // 5).
        assert_eq!(atom.get_hand_size(), 5);
    }

    /// Round 136 —
    /// `end_turn`
    /// discards every
    /// card in hand
    /// (hand size
    /// becomes 0,
    /// discard grows).
    #[test]
    fn card_atom_end_turn_discards_hand_round_136() {
        let mut atom = make_running_atom();
        // At
        // least
        // 1
        // card
        // in
        // hand
        // after
        // on_enter.
        assert!(atom.get_hand_size() > 0);
        let hand_size = atom.get_hand_size();
        atom.end_turn();
        assert_eq!(atom.get_hand_size(), 0);
        assert_eq!(atom.get_discard_size(), hand_size);
    }

    /// Round 136 —
    /// `shuffle_deck`
    /// is a no-op
    /// when the deck
    /// is empty (no
    /// panic).
    #[test]
    fn card_atom_shuffle_empty_deck_is_no_op_round_136() {
        let mut atom = CardAtom::new(10, 10);
        atom.shuffle_deck(); // No panic.
        assert_eq!(atom.get_deck_size(), 0);
    }

    // --- CardAtom save/load ---

    /// Round 136 —
    /// `save_state`
    /// includes the
    /// documented set
    /// of keys
    /// (energy, score,
    /// cards_played,
    /// deck_size,
    /// hand_size).
    #[test]
    fn card_atom_save_state_has_5_documented_keys_round_136() {
        let atom = make_running_atom();
        let state = atom.save_state();
        for k in &["energy", "score", "cards_played", "deck_size", "hand_size"] {
            assert!(state.contains_key(*k), "save_state should contain key '{}'", k);
        }
    }

    /// Round 136 —
    /// `load_state`
    /// silently ignores
    /// unknown /
    /// wrong-type
    /// values (defensive:
    /// don't panic on
    /// stale saves).
    #[test]
    fn card_atom_load_state_ignores_unknown_keys_round_136() {
        let mut atom = CardAtom::new(10, 10);
        let mut bogus = ValueMap::new();
        bogus.insert("energy".to_string(),       Value::Integer(7));
        bogus.insert("unknown_field".to_string(), Value::Integer(999));
        atom.load_state(&bogus);
        assert_eq!(atom.get_energy(), 7);
    }

    /// Round 136 —
    /// `load_state` with
    /// an empty map
    /// does not modify
    /// existing state
    /// (idempotent).
    #[test]
    fn card_atom_load_state_empty_map_is_idempotent_round_136() {
        let mut atom = CardAtom::new(10, 10);
        atom.energy = 7;
        atom.score  = 100;
        atom.load_state(&ValueMap::new());
        assert_eq!(atom.get_energy(), 7);
        assert_eq!(atom.get_score(), 100);
    }
}
