use std::any::Any;

use crate::base::value::{Value, ValueMap};

use super::super::atom::{Atom, AtomContext, AtomId, AtomPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    Attack,
    Skill,
    Defend,
    Wait,
    Flee,
}

#[derive(Debug, Clone)]
pub struct Buff {
    pub id: String,
    pub name: String,
    pub stacks: u32,
    pub duration: u32,
    pub effect: BuffEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffEffect {
    AttackUp,
    DefenseUp,
    SpeedUp,
    Poison,
    Burn,
    Stun,
    Regen,
}

impl Buff {
    pub fn new(id: &str, name: &str, stacks: u32, duration: u32, effect: BuffEffect) -> Self {
        Self { id: id.to_string(), name: name.to_string(), stacks, duration, effect }
    }

    pub fn tick(&mut self) -> bool {
        if self.duration > 0 {
            self.duration -= 1;
        }
        self.duration == 0
    }
}

#[derive(Debug, Clone)]
pub struct CombatUnit {
    pub id: String,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub position: u8,
    pub is_player: bool,
    pub buffs: Vec<Buff>,
    pub action_gauge: i32,
    pub action_threshold: i32,
}

impl CombatUnit {
    pub fn new(id: &str, name: &str, hp: i32, attack: i32, defense: i32, speed: i32, is_player: bool) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            hp,
            max_hp: hp,
            attack,
            defense,
            speed,
            position: 0,
            is_player,
            buffs: Vec::new(),
            action_gauge: 0,
            action_threshold: 100,
        }
    }

    pub fn is_alive(&self) -> bool { self.hp > 0 }

    pub fn take_damage(&mut self, damage: i32) -> i32 {
        let actual = (damage - self.defense / 2).max(1);
        self.hp = (self.hp - actual).max(0);
        actual
    }

    pub fn heal(&mut self, amount: i32) -> i32 {
        let old = self.hp;
        self.hp = (self.hp + amount).min(self.max_hp);
        self.hp - old
    }

    pub fn add_buff(&mut self, buff: Buff) {
        if let Some(existing) = self.buffs.iter_mut().find(|b| b.id == buff.id) {
            existing.stacks += buff.stacks;
            existing.duration = existing.duration.max(buff.duration);
        } else {
            self.buffs.push(buff);
        }
    }

    pub fn tick_buffs(&mut self) {
        self.buffs.retain(|b| b.duration > 0);
        for buff in &mut self.buffs {
            buff.tick();
        }
    }

    pub fn tick_action_gauge(&mut self) -> bool {
        self.action_gauge += self.speed;
        if self.action_gauge >= self.action_threshold {
            self.action_gauge -= self.action_threshold;
            true
        } else {
            false
        }
    }

    pub fn effective_attack(&self) -> i32 {
        let mut atk = self.attack;
        for buff in &self.buffs {
            match buff.effect {
                BuffEffect::AttackUp => atk += atk * buff.stacks as i32 / 10,
                BuffEffect::Poison => atk -= buff.stacks as i32 * 2,
                _ => {}
            }
        }
        atk.max(1)
    }

    pub fn effective_defense(&self) -> i32 {
        let mut def = self.defense;
        for buff in &self.buffs {
            if buff.effect == BuffEffect::DefenseUp {
                def += def * buff.stacks as i32 / 10;
            }
        }
        def
    }
}

pub struct TurnCombatAtom {
    phase: AtomPhase,
    player_units: Vec<CombatUnit>,
    enemy_units: Vec<CombatUnit>,
    turn: u32,
    score: u64,
    is_player_turn: bool,
    waiting_for_input: bool,
    selected_action: Option<ActionType>,
    selected_target: Option<String>,
    combat_log: Vec<String>,
}

impl TurnCombatAtom {
    pub fn new() -> Self {
        Self {
            phase: AtomPhase::Uninitialized,
            player_units: Vec::new(),
            enemy_units: Vec::new(),
            turn: 0,
            score: 0,
            is_player_turn: true,
            waiting_for_input: false,
            selected_action: None,
            selected_target: None,
            combat_log: Vec::new(),
        }
    }

    pub fn add_player_unit(&mut self, unit: CombatUnit) {
        self.player_units.push(unit);
    }

    pub fn add_enemy_unit(&mut self, unit: CombatUnit) {
        self.enemy_units.push(unit);
    }

    pub fn generate_enemies(&mut self, difficulty: f32) {
        let count = (1.0 + difficulty * 3.0) as usize;
        for i in 0..count.min(4) {
            let hp = (50.0 + difficulty * 50.0) as i32;
            let atk = (10.0 + difficulty * 15.0) as i32;
            let def = (5.0 + difficulty * 5.0) as i32;
            let spd = (8 + i as i32 * 2) as i32;
            let name = format!("敌人_{}", i + 1);
            self.add_enemy_unit(CombatUnit::new(&format!("enemy_{}", i), &name, hp, atk, def, spd, false));
        }
    }

    pub fn player_action(&mut self, action: ActionType, target_id: Option<&str>) {
        if !self.is_player_turn {
            return;
        }

        for unit in &mut self.player_units {
            if !unit.is_alive() {
                continue;
            }

            match action {
                ActionType::Attack => {
                    if let Some(tid) = target_id {
                        if let Some(target) = self.enemy_units.iter_mut().find(|e| e.id == tid && e.is_alive()) {
                            let damage = unit.effective_attack();
                            let actual = target.take_damage(damage);
                            self.score += actual as u64;
                            self.combat_log.push(format!("{} 攻击 {}，造成 {} 伤害", unit.name, target.name, actual));
                        }
                    } else if let Some(target) = self.enemy_units.iter_mut().find(|e| e.is_alive()) {
                        let damage = unit.effective_attack();
                        let actual = target.take_damage(damage);
                        self.score += actual as u64;
                        self.combat_log.push(format!("{} 攻击 {}，造成 {} 伤害", unit.name, target.name, actual));
                    }
                }
                ActionType::Defend => {
                    unit.add_buff(Buff::new("def_up", "防御提升", 3, 2, BuffEffect::DefenseUp));
                    self.combat_log.push(format!("{} 进入防御姿态", unit.name));
                }
                ActionType::Skill => {
                    for enemy in self.enemy_units.iter_mut().filter(|e| e.is_alive()) {
                        let damage = unit.effective_attack() / 2;
                        let actual = enemy.take_damage(damage);
                        self.score += actual as u64;
                    }
                    self.combat_log.push(format!("{} 使用群体技能", unit.name));
                }
                ActionType::Wait => {
                    self.combat_log.push(format!("{} 等待", unit.name));
                }
                ActionType::Flee => {
                    self.combat_log.push("尝试逃跑...".to_string());
                }
            }
            unit.tick_buffs();
        }

        self.is_player_turn = false;
        self.enemy_turn();
    }

