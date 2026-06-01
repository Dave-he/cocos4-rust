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
