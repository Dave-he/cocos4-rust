use std::any::Any;

use crate::base::value::{Value, ValueMap};

use super::super::atom::{Atom, AtomContext, AtomId, AtomPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObstacleType {
    Low,
    High,
    Gap,
    Spike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectibleType {
    Coin,
    Gem,
    PowerUp,
    Shield,
}

#[derive(Debug, Clone)]
pub struct Obstacle {
    pub position: f32,
    pub lane: u8,
    pub obstacle_type: ObstacleType,
    pub width: f32,
    pub passed: bool,
}

#[derive(Debug, Clone)]
pub struct Collectible {
    pub position: f32,
    pub lane: u8,
    pub collectible_type: CollectibleType,
    pub value: u32,
    pub collected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerAction {
    Jump,
    Slide,
    Dash,
    LaneLeft,
    LaneRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerState {
    Running,
    Jumping,
    Sliding,
    Dashing,
    Hit,
    Dead,
}

pub struct ParkourAtom {
    phase: AtomPhase,
    state: RunnerState,
    lane: u8,
    num_lanes: u8,
    position: f32,
    speed: f32,
    base_speed: f32,
    max_speed: f32,
    distance: f32,
    score: u64,
    coins: u32,
    hp: i32,
    max_hp: i32,
    obstacles: Vec<Obstacle>,
    collectibles: Vec<Collectible>,
    action_timer: f32,
    dash_timer: f32,
    invincible_timer: f32,
    spawn_distance: f32,
    next_spawn_distance: f32,
    difficulty: f32,
}

impl ParkourAtom {
    pub fn new(num_lanes: u8, base_speed: f32, hp: i32) -> Self {
        Self {
            phase: AtomPhase::Uninitialized,
            state: RunnerState::Running,
            lane: num_lanes / 2,
            num_lanes,
            position: 0.0,
            speed: base_speed,
            base_speed,
            max_speed: base_speed * 3.0,
            distance: 0.0,
            score: 0,
            coins: 0,
            hp,
            max_hp: hp,
            obstacles: Vec::new(),
            collectibles: Vec::new(),
            action_timer: 0.0,
            dash_timer: 0.0,
            invincible_timer: 0.0,
            spawn_distance: 0.0,
            next_spawn_distance: 5.0,
            difficulty: 0.5,
        }
    }

    pub fn perform_action(&mut self, action: RunnerAction) {
        match action {
            RunnerAction::Jump => {
                if self.state == RunnerState::Running {
                    self.state = RunnerState::Jumping;
                    self.action_timer = 0.6;
                }
            }
            RunnerAction::Slide => {
                if self.state == RunnerState::Running {
                    self.state = RunnerState::Sliding;
                    self.action_timer = 0.5;
                }
            }
            RunnerAction::Dash => {
                if self.dash_timer <= 0.0 {
                    self.state = RunnerState::Dashing;
                    self.dash_timer = 0.3;
                    self.speed = self.base_speed * 2.5;
                    self.invincible_timer = 0.3;
                }
            }
            RunnerAction::LaneLeft => {
                if self.lane > 0 {
                    self.lane -= 1;
                }
            }
            RunnerAction::LaneRight => {
                if self.lane < self.num_lanes - 1 {
                    self.lane += 1;
                }
            }
        }
    }

    fn spawn_content(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if self.spawn_distance >= self.next_spawn_distance {
            self.spawn_distance = 0.0;
            self.next_spawn_distance = 2.0 + rng.gen::<f32>() * 3.0 * (1.0 - self.difficulty * 0.3);

            let lane = rng.gen_range(0..self.num_lanes);

            if rng.gen_bool(0.7) {
                let obs_type = match rng.gen_range(0..4) {
                    0 => ObstacleType::Low,
                    1 => ObstacleType::High,
                    2 => ObstacleType::Gap,
                    _ => ObstacleType::Spike,
                };
                self.obstacles.push(Obstacle {
                    position: 50.0,
                    lane,
                    obstacle_type: obs_type,
                    width: 1.0,
                    passed: false,
                });
            }

            if rng.gen_bool(0.5) {
                let col_type = match rng.gen_bool(0.7) {
                    true => CollectibleType::Coin,
                    false if rng.gen_bool(0.5) => CollectibleType::Gem,
                    false if rng.gen_bool(0.5) => CollectibleType::PowerUp,
                    _ => CollectibleType::Shield,
                };
                let value = match col_type {
                    CollectibleType::Coin => 10,
                    CollectibleType::Gem => 50,
                    CollectibleType::PowerUp => 25,
                    CollectibleType::Shield => 30,
                };
                let col_lane = if rng.gen_bool(0.3) { lane } else { rng.gen_range(0..self.num_lanes) };
                self.collectibles.push(Collectible {
                    position: 50.0,
                    lane: col_lane,
                    collectible_type: col_type,
                    value,
                    collected: false,
                });
            }
        }
    }

    fn update_movement(&mut self, dt: f32) {
        let move_speed = self.speed * dt;
        self.position += move_speed;
        self.distance += move_speed;
        self.score += (move_speed * 10.0) as u64;

        for obs in &mut self.obstacles {
            obs.position -= move_speed;
        }
        for col in &mut self.collectibles {
            col.position -= move_speed;
        }

        self.obstacles.retain(|o| o.position > -5.0);
        self.collectibles.retain(|c| c.position > -5.0);

        self.spawn_distance += move_speed;
    }

    fn update_timers(&mut self, dt: f32) {
        if self.action_timer > 0.0 {
            self.action_timer -= dt;
            if self.action_timer <= 0.0 {
                self.state = RunnerState::Running;
                self.action_timer = 0.0;
            }
        }

        if self.dash_timer > 0.0 {
            self.dash_timer -= dt;
            if self.dash_timer <= 0.0 {
                self.speed = self.base_speed;
            }
        }

        if self.invincible_timer > 0.0 {
            self.invincible_timer -= dt;
        }

        self.speed = (self.speed + dt * 0.1).min(self.max_speed);
        self.difficulty = (0.3 + self.distance / 1000.0).min(1.0);
    }

    fn check_collisions(&mut self) {
        let player_lane = self.lane;

        for obs in &mut self.obstacles {
            if obs.passed || obs.lane != player_lane {
                continue;
            }
            if obs.position.abs() < 0.8 {
                let dodged = match obs.obstacle_type {
                    ObstacleType::Low => self.state == RunnerState::Jumping,
                    ObstacleType::High => self.state == RunnerState::Sliding,
                    ObstacleType::Gap => self.state == RunnerState::Jumping,
                    ObstacleType::Spike => self.state != RunnerState::Running,
                };

                if dodged {
                    obs.passed = true;
                    self.score += 50;
                } else if self.invincible_timer <= 0.0 {
                    self.hp -= 1;
                    obs.passed = true;
                    self.state = RunnerState::Hit;
                    self.action_timer = 0.3;
                    if self.hp <= 0 {
                        self.state = RunnerState::Dead;
                    }
                }
            }
        }

        for col in &mut self.collectibles {
            if col.collected || col.lane != player_lane {
                continue;
            }
            if col.position.abs() < 1.0 {
                col.collected = true;
                self.coins += col.value;
                self.score += col.value as u64 * 5;
                match col.collectible_type {
                    CollectibleType::Shield => self.invincible_timer = 3.0,
                    CollectibleType::PowerUp => self.speed = (self.speed * 1.2).min(self.max_speed),
                    _ => {}
                }
            }
        }
    }

    pub fn get_score(&self) -> u64 { self.score }
    pub fn get_distance(&self) -> f32 { self.distance }
    pub fn get_hp(&self) -> i32 { self.hp }
    pub fn get_coins(&self) -> u32 { self.coins }
    pub fn get_lane(&self) -> u8 { self.lane }
    pub fn get_state(&self) -> RunnerState { self.state }
    pub fn is_dead(&self) -> bool { self.hp <= 0 }
}

impl Atom for ParkourAtom {
    fn atom_id(&self) -> AtomId { "parkour".to_string() }
    fn atom_name(&self) -> &str { "跑酷" }

    fn on_init(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Initialized; }

    fn on_enter(&mut self, _ctx: &mut AtomContext) {
        self.state = RunnerState::Running;
        self.lane = self.num_lanes / 2;
        self.position = 0.0;
        self.speed = self.base_speed;
        self.distance = 0.0;
        self.score = 0;
        self.coins = 0;
        self.hp = self.max_hp;
        self.obstacles.clear();
        self.collectibles.clear();
        self.action_timer = 0.0;
        self.dash_timer = 0.0;
        self.invincible_timer = 0.0;
        self.spawn_distance = 0.0;
        self.difficulty = 0.3;
        self.phase = AtomPhase::Running;
    }

    fn on_update(&mut self, ctx: &mut AtomContext) {
        if self.is_dead() {
            self.phase = AtomPhase::Completed;
            return;
        }
        let dt = ctx.delta_time;
        self.update_movement(dt);
        self.update_timers(dt);
        self.spawn_content();
        self.check_collisions();
    }

    fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
    fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
    fn on_exit(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Completed; }
    fn on_destroy(&mut self) { self.phase = AtomPhase::Uninitialized; }

    fn save_state(&self) -> ValueMap {
        let mut map = ValueMap::new();
        map.insert("score".to_string(), Value::Int(self.score as i64));
        map.insert("distance".to_string(), Value::Float(self.distance as f64));
        map.insert("hp".to_string(), Value::Int(self.hp as i64));
        map.insert("coins".to_string(), Value::Int(self.coins as i64));
        map.insert("lane".to_string(), Value::Int(self.lane as i64));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Int(n)) = state.get("score") { self.score = *n as u64; }
        if let Some(Value::Float(n)) = state.get("distance") { self.distance = *n as f32; }
        if let Some(Value::Int(n)) = state.get("hp") { self.hp = *n as i32; }
        if let Some(Value::Int(n)) = state.get("coins") { self.coins = *n as u32; }
        if let Some(Value::Int(n)) = state.get("lane") { self.lane = *n as u8; }
    }

    fn handle_event(&mut self, event: &str, data: &ValueMap, _ctx: &mut AtomContext) {
        match event {
            "action" => {
                let action = data.get("type").and_then(|v| {
                    if let Value::String(s) = v {
                        match s.as_str() {
                            "jump" => Some(RunnerAction::Jump),
                            "slide" => Some(RunnerAction::Slide),
                            "dash" => Some(RunnerAction::Dash),
                            "left" => Some(RunnerAction::LaneLeft),
                            "right" => Some(RunnerAction::LaneRight),
                            _ => None,
                        }
                    } else { None }
                }).unwrap_or(RunnerAction::Jump);
                self.perform_action(action);
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
    fn test_parkour_init() {
        let mut atom = ParkourAtom::new(3, 5.0, 3);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);
        assert_eq!(atom.get_hp(), 3);
        assert_eq!(atom.get_lane(), 1);
    }

    #[test]
    fn test_parkour_actions() {
        let mut atom = ParkourAtom::new(3, 5.0, 3);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        atom.perform_action(RunnerAction::Jump);
        assert_eq!(atom.get_state(), RunnerState::Jumping);

        atom.state = RunnerState::Running;
        atom.perform_action(RunnerAction::LaneRight);
        assert_eq!(atom.get_lane(), 2);

        atom.perform_action(RunnerAction::LaneRight);
        assert_eq!(atom.get_lane(), 2);
    }

    #[test]
    fn test_parkour_update() {
        let mut atom = ParkourAtom::new(3, 5.0, 3);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        for _ in 0..100 {
            atom.on_update(&mut ctx);
        }
        assert!(atom.get_distance() > 0.0);
        assert!(atom.get_score() > 0);
    }

    #[test]
    fn test_parkour_save_load() {
        let mut atom = ParkourAtom::new(3, 5.0, 3);
        let mut ctx = make_ctx();
        atom.on_init(&mut ctx);
        atom.on_enter(&mut ctx);

        for _ in 0..50 {
            atom.on_update(&mut ctx);
        }

        let state = atom.save_state();
        let mut atom2 = ParkourAtom::new(3, 5.0, 3);
        atom2.load_state(&state);
        assert_eq!(atom2.get_score(), atom.get_score());
    }
}