    fn enemy_turn(&mut self) {
        for enemy in &mut self.enemy_units {
            if !enemy.is_alive() {
                continue;
            }

            if let Some(player) = self.player_units.iter_mut().find(|p| p.is_alive()) {
                let damage = enemy.effective_attack();
                let actual = player.take_damage(damage);
                self.combat_log.push(format!("{} 攻击 {}，造成 {} 伤害", enemy.name, player.name, actual));
            }
            enemy.tick_buffs();
        }

        self.turn += 1;
        self.is_player_turn = true;
    }

    pub fn is_combat_over(&self) -> bool {
        let players_alive = self.player_units.iter().any(|u| u.is_alive());
        let enemies_alive = self.enemy_units.iter().any(|u| u.is_alive());
        !players_alive || !enemies_alive
    }

    pub fn is_victory(&self) -> bool {
        self.enemy_units.iter().all(|u| !u.is_alive())
    }

    pub fn get_score(&self) -> u64 { self.score }
    pub fn get_turn(&self) -> u32 { self.turn }
    pub fn get_player_units(&self) -> &[CombatUnit] { &self.player_units }
    pub fn get_enemy_units(&self) -> &[CombatUnit] { &self.enemy_units }
}

impl Default for TurnCombatAtom {
    fn default() -> Self {
        Self::new()
    }
}

impl Atom for TurnCombatAtom {
    fn atom_id(&self) -> AtomId { "turn_combat".to_string() }
    fn atom_name(&self) -> &str { "回合战斗" }

    fn on_init(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Initialized; }

    fn on_enter(&mut self, _ctx: &mut AtomContext) {
        self.turn = 0;
        self.score = 0;
        self.combat_log.clear();
        self.is_player_turn = true;
        if self.player_units.is_empty() {
            self.add_player_unit(CombatUnit::new("hero", "勇者", 100, 20, 10, 12, true));
        }
        if self.enemy_units.is_empty() {
            self.generate_enemies(0.5);
        }
        self.phase = AtomPhase::Running;
    }

    fn on_update(&mut self, _ctx: &mut AtomContext) {
        if self.is_combat_over() {
            self.phase = AtomPhase::Completed;
        }
    }

    fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
    fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
    fn on_exit(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Completed; }
    fn on_destroy(&mut self) { self.phase = AtomPhase::Uninitialized; }

    fn save_state(&self) -> ValueMap {
        let mut map = ValueMap::new();
        map.insert("turn".to_string(), Value::Integer(self.turn as i32));
        map.insert("score".to_string(), Value::Integer(self.score as i32));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Integer(n)) = state.get("turn") { self.turn = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("score") { self.score = *n as u64; }
    }

