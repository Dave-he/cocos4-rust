use std::any::Any;

use crate::base::value::{Value, ValueMap};

use super::super::atom::{Atom, AtomContext, AtomId, AtomPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyType {
    Normal,
    Fast,
    Tank,
    Boss,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: String,
    pub enemy_type: EnemyType,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub path_index: usize,
    pub position_on_path: f32,
    pub reward: u32,
}

impl Enemy {
    pub fn new(id: &str, enemy_type: EnemyType, hp: f32, speed: f32, reward: u32) -> Self {
        Self {
            id: id.to_string(),
            enemy_type,
            hp,
            max_hp: hp,
            speed,
            path_index: 0,
            position_on_path: 0.0,
            reward,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
    }

    pub fn take_damage(&mut self, damage: f32) -> bool {
        self.hp = (self.hp - damage).max(0.0);
        self.hp <= 0.0
    }

    pub fn hp_ratio(&self) -> f32 {
        if self.max_hp <= 0.0 { 0.0 } else { self.hp / self.max_hp }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TowerType {
    Arrow,
    Cannon,
    Ice,
    Laser,
}

#[derive(Debug, Clone)]
pub struct Tower {
    pub id: String,
    pub tower_type: TowerType,
    pub row: usize,
    pub col: usize,
    pub level: u32,
    pub damage: f32,
    pub range: f32,
    pub attack_speed: f32,
    pub attack_timer: f32,
    pub target_id: Option<String>,
}

impl Tower {
    pub fn new(id: &str, tower_type: TowerType, row: usize, col: usize) -> Self {
        let (damage, range, attack_speed) = match tower_type {
            TowerType::Arrow => (10.0, 3.0, 1.0),
            TowerType::Cannon => (30.0, 2.5, 0.5),
            TowerType::Ice => (5.0, 3.5, 0.8),
            TowerType::Laser => (20.0, 4.0, 1.5),
        };

        Self {
            id: id.to_string(),
            tower_type,
            row,
            col,
            level: 1,
            damage,
            range,
            attack_speed,
            attack_timer: 0.0,
            target_id: None,
        }
    }

    pub fn upgrade(&mut self) -> u32 {
        self.level += 1;
        self.damage *= 1.5;
        self.range *= 1.1;
        self.level
    }

    pub fn upgrade_cost(&self) -> u32 {
        50 * self.level
    }
}

#[derive(Debug, Clone)]
pub struct Wave {
    pub wave_number: u32,
    pub enemies: Vec<EnemySpawn>,
    pub spawn_interval: f32,
    pub started: bool,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct EnemySpawn {
    pub enemy_type: EnemyType,
    pub count: u32,
}

impl Wave {
    pub fn new(wave_number: u32) -> Self {
        Self {
            wave_number,
            enemies: Vec::new(),
            spawn_interval: 1.0,
            started: false,
            completed: false,
        }
    }

    pub fn with_enemy(mut self, enemy_type: EnemyType, count: u32) -> Self {
        self.enemies.push(EnemySpawn { enemy_type, count });
        self
    }

    pub fn total_enemies(&self) -> u32 {
        self.enemies.iter().map(|e| e.count).sum()
    }
}

pub struct TowerDefenseAtom {
    phase: AtomPhase,
    grid_rows: usize,
    grid_cols: usize,
    towers: Vec<Tower>,
    enemies: Vec<Enemy>,
    waves: Vec<Wave>,
    current_wave: usize,
    spawn_timer: f32,
    enemies_spawned: u32,
    base_hp: f32,
    max_base_hp: f32,
    gold: u32,
    score: u64,
    path: Vec<(usize, usize)>,
}

impl TowerDefenseAtom {
    pub fn new(grid_rows: usize, grid_cols: usize, base_hp: f32, starting_gold: u32) -> Self {
        let path = Self::generate_path(grid_rows, grid_cols);
        Self {
            phase: AtomPhase::Uninitialized,
            grid_rows,
            grid_cols,
            towers: Vec::new(),
            enemies: Vec::new(),
            waves: Vec::new(),
            current_wave: 0,
            spawn_timer: 0.0,
            enemies_spawned: 0,
            base_hp,
            max_base_hp: base_hp,
            gold: starting_gold,
            score: 0,
            path,
        }
    }

    fn generate_path(rows: usize, cols: usize) -> Vec<(usize, usize)> {
        let mut path = Vec::new();
        let mid_row = rows / 2;
        for c in 0..cols {
            path.push((mid_row, c));
        }
        path
    }

    pub fn add_wave(&mut self, wave: Wave) {
        self.waves.push(wave);
    }

    pub fn generate_waves(&mut self, count: u32, difficulty: f32) {
        for i in 0..count {
            let mut wave = Wave::new(i + 1);
            let normal_count = (3.0 + i as f32 * difficulty * 2.0) as u32;
            wave = wave.with_enemy(EnemyType::Normal, normal_count);

            if i > 0 {
                let fast_count = (1.0 + i as f32 * difficulty) as u32;
                wave = wave.with_enemy(EnemyType::Fast, fast_count);
            }

            if i > 2 {
                let tank_count = (i as f32 * difficulty * 0.5) as u32;
                wave = wave.with_enemy(EnemyType::Tank, tank_count.max(1));
            }

            if i > 0 && i % 5 == 0 {
                wave = wave.with_enemy(EnemyType::Boss, 1);
            }

            wave.spawn_interval = (1.5 - difficulty * 0.5).max(0.3);
            self.waves.push(wave);
        }
    }

    pub fn place_tower(&mut self, tower_type: TowerType, row: usize, col: usize) -> bool {
        let cost = match tower_type {
            TowerType::Arrow => 50,
            TowerType::Cannon => 100,
            TowerType::Ice => 75,
            TowerType::Laser => 150,
        };

        if self.gold < cost {
            return false;
        }

        if self.path.contains(&(row, col)) {
            return false;
        }

        if self.towers.iter().any(|t| t.row == row && t.col == col) {
            return false;
        }

        if row >= self.grid_rows || col >= self.grid_cols {
            return false;
        }

        self.gold -= cost;
        let id = format!("tower_{}", self.towers.len());
        self.towers.push(Tower::new(&id, tower_type, row, col));
        true
    }

    pub fn upgrade_tower(&mut self, tower_id: &str) -> bool {
        if let Some(tower) = self.towers.iter_mut().find(|t| t.id == tower_id) {
            let cost = tower.upgrade_cost();
            if self.gold >= cost {
                self.gold -= cost;
                tower.upgrade();
                return true;
            }
        }
        false
    }

    fn update_towers(&mut self, dt: f32) {
        for tower in &mut self.towers {
            tower.attack_timer += dt;
            if tower.attack_timer < 1.0 / tower.attack_speed {
                continue;
            }
            tower.attack_timer = 0.0;

            let mut best_target: Option<(usize, f32)> = None;
            for (i, enemy) in self.enemies.iter().enumerate() {
                if !enemy.is_alive() {
                    continue;
                }
                let dr = (tower.row as f32 - self.path[enemy.path_index].0 as f32).abs();
                let dc = (tower.col as f32 - self.path[enemy.path_index].1 as f32).abs();
                let dist = (dr * dr + dc * dc).sqrt();
                if dist <= tower.range {
                    match best_target {
                        None => best_target = Some((i, dist)),
                        Some((_, best_dist)) if dist < best_dist => best_target = Some((i, dist)),
                        _ => {}
                    }
                }
            }

            if let Some((target_idx, _)) = best_target {
                let killed = self.enemies[target_idx].take_damage(tower.damage);
                tower.target_id = Some(self.enemies[target_idx].id.clone());
                if killed {
                    self.gold += self.enemies[target_idx].reward;
                    self.score += self.enemies[target_idx].reward as u64 * 10;
                }
            } else {
                tower.target_id = None;
            }
        }
    }

    fn update_enemies(&mut self, dt: f32) {
        for enemy in &mut self.enemies {
            if !enemy.is_alive() {
                continue;
            }
            enemy.position_on_path += enemy.speed * dt;
            while enemy.position_on_path >= 1.0 && enemy.path_index < self.path.len() - 1 {
                enemy.position_on_path -= 1.0;
                enemy.path_index += 1;
            }

            if enemy.path_index >= self.path.len() - 1 && enemy.position_on_path >= 0.5 {
                self.base_hp -= 10.0;
                enemy.hp = 0.0;
            }
        }

        self.enemies.retain(|e| e.is_alive());
    }

    fn spawn_enemies(&mut self, dt: f32) {
        if self.current_wave >= self.waves.len() {
            return;
        }

        let wave = &mut self.waves[self.current_wave];
        if !wave.started {
            wave.started = true;
        }

        self.spawn_timer += dt;
        if self.spawn_timer < wave.spawn_interval {
            return;
        }
        self.spawn_timer = 0.0;

        let total = wave.total_enemies();
        if self.enemies_spawned < total {
            let mut spawned = 0u32;
            for spawn in &wave.enemies {
                if spawned + spawn.count > self.enemies_spawned {
                    let (hp, speed, reward) = match spawn.enemy_type {
                        EnemyType::Normal => (50.0, 1.0, 10),
                        EnemyType::Fast => (30.0, 2.0, 15),
                        EnemyType::Tank => (150.0, 0.5, 25),
                        EnemyType::Boss => (500.0, 0.3, 100),
                    };
                    let id = format!("e_{}_{}", self.current_wave, self.enemies_spawned);
                    self.enemies.push(Enemy::new(&id, spawn.enemy_type, hp, speed, reward));
                    self.enemies_spawned += 1;
                    break;
                }
                spawned += spawn.count;
            }
        }

        if self.enemies_spawned >= total && self.enemies.is_empty() {
            self.waves[self.current_wave].completed = true;
            self.current_wave += 1;
            self.enemies_spawned = 0;
            self.gold += 50;
        }
    }

    pub fn get_base_hp(&self) -> f32 { self.base_hp }
    pub fn get_gold(&self) -> u32 { self.gold }
    pub fn get_score(&self) -> u64 { self.score }
    pub fn get_wave_number(&self) -> u32 { self.current_wave as u32 + 1 }
    pub fn is_game_over(&self) -> bool { self.base_hp <= 0.0 }
    pub fn is_victory(&self) -> bool { self.current_wave >= self.waves.len() && self.enemies.is_empty() }
}

impl Atom for TowerDefenseAtom {
    fn atom_id(&self) -> AtomId { "tower_defense".to_string() }
    fn atom_name(&self) -> &str { "塔防" }

    fn on_init(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Initialized; }

    fn on_enter(&mut self, _ctx: &mut AtomContext) {
        self.base_hp = self.max_base_hp;
        self.gold = 200;
        self.score = 0;
        self.current_wave = 0;
        self.enemies.clear();
        self.towers.clear();
        self.enemies_spawned = 0;
        self.spawn_timer = 0.0;
        if self.waves.is_empty() {
            self.generate_waves(10, 0.5);
        }
        self.phase = AtomPhase::Running;
    }

    fn on_update(&mut self, ctx: &mut AtomContext) {
        if self.is_game_over() || self.is_victory() {
            self.phase = AtomPhase::Completed;
            return;
        }
        let dt = ctx.delta_time;
        self.spawn_enemies(dt);
        self.update_towers(dt);
        self.update_enemies(dt);
    }

    fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
    fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
    fn on_exit(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Completed; }
    fn on_destroy(&mut self) { self.phase = AtomPhase::Uninitialized; }

    fn save_state(&self) -> ValueMap {
        let mut map = ValueMap::new();
        map.insert("base_hp".to_string(), Value::Float(self.base_hp as f64));
        map.insert("gold".to_string(), Value::Int(self.gold as i64));
        map.insert("score".to_string(), Value::Int(self.score as i64));
        map.insert("current_wave".to_string(), Value::Int(self.current_wave as i64));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Float(n)) = state.get("base_hp") { self.base_hp = *n as f32; }
        if let Some(Value::Int(n)) = state.get("gold") { self.gold = *n as u32; }
        if let Some(Value::Int(n)) = state.get("score") { self.score = *n as u64; }
        if let Some(Value::Int(n)) = state.get("current_wave") { self.current_wave = *n as usize; }
    }

    fn handle_event(&mut self, event: &str, data: &ValueMap, _ctx: &mut AtomContext) {
        match event {
            "place_tower" => {
                let tt = data.get("type").and_then(|v| if let Value::String(s) = v {
                    match s.as_str() {
                        "arrow" => Some(TowerType::Arrow),
                        "cannon" => Some(TowerType::Cannon),
                        "ice" => Some(TowerType::Ice),
                        "laser" => Some(TowerType::Laser),
                        _ => None,
                    }
                } else { None }).unwrap_or(TowerType::Arrow);
                let row = data.get("row").and_then(|v| if let Value::Int(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
                let col = data.get("col").and_then(|v| if let Value::Int(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
                self.place_tower(tt, row, col);
            }
            "upgrade_tower" => {
                if let Some(Value::String(id)) = data.get("tower_id") {
                    self.upgrade_tower(id);
                }
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
        AtomContext::new(ws).with_delta_time(0.016)
    }

    #[test]
    fn test_td_init() {
        let mut atom = TowerDefenseAtom::new(10, 10, 100.0, 200);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        assert_eq!(atom.get_base_hp(), 100.0);
        assert_eq!(atom.get_gold(), 200);
    }

    #[test]
    fn test_place_tower() {
        let mut atom = TowerDefenseAtom::new(10, 10, 100.0, 200);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        assert!(atom.place_tower(TowerType::Arrow, 0, 0));
        assert_eq!(atom.get_gold(), 150);
        assert!(!atom.place_tower(TowerType::Arrow, 0, 0));
    }

    #[test]
    fn test_cannot_place_on_path() {
        let mut atom = TowerDefenseAtom::new(10, 10, 100.0, 200);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        let mid_row = 10 / 2;
        assert!(!atom.place_tower(TowerType::Arrow, mid_row, 0));
    }

    #[test]
    fn test_enemy_damage() {
        let mut enemy = Enemy::new("e1", EnemyType::Normal, 50.0, 1.0, 10);
        assert!(enemy.is_alive());
        assert!(!enemy.take_damage(30.0));
        assert!(enemy.is_alive());
        assert!(enemy.take_damage(20.0));
        assert!(!enemy.is_alive());
    }

    #[test]
    fn test_tower_upgrade() {
        let mut tower = Tower::new("t1", TowerType::Arrow, 0, 0);
        let orig_damage = tower.damage;
        tower.upgrade();
        assert_eq!(tower.level, 2);
        assert!(tower.damage > orig_damage);
    }

    #[test]
    fn test_wave_generation() {
        let mut atom = TowerDefenseAtom::new(10, 10, 100.0, 200);
        atom.generate_waves(5, 0.5);
        assert_eq!(atom.waves.len(), 5);
        assert!(atom.waves[0].total_enemies() > 0);
    }

    #[test]
    fn test_td_save_load() {
        let mut atom = TowerDefenseAtom::new(10, 10, 100.0, 200);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        atom.score = 1000;
        atom.gold = 300;

        let state = atom.save_state();
        let mut atom2 = TowerDefenseAtom::new(10, 10, 100.0, 200);
        atom2.load_state(&state);
        assert_eq!(atom2.score, 1000);
        assert_eq!(atom2.gold, 300);
    }
}
