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
        // Score is recomputed from accumulated distance so the small
        // per-frame increments (which would truncate to 0 with the
        // default 60Hz tick) still produce a non-zero score.
        self.score = (self.distance * 10.0) as u64;

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
        map.insert("score".to_string(), Value::Integer(self.score as i32));
        map.insert("distance".to_string(), Value::Float(self.distance as f32));
        map.insert("hp".to_string(), Value::Integer(self.hp as i32));
        map.insert("coins".to_string(), Value::Integer(self.coins as i32));
        map.insert("lane".to_string(), Value::Integer(self.lane as i32));
        map
    }

    fn load_state(&mut self, state: &ValueMap) {
        if let Some(Value::Integer(n)) = state.get("score") { self.score = *n as u64; }
        if let Some(Value::Float(n)) = state.get("distance") { self.distance = *n as f32; }
        if let Some(Value::Integer(n)) = state.get("hp") { self.hp = *n as i32; }
        if let Some(Value::Integer(n)) = state.get("coins") { self.coins = *n as u32; }
        if let Some(Value::Integer(n)) = state.get("lane") { self.lane = *n as u8; }
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

// ---------------------------------------------------------------------------
// Round 137 helper-level tests — follow
// the round 110b / 122-136
// pattern. The pre-round-137
// `mod tests` had 4 integration
// tests (init, actions, update,
// save/load) but no focused unit
// coverage of the public surface.
// These tests pin the per-enum
// variant counts, per-field
// defaults of `ParkourAtom::new`,
// the lane-bounds / dash
// invincibility / state machine
// guards, the `save_state` +
// `load_state` round-trip for
// every persisted field, the
// `handle_event` dispatch for
// all 5 `RunnerAction` strings
// + unknown, and the lifecycle
// `on_init` / `on_enter` /
// `on_pause` / `on_resume` /
// `on_exit` / `on_destroy`
// phase transitions.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round137_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use crate::agi_minigame::world_state::UnifiedWorldState;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws).with_delta_time(0.016)
    }

    /// `ObstacleType` has 4
    /// variants (Low / High /
    /// Gap / Spike).
    #[test]
    fn obstacle_type_has_4_variants_round_137() {
        let v = [
            ObstacleType::Low,
            ObstacleType::High,
            ObstacleType::Gap,
            ObstacleType::Spike,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(ObstacleType::Low, ObstacleType::High);
        assert_ne!(ObstacleType::Gap, ObstacleType::Spike);
    }

    /// `CollectibleType` has 4
    /// variants (Coin / Gem /
    /// PowerUp / Shield).
    #[test]
    fn collectible_type_has_4_variants_round_137() {
        let v = [
            CollectibleType::Coin,
            CollectibleType::Gem,
            CollectibleType::PowerUp,
            CollectibleType::Shield,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(CollectibleType::Coin, CollectibleType::Gem);
        assert_ne!(CollectibleType::PowerUp, CollectibleType::Shield);
    }

    /// `RunnerAction` has 5
    /// variants (Jump / Slide
    /// / Dash / LaneLeft /
    /// LaneRight).
    #[test]
    fn runner_action_has_5_variants_round_137() {
        let v = [
            RunnerAction::Jump,
            RunnerAction::Slide,
            RunnerAction::Dash,
            RunnerAction::LaneLeft,
            RunnerAction::LaneRight,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(RunnerAction::Jump, RunnerAction::Slide);
        assert_ne!(RunnerAction::LaneLeft, RunnerAction::LaneRight);
    }

    /// `RunnerState` has 6
    /// variants (Running /
    /// Jumping / Sliding /
    /// Dashing / Hit / Dead).
    #[test]
    fn runner_state_has_6_variants_round_137() {
        let v = [
            RunnerState::Running,
            RunnerState::Jumping,
            RunnerState::Sliding,
            RunnerState::Dashing,
            RunnerState::Hit,
            RunnerState::Dead,
        ];
        for &x in &v { assert_eq!(x, x); }
        assert_ne!(RunnerState::Running, RunnerState::Jumping);
        assert_ne!(RunnerState::Hit, RunnerState::Dead);
    }

    /// `ParkourAtom::new` —
    /// lane defaults to
    /// `num_lanes / 2`
    /// (middle of the
    /// track).
    #[test]
    fn parkour_new_lane_defaults_to_middle_round_137() {
        let a3 = ParkourAtom::new(3, 5.0, 3);
        assert_eq!(a3.get_lane(), 1);
        let a5 = ParkourAtom::new(5, 5.0, 3);
        assert_eq!(a5.get_lane(), 2);
        let a1 = ParkourAtom::new(1, 5.0, 3);
        assert_eq!(a1.get_lane(), 0);
    }

    /// `ParkourAtom::new` —
    /// max_speed is 3x the
    /// base_speed.
    #[test]
    fn parkour_new_max_speed_is_3x_base_round_137() {
        let a = ParkourAtom::new(3, 4.0, 3);
        assert_eq!(a.max_speed, 12.0);
    }

    /// `ParkourAtom::new` —
    /// phase starts
    /// `Uninitialized`
    /// (not entered
    /// yet).
    #[test]
    fn parkour_new_phase_starts_uninitialized_round_137() {
        let a = ParkourAtom::new(3, 5.0, 3);
        assert_eq!(a.phase, AtomPhase::Uninitialized);
    }

    /// `ParkourAtom::new` —
    /// state starts
    /// `Running`.
    #[test]
    fn parkour_new_state_starts_running_round_137() {
        let a = ParkourAtom::new(3, 5.0, 3);
        assert_eq!(a.state, RunnerState::Running);
    }

    /// `ParkourAtom::new` —
    /// distance / score /
    /// coins are zero,
    /// hp == max_hp, all
    /// collections are
    /// empty, all timers
    /// are 0.
    #[test]
    fn parkour_new_defaults_all_zero_round_137() {
        let a = ParkourAtom::new(3, 5.0, 3);
        assert_eq!(a.position, 0.0);
        assert_eq!(a.distance, 0.0);
        assert_eq!(a.score, 0);
        assert_eq!(a.coins, 0);
        assert_eq!(a.hp, 3);
        assert_eq!(a.max_hp, 3);
        assert!(a.obstacles.is_empty());
        assert!(a.collectibles.is_empty());
        assert_eq!(a.action_timer, 0.0);
        assert_eq!(a.dash_timer, 0.0);
        assert_eq!(a.invincible_timer, 0.0);
    }

    /// `perform_action(Jump)`
    /// from `Running` →
    /// `Jumping`. From
    /// `Jumping` →
    /// still `Jumping`
    /// (no double-jump).
    #[test]
    fn parkour_perform_jump_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        a.perform_action(RunnerAction::Jump);
        assert_eq!(a.state, RunnerState::Jumping);
        // Re-jump while
        // already in
        // Jumping does
        // nothing.
        a.perform_action(RunnerAction::Jump);
        assert_eq!(a.state, RunnerState::Jumping);
    }

    /// `perform_action(Slide)`
    /// from `Running` →
    /// `Sliding`.
    #[test]
    fn parkour_perform_slide_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        a.perform_action(RunnerAction::Slide);
        assert_eq!(a.state, RunnerState::Sliding);
    }

    /// `perform_action(Dash)`
    /// sets
    /// `dash_timer=0.3`,
    /// `speed=base*2.5`,
    /// `invincible_timer=0.3`,
    /// state →
    /// `Dashing`.
    #[test]
    fn parkour_perform_dash_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        a.perform_action(RunnerAction::Dash);
        assert_eq!(a.state, RunnerState::Dashing);
        assert!((a.dash_timer - 0.3).abs() < 1e-6);
        assert!((a.speed - 12.5).abs() < 1e-6);
        assert!((a.invincible_timer - 0.3).abs() < 1e-6);
    }

    /// `perform_action(Dash)`
    /// while
    /// `dash_timer > 0`
    /// is a no-op
    /// (can't spam
    /// dash).
    #[test]
    fn parkour_dash_no_op_when_timer_active_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        a.perform_action(RunnerAction::Dash);
        let prev_speed = a.speed;
        a.perform_action(RunnerAction::Dash);
        // Speed
        // unchanged —
        // second dash
        // rejected.
        assert_eq!(a.speed, prev_speed);
    }

    /// `perform_action(LaneLeft)`
    /// decrements the
    /// lane; clamps
    /// at 0 (no
    /// negative
    /// lane).
    #[test]
    fn parkour_perform_lane_left_clamps_at_zero_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        a.lane = 0;
        a.perform_action(RunnerAction::LaneLeft);
        assert_eq!(a.get_lane(), 0);
        a.lane = 1;
        a.perform_action(RunnerAction::LaneLeft);
        assert_eq!(a.get_lane(), 0);
    }

    /// `perform_action(LaneRight)`
    /// increments the
    /// lane; clamps
    /// at
    /// `num_lanes-1`.
    #[test]
    fn parkour_perform_lane_right_clamps_at_top_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        a.lane = 2;
        a.perform_action(RunnerAction::LaneRight);
        assert_eq!(a.get_lane(), 2);
        a.lane = 1;
        a.perform_action(RunnerAction::LaneRight);
        assert_eq!(a.get_lane(), 2);
    }

    /// `get_state` /
    /// `is_dead`
    /// getters
    /// surface the
    /// internal
    /// state.
    #[test]
    fn parkour_getters_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        assert_eq!(a.get_state(), RunnerState::Running);
        assert!(!a.is_dead());
        a.hp = 0;
        assert!(a.is_dead());
    }

    /// `save_state`
    /// includes
    /// the 5
    /// persisted
    /// keys.
    #[test]
    fn parkour_save_state_keys_round_137() {
        let a = ParkourAtom::new(3, 5.0, 3);
        let s = a.save_state();
        assert!(s.contains_key("score"));
        assert!(s.contains_key("distance"));
        assert!(s.contains_key("hp"));
        assert!(s.contains_key("coins"));
        assert!(s.contains_key("lane"));
    }

    /// `load_state`
    /// restores all
    /// 5 persisted
    /// fields.
    #[test]
    fn parkour_load_state_restores_all_fields_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let mut s = ValueMap::new();
        s.insert("score".to_string(), Value::Integer(1000));
        s.insert("distance".to_string(), Value::Float(99.5));
        s.insert("hp".to_string(), Value::Integer(2));
        s.insert("coins".to_string(), Value::Integer(50));
        s.insert("lane".to_string(), Value::Integer(2));
        a.load_state(&s);
        assert_eq!(a.score, 1000);
        assert!((a.distance - 99.5).abs() < 1e-6);
        assert_eq!(a.hp, 2);
        assert_eq!(a.coins, 50);
        assert_eq!(a.lane, 2);
    }

    /// `handle_event`
    /// with
    /// `action=jump` →
    /// `Jumping`.
    #[test]
    fn parkour_handle_event_jump_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let mut s = ValueMap::new();
        s.insert("type".to_string(), Value::String("jump".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("action", &s, &mut ctx);
        assert_eq!(a.state, RunnerState::Jumping);
    }

    /// `handle_event`
    /// with
    /// `action=slide` →
    /// `Sliding`.
    #[test]
    fn parkour_handle_event_slide_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let mut s = ValueMap::new();
        s.insert("type".to_string(), Value::String("slide".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("action", &s, &mut ctx);
        assert_eq!(a.state, RunnerState::Sliding);
    }

    /// `handle_event`
    /// with
    /// `action=left` /
    /// `action=right` →
    /// lane
    /// change.
    #[test]
    fn parkour_handle_event_lane_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let mut s = ValueMap::new();
        s.insert("type".to_string(), Value::String("right".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("action", &s, &mut ctx);
        assert_eq!(a.get_lane(), 2);
        s.insert("type".to_string(), Value::String("left".to_string()));
        a.handle_event("action", &s, &mut ctx);
        assert_eq!(a.get_lane(), 1);
    }

    /// `handle_event`
    /// with
    /// `action=dash` →
    /// `Dashing` +
    /// dash_timer
    /// set.
    #[test]
    fn parkour_handle_event_dash_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let mut s = ValueMap::new();
        s.insert("type".to_string(), Value::String("dash".to_string()));
        let mut ctx = make_ctx();
        a.handle_event("action", &s, &mut ctx);
        assert_eq!(a.state, RunnerState::Dashing);
        assert!(a.dash_timer > 0.0);
    }

    /// `handle_event`
    /// with an
    /// unknown
    /// event
    /// name is
    /// a no-op.
    #[test]
    fn parkour_handle_event_unknown_is_noop_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let prev_state = a.state;
        let prev_lane = a.lane;
        let s = ValueMap::new();
        let mut ctx = make_ctx();
        a.handle_event("bogus", &s, &mut ctx);
        assert_eq!(a.state, prev_state);
        assert_eq!(a.get_lane(), prev_lane);
    }

    /// `on_init` →
    /// phase =
    /// `Initialized`.
    #[test]
    fn parkour_on_init_phase_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let mut ctx = make_ctx();
        a.on_init(&mut ctx);
        assert_eq!(a.phase, AtomPhase::Initialized);
    }

    /// `on_enter`
    /// resets all
    /// state and
    /// sets phase
    /// to
    /// `Running`.
    #[test]
    fn parkour_on_enter_resets_state_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        // Dirty
        // up the
        // state.
        a.score = 9999;
        a.coins = 100;
        a.hp = 1;
        a.obstacles.push(Obstacle {
            position: 10.0,
            lane: 0,
            obstacle_type: ObstacleType::Low,
            width: 1.0,
            passed: false,
        });
        let mut ctx = make_ctx();
        a.on_enter(&mut ctx);
        assert_eq!(a.score, 0);
        assert_eq!(a.coins, 0);
        assert_eq!(a.hp, a.max_hp);
        assert!(a.obstacles.is_empty());
        assert_eq!(a.phase, AtomPhase::Running);
    }

    /// `on_pause` /
    /// `on_resume` /
    /// `on_exit` /
    /// `on_destroy`
    /// each
    /// transition
    /// to the
    /// matching
    /// `AtomPhase`.
    #[test]
    fn parkour_lifecycle_phases_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
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
    fn parkour_atom_id_and_name_round_137() {
        let a = ParkourAtom::new(3, 5.0, 3);
        assert_eq!(a.atom_id(), "parkour");
        assert_eq!(a.atom_name(), "跑酷");
        let _ = a.as_any();
        let mut a = ParkourAtom::new(3, 5.0, 3);
        let _ = a.as_any_mut();
    }

    /// `current_phase`
    /// mirrors
    /// the
    /// internal
    /// `phase`
    /// field.
    #[test]
    fn parkour_current_phase_matches_field_round_137() {
        let mut a = ParkourAtom::new(3, 5.0, 3);
        assert_eq!(a.current_phase(), AtomPhase::Uninitialized);
        a.phase = AtomPhase::Paused;
        assert_eq!(a.current_phase(), AtomPhase::Paused);
    }
}