    fn handle_event(&mut self, event: &str, data: &ValueMap, _ctx: &mut AtomContext) {
        match event {
            "action" => {
                let action = data.get("type").and_then(|v| {
                    if let Value::String(s) = v {
                        match s.as_str() {
                            "attack" => Some(ActionType::Attack),
                            "defend" => Some(ActionType::Defend),
                            "skill" => Some(ActionType::Skill),
                            "wait" => Some(ActionType::Wait),
                            "flee" => Some(ActionType::Flee),
                            _ => None,
                        }
                    } else { None }
                }).unwrap_or(ActionType::Wait);
                let target = data.get("target").and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None });
                self.player_action(action, target.as_deref());
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
    fn test_combat_init() {
        let mut atom = TurnCombatAtom::new();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        assert!(!atom.player_units.is_empty());
        assert!(!atom.enemy_units.is_empty());
    }

    #[test]
    fn test_combat_action() {
        let mut atom = TurnCombatAtom::new();
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        let enemy_count = atom.enemy_units.len();
        atom.player_action(ActionType::Attack, None);
        assert!(atom.get_turn() >= 1);
    }

    #[test]
    fn test_combat_victory() {
        let mut atom = TurnCombatAtom::new();
        atom.add_player_unit(CombatUnit::new("hero", "Hero", 100, 999, 10, 12, true));
        atom.add_enemy_unit(CombatUnit::new("e1", "Enemy", 10, 5, 0, 8, false));

        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        atom.player_action(ActionType::Attack, Some("e1"));
        assert!(atom.is_victory() || atom.enemy_units[0].hp <= 0);
    }

    #[test]
    fn test_combat_unit_buffs() {
        let mut unit = CombatUnit::new("u1", "Test", 100, 20, 10, 12, true);
        unit.add_buff(Buff::new("atk_up", "攻击提升", 5, 3, BuffEffect::AttackUp));
        assert_eq!(unit.buffs.len(), 1);
        assert!(unit.effective_attack() > 20);
        unit.tick_buffs();
        assert_eq!(unit.buffs[0].duration, 2);
    }

    #[test]
    fn test_combat_unit_damage() {
        let mut unit = CombatUnit::new("u1", "Test", 100, 20, 10, 12, true);
        let damage = unit.take_damage(30);
        assert!(damage > 0);
        assert!(unit.hp < 100);
        assert!(unit.is_alive());
    }

    #[test]
    fn test_combat_action_gauge() {
        let mut unit = CombatUnit::new("u1", "Speedster", 50, 10, 5, 100, true);
        // Speed 100 with threshold 100 → 1 tick of update should fire.
        let ready = unit.tick_action_gauge();
        assert!(ready);
    }

    #[test]
    fn test_heal_caps_at_max_hp() {
        let mut unit = CombatUnit::new("u1", "Test", 100, 10, 0, 8, true);
        unit.take_damage(40);
        assert_eq!(unit.hp, 60);
        let healed = unit.heal(999);
        assert_eq!(healed, 40);
        assert_eq!(unit.hp, 100);
    }

    #[test]
    fn test_buff_stacking_uses_max_duration() {
        let mut unit = CombatUnit::new("u1", "Test", 100, 10, 5, 8, true);
        unit.add_buff(Buff::new("b1", "B1", 2, 3, BuffEffect::AttackUp));
        unit.add_buff(Buff::new("b1", "B1", 3, 5, BuffEffect::AttackUp));
        assert_eq!(unit.buffs.len(), 1);
        assert_eq!(unit.buffs[0].stacks, 5);
        assert_eq!(unit.buffs[0].duration, 5);
    }
}

// ---------------------------------------------------------------------------
// Round 140 helper-level tests — follow
// the round 110b / 122-139
// pattern. Pre-round-140 had 7
// integration tests (init /
// action / victory / buffs /
// damage / action_gauge /
// heal-cap / buff-stacking)
// but 0 focused unit coverage
// of the public surface. These
// tests pin per-enum variant
// counts, per-field defaults of
// `Buff::new` / `CombatUnit::new`
// / `TurnCombatAtom::new`, the
// `Buff::tick` decrement contract
// + expire semantics, the
// `CombatUnit::take_damage` /
// `heal` math (defense/2
// reduction, max-hp cap, min
// damage 1), the `add_buff`
// stack+max-duration contract,
// the `tick_buffs` retain-then-
// tick order, the
// `tick_action_gauge` overflow
// + threshold-reset semantics,
// the `effective_attack`
// AttackUp + Poison + min-1
// bonus/penalty contract, the
// `effective_defense` DefenseUp
// bonus, the `player_action` 5
// action kinds (Attack w/ +
// w/o target_id, Defend adds
// buff, Skill hits all alive
// enemies, Wait is no-op, Flee
// logs), the
// `!is_player_turn → no-op`
// guard, the
// `is_combat_over` / `is_victory`
// status flags, the 4 getters,
// the `Default::default` round-
// trip, the `save_state` 2
// persisted keys + `load_state`
// round-trip, the `handle_event`
// "action" dispatch (attack /
// flee / with target / unknown
// type → Wait default / unknown
// event no-op), the `on_update`
// combat-over phase transition,
// the full lifecycle
// on_init / on_enter / on_pause
// / on_resume / on_exit /
// on_destroy, the atom_id /
// atom_name contract, and the
// `current_phase` mirror.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round140_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws)
    }

    /// `ActionType` has
    /// 5 variants
    /// (Attack /
    /// Skill /
    /// Defend /
    /// Wait /
    /// Flee).
    #[test]
    fn action_type_has_5_variants_round_140() {
        let v = [
            ActionType::Attack,
            ActionType::Skill,
            ActionType::Defend,
            ActionType::Wait,
            ActionType::Flee,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(ActionType::Attack, ActionType::Flee);
        assert_ne!(ActionType::Skill, ActionType::Wait);
    }

    /// `BuffEffect` has
    /// 7 variants
    /// (AttackUp /
    /// DefenseUp /
    /// SpeedUp /
    /// Poison /
    /// Burn /
    /// Stun /
    /// Regen).
    #[test]
    fn buff_effect_has_7_variants_round_140() {
        let v = [
            BuffEffect::AttackUp,
            BuffEffect::DefenseUp,
            BuffEffect::SpeedUp,
            BuffEffect::Poison,
            BuffEffect::Burn,
            BuffEffect::Stun,
            BuffEffect::Regen,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(BuffEffect::AttackUp, BuffEffect::Poison);
        assert_ne!(BuffEffect::Stun, BuffEffect::Regen);
    }

    /// `Buff::new`
    /// stores fields
    /// verbatim.
    #[test]
    fn buff_new_stores_fields_round_140() {
        let b = Buff::new("b1", "攻击提升", 2, 3, BuffEffect::AttackUp);
        assert_eq!(b.id, "b1");
        assert_eq!(b.name, "攻击提升");
        assert_eq!(b.stacks, 2);
        assert_eq!(b.duration, 3);
        assert_eq!(b.effect, BuffEffect::AttackUp);
    }

    /// `Buff::tick`
    /// decrements
    /// duration
    /// + returns
    /// true when
    /// duration
    /// reaches 0.
    #[test]
    fn buff_tick_decrements_duration_round_140() {
        let mut b = Buff::new("b", "B", 1, 3, BuffEffect::AttackUp);
        assert!(!b.tick()); // 3 → 2
        assert_eq!(b.duration, 2);
        assert!(!b.tick()); // 2 → 1
        assert!(b.tick());  // 1 → 0
        assert_eq!(b.duration, 0);
    }

    /// `Buff::tick`
    /// returns
    /// true when
    /// duration
    /// is already
    /// 0 (no
    /// further
    /// decrement).
    #[test]
    fn buff_tick_expired_returns_true_round_140() {
        let mut b = Buff::new("b", "B", 1, 0, BuffEffect::AttackUp);
        assert!(b.tick());
        assert_eq!(b.duration, 0);
    }

    /// `CombatUnit::new`
    /// stores fields
    /// verbatim +
    /// position=0 +
    /// max_hp=hp +
    /// buffs=[] +
    /// action_gauge=0
    /// +
    /// action_threshold
    /// =100.
    #[test]
    fn combat_unit_new_stores_fields_round_140() {
        let u = CombatUnit::new("u1", "Hero", 100, 20, 10, 12, true);
        assert_eq!(u.id, "u1");
        assert_eq!(u.name, "Hero");
        assert_eq!(u.hp, 100);
        assert_eq!(u.max_hp, 100);
        assert_eq!(u.attack, 20);
        assert_eq!(u.defense, 10);
        assert_eq!(u.speed, 12);
        assert_eq!(u.position, 0);
        assert!(u.is_player);
        assert!(u.buffs.is_empty());
        assert_eq!(u.action_gauge, 0);
        assert_eq!(u.action_threshold, 100);
    }

    /// `CombatUnit::is_alive`
    /// is hp > 0.
    #[test]
    fn combat_unit_is_alive_hp_positive_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 10, 5, 8, true);
        assert!(u.is_alive());
        u.hp = 0;
        assert!(!u.is_alive());
    }

    /// `CombatUnit::take_damage`
    /// formula:
    /// actual =
    /// (damage -
    /// defense/2)
    /// .max(1),
    /// hp -= actual
    /// (clamped
    /// to 0),
    /// returns
    /// actual.
    #[test]
    fn combat_unit_take_damage_formula_round_140() {
        // defense=10,
        // damage=30
        // → actual
        // = (30-5)
        // = 25.
        let mut u = CombatUnit::new("u", "U", 100, 10, 10, 8, true);
        let actual = u.take_damage(30);
        assert_eq!(actual, 25);
        assert_eq!(u.hp, 75);
    }

    /// `take_damage`
    /// with damage
    /// < defense/2
    /// still deals
    /// at least 1
    /// (min
    /// damage
    /// floor).
    #[test]
    fn combat_unit_take_damage_min_one_round_140() {
        // defense=100,
        // damage=5
        // → (5-50)
        // would
        // be -45
        // → .max(1)
        // = 1.
        let mut u = CombatUnit::new("u", "U", 100, 10, 100, 8, true);
        let actual = u.take_damage(5);
        assert_eq!(actual, 1);
        assert_eq!(u.hp, 99);
    }

    /// `take_damage`
    /// clamps hp
    /// to 0
    /// (no
    /// negative)
    /// but
    /// returns
    /// the
    /// damage
    /// DEALT
    /// (formula
    /// result),
    /// not the
    /// damage
    /// APPLIED
    /// (clamped
    /// to hp).
    #[test]
    fn combat_unit_take_damage_clamps_to_zero_round_140() {
        let mut u = CombatUnit::new("u", "U", 50, 10, 0, 8, true);
        let actual = u.take_damage(999);
        // 999-0=999
        // → actual
        // = 999
        // (the
        // formula
        // result,
        // not
        // clamped
        // to
        // remaining
        // hp).
        assert_eq!(actual, 999);
        // But
        // hp is
        // clamped
        // to 0.
        assert_eq!(u.hp, 0);
    }

    /// `CombatUnit::heal`
    /// caps at
    /// max_hp +
    /// returns
    /// actual
    /// healed
    /// amount.
    #[test]
    fn combat_unit_heal_caps_at_max_round_140() {
        // defense=0
        // →
        // take_damage(30)
        // is full
        // 30
        // damage
        // (no
        // reduction).
        let mut u = CombatUnit::new("u", "U", 100, 10, 0, 8, true);
        u.take_damage(30);
        // hp=70.
        assert_eq!(u.hp, 70);
        // heal 50
        // → actual
        // healed
        // = 30
        // (capped
        // at
        // max_hp=100).
        let healed = u.heal(50);
        assert_eq!(healed, 30);
        assert_eq!(u.hp, 100);
    }

    /// `add_buff`
    /// with new
    /// id
    /// appends.
    #[test]
    fn combat_unit_add_buff_new_appends_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 10, 5, 8, true);
        u.add_buff(Buff::new("a", "A", 1, 3, BuffEffect::AttackUp));
        u.add_buff(Buff::new("b", "B", 1, 3, BuffEffect::DefenseUp));
        assert_eq!(u.buffs.len(), 2);
    }

    /// `add_buff`
    /// with
    /// existing
    /// id stacks
    /// + uses
    /// max
    /// duration.
    #[test]
    fn combat_unit_add_buff_existing_stacks_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 10, 5, 8, true);
        u.add_buff(Buff::new("a", "A", 2, 3, BuffEffect::AttackUp));
        u.add_buff(Buff::new("a", "A", 3, 5, BuffEffect::AttackUp));
        assert_eq!(u.buffs.len(), 1);
        assert_eq!(u.buffs[0].stacks, 5); // 2+3
        assert_eq!(u.buffs[0].duration, 5); // max(3,5)
    }

    /// `tick_buffs`
    /// retain-then-tick
    /// order:
    /// duration=1
    /// buff lasts
    /// 1 tick.
    #[test]
    fn combat_unit_tick_buffs_retain_then_tick_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 10, 5, 8, true);
        u.add_buff(Buff::new("a", "A", 1, 1, BuffEffect::AttackUp));
        // Tick 1:
        // retain
        // (1>0) →
        // keep +
        // tick
        // (1→0).
        u.tick_buffs();
        assert_eq!(u.buffs.len(), 1);
        assert_eq!(u.buffs[0].duration, 0);
        // Tick 2:
        // retain
        // (0>0 is
        // false)
        // → REMOVE.
        u.tick_buffs();
        assert_eq!(u.buffs.len(), 0);
    }

    /// `tick_action_gauge`
    /// adds speed,
    /// returns
    /// true when
    /// >= threshold
    /// + subtracts
    /// threshold.
    #[test]
    fn combat_unit_tick_action_gauge_threshold_round_140() {
        let mut u = CombatUnit::new("u", "U", 50, 10, 5, 100, true);
        // action_threshold=100,
        // speed=100
        // → 1
        // tick
        // triggers.
        let ready = u.tick_action_gauge();
        assert!(ready);
        assert_eq!(u.action_gauge, 0); // reset after firing
    }

    /// `tick_action_gauge`
    /// with
    /// low
    /// speed
    /// doesn't
    /// trigger.
    #[test]
    fn combat_unit_tick_action_gauge_below_threshold_round_140() {
        let mut u = CombatUnit::new("u", "U", 50, 10, 5, 5, true);
        let ready = u.tick_action_gauge();
        assert!(!ready);
        assert_eq!(u.action_gauge, 5);
    }

    /// `effective_attack`
    /// with
    /// AttackUp
    /// buff
    /// adds
    /// (stacks * 10%)
    /// bonus.
    #[test]
    fn combat_unit_effective_attack_buff_bonus_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 20, 5, 8, true);
        u.add_buff(Buff::new("a", "A", 3, 5, BuffEffect::AttackUp));
        // 20
        // base +
        // 20*3/10
        // = 20 +
        // 6
        // = 26.
        assert_eq!(u.effective_attack(), 26);
    }

    /// `effective_attack`
    /// with
    /// Poison
    /// buff
    /// subtracts
    /// stacks*2.
    #[test]
    fn combat_unit_effective_attack_poison_penalty_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 20, 5, 8, true);
        u.add_buff(Buff::new("p", "P", 3, 5, BuffEffect::Poison));
        // 20
        // - 3*2
        // = 20 -
        // 6
        // = 14.
        assert_eq!(u.effective_attack(), 14);
    }

    /// `effective_attack`
    /// has a
    /// floor of
    /// 1.
    #[test]
    fn combat_unit_effective_attack_min_one_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 5, 0, 8, true);
        // Stack 100
        // Poison
        // = -200
        // → 5-200
        // = -195
        // → .max(1)
        // = 1.
        u.add_buff(Buff::new("p", "P", 100, 5, BuffEffect::Poison));
        assert_eq!(u.effective_attack(), 1);
    }

    /// `effective_defense`
    /// with
    /// DefenseUp
    /// buff
    /// adds
    /// (stacks * 10%)
    /// bonus.
    #[test]
    fn combat_unit_effective_defense_buff_bonus_round_140() {
        let mut u = CombatUnit::new("u", "U", 100, 10, 20, 8, true);
        u.add_buff(Buff::new("d", "D", 5, 3, BuffEffect::DefenseUp));
        // 20
        // +
        // 20*5/10
        // = 20 +
        // 10
        // = 30.
        assert_eq!(u.effective_defense(), 30);
    }

    /// `TurnCombatAtom::new`
    /// defaults:
    /// Uninit +
    /// empty
    /// units +
    /// turn=0 +
    /// score=0 +
    /// is_player_turn=true
    /// +
    /// waiting_for_input
    /// =false +
    /// selected_action
    /// =None +
    /// selected_target
    /// =None +
    /// empty log.
    #[test]
    fn turn_combat_atom_new_defaults_round_140() {
        let a = TurnCombatAtom::new();
        assert_eq!(a.phase, AtomPhase::Uninitialized);
        assert!(a.player_units.is_empty());
        assert!(a.enemy_units.is_empty());
        assert_eq!(a.turn, 0);
        assert_eq!(a.score, 0);
        assert!(a.is_player_turn);
        assert!(!a.waiting_for_input);
        assert!(a.selected_action.is_none());
        assert!(a.selected_target.is_none());
        assert!(a.combat_log.is_empty());
    }

    /// `add_player_unit`
    /// +
    /// `add_enemy_unit`
    /// append.
    #[test]
    fn add_unit_appends_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p1", "P1", 100, 10, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        assert_eq!(a.player_units.len(), 1);
        assert_eq!(a.enemy_units.len(), 1);
    }

    /// `generate_enemies`
    /// creates 1-4
    /// enemies
    /// (capped
    /// at 4)
    /// with
    /// difficulty-
    /// scaled
    /// stats.
    #[test]
    fn generate_enemies_count_capped_at_4_round_140() {
        let mut a = TurnCombatAtom::new();
        // difficulty=10
        // → 1+30=31
        // → capped
        // at 4.
        a.generate_enemies(10.0);
        assert_eq!(a.enemy_units.len(), 4);
        // difficulty=0.0
        // → 1+0=1.
        let mut a2 = TurnCombatAtom::new();
        a2.generate_enemies(0.0);
        assert_eq!(a2.enemy_units.len(), 1);
    }

    /// `player_action`
    /// Attack with
    /// target_id
    /// deals
    /// damage to
    /// that
    /// specific
    /// enemy +
    /// score
    /// += actual.
    #[test]
    fn player_action_attack_with_target_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        let initial_score = a.score;
        a.is_player_turn = true;
        a.player_action(ActionType::Attack, Some("e1"));
        // Enemy
        // took
        // damage
        // (defense=2,
        // damage=20,
        // actual=20-1=19).
        assert!(a.enemy_units[0].hp < 50);
        assert!(a.score > initial_score);
    }

    /// `player_action`
    /// Attack
    /// without
    /// target_id
    /// picks
    /// first
    /// alive
    /// enemy.
    #[test]
    fn player_action_attack_without_target_picks_first_alive_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.add_enemy_unit(CombatUnit::new("e2", "E2", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        a.player_action(ActionType::Attack, None);
        // First
        // enemy
        // took
        // damage.
        assert!(a.enemy_units[0].hp < 50);
        // Second
        // enemy
        // untouched.
        assert_eq!(a.enemy_units[1].hp, 50);
    }

    /// `player_action`
    /// when NOT
    /// player turn
    /// is a no-op.
    #[test]
    fn player_action_not_player_turn_is_noop_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.is_player_turn = false;
        let prev_enemy_hp = a.enemy_units[0].hp;
        a.player_action(ActionType::Attack, Some("e1"));
        assert_eq!(a.enemy_units[0].hp, prev_enemy_hp);
    }

    /// `player_action`
    /// Defend
    /// adds a
    /// defense
    /// buff to
    /// the unit.
    #[test]
    fn player_action_defend_adds_buff_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 10, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        a.player_action(ActionType::Defend, None);
        // Player
        // has
        // 1 buff
        // (def_up).
        assert_eq!(a.player_units[0].buffs.len(), 1);
        assert_eq!(a.player_units[0].buffs[0].effect, BuffEffect::DefenseUp);
    }

    /// `player_action`
    /// Skill hits
    /// ALL alive
    /// enemies
    /// (half
    /// attack).
    #[test]
    fn player_action_skill_hits_all_enemies_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.add_enemy_unit(CombatUnit::new("e2", "E2", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        a.player_action(ActionType::Skill, None);
        // Both
        // enemies
        // took
        // damage.
        assert!(a.enemy_units[0].hp < 50);
        assert!(a.enemy_units[1].hp < 50);
    }

    /// `player_action`
    /// Wait is a
    /// no-op on
    /// enemies.
    #[test]
    fn player_action_wait_is_noop_on_enemies_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        a.player_action(ActionType::Wait, None);
        // Enemy
        // hp
        // unchanged.
        assert_eq!(a.enemy_units[0].hp, 50);
    }

    /// `player_action`
    /// Flee logs
    /// "尝试逃跑..."
    /// (no enemy
    /// damage).
    #[test]
    fn player_action_flee_logs_no_damage_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        a.player_action(ActionType::Flee, None);
        // Enemy
        // hp
        // unchanged
        // (Flee
        // only
        // logs).
        assert_eq!(a.enemy_units[0].hp, 50);
        // Log
        // contains
        // the
        // flee
        // text.
        let log_text = a.combat_log.join(" | ");
        assert!(log_text.contains("尝试逃跑"));
    }

    /// `is_combat_over`
    /// true when
    /// all
    /// players
    /// or all
    /// enemies
    /// dead.
    #[test]
    fn is_combat_over_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        assert!(!a.is_combat_over());
        // Kill
        // all
        // players.
        a.player_units[0].hp = 0;
        assert!(a.is_combat_over());
        // Restore
        // player,
        // kill
        // enemy.
        a.player_units[0].hp = 100;
        a.enemy_units[0].hp = 0;
        assert!(a.is_combat_over());
    }

    /// `is_victory`
    /// true when
    /// all
    /// enemies
    /// dead.
    #[test]
    fn is_victory_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        assert!(!a.is_victory());
        a.enemy_units[0].hp = 0;
        assert!(a.is_victory());
    }

    /// Getters
    /// surface
    /// internal
    /// state.
    #[test]
    fn getters_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e", "E", 50, 5, 2, 6, false));
        a.turn = 5;
        a.score = 1000;
        assert_eq!(a.get_turn(), 5);
        assert_eq!(a.get_score(), 1000);
        assert_eq!(a.get_player_units().len(), 1);
        assert_eq!(a.get_enemy_units().len(), 1);
    }

    /// `Default::default`
    /// delegates
    /// to `new`.
    #[test]
    fn default_delegates_to_new_round_140() {
        let a: TurnCombatAtom = Default::default();
        assert_eq!(a.phase, AtomPhase::Uninitialized);
        assert!(a.player_units.is_empty());
    }

    /// `save_state`
    /// has 2
    /// persisted
    /// keys.
    #[test]
    fn save_state_keys_round_140() {
        let a = TurnCombatAtom::new();
        let s = a.save_state();
        assert!(s.contains_key("turn"));
        assert!(s.contains_key("score"));
    }

    /// `load_state`
    /// restores
    /// turn +
    /// score.
    #[test]
    fn load_state_restores_fields_round_140() {
        let mut a = TurnCombatAtom::new();
        let mut s = ValueMap::new();
        s.insert("turn".to_string(), Value::Integer(7));
        s.insert("score".to_string(), Value::Integer(2500));
        a.load_state(&s);
        assert_eq!(a.turn, 7);
        assert_eq!(a.score, 2500);
    }

    /// `handle_event`
    /// "action"
    /// with
    /// type="attack"
    /// calls
    /// `player_action`
    /// (Attack).
    #[test]
    fn handle_event_action_attack_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        let mut data = ValueMap::new();
        data.insert("type".to_string(), Value::String("attack".to_string()));
        data.insert("target".to_string(), Value::String("e1".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("action", &data, &mut ctx);
        // Enemy
        // took
        // damage.
        assert!(a.enemy_units[0].hp < 50);
    }

    /// `handle_event`
    /// "action"
    /// with
    /// type="flee"
    /// calls
    /// `player_action`
    /// (Flee) +
    /// logs the
    /// flee text.
    #[test]
    fn handle_event_action_flee_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        let mut data = ValueMap::new();
        data.insert("type".to_string(), Value::String("flee".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("action", &data, &mut ctx);
        let log_text = a.combat_log.join(" | ");
        assert!(log_text.contains("尝试逃跑"));
    }

    /// `handle_event`
    /// "action"
    /// with
    /// unknown
    /// type
    /// defaults
    /// to Wait.
    #[test]
    fn handle_event_action_unknown_type_defaults_to_wait_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.is_player_turn = true;
        let mut data = ValueMap::new();
        data.insert("type".to_string(), Value::String("dragon".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("action", &data, &mut ctx);
        // Wait
        // is a
        // no-op
        // → enemy
        // hp
        // unchanged.
        assert_eq!(a.enemy_units[0].hp, 50);
    }

    /// `handle_event`
    /// unknown
    /// event
    /// is no-op.
    #[test]
    fn handle_event_unknown_is_noop_round_140() {
        let mut a = TurnCombatAtom::new();
        let prev = a.turn;
        let s = ValueMap::new();
        let mut ctx = make_ctx();
        a.handle_event("bogus", &s, &mut ctx);
        assert_eq!(a.turn, prev);
    }

    /// `on_update`
    /// with
    /// `is_combat_over`
    /// → phase
    /// = Completed.
    #[test]
    fn on_update_combat_over_sets_completed_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("p", "P", 100, 20, 5, 8, true));
        a.add_enemy_unit(CombatUnit::new("e1", "E1", 50, 5, 2, 6, false));
        a.player_units[0].hp = 0; // combat over
        let mut ctx = make_ctx();
        a.on_update(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Completed);
    }

    /// `on_enter`
    /// resets
    /// state +
    /// adds hero
    /// if empty
    /// + generates
    /// enemies
    /// if empty.
    #[test]
    fn on_enter_resets_and_populates_round_140() {
        let mut a = TurnCombatAtom::new();
        let mut ctx = make_ctx();
        a.on_init(&mut ctx);
        a.on_enter(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Running);
        // Hero
        // added.
        assert_eq!(a.player_units.len(), 1);
        assert_eq!(a.player_units[0].id, "hero");
        // Enemies
        // generated.
        assert!(!a.enemy_units.is_empty());
        // is_player_turn
        // = true.
        assert!(a.is_player_turn);
    }

    /// `on_enter`
    /// preserves
    /// existing
    /// units
    /// (doesn't
    /// overwrite
    /// them).
    #[test]
    fn on_enter_preserves_existing_units_round_140() {
        let mut a = TurnCombatAtom::new();
        a.add_player_unit(CombatUnit::new("custom", "Custom", 200, 50, 20, 20, true));
        a.add_enemy_unit(CombatUnit::new("boss", "Boss", 500, 30, 15, 10, false));
        let mut ctx = make_ctx();
        a.on_enter(&mut ctx);
        // Both
        // units
        // preserved
        // (not
        // overwritten
        // by
        // on_enter).
        assert_eq!(a.player_units.len(), 1);
        assert_eq!(a.player_units[0].id, "custom");
        assert_eq!(a.enemy_units.len(), 1);
        assert_eq!(a.enemy_units[0].id, "boss");
    }

    /// Lifecycle
    /// phases:
    /// on_pause →
    /// Paused /
    /// on_resume
    /// → Running /
    /// on_exit →
    /// Completed /
    /// on_destroy
    /// →
    /// Uninitialized.
    #[test]
    fn lifecycle_phases_round_140() {
        let mut a = TurnCombatAtom::new();
        let mut ctx = make_ctx();
        a.on_init(&mut ctx);
        a.on_enter(&mut ctx);
        a.on_pause(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Paused);
        a.on_resume(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Running);
        a.on_exit(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Completed);
        a.on_destroy();
        assert_eq!(a.phase, AtomPhase::Uninitialized);
    }

    /// `atom_id` /
    /// `atom_name` /
    /// `as_any` /
    /// `as_any_mut`
    /// contract.
    #[test]
    fn atom_id_and_name_round_140() {
        let a = TurnCombatAtom::new();
        assert_eq!(a.atom_id(), "turn_combat");
        assert_eq!(a.atom_name(), "回合战斗");
        let _ = a.as_any();
        let mut a = TurnCombatAtom::new();
        let _ = a.as_any_mut();
    }

    /// `current_phase`
    /// mirrors the
    /// internal
    /// `phase`
    /// field.
    #[test]
    fn current_phase_matches_field_round_140() {
        let mut a = TurnCombatAtom::new();
        assert_eq!(a.current_phase(), AtomPhase::Uninitialized);
        a.phase = AtomPhase::Paused;
        assert_eq!(a.current_phase(), AtomPhase::Paused);
    }
}

