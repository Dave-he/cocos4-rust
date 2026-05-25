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
        let count = (1 + difficulty * 3.0) as usize;
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
        map.insert("turn".to_string(), Value::Int(self.turn as i64));
        map.insert("score".to_string(), Value::Int(self.score as i64));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Int(n)) = state.get("turn") { self.turn = *n as u32; }
        if let Some(Value::Int(n)) = state.get("score") { self.score = *n as u64; }
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
}
