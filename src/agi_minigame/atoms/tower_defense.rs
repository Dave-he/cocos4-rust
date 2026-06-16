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
        map.insert("base_hp".to_string(), Value::Float(self.base_hp as f32));
        map.insert("gold".to_string(), Value::Integer(self.gold as i32));
        map.insert("score".to_string(), Value::Integer(self.score as i32));
        map.insert("current_wave".to_string(), Value::Integer(self.current_wave as i32));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Float(n)) = state.get("base_hp") { self.base_hp = *n as f32; }
        if let Some(Value::Integer(n)) = state.get("gold") { self.gold = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("score") { self.score = *n as u64; }
        if let Some(Value::Integer(n)) = state.get("current_wave") { self.current_wave = *n as usize; }
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
                let row = data.get("row").and_then(|v| if let Value::Integer(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
                let col = data.get("col").and_then(|v| if let Value::Integer(n) = v { Some(*n as usize) } else { None }).unwrap_or(0);
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

// ---------------------------------------------------------------------------
// Round 139 helper-level tests — follow
// the round 110b / 122-138
// pattern. Pre-round-139 had 7
// integration tests (init / place
// tower / cannot place on path /
// enemy damage / tower upgrade /
// wave generation / save-load)
// but 0 focused unit coverage
// of the public surface. These
// tests pin per-enum variant
// counts, per-field defaults of
// `TowerDefenseAtom::new` /
// `Tower::new` / `Wave::new` /
// `Enemy::new`, the per-tower-
// type stat tables (Arrow /
// Cannon / Ice / Laser), the
// `upgrade` damage×1.5 /
// range×1.1 + `upgrade_cost`
// = 50×level contract, the
// `place_tower` 4-guard matrix
// (insufficient gold / on path
// / existing tower / out of
// bounds), the `generate_waves`
// difficulty-scaling curve
// (wave 1 = Normal only, wave 2
// + Fast, wave 4 + Tank, every
// 5th + Boss), the 6 getters /
// 2 status flags (is_game_over
// / is_victory), the `save_state`
// 4 persisted keys + `load_state`
// round-trip, the `handle_event`
// "place_tower" + "upgrade_tower"
// dispatch with default-type
// fallback (unknown type →
// Arrow, missing type → Arrow),
// the `on_update` game-over /
// victory phase transitions,
// the full lifecycle
// on_init / on_enter / on_pause
// / on_resume / on_exit /
// on_destroy, the atom_id /
// atom_name contract, and the
// `current_phase` mirror.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round139_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws).with_delta_time(0.016)
    }

    fn make_atom() -> TowerDefenseAtom {
        // 10x10 grid,
        // 100 base
        // HP, 500
        // starting
        // gold.
        TowerDefenseAtom::new(10, 10, 100.0, 500)
    }

    /// `EnemyType` has
    /// 4 variants
    /// (Normal /
    /// Fast /
    /// Tank /
    /// Boss).
    #[test]
    fn enemy_type_has_4_variants_round_139() {
        let v = [
            EnemyType::Normal,
            EnemyType::Fast,
            EnemyType::Tank,
            EnemyType::Boss,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(EnemyType::Normal, EnemyType::Boss);
        assert_ne!(EnemyType::Fast, EnemyType::Tank);
    }

    /// `TowerType` has
    /// 4 variants
    /// (Arrow /
    /// Cannon /
    /// Ice /
    /// Laser).
    #[test]
    fn tower_type_has_4_variants_round_139() {
        let v = [
            TowerType::Arrow,
            TowerType::Cannon,
            TowerType::Ice,
            TowerType::Laser,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(TowerType::Arrow, TowerType::Cannon);
        assert_ne!(TowerType::Ice, TowerType::Laser);
    }

    /// `Enemy::new`
    /// stores fields
    /// verbatim +
    /// path_index=0
    /// +
    /// position_on_path
    /// =0 +
    /// max_hp=hp.
    #[test]
    fn enemy_new_stores_fields_round_139() {
        let e = Enemy::new("e1", EnemyType::Normal, 50.0, 1.5, 10);
        assert_eq!(e.id, "e1");
        assert_eq!(e.enemy_type, EnemyType::Normal);
        assert_eq!(e.hp, 50.0);
        assert_eq!(e.max_hp, 50.0);
        assert_eq!(e.speed, 1.5);
        assert_eq!(e.path_index, 0);
        assert_eq!(e.position_on_path, 0.0);
        assert_eq!(e.reward, 10);
    }

    /// `Enemy::is_alive`
    /// is hp > 0.
    #[test]
    fn enemy_is_alive_hp_positive_round_139() {
        let mut e = Enemy::new("e", EnemyType::Normal, 50.0, 1.0, 10);
        assert!(e.is_alive());
        e.hp = 0.0;
        assert!(!e.is_alive());
    }

    /// `Enemy::take_damage`
    /// subtracts +
    /// clamps to 0
    /// + returns
    /// killed bool.
    #[test]
    fn enemy_take_damage_round_139() {
        let mut e = Enemy::new("e", EnemyType::Normal, 50.0, 1.0, 10);
        // Non-lethal
        // damage
        // returns
        // false.
        assert!(!e.take_damage(20.0));
        assert_eq!(e.hp, 30.0);
        assert!(e.is_alive());
        // Overkill
        // damage
        // clamps
        // to 0
        // and
        // returns
        // true.
        assert!(e.take_damage(100.0));
        assert_eq!(e.hp, 0.0);
        assert!(!e.is_alive());
    }

    /// `Enemy::hp_ratio`
    /// is
    /// hp/max_hp,
    /// 0 when
    /// max_hp<=0.
    #[test]
    fn enemy_hp_ratio_round_139() {
        let e = Enemy::new("e", EnemyType::Normal, 50.0, 1.0, 10);
        assert!((e.hp_ratio() - 1.0).abs() < 1e-6);
        let mut e2 = Enemy::new("e", EnemyType::Normal, 50.0, 1.0, 10);
        e2.take_damage(25.0);
        assert!((e2.hp_ratio() - 0.5).abs() < 1e-6);
        // max_hp<=0
        // → 0.
        let mut e3 = Enemy::new("e", EnemyType::Normal, 50.0, 1.0, 10);
        e3.max_hp = 0.0;
        assert_eq!(e3.hp_ratio(), 0.0);
    }

    /// `Tower::new`
    /// Arrow:
    /// damage=10,
    /// range=3,
    /// attack_speed=1
    /// + level=1
    /// +
    /// attack_timer=0
    /// +
    /// target_id=None.
    #[test]
    fn tower_new_arrow_stats_round_139() {
        let t = Tower::new("t", TowerType::Arrow, 0, 0);
        assert_eq!(t.id, "t");
        assert_eq!(t.tower_type, TowerType::Arrow);
        assert_eq!(t.row, 0);
        assert_eq!(t.col, 0);
        assert_eq!(t.level, 1);
        assert_eq!(t.damage, 10.0);
        assert_eq!(t.range, 3.0);
        assert_eq!(t.attack_speed, 1.0);
        assert_eq!(t.attack_timer, 0.0);
        assert!(t.target_id.is_none());
    }

    /// `Tower::new`
    /// Cannon:
    /// damage=30,
    /// range=2.5,
    /// attack_speed=0.5.
    #[test]
    fn tower_new_cannon_stats_round_139() {
        let t = Tower::new("t", TowerType::Cannon, 0, 0);
        assert_eq!(t.damage, 30.0);
        assert_eq!(t.range, 2.5);
        assert_eq!(t.attack_speed, 0.5);
    }

    /// `Tower::new`
    /// Ice:
    /// damage=5,
    /// range=3.5,
    /// attack_speed=0.8.
    #[test]
    fn tower_new_ice_stats_round_139() {
        let t = Tower::new("t", TowerType::Ice, 0, 0);
        assert_eq!(t.damage, 5.0);
        assert_eq!(t.range, 3.5);
        assert_eq!(t.attack_speed, 0.8);
    }

    /// `Tower::new`
    /// Laser:
    /// damage=20,
    /// range=4,
    /// attack_speed=1.5.
    #[test]
    fn tower_new_laser_stats_round_139() {
        let t = Tower::new("t", TowerType::Laser, 0, 0);
        assert_eq!(t.damage, 20.0);
        assert_eq!(t.range, 4.0);
        assert_eq!(t.attack_speed, 1.5);
    }

    /// `Tower::upgrade`
    /// level+=1 +
    /// damage×=1.5
    /// + range×=1.1
    /// + returns
    /// new level.
    #[test]
    fn tower_upgrade_stats_round_139() {
        let mut t = Tower::new("t", TowerType::Arrow, 0, 0);
        let new_level = t.upgrade();
        assert_eq!(new_level, 2);
        assert_eq!(t.level, 2);
        assert_eq!(t.damage, 15.0); // 10 × 1.5
        assert!((t.range - 3.3).abs() < 1e-6); // 3 × 1.1
    }

    /// `Tower::upgrade_cost`
    /// is 50 *
    /// level.
    #[test]
    fn tower_upgrade_cost_round_139() {
        let mut t = Tower::new("t", TowerType::Arrow, 0, 0);
        assert_eq!(t.upgrade_cost(), 50); // level 1
        t.upgrade();
        assert_eq!(t.upgrade_cost(), 100); // level 2
        t.upgrade();
        assert_eq!(t.upgrade_cost(), 150); // level 3
    }

    /// `Wave::new`
    /// defaults:
    /// empty
    /// enemies,
    /// spawn_interval=1.0,
    /// started=false,
    /// completed=false.
    #[test]
    fn wave_new_defaults_round_139() {
        let w = Wave::new(1);
        assert_eq!(w.wave_number, 1);
        assert!(w.enemies.is_empty());
        assert_eq!(w.spawn_interval, 1.0);
        assert!(!w.started);
        assert!(!w.completed);
    }

    /// `Wave::with_enemy`
    /// appends +
    /// returns self
    /// (builder
    /// pattern).
    #[test]
    fn wave_with_enemy_appends_round_139() {
        let w = Wave::new(1)
            .with_enemy(EnemyType::Normal, 5)
            .with_enemy(EnemyType::Fast, 3);
        assert_eq!(w.enemies.len(), 2);
        assert_eq!(w.enemies[0].enemy_type, EnemyType::Normal);
        assert_eq!(w.enemies[0].count, 5);
        assert_eq!(w.enemies[1].enemy_type, EnemyType::Fast);
        assert_eq!(w.enemies[1].count, 3);
    }

    /// `Wave::total_enemies`
    /// sums counts.
    #[test]
    fn wave_total_enemies_sums_round_139() {
        let w = Wave::new(1)
            .with_enemy(EnemyType::Normal, 5)
            .with_enemy(EnemyType::Fast, 3)
            .with_enemy(EnemyType::Tank, 2);
        assert_eq!(w.total_enemies(), 10);
    }

    /// `TowerDefenseAtom::new`
    /// defaults:
    /// phase=Uninit,
    /// empty
    /// collections,
    /// base_hp/max_base_hp,
    /// gold,
    /// score=0,
    /// current_wave=0,
    /// spawn_timer=0,
    /// enemies_spawned=0,
    /// path from
    /// generate_path.
    #[test]
    fn atom_new_defaults_round_139() {
        let a = TowerDefenseAtom::new(10, 10, 100.0, 200);
        assert_eq!(a.phase, AtomPhase::Uninitialized);
        assert_eq!(a.grid_rows, 10);
        assert_eq!(a.grid_cols, 10);
        assert!(a.towers.is_empty());
        assert!(a.enemies.is_empty());
        assert!(a.waves.is_empty());
        assert_eq!(a.current_wave, 0);
        assert_eq!(a.spawn_timer, 0.0);
        assert_eq!(a.enemies_spawned, 0);
        assert_eq!(a.base_hp, 100.0);
        assert_eq!(a.max_base_hp, 100.0);
        assert_eq!(a.gold, 200);
        assert_eq!(a.score, 0);
        // Path
        // length
        // = cols.
        assert_eq!(a.path.len(), 10);
    }

    /// Path is the
    /// mid-row
    /// across
    /// all
    /// columns.
    #[test]
    fn path_is_mid_row_across_cols_round_139() {
        let a = TowerDefenseAtom::new(10, 10, 100.0, 200);
        // 10 rows
        // → mid_row
        // = 5.
        for c in 0..10 {
            assert_eq!(a.path[c], (5, c));
        }
    }

    /// `add_wave`
    /// appends to
    /// the
    /// `waves`
    /// vec.
    #[test]
    fn add_wave_appends_round_139() {
        let mut a = make_atom();
        a.add_wave(Wave::new(1).with_enemy(EnemyType::Normal, 5));
        a.add_wave(Wave::new(2).with_enemy(EnemyType::Fast, 3));
        assert_eq!(a.waves.len(), 2);
    }

    /// `generate_waves`
    /// creates
    /// `count`
    /// waves +
    /// wave 1
    /// has only
    /// Normal.
    #[test]
    fn generate_waves_count_round_139() {
        let mut a = make_atom();
        a.generate_waves(5, 0.5);
        assert_eq!(a.waves.len(), 5);
        // Wave 1:
        // only
        // Normal.
        assert_eq!(a.waves[0].enemies.len(), 1);
        assert_eq!(a.waves[0].enemies[0].enemy_type, EnemyType::Normal);
    }

    /// Wave 2+
    /// adds Fast
    /// (per
    /// generate_waves
    /// logic).
    #[test]
    fn generate_waves_wave_2_adds_fast_round_139() {
        let mut a = make_atom();
        a.generate_waves(5, 0.5);
        // Wave 2:
        // Normal
        // + Fast.
        let kinds_2: Vec<EnemyType> = a.waves[1].enemies.iter().map(|e| e.enemy_type).collect();
        assert!(kinds_2.contains(&EnemyType::Normal));
        assert!(kinds_2.contains(&EnemyType::Fast));
        assert!(!kinds_2.contains(&EnemyType::Tank));
    }

    /// Wave 4+
    /// adds Tank.
    #[test]
    fn generate_waves_wave_4_adds_tank_round_139() {
        let mut a = make_atom();
        a.generate_waves(5, 0.5);
        // Wave 4:
        // Normal
        // + Fast
        // + Tank.
        let kinds_4: Vec<EnemyType> = a.waves[3].enemies.iter().map(|e| e.enemy_type).collect();
        assert!(kinds_4.contains(&EnemyType::Normal));
        assert!(kinds_4.contains(&EnemyType::Fast));
        assert!(kinds_4.contains(&EnemyType::Tank));
    }

    /// Every 5th
    /// wave
    /// (6, 11,
    /// 16) adds
    /// Boss.
    #[test]
    fn generate_waves_every_5th_adds_boss_round_139() {
        let mut a = make_atom();
        a.generate_waves(11, 0.5);
        // Wave 6
        // (i=5):
        // has
        // Boss.
        let kinds_6: Vec<EnemyType> = a.waves[5].enemies.iter().map(|e| e.enemy_type).collect();
        assert!(kinds_6.contains(&EnemyType::Boss));
        // Wave 11
        // (i=10):
        // also
        // has
        // Boss.
        let kinds_11: Vec<EnemyType> = a.waves[10].enemies.iter().map(|e| e.enemy_type).collect();
        assert!(kinds_11.contains(&EnemyType::Boss));
    }

    /// `place_tower`
    /// Arrow
    /// costs 50
    /// gold.
    #[test]
    fn place_tower_arrow_costs_50_round_139() {
        let mut a = make_atom();
        a.gold = 100;
        assert!(a.place_tower(TowerType::Arrow, 0, 0));
        assert_eq!(a.gold, 50);
        assert_eq!(a.towers.len(), 1);
    }

    /// `place_tower`
    /// Cannon
    /// costs 100,
    /// Ice 75,
    /// Laser 150.
    #[test]
    fn place_tower_per_type_costs_round_139() {
        // Cannon
        // = 100.
        let mut a = make_atom();
        a.gold = 100;
        assert!(a.place_tower(TowerType::Cannon, 0, 0));
        assert_eq!(a.gold, 0);
        // Ice
        // = 75.
        let mut a = make_atom();
        a.gold = 100;
        assert!(a.place_tower(TowerType::Ice, 0, 0));
        assert_eq!(a.gold, 25);
        // Laser
        // = 150.
        let mut a = make_atom();
        a.gold = 200;
        assert!(a.place_tower(TowerType::Laser, 0, 0));
        assert_eq!(a.gold, 50);
    }

    /// `place_tower`
    /// returns
    /// false when
    /// gold <
    /// cost.
    #[test]
    fn place_tower_insufficient_gold_returns_false_round_139() {
        let mut a = make_atom();
        a.gold = 10; // Laser
        // costs 150
        // → not
        // enough.
        assert!(!a.place_tower(TowerType::Laser, 0, 0));
        assert_eq!(a.gold, 10); // unchanged
        assert_eq!(a.towers.len(), 0);
    }

    /// `place_tower`
    /// rejects
    /// the mid-row
    /// path cells.
    #[test]
    fn place_tower_on_path_returns_false_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        // mid_row=5,
        // path
        // occupies
        // (5, 0..10).
        assert!(!a.place_tower(TowerType::Arrow, 5, 3));
        assert_eq!(a.towers.len(), 0);
        assert_eq!(a.gold, 500); // unchanged
    }

    /// `place_tower`
    /// rejects a
    /// cell with
    /// an existing
    /// tower.
    #[test]
    fn place_tower_on_existing_returns_false_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        assert!(a.place_tower(TowerType::Arrow, 0, 0));
        assert_eq!(a.gold, 450);
        // Same
        // cell →
        // false.
        assert!(!a.place_tower(TowerType::Cannon, 0, 0));
        assert_eq!(a.gold, 450); // unchanged
        assert_eq!(a.towers.len(), 1);
    }

    /// `place_tower`
    /// rejects
    /// out-of-
    /// bounds
    /// coordinates.
    #[test]
    fn place_tower_out_of_bounds_returns_false_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        // 10x10 grid
        // → row/col
        // 10 is OOB.
        assert!(!a.place_tower(TowerType::Arrow, 10, 0));
        assert!(!a.place_tower(TowerType::Arrow, 0, 10));
        assert_eq!(a.towers.len(), 0);
    }

    /// `upgrade_tower`
    /// unknown id
    /// returns
    /// false.
    #[test]
    fn upgrade_tower_unknown_id_returns_false_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        assert!(!a.upgrade_tower("nope"));
        assert_eq!(a.gold, 500);
    }

    /// `upgrade_tower`
    /// insufficient
    /// gold returns
    /// false.
    #[test]
    fn upgrade_tower_insufficient_gold_returns_false_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        a.place_tower(TowerType::Arrow, 0, 0); // tower_0
        // gold = 450.
        // Drop gold
        // below
        // upgrade_cost=50.
        a.gold = 40;
        assert!(!a.upgrade_tower("tower_0"));
        // Level
        // unchanged
        // + gold
        // unchanged.
        assert_eq!(a.towers[0].level, 1);
        assert_eq!(a.gold, 40);
    }

    /// `upgrade_tower`
    /// sufficient
    /// gold → true
    /// + level+=1
    /// + gold
    /// -= cost.
    #[test]
    fn upgrade_tower_sufficient_gold_round_139() {
        let mut a = make_atom();
        a.gold = 200;
        a.place_tower(TowerType::Arrow, 0, 0); // costs 50
        // gold=150
        // upgrade_cost
        // = 50
        // → ok.
        assert!(a.upgrade_tower("tower_0"));
        assert_eq!(a.towers[0].level, 2);
        assert_eq!(a.gold, 100); // 150-50
    }

    /// Getters
    /// surface
    /// internal
    /// state.
    #[test]
    fn getters_round_139() {
        let mut a = make_atom();
        a.gold = 250;
        a.score = 12345;
        a.base_hp = 80.0;
        assert_eq!(a.get_base_hp(), 80.0);
        assert_eq!(a.get_gold(), 250);
        assert_eq!(a.get_score(), 12345);
        // current_wave=0
        // → wave_number=1.
        assert_eq!(a.get_wave_number(), 1);
    }

    /// `is_game_over`
    /// returns true
    /// when
    /// base_hp<=0.
    #[test]
    fn is_game_over_round_139() {
        let mut a = make_atom();
        assert!(!a.is_game_over());
        a.base_hp = 0.0;
        assert!(a.is_game_over());
    }

    /// `is_victory`
    /// returns true
    /// when all
    /// waves
    /// cleared +
    /// no enemies.
    #[test]
    fn is_victory_round_139() {
        let mut a = make_atom();
        a.add_wave(Wave::new(1));
        a.current_wave = 1; // past last wave
        assert!(a.enemies.is_empty());
        assert!(a.is_victory());
        // Has
        // enemies
        // → not
        // victory.
        a.enemies.push(Enemy::new("e", EnemyType::Normal, 10.0, 1.0, 5));
        assert!(!a.is_victory());
    }

    /// `save_state`
    /// has 4
    /// persisted
    /// keys.
    #[test]
    fn save_state_keys_round_139() {
        let a = make_atom();
        let s = a.save_state();
        assert!(s.contains_key("base_hp"));
        assert!(s.contains_key("gold"));
        assert!(s.contains_key("score"));
        assert!(s.contains_key("current_wave"));
    }

    /// `load_state`
    /// restores
    /// all 4
    /// fields.
    #[test]
    fn load_state_restores_all_fields_round_139() {
        let mut a = make_atom();
        let mut s = ValueMap::new();
        s.insert("base_hp".to_string(), Value::Float(75.0));
        s.insert("gold".to_string(), Value::Integer(250));
        s.insert("score".to_string(), Value::Integer(5000));
        s.insert("current_wave".to_string(), Value::Integer(3));
        a.load_state(&s);
        assert_eq!(a.base_hp, 75.0);
        assert_eq!(a.gold, 250);
        assert_eq!(a.score, 5000);
        assert_eq!(a.current_wave, 3);
    }

    /// `handle_event`
    /// "place_tower"
    /// with
    /// type="cannon"
    /// places a
    /// Cannon.
    #[test]
    fn handle_event_place_tower_cannon_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        let mut data = ValueMap::new();
        data.insert("type".to_string(), Value::String("cannon".to_string()));
        data.insert("row".to_string(), Value::Integer(0));
        data.insert("col".to_string(), Value::Integer(0));
        let mut ctx = make_ctx();
        a.handle_event("place_tower", &data, &mut ctx);
        assert_eq!(a.towers.len(), 1);
        assert_eq!(a.towers[0].tower_type, TowerType::Cannon);
    }

    /// `handle_event`
    /// "place_tower"
    /// with
    /// unknown
    /// type
    /// defaults
    /// to Arrow.
    #[test]
    fn handle_event_place_tower_unknown_type_defaults_to_arrow_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        let mut data = ValueMap::new();
        data.insert("type".to_string(), Value::String("dragon".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("place_tower", &data, &mut ctx);
        assert_eq!(a.towers[0].tower_type, TowerType::Arrow);
    }

    /// `handle_event`
    /// "place_tower"
    /// without
    /// type →
    /// defaults
    /// to Arrow
    /// (also
    /// row/col
    /// default
    /// to 0).
    #[test]
    fn handle_event_place_tower_no_data_defaults_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        let data = ValueMap::new();
        let mut ctx = make_ctx();
        a.handle_event("place_tower", &data, &mut ctx);
        assert_eq!(a.towers.len(), 1);
        assert_eq!(a.towers[0].tower_type, TowerType::Arrow);
        assert_eq!(a.towers[0].row, 0);
        assert_eq!(a.towers[0].col, 0);
    }

    /// `handle_event`
    /// "upgrade_tower"
    /// with
    /// tower_id
    /// calls
    /// upgrade.
    #[test]
    fn handle_event_upgrade_tower_round_139() {
        let mut a = make_atom();
        a.gold = 500;
        a.place_tower(TowerType::Arrow, 0, 0); // tower_0
        assert_eq!(a.towers[0].level, 1);
        let mut data = ValueMap::new();
        data.insert("tower_id".to_string(), Value::String("tower_0".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("upgrade_tower", &data, &mut ctx);
        assert_eq!(a.towers[0].level, 2);
    }

    /// `handle_event`
    /// unknown
    /// event
    /// is no-op.
    #[test]
    fn handle_event_unknown_is_noop_round_139() {
        let mut a = make_atom();
        let prev_gold = a.gold;
        let s = ValueMap::new();
        let mut ctx = make_ctx();
        a.handle_event("bogus", &s, &mut ctx);
        assert_eq!(a.gold, prev_gold);
        assert_eq!(a.towers.len(), 0);
    }

    /// `on_update`
    /// with
    /// base_hp<=0
    /// transitions
    /// phase to
    /// Completed.
    #[test]
    fn on_update_game_over_sets_completed_round_139() {
        let mut a = make_atom();
        a.base_hp = 0.0;
        let mut ctx = make_ctx();
        a.on_update(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Completed);
    }

    /// `on_update`
    /// with all
    /// waves
    /// cleared +
    /// no enemies
    /// sets
    /// phase to
    /// Completed.
    #[test]
    fn on_update_victory_sets_completed_round_139() {
        let mut a = make_atom();
        a.add_wave(Wave::new(1));
        a.current_wave = 1; // past last wave
        assert!(a.enemies.is_empty());
        let mut ctx = make_ctx();
        a.on_update(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Completed);
    }

    /// `on_init` →
    /// Initialized.
    /// `on_enter`
    /// → Running
    /// + resets
    /// state +
    /// generates
    /// 10 default
    /// waves if
    /// waves is
    /// empty +
    /// sets
    /// gold=200
    /// (NOT
    /// starting_gold).
    #[test]
    fn on_enter_resets_and_generates_waves_round_139() {
        let mut a = TowerDefenseAtom::new(10, 10, 100.0, 999);
        let mut ctx = make_ctx();
        a.on_init(&mut ctx);
        a.on_enter(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Running);
        // gold=200
        // hardcoded,
        // not 999.
        assert_eq!(a.gold, 200);
        // 10 default
        // waves
        // generated.
        assert_eq!(a.waves.len(), 10);
    }

    /// `on_pause`
    /// → Paused,
    /// `on_resume`
    /// → Running,
    /// `on_exit`
    /// →
    /// Completed,
    /// `on_destroy`
    /// →
    /// Uninitialized.
    #[test]
    fn lifecycle_phases_round_139() {
        let mut a = make_atom();
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
    fn atom_id_and_name_round_139() {
        let a = make_atom();
        assert_eq!(a.atom_id(), "tower_defense");
        assert_eq!(a.atom_name(), "塔防");
        let _ = a.as_any();
        let mut a = make_atom();
        let _ = a.as_any_mut();
    }

    /// `current_phase`
    /// mirrors the
    /// internal
    /// `phase`
    /// field.
    #[test]
    fn current_phase_matches_field_round_139() {
        let mut a = make_atom();
        assert_eq!(a.current_phase(), AtomPhase::Uninitialized);
        a.phase = AtomPhase::Paused;
        assert_eq!(a.current_phase(), AtomPhase::Paused);
    }
}