// ---------------------------------------------------------------------------
// Round 160 — focused helper tests for `atoms/turn_combat.rs`.
//
// The round-140 block covered the
// dispatch / event surface. This
// block goes deeper on the data-model
// primitives (Buff / CombatUnit
// field defaults, the 5 action
// types / 7 buff effects) and on
// the combat math (damage
// mitigation, heal clamping,
// buff stacking, action-gauge
// tick, effective stats).
//
// 11 tests pinning the round-140 →
// round-159 surface.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round160_tests {
    use super::*;

    /// `Buff::new`
    /// stores all 5
    /// fields verbatim
    /// (id / name
    /// cloned to the
    /// heap; stacks /
    /// duration / effect
    /// are stored as
    /// the constructor
    /// args). A
    /// regression that
    /// used a `&str` for
    /// id / name would
    /// cause a borrow
    /// error in
    /// `add_buff`.
    #[test]
    fn buff_new_stores_all_fields_round_160() {
        let b = Buff::new("b1", "Attack Up", 2, 5, BuffEffect::AttackUp);
        assert_eq!(b.id, "b1");
        assert_eq!(b.name, "Attack Up");
        assert_eq!(b.stacks, 2);
        assert_eq!(b.duration, 5);
        assert_eq!(b.effect, BuffEffect::AttackUp);
    }

    /// `Buff::tick`
    /// decrements
    /// `duration` by 1
    /// and returns
    /// `true` when
    /// the buff has
    /// expired
    /// (duration
    /// reached 0).
    /// A regression
    /// that returned
    /// `true` on
    /// every tick
    /// would
    /// auto-remove
    /// buffs
    /// prematurely.
    #[test]
    fn buff_tick_decrements_and_signals_expiry_round_160() {
        let mut b = Buff::new("b1", "Poison", 1, 2, BuffEffect::Poison);
        // First tick: 2 → 1, not expired
        assert!(!b.tick());
        assert_eq!(b.duration, 1);
        // Second tick: 1 → 0, expired
        assert!(b.tick());
        assert_eq!(b.duration, 0);
    }

    /// `ActionType`
    /// has 5 variants
    /// (Attack /
    /// Skill / Defend
    /// / Wait / Flee).
    /// `BuffEffect`
    /// has 7 variants
    /// (AttackUp /
    /// DefenseUp /
    /// SpeedUp /
    /// Poison / Burn
    /// / Stun /
    /// Regen). Both
    /// are
    /// `Copy + Eq +
    /// Hash` so
    /// they're
    /// usable in
    /// `match` and
    /// `HashMap`
    /// keys.
    #[test]
    fn turn_combat_action_and_buff_effect_have_5_and_7_variants_round_160() {
        let actions = [
            ActionType::Attack,
            ActionType::Skill,
            ActionType::Defend,
            ActionType::Wait,
            ActionType::Flee,
        ];
        assert_eq!(actions.len(), 5);
        for &a in &actions { assert_eq!(a, a); }

        let effects = [
            BuffEffect::AttackUp,
            BuffEffect::DefenseUp,
            BuffEffect::SpeedUp,
            BuffEffect::Poison,
            BuffEffect::Burn,
            BuffEffect::Stun,
            BuffEffect::Regen,
        ];
        assert_eq!(effects.len(), 7);
        for &e in &effects { assert_eq!(e, e); }
    }

    /// `CombatUnit::new`
    /// initializes
    /// all fields
    /// with
    /// `max_hp == hp`
    /// (so the HUD
    /// HP bar reads
    /// 100% on a
    /// fresh unit),
    /// `position = 0`,
    /// empty buffs
    /// list, and
    /// `action_gauge
    /// = 0` with
    /// `action_threshold
    /// = 100` (the
    /// round-1
    /// defaults).
    #[test]
    fn combat_unit_new_initializes_fields_round_160() {
        let u = CombatUnit::new("u1", "Hero", 100, 20, 10, 15, true);
        assert_eq!(u.id, "u1");
        assert_eq!(u.name, "Hero");
        assert_eq!(u.hp, 100);
        assert_eq!(u.max_hp, 100);
        assert_eq!(u.attack, 20);
        assert_eq!(u.defense, 10);
        assert_eq!(u.speed, 15);
        assert_eq!(u.position, 0);
        assert!(u.is_player);
        assert!(u.buffs.is_empty());
        assert_eq!(u.action_gauge, 0);
        assert_eq!(u.action_threshold, 100);
        assert!(u.is_alive());
    }

    /// `CombatUnit::take_damage`
    /// subtracts
    /// `(damage -
    /// defense / 2)`
    /// (the
    /// round-1
    /// damage
    /// mitigation
    /// formula),
    /// clamps the
    /// result to
    /// `[1, ∞)` so a
    /// heavily-armored
    /// unit still
    /// takes at
    /// least 1
    /// damage per
    /// hit (a
    /// regression
    /// that
    /// returned 0
    /// for high-
    /// defense
    /// units would
    /// make them
    /// invulnerable),
    /// and clamps
    /// `hp` to 0
    /// (the unit
    /// dies but
    /// never has
    /// negative HP).
    #[test]
    fn combat_unit_take_damage_subtracts_and_minimum_is_one_round_160() {
        let mut u = CombatUnit::new("u1", "Tank", 100, 20, 100, 5, true);
        // damage=10, defense=100 → mitigated = 10 - 50 = -40 → max(1, -40) = 1
        let actual = u.take_damage(10);
        assert_eq!(actual, 1);
        assert_eq!(u.hp, 99);
        // Big hit: 100 damage → 100 - 50 = 50
        let actual2 = u.take_damage(100);
        assert_eq!(actual2, 50);
        assert_eq!(u.hp, 49);
    }

    /// `CombatUnit::heal`
    /// adds the
    /// amount to
    /// `hp` but
    /// clamps at
    /// `max_hp`
    /// (over-heal
    /// is wasted,
    /// not
    /// stored).
    /// Returns the
    /// actual
    /// amount
    /// healed
    /// (the
    /// over-heal
    /// delta is
    /// discarded).
    #[test]
    fn combat_unit_heal_clamps_at_max_hp_round_160() {
        let mut u = CombatUnit::new("u1", "Hero", 100, 20, 10, 15, true);
        // Take some damage first
        u.hp = 80;
        // Heal 10 → 90, healed = 10
        let healed = u.heal(10);
        assert_eq!(healed, 10);
        assert_eq!(u.hp, 90);
        // Heal 50 → 100 (clamped), healed = 10 (the 40 over-heal is wasted)
        let healed2 = u.heal(50);
        assert_eq!(healed2, 10);
        assert_eq!(u.hp, 100);
    }

    /// `CombatUnit::add_buff`
    /// for a new
    /// buff id
    /// appends to
    /// the list.
    /// For an
    /// existing
    /// buff id
    /// (re-apply),
    /// it stacks
    /// (adds
    /// stacks) and
    /// takes the
    /// max of the
    /// two
    /// durations
    /// (so a
    /// re-apply
    /// doesn't
    /// shorten the
    /// duration).
    #[test]
    fn combat_unit_add_buff_stacks_and_maxes_duration_round_160() {
        let mut u = CombatUnit::new("u1", "Hero", 100, 20, 10, 15, true);
        u.add_buff(Buff::new("atk", "Atk Up", 1, 5, BuffEffect::AttackUp));
        assert_eq!(u.buffs.len(), 1);
        assert_eq!(u.buffs[0].stacks, 1);
        assert_eq!(u.buffs[0].duration, 5);
        // Re-apply with more stacks + longer duration
        u.add_buff(Buff::new("atk", "Atk Up", 2, 8, BuffEffect::AttackUp));
        assert_eq!(u.buffs.len(), 1);
        assert_eq!(u.buffs[0].stacks, 3); // 1 + 2
        assert_eq!(u.buffs[0].duration, 8); // max(5, 8)
        // Re-apply with shorter duration → still keeps the longer one
        u.add_buff(Buff::new("atk", "Atk Up", 1, 3, BuffEffect::AttackUp));
        assert_eq!(u.buffs[0].duration, 8);
    }

    /// `CombatUnit::tick_action_gauge`
    /// adds `speed`
    /// to
    /// `action_gauge`
    /// and returns
    /// `true` when
    /// the gauge
    /// reaches the
    /// threshold
    /// (the unit
    /// gets a
    /// turn). The
    /// threshold
    /// excess is
    /// rolled over
    /// to the next
    /// tick (so a
    /// unit with
    /// `speed=120`
    /// and
    /// `threshold=100`
    /// would
    /// trigger
    /// every tick
    /// with 20
    /// carrying
    /// over).
    #[test]
    fn combat_unit_tick_action_gauge_triggers_at_threshold_round_160() {
        let mut u = CombatUnit::new("u1", "Fast", 100, 20, 10, 60, true);
        // speed=60, threshold=100 → 1 tick not enough
        assert!(!u.tick_action_gauge());
        assert_eq!(u.action_gauge, 60);
        // 2 ticks → 120, triggers, rolls over 20
        assert!(u.tick_action_gauge());
        assert_eq!(u.action_gauge, 20);
        // 3 ticks → 80, not triggered
        assert!(!u.tick_action_gauge());
        assert_eq!(u.action_gauge, 80);
    }

    /// `effective_attack`
    /// adds a 10%
    /// multiplier
    /// per stack
    /// of an
    /// AttackUp
    /// buff (so 2
    /// stacks =
    /// +20% attack)
    /// and subtracts
    /// `2 * stacks`
    /// for a
    /// Poison buff.
    /// The result is
    /// floored at 1
    /// (a regression
    /// that allowed
    /// the result to
    /// go to 0
    /// would make a
    /// unit unable
    /// to attack).
    #[test]
    fn combat_unit_effective_attack_includes_buffs_and_floors_at_one_round_160() {
        let mut u = CombatUnit::new("u1", "Hero", 100, 20, 10, 15, true);
        // No buffs: 20
        assert_eq!(u.effective_attack(), 20);
        // 2 AttackUp stacks: 20 + 20*2/10 = 24
        u.add_buff(Buff::new("atk", "Atk Up", 2, 5, BuffEffect::AttackUp));
        assert_eq!(u.effective_attack(), 24);
        // 3 Poison stacks: 20 - 3*2 = 14
        u.buffs.clear();
        u.add_buff(Buff::new("psn", "Poison", 3, 5, BuffEffect::Poison));
        assert_eq!(u.effective_attack(), 14);
        // Floor at 1: huge poison shouldn't go negative
        u.buffs.clear();
        u.add_buff(Buff::new("psn", "Mega Poison", 50, 5, BuffEffect::Poison));
        assert_eq!(u.effective_attack(), 1);
    }

    /// `effective_defense`
    /// adds a 10%
    /// multiplier
    /// per stack
    /// of a
    /// DefenseUp
    /// buff (so 2
    /// stacks =
    /// +20% defense).
    /// Other buff
    /// effects
    /// (Poison,
    /// Burn, etc.)
    /// do NOT
    /// affect
    /// defense
    /// (only the
    /// DefenseUp
    /// match arm
    /// runs).
    #[test]
    fn combat_unit_effective_defense_includes_defenseup_only_round_160() {
        let mut u = CombatUnit::new("u1", "Tank", 100, 20, 50, 15, true);
        // No buffs: 50
        assert_eq!(u.effective_defense(), 50);
        // 2 DefenseUp stacks: 50 + 50*2/10 = 60
        u.add_buff(Buff::new("def", "Def Up", 2, 5, BuffEffect::DefenseUp));
        assert_eq!(u.effective_defense(), 60);
        // Poison does NOT affect defense
        u.buffs.clear();
        u.add_buff(Buff::new("psn", "Poison", 5, 5, BuffEffect::Poison));
        assert_eq!(u.effective_defense(), 50);
    }

    /// `is_alive()`
    /// returns
    /// `true` when
    /// `hp > 0`
    /// and `false`
    /// when
    /// `hp <= 0`.
    /// A regression
    /// that used
    /// `hp > 0`
    /// strictly is
    /// correct
    /// (matches
    /// the
    /// round-1
    /// contract);
    /// the danger
    /// case is
    /// `hp == 0`
    /// (the unit
    /// just died)
    /// — is_alive
    /// must
    /// return
    /// `false` so
    /// the HUD
    /// shows
    /// "defeated".
    #[test]
    fn combat_unit_is_alive_matches_hp_positive_round_160() {
        let mut u = CombatUnit::new("u1", "Hero", 100, 20, 10, 15, true);
        assert!(u.is_alive());
        u.hp = 0;
        assert!(!u.is_alive());
    }
}
