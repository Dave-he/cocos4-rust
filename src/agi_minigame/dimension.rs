use std::sync::{Arc, Mutex};

use crate::base::value::{Value, ValueMap};

use super::atom::{Atom, AtomContext, AtomId, AtomFactory, AtomPhase, AtomRegistry, AtomRunner};
use super::ai_engine::DimensionBlueprint;
use super::world_state::UnifiedWorldState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionState {
    Uninitialized,
    Loading,
    Ready,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct DimensionConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub atom_ids: Vec<AtomId>,
    pub difficulty: f32,
    pub time_limit_secs: Option<u32>,
    pub rules: Vec<DimensionRule>,
    pub rewards: Vec<DimensionReward>,
    pub objectives: Vec<DimensionObjective>,
}

#[derive(Debug, Clone)]
pub struct DimensionRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub params: ValueMap,
}

#[derive(Debug, Clone)]
pub struct DimensionReward {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct DimensionObjective {
    pub id: String,
    pub description: String,
    pub target: u64,
    pub current: u64,
    pub is_optional: bool,
    pub is_completed: bool,
}

impl DimensionObjective {
    pub fn new(id: &str, description: &str, target: u64, is_optional: bool) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            target,
            current: 0,
            is_optional,
            is_completed: false,
        }
    }

    pub fn progress(&mut self, amount: u64) -> bool {
        self.current = (self.current + amount).min(self.target);
        if self.current >= self.target && !self.is_completed {
            self.is_completed = true;
            return true;
        }
        false
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.target == 0 {
            return 1.0;
        }
        self.current as f32 / self.target as f32
    }
}

impl DimensionConfig {
    pub fn from_blueprint(blueprint: &DimensionBlueprint) -> Self {
        Self {
            id: blueprint.id.clone(),
            name: blueprint.name.clone(),
            description: blueprint.description.clone(),
            atom_ids: blueprint.atom_ids.clone(),
            difficulty: blueprint.difficulty,
            time_limit_secs: blueprint.time_limit_secs,
            rules: blueprint
                .rules
                .iter()
                .map(|r| DimensionRule {
                    rule_id: r.rule_id.clone(),
                    name: r.name.clone(),
                    description: r.description.clone(),
                    is_active: true,
                    params: r.params.clone(),
                })
                .collect(),
            rewards: blueprint
                .rewards
                .iter()
                .map(|r| DimensionReward {
                    item_id: r.item_id.clone(),
                    quantity: r.base_quantity,
                })
                .collect(),
            objectives: blueprint
                .objectives
                .iter()
                .map(|o| DimensionObjective::new(&o.id, &o.description, o.target_value, o.is_optional))
                .collect(),
        }
    }
}

pub struct Dimension {
    pub config: DimensionConfig,
    pub state: DimensionState,
    pub atom_runners: Vec<AtomRunner>,
    pub elapsed_time: f32,
    pub score: u64,
    pub completed_objectives: Vec<String>,
    pub active_rules: Vec<DimensionRule>,
    pub event_log: Vec<DimensionEvent>,
}

#[derive(Debug, Clone)]
pub struct DimensionEvent {
    pub timestamp: f32,
    pub event_type: String,
    pub data: ValueMap,
}

impl Dimension {
    pub fn new(config: DimensionConfig) -> Self {
        Self {
            config,
            state: DimensionState::Uninitialized,
            atom_runners: Vec::new(),
            elapsed_time: 0.0,
            score: 0,
            completed_objectives: Vec::new(),
            active_rules: Vec::new(),
            event_log: Vec::new(),
        }
    }

    pub fn load(&mut self, registry: &AtomRegistry) -> bool {
        self.state = DimensionState::Loading;

        for atom_id in &self.config.atom_ids {
            if let Some(atom) = registry.create(atom_id) {
                self.atom_runners.push(AtomRunner::new(atom));
            } else {
                self.state = DimensionState::Failed;
                return false;
            }
        }

        self.active_rules = self.config.rules.clone();
        self.state = DimensionState::Ready;
        true
    }

    pub fn start(&mut self, ctx: &mut AtomContext) {
        if self.state != DimensionState::Ready {
            return;
        }

        for runner in &mut self.atom_runners {
            runner.init(ctx);
            runner.enter(ctx);
        }

        self.state = DimensionState::Running;
        self.elapsed_time = 0.0;

        self.log_event("dimension_start", ValueMap::new());
    }

    pub fn update(&mut self, dt: f32, ctx: &mut AtomContext) {
        if self.state != DimensionState::Running {
            return;
        }

        self.elapsed_time += dt;

        if let Some(time_limit) = self.config.time_limit_secs {
            if self.elapsed_time >= time_limit as f32 {
                self.complete(ctx);
                return;
            }
        }

        for runner in &mut self.atom_runners {
            runner.update(ctx);
        }

        self.apply_rules(dt);

        if self.check_all_mandatory_objectives() {
            self.complete(ctx);
        }
    }

    pub fn pause(&mut self, ctx: &mut AtomContext) {
        if self.state != DimensionState::Running {
            return;
        }
        for runner in &mut self.atom_runners {
            runner.pause(ctx);
        }
        self.state = DimensionState::Paused;
        self.log_event("dimension_pause", ValueMap::new());
    }

    pub fn resume(&mut self, ctx: &mut AtomContext) {
        if self.state != DimensionState::Paused {
            return;
        }
        for runner in &mut self.atom_runners {
            runner.resume(ctx);
        }
        self.state = DimensionState::Running;
        self.log_event("dimension_resume", ValueMap::new());
    }

    pub fn complete(&mut self, ctx: &mut AtomContext) {
        for runner in &mut self.atom_runners {
            runner.exit(ctx);
        }
        self.state = DimensionState::Completed;

        let mut data = ValueMap::new();
        data.insert("score".to_string(), Value::Integer(self.score as i32));
        data.insert("time".to_string(), Value::Float(self.elapsed_time as f32));
        self.log_event("dimension_complete", data);
    }

    pub fn fail(&mut self, ctx: &mut AtomContext, reason: &str) {
        for runner in &mut self.atom_runners {
            runner.exit(ctx);
        }
        self.state = DimensionState::Failed;

        let mut data = ValueMap::new();
        data.insert("reason".to_string(), Value::String(reason.to_string()));
        self.log_event("dimension_fail", data);
    }

    pub fn add_score(&mut self, amount: u64) {
        self.score += amount;
    }

    pub fn progress_objective(&mut self, objective_id: &str, amount: u64) -> bool {
        if let Some(obj) = self.config.objectives.iter_mut().find(|o| o.id == objective_id) {
            let just_completed = obj.progress(amount);
            if just_completed {
                self.completed_objectives.push(objective_id.to_string());
            }
            just_completed
        } else {
            false
        }
    }

    pub fn get_objective(&self, objective_id: &str) -> Option<&DimensionObjective> {
        self.config.objectives.iter().find(|o| o.id == objective_id)
    }

    pub fn broadcast_event(&mut self, event: &str, data: &ValueMap, ctx: &mut AtomContext) {
        for runner in &mut self.atom_runners {
            runner.handle_event(event, data, ctx);
        }
        self.log_event(event, data.clone());
    }

    fn apply_rules(&mut self, _dt: f32) {
        for rule in &self.active_rules {
            if !rule.is_active {
                continue;
            }
        }
    }

    fn check_all_mandatory_objectives(&self) -> bool {
        let mandatory: Vec<&DimensionObjective> = self
            .config
            .objectives
            .iter()
            .filter(|o| !o.is_optional)
            .collect();
        // A dimension with no objectives never auto-completes — only the
        // runner's `complete()` call or a time-limit expiry ends it.
        if mandatory.is_empty() {
            return false;
        }
        mandatory.iter().all(|o| o.is_completed)
    }

    fn log_event(&mut self, event_type: &str, data: ValueMap) {
        self.event_log.push(DimensionEvent {
            timestamp: self.elapsed_time,
            event_type: event_type.to_string(),
            data,
        });
    }

    pub fn get_progress(&self) -> DimensionProgress {
        let total_objectives = self.config.objectives.len();
        let completed = self.completed_objectives.len();
        let mandatory_total = self.config.objectives.iter().filter(|o| !o.is_optional).count();
        let mandatory_done = self.config.objectives.iter().filter(|o| !o.is_optional && o.is_completed).count();

        DimensionProgress {
            state: self.state,
            elapsed_time: self.elapsed_time,
            score: self.score,
            total_objectives,
            completed_objectives: completed,
            mandatory_progress: if mandatory_total > 0 {
                mandatory_done as f32 / mandatory_total as f32
            } else {
                1.0
            },
            time_remaining: self.config.time_limit_secs.map(|t| t as f32 - self.elapsed_time),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DimensionProgress {
    pub state: DimensionState,
    pub elapsed_time: f32,
    pub score: u64,
    pub total_objectives: usize,
    pub completed_objectives: usize,
    pub mandatory_progress: f32,
    pub time_remaining: Option<f32>,
}

pub struct DimensionRunner {
    world_state: Arc<Mutex<UnifiedWorldState>>,
    registry: Arc<Mutex<AtomRegistry>>,
    active_dimension: Option<Dimension>,
}

impl DimensionRunner {
    pub fn new(
        world_state: Arc<Mutex<UnifiedWorldState>>,
        registry: Arc<Mutex<AtomRegistry>>,
    ) -> Self {
        Self {
            world_state,
            registry,
            active_dimension: None,
        }
    }

    pub fn start_dimension(&mut self, config: DimensionConfig) -> bool {
        let registry = self.registry.lock().unwrap();
        let mut dimension = Dimension::new(config);

        if !dimension.load(&registry) {
            return false;
        }

        let ctx = self.make_ctx();
        drop(registry);

        let mut dimension_ref = dimension;
        let mut ctx_mut = ctx;
        dimension_ref.start(&mut ctx_mut);

        self.active_dimension = Some(dimension_ref);
        true
    }

    pub fn update(&mut self, dt: f32) {
        let mut ctx = self.make_ctx();
        if let Some(ref mut dim) = self.active_dimension {
            dim.update(dt, &mut ctx);
        }
    }

    pub fn pause(&mut self) {
        let mut ctx = self.make_ctx();
        if let Some(ref mut dim) = self.active_dimension {
            dim.pause(&mut ctx);
        }
    }

    pub fn resume(&mut self) {
        let mut ctx = self.make_ctx();
        if let Some(ref mut dim) = self.active_dimension {
            dim.resume(&mut ctx);
        }
    }

    pub fn get_progress(&self) -> Option<DimensionProgress> {
        self.active_dimension.as_ref().map(|d| d.get_progress())
    }

    pub fn get_active_dimension(&self) -> Option<&Dimension> {
        self.active_dimension.as_ref()
    }

    pub fn get_active_dimension_mut(&mut self) -> Option<&mut Dimension> {
        self.active_dimension.as_mut()
    }

    pub fn is_running(&self) -> bool {
        self.active_dimension
            .as_ref()
            .map(|d| d.state == DimensionState::Running)
            .unwrap_or(false)
    }

    fn make_ctx(&self) -> AtomContext {
        AtomContext::new(Arc::clone(&self.world_state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::agi_minigame::atom::{Atom, AtomMetadata, AtomPhase};
    use crate::agi_minigame::player::PlayerProfile;

    struct MockAtom {
        id: String,
        name: String,
        phase: AtomPhase,
        update_count: u32,
    }

    impl MockAtom {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                name: format!("Mock_{}", id),
                phase: AtomPhase::Uninitialized,
                update_count: 0,
            }
        }
    }

    impl Atom for MockAtom {
        fn atom_id(&self) -> AtomId { self.id.clone() }
        fn atom_name(&self) -> &str { &self.name }
        fn on_init(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Initialized; }
        fn on_enter(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
        fn on_update(&mut self, _ctx: &mut AtomContext) {
            self.update_count += 1;
        }
        fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
        fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
        fn on_exit(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Completed; }
        fn on_destroy(&mut self) { self.phase = AtomPhase::Uninitialized; }
        fn save_state(&self) -> ValueMap {
            let mut m = ValueMap::new();
            m.insert("update_count".to_string(), Value::Integer(self.update_count as i32));
            m
        }
        fn load_state(&mut self, state: &ValueMap) {
            if let Some(Value::Integer(n)) = state.get("update_count") {
                self.update_count = *n as u32;
            }
        }
        fn handle_event(&mut self, _event: &str, _data: &ValueMap, _ctx: &mut AtomContext) {}
        fn current_phase(&self) -> AtomPhase { self.phase }
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    }

    fn make_registry() -> AtomRegistry {
        let mut registry = AtomRegistry::new();
        let atoms = vec![
            ("match3", "Match3", "puzzle"),
            ("tower_defense", "TowerDefense", "strategy"),
            ("card", "Card", "card"),
        ];
        for (id, name, gt) in atoms {
            let id_owned = id.to_string();
            let factory: AtomFactory = Box::new(move || Box::new(MockAtom::new(&id_owned)));
            registry.register(
                id.to_string(),
                AtomMetadata {
                    id: id.to_string(),
                    name: name.to_string(),
                    version: 1,
                    gameplay_type: gt.to_string(),
                    description: format!("{} atom", name),
                    tags: vec![gt.to_string()],
                },
                factory,
            );
        }
        registry
    }

    fn make_ws() -> Arc<Mutex<UnifiedWorldState>> {
        Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))))
    }

    #[test]
    fn test_dimension_lifecycle() {
        let registry = make_registry();
        let ws = make_ws();

        let config = DimensionConfig {
            id: "test_dim".to_string(),
            name: "Test Dimension".to_string(),
            description: "Test".to_string(),
            atom_ids: vec!["match3".to_string(), "card".to_string()],
            difficulty: 0.5,
            time_limit_secs: None,
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives: vec![
                DimensionObjective::new("obj1", "Score 100", 100, false),
            ],
        };

        let mut dim = Dimension::new(config);
        assert_eq!(dim.state, DimensionState::Uninitialized);

        assert!(dim.load(&registry));
        assert_eq!(dim.state, DimensionState::Ready);

        let mut ctx = AtomContext::new(Arc::clone(&ws));
        dim.start(&mut ctx);
        assert_eq!(dim.state, DimensionState::Running);

        dim.update(0.016, &mut ctx);
        assert_eq!(dim.state, DimensionState::Running);

        dim.pause(&mut ctx);
        assert_eq!(dim.state, DimensionState::Paused);

        dim.resume(&mut ctx);
        assert_eq!(dim.state, DimensionState::Running);

        dim.complete(&mut ctx);
        assert_eq!(dim.state, DimensionState::Completed);
    }

    #[test]
    fn test_dimension_objectives() {
        let mut obj = DimensionObjective::new("obj1", "Score 100", 100, false);
        assert!(!obj.is_completed);
        assert_eq!(obj.progress_ratio(), 0.0);

        let just_completed = obj.progress(50);
        assert!(!just_completed);
        assert!(!obj.is_completed);

        let just_completed = obj.progress(50);
        assert!(just_completed);
        assert!(obj.is_completed);
        assert_eq!(obj.progress_ratio(), 1.0);
    }

    #[test]
    fn test_dimension_time_limit() {
        let registry = make_registry();
        let ws = make_ws();

        let config = DimensionConfig {
            id: "timed".to_string(),
            name: "Timed".to_string(),
            description: "Test".to_string(),
            atom_ids: vec!["match3".to_string()],
            difficulty: 0.5,
            time_limit_secs: Some(5),
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives: Vec::new(),
        };

        let mut dim = Dimension::new(config);
        dim.load(&registry);

        let mut ctx = AtomContext::new(Arc::clone(&ws));
        dim.start(&mut ctx);

        for _ in 0..400 {
            dim.update(0.016, &mut ctx);
        }

        assert_eq!(dim.state, DimensionState::Completed);
    }

    #[test]
    fn test_dimension_score() {
        let registry = make_registry();
        let ws = make_ws();

        let config = DimensionConfig {
            id: "scored".to_string(),
            name: "Scored".to_string(),
            description: "Test".to_string(),
            atom_ids: vec!["match3".to_string()],
            difficulty: 0.5,
            time_limit_secs: None,
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives: Vec::new(),
        };

        let mut dim = Dimension::new(config);
        dim.load(&registry);

        let mut ctx = AtomContext::new(Arc::clone(&ws));
        dim.start(&mut ctx);

        dim.add_score(100);
        dim.add_score(50);
        assert_eq!(dim.score, 150);
    }

    #[test]
    fn test_dimension_progress() {
        let registry = make_registry();
        let ws = make_ws();

        let config = DimensionConfig {
            id: "prog".to_string(),
            name: "Progress".to_string(),
            description: "Test".to_string(),
            atom_ids: vec!["match3".to_string()],
            difficulty: 0.5,
            time_limit_secs: Some(60),
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives: vec![
                DimensionObjective::new("obj1", "Score 100", 100, false),
                DimensionObjective::new("obj2", "Collect 10", 10, true),
            ],
        };

        let mut dim = Dimension::new(config);
        dim.load(&registry);

        let mut ctx = AtomContext::new(Arc::clone(&ws));
        dim.start(&mut ctx);

        dim.progress_objective("obj1", 50);
        let progress = dim.get_progress();
        assert_eq!(progress.completed_objectives, 0);
        assert!((progress.mandatory_progress - 0.0).abs() < 0.01);

        dim.progress_objective("obj1", 50);
        let progress = dim.get_progress();
        assert_eq!(progress.completed_objectives, 1);
        assert!((progress.mandatory_progress - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_dimension_runner() {
        let registry = Arc::new(Mutex::new(make_registry()));
        let ws = make_ws();

        let mut runner = DimensionRunner::new(Arc::clone(&ws), Arc::clone(&registry));

        let config = DimensionConfig {
            id: "runner_test".to_string(),
            name: "Runner Test".to_string(),
            description: "Test".to_string(),
            atom_ids: vec!["match3".to_string()],
            difficulty: 0.5,
            time_limit_secs: None,
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives: Vec::new(),
        };

        assert!(runner.start_dimension(config));
        assert!(runner.is_running());

        runner.update(0.016);
        runner.pause();
        assert!(!runner.is_running());
        runner.resume();
        assert!(runner.is_running());

        let progress = runner.get_progress();
        assert!(progress.is_some());
    }

    #[test]
    fn test_dimension_from_blueprint() {
        use super::super::ai_engine::{DimensionBlueprint, DimensionTheme, GeneratedReward, GeneratedRule, Objective, ObjectiveType, RuleType};

        let blueprint = DimensionBlueprint {
            id: "bp1".to_string(),
            name: "Test BP".to_string(),
            description: "From blueprint".to_string(),
            atom_ids: vec!["match3".to_string()],
            atom_weights: HashMap::new(),
            difficulty: 0.7,
            rules: vec![GeneratedRule {
                rule_id: "speed".to_string(),
                name: "Speed".to_string(),
                description: "Faster".to_string(),
                rule_type: RuleType::Modifier,
                params: ValueMap::new(),
            }],
            rewards: vec![GeneratedReward {
                item_id: "gold".to_string(),
                base_quantity: 100,
                scaling_factor: 1.0,
            }],
            theme: DimensionTheme {
                name: "Test Theme".to_string(),
                visual_style: "cyber".to_string(),
                music_mood: "epic".to_string(),
                color_palette: vec![],
            },
            time_limit_secs: Some(120),
            objectives: vec![Objective {
                id: "score".to_string(),
                description: "Get 500".to_string(),
                objective_type: ObjectiveType::Score,
                target_value: 500,
                is_optional: false,
            }],
        };

        let config = DimensionConfig::from_blueprint(&blueprint);
        assert_eq!(config.id, "bp1");
        assert_eq!(config.atom_ids.len(), 1);
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rewards.len(), 1);
        assert_eq!(config.objectives.len(), 1);
    }
}

// ---------------------------------------------------------------------------
// Round 16 — additional lifecycle / state-transition tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round16_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::agi_minigame::atom::AtomContext;
    use crate::agi_minigame::atoms;
    use crate::agi_minigame::player::PlayerProfile;
    use crate::agi_minigame::world_state::UnifiedWorldState;

    fn make_ctx() -> AtomContext {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        AtomContext::new(ws).with_delta_time(0.016)
    }

    fn make_config() -> DimensionConfig {
        DimensionConfig {
            id: "round16".to_string(),
            name: "Round 16 Test".to_string(),
            description: "tests".to_string(),
            atom_ids: vec!["match3".to_string()],
            difficulty: 0.5,
            time_limit_secs: Some(120),
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives: Vec::new(),
        }
    }

    fn make_registry() -> AtomRegistry {
        let mut reg = AtomRegistry::new();
        atoms::register_all_atoms(&mut reg);
        reg
    }

    #[test]
    fn new_dimension_starts_in_uninitialized_state() {
        let d = Dimension::new(make_config());
        // state is private; verify via get_progress / is_running / is_completed
        assert!(!(d.get_progress().state == DimensionState::Running));
        assert!(!(d.get_progress().state == DimensionState::Completed));
    }

    #[test]
    fn load_then_start_transitions_to_running() {
        let mut d = Dimension::new(make_config());
        let reg = make_registry();
        assert!(d.load(&reg));
        let mut ctx = make_ctx();
        d.start(&mut ctx);
        assert!((d.get_progress().state == DimensionState::Running));
    }

    #[test]
    fn load_with_unknown_atom_fails() {
        let mut bad = make_config();
        bad.atom_ids = vec!["not.an.atom".to_string()];
        let mut d = Dimension::new(bad);
        let reg = make_registry();
        assert!(!d.load(&reg));
        // Failed state — is_completed remains false
        assert!(!(d.get_progress().state == DimensionState::Completed));
    }

    #[test]
    fn update_accumulates_elapsed_time() {
        let mut d = Dimension::new(make_config());
        let reg = make_registry();
        d.load(&reg);
        let mut ctx = make_ctx();
        d.start(&mut ctx);
        d.update(0.5, &mut ctx);
        d.update(0.5, &mut ctx);
        // 1.0s of simulated time, but time_limit = 120 so still running
        assert!((d.get_progress().state == DimensionState::Running));
    }

    #[test]
    fn pause_resume_round_trip() {
        let mut d = Dimension::new(make_config());
        let reg = make_registry();
        d.load(&reg);
        let mut ctx = make_ctx();
        d.start(&mut ctx);
        d.pause(&mut ctx);
        // While paused, update is a no-op
        d.update(5.0, &mut ctx);
        // Resume
        d.resume(&mut ctx);
        assert!((d.get_progress().state == DimensionState::Running));
    }

    #[test]
    fn complete_transitions_to_completed() {
        let mut d = Dimension::new(make_config());
        let reg = make_registry();
        d.load(&reg);
        let mut ctx = make_ctx();
        d.start(&mut ctx);
        d.complete(&mut ctx);
        assert!((d.get_progress().state == DimensionState::Completed));
        // After completion, update is a no-op
        d.update(1.0, &mut ctx);
        assert!((d.get_progress().state == DimensionState::Completed));
    }

    #[test]
    fn fail_transitions_to_failed() {
        let mut d = Dimension::new(make_config());
        let reg = make_registry();
        d.load(&reg);
        let mut ctx = make_ctx();
        d.start(&mut ctx);
        d.fail(&mut ctx, "out of mana");
        // After failure, is_completed is false, is_running is false
        assert!(!(d.get_progress().state == DimensionState::Running));
        assert!(!(d.get_progress().state == DimensionState::Completed));
    }
}

// ---------------------------------------------------------------------------
// Round 17 — DimensionRunner end-to-end: load → update → state asserts.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round17_tests {
    use super::*;
    use std::sync::Arc;
    use crate::agi_minigame::atoms;
    use crate::agi_minigame::player::PlayerProfile;
    use crate::agi_minigame::world_state::UnifiedWorldState;

    fn make_registry() -> AtomRegistry {
        let mut reg = AtomRegistry::new();
        atoms::register_all_atoms(&mut reg);
        reg
    }

    fn make_config(id: &str, atom_ids: Vec<String>) -> DimensionConfig {
        DimensionConfig {
            id: id.to_string(),
            name: id.to_string(),
            description: "round 17".to_string(),
            atom_ids,
            difficulty: 0.5,
            time_limit_secs: Some(60),
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives: Vec::new(),
        }
    }

    #[test]
    fn runner_loads_a_dimension_with_known_atoms() {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("runner"))));
        let reg = make_registry();
        let mut runner = DimensionRunner::new(ws, Arc::new(Mutex::new(reg)));
        let cfg = make_config("round17_dim", vec!["match3".into(), "tower_defense".into()]);
        assert!(runner.start_dimension(cfg));
        let dim = runner.get_active_dimension().expect("active dim");
        assert_eq!(dim.config.id, "round17_dim");
        assert_eq!(dim.config.atom_ids.len(), 2);
    }

    #[test]
    fn runner_drives_a_dimension_through_updates() {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("runner"))));
        let reg = make_registry();
        let mut runner = DimensionRunner::new(ws, Arc::new(Mutex::new(reg)));
        let cfg = make_config("round17_life", vec!["parkour".into()]);
        assert!(runner.start_dimension(cfg));
        // The runner's update takes only dt (it builds its own ctx).
        runner.update(0.016);
        runner.update(0.016);
        runner.update(0.016);
        let dim = runner.get_active_dimension().unwrap();
        assert!(dim.elapsed_time > 0.0);
    }

    #[test]
    fn runner_rejects_unknown_atom_ids() {
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("runner"))));
        let reg = make_registry();
        let mut runner = DimensionRunner::new(ws, Arc::new(Mutex::new(reg)));
        let cfg = make_config("bad", vec!["definitely.not.an.atom".into()]);
        assert!(!runner.start_dimension(cfg));
        // No active dimension after a failed start.
        assert!(runner.get_active_dimension().is_none());
    }
}

// ---------------------------------------------------------------------------
// Round 130 — dimension.rs helper-level unit tests.
// Mirrors the round-110b / 122 / 123 / 124 / 125 / 126 / 127 / 128 / 129
// pattern: pin the small public helpers' contracts
// (`DimensionObjective::new/progress/progress_ratio`,
// `DimensionState` PartialEq) so a refactor can't silently
// change the objective / state-machine semantics that
// the runner relies on for completion signalling.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round130_tests {
    use super::*;

    // -----------------------------------------------------------------
    // DimensionState PartialEq round-trip across all 7 variants.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_state_partial_eq_for_all_7_variants() {
        use DimensionState::*;
        assert_eq!(Uninitialized, Uninitialized);
        assert_eq!(Loading,      Loading);
        assert_eq!(Ready,        Ready);
        assert_eq!(Running,      Running);
        assert_eq!(Paused,       Paused);
        assert_eq!(Completed,    Completed);
        assert_eq!(Failed,       Failed);
        assert_ne!(Uninitialized, Loading);
        assert_ne!(Running,      Paused);
        assert_ne!(Completed,    Failed);
    }

    // -----------------------------------------------------------------
    // DimensionObjective::new — initial state pinning.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_objective_new_initial_state() {
        // `new` should set current=0, is_completed=false,
        // and copy through id/description/target/is_optional.
        // A regression that pre-filled current=target
        // would silently mark every objective as completed.
        let obj = DimensionObjective::new("kill_10", "Kill 10 enemies", 10, false);
        assert_eq!(obj.id, "kill_10");
        assert_eq!(obj.description, "Kill 10 enemies");
        assert_eq!(obj.target, 10);
        assert_eq!(obj.current, 0);
        assert!(!obj.is_completed);
        assert!(!obj.is_optional);
    }

    #[test]
    fn dimension_objective_new_optional_flag_passes_through() {
        // Optional objective flag must round-trip.
        let obj = DimensionObjective::new("bonus", "Bonus objective", 5, true);
        assert!(obj.is_optional);
        assert!(!obj.is_completed);
    }

    // -----------------------------------------------------------------
    // DimensionObjective::progress — completion detection.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_objective_progress_returns_false_below_target() {
        // A progress call that doesn't reach
        // the target returns false and leaves
        // is_completed=false.
        let mut obj = DimensionObjective::new("o", "o", 10, false);
        assert!(!obj.progress(3));
        assert_eq!(obj.current, 3);
        assert!(!obj.is_completed);
        assert!(!obj.progress(5));
        assert_eq!(obj.current, 8);
        assert!(!obj.is_completed);
    }

    #[test]
    fn dimension_objective_progress_returns_true_on_first_completion() {
        // The first progress call that reaches
        // the target returns true and sets
        // is_completed=true. This is the
        // primary signal the runner uses to
        // decide when an objective is done.
        let mut obj = DimensionObjective::new("o", "o", 10, false);
        assert!(obj.progress(10));
        assert_eq!(obj.current, 10);
        assert!(obj.is_completed);
    }

    #[test]
    fn dimension_objective_progress_clamps_overshoot_to_target() {
        // An overshooting progress call
        // (amount > remaining) clamps to the
        // target. This is documented in the
        // source as `(self.current + amount)
        // .min(self.target)`.
        let mut obj = DimensionObjective::new("o", "o", 10, false);
        assert!(obj.progress(15));
        assert_eq!(obj.current, 10);
        assert!(obj.is_completed);
    }

    #[test]
    fn dimension_objective_progress_returns_false_on_subsequent_calls_after_complete() {
        // Once completed, additional progress
        // calls do NOT keep returning true.
        // The contract: only the FIRST call
        // that reaches the target returns
        // true. Pin this so a refactor can't
        // accidentally re-emit completion
        // signals (which would double-count
        // rewards).
        let mut obj = DimensionObjective::new("o", "o", 10, false);
        assert!(obj.progress(10));
        assert!(!obj.progress(1));
        assert!(!obj.progress(100));
        assert_eq!(obj.current, 10);
        assert!(obj.is_completed);
    }

    #[test]
    fn dimension_objective_progress_stale_call_after_completion_keeps_clamping() {
        // A progress(0) after completion is
        // a no-op for state but should not
        // panic and not return true.
        let mut obj = DimensionObjective::new("o", "o", 10, false);
        assert!(obj.progress(10));
        assert!(!obj.progress(0));
        assert_eq!(obj.current, 10);
        assert!(obj.is_completed);
    }

    // -----------------------------------------------------------------
    // DimensionObjective::progress_ratio — fractional completion.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_objective_progress_ratio_zero_for_fresh_objective() {
        // A fresh objective (current=0) has
        // progress_ratio == 0.0.
        let obj = DimensionObjective::new("o", "o", 10, false);
        assert_eq!(obj.progress_ratio(), 0.0);
    }

    #[test]
    fn dimension_objective_progress_ratio_for_partial_completion() {
        // 5/10 → 0.5.
        let mut obj = DimensionObjective::new("o", "o", 10, false);
        obj.progress(5);
        assert!((obj.progress_ratio() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn dimension_objective_progress_ratio_for_completed_objective_is_one() {
        // After completion (current == target),
        // progress_ratio should be exactly 1.0.
        // This is what the UI uses to draw
        // the completion bar.
        let mut obj = DimensionObjective::new("o", "o", 10, false);
        obj.progress(10);
        assert!((obj.progress_ratio() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn dimension_objective_progress_ratio_for_zero_target_is_one() {
        // Documented edge case: a target=0
        // objective is treated as "already
        // complete" by progress_ratio (returns
        // 1.0) to avoid division by zero. Pin
        // this so a future change can't
        // accidentally produce NaN for
        // zero-target objectives (the AI
        // engine emits these as auto-pass
        // conditions).
        let mut obj = DimensionObjective::new("auto", "auto-pass", 0, true);
        assert_eq!(obj.progress_ratio(), 1.0);
        // The FIRST progress() call (even with
        // amount=0) flips is_completed=true
        // because `0 >= 0` is true. This is
        // a side-effect of the target=0
        // design: it auto-completes on first
        // tick. Subsequent calls return
        // false (the once-only signal).
        assert!(!obj.is_completed);
        assert!(obj.progress(0));
        assert!(obj.is_completed);
        assert!(!obj.progress(0));
    }
}

// ---------------------------------------------------------------------------
// Round 153 helper-level tests for `dimension.rs`.
//
// Round 153 closes surface-area gaps left after
// the round-130 / round-132 sweep — specifically
// the DimensionState lifecycle guards, the
// objective-lookup paths, and the
// DimensionRunner no-active-dim paths.
//
// Every test is fully self-contained: each builds
// its own `Dimension` (or `DimensionRunner`) via
// the local `make_dim` / `make_runner` helpers
// below, so a regression in one fixture doesn't
// poison the others.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round153_tests {
    use super::*;
    use std::sync::Arc;
    use crate::agi_minigame::atoms;
    use crate::agi_minigame::player::PlayerProfile;
    use crate::agi_minigame::world_state::UnifiedWorldState;

    fn make_registry() -> AtomRegistry {
        let mut reg = AtomRegistry::new();
        atoms::register_all_atoms(&mut reg);
        reg
    }

    fn make_world() -> Arc<Mutex<UnifiedWorldState>> {
        Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("round153"))))
    }

    fn make_runner() -> DimensionRunner {
        let ws = make_world();
        let reg = make_registry();
        DimensionRunner::new(ws, Arc::new(Mutex::new(reg)))
    }

    fn make_dim_with_objectives(
        id: &str,
        objective_specs: Vec<(&str, u64, bool)>,
    ) -> Dimension {
        let objectives = objective_specs
            .into_iter()
            .map(|(oid, target, is_optional)| {
                DimensionObjective::new(oid, oid, target, is_optional)
            })
            .collect();
        let cfg = DimensionConfig {
            id: id.to_string(),
            name: id.to_string(),
            description: "round153".to_string(),
            atom_ids: Vec::new(),
            difficulty: 0.5,
            time_limit_secs: Some(60),
            rules: Vec::new(),
            rewards: Vec::new(),
            objectives,
        };
        Dimension::new(cfg)
    }

    fn make_ctx() -> AtomContext {
        AtomContext::new(make_world())
    }

    // -----------------------------------------------------------------
    // DimensionState lifecycle guards.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_start_is_no_op_when_not_ready_round153() {
        // `start` guards on `state == Ready`. A
        // fresh Dimension is `Uninitialized`, so
        // start() must not flip state and must
        // not crash. The guard prevents a
        // double-start path that would otherwise
        // produce an extra `dimension_complete`
        // event or duplicate atom_runner init.
        let mut dim = make_dim_with_objectives("start_guard", vec![]);
        assert_eq!(dim.state, DimensionState::Uninitialized);
        let mut ctx = make_ctx();
        dim.start(&mut ctx);
        assert_eq!(
            dim.state,
            DimensionState::Uninitialized,
            "start() must not promote Uninitialized → anything else"
        );
        assert!(dim.event_log.is_empty());
    }

    #[test]
    fn dimension_pause_is_no_op_when_not_running_round153() {
        // Symmetric guard: pause() must only
        // act when state == Running. A
        // Ready→pause call (or Running→pause
        // twice) must not flip state and must
        // not push a `dimension_pause` event.
        let mut dim = make_dim_with_objectives("pause_guard", vec![]);
        assert_eq!(dim.state, DimensionState::Uninitialized);
        let mut ctx = make_ctx();
        dim.pause(&mut ctx);
        assert_eq!(
            dim.state,
            DimensionState::Uninitialized,
            "pause() on Uninitialized must stay Uninitialized"
        );
        assert!(dim.event_log.is_empty());
    }

    #[test]
    fn dimension_resume_is_no_op_when_not_paused_round153() {
        // Symmetric guard: resume() must only
        // act when state == Paused. A fresh
        // dimension is `Uninitialized`, so
        // resume() must be a no-op.
        let mut dim = make_dim_with_objectives("resume_guard", vec![]);
        let mut ctx = make_ctx();
        dim.resume(&mut ctx);
        assert_eq!(dim.state, DimensionState::Uninitialized);
        assert!(dim.event_log.is_empty());
    }

    // -----------------------------------------------------------------
    // Objective lookup paths.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_progress_objective_for_unknown_id_returns_false_round153() {
        // progress_objective must return
        // false (no side-effects, no
        // completed_objectives push) when
        // the id isn't in the config.
        let mut dim = make_dim_with_objectives(
            "unknown_obj",
            vec![("kill", 10, false)],
        );
        let just_completed = dim.progress_objective("does_not_exist", 1);
        assert!(!just_completed);
        assert!(dim.completed_objectives.is_empty());
    }

    #[test]
    fn dimension_get_objective_for_unknown_id_returns_none_round153() {
        // get_objective's mirror: unknown id
        // returns None (not a default-zero
        // objective, which would silently
        // mark a mandatory objective as 0%).
        let dim = make_dim_with_objectives(
            "get_unknown",
            vec![("kill", 10, false)],
        );
        assert!(dim.get_objective("does_not_exist").is_none());
        // Sanity: the real objective is still findable.
        let obj = dim.get_objective("kill").expect("real obj");
        assert_eq!(obj.target, 10);
    }

    // -----------------------------------------------------------------
    // Event log paths.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_broadcast_event_writes_to_event_log_round153() {
        // broadcast_event writes the event to
        // event_log (in addition to
        // dispatching to atom runners).
        // With no atom runners the dispatch
        // is a no-op but the log entry must
        // still appear.
        let mut dim = make_dim_with_objectives("broadcast_log", vec![]);
        let mut data = ValueMap::new();
        data.insert("k".to_string(), Value::Integer(42));
        let mut ctx = make_ctx();
        dim.broadcast_event("custom_event", &data, &mut ctx);
        assert_eq!(dim.event_log.len(), 1);
        let logged = &dim.event_log[0];
        assert_eq!(logged.event_type, "custom_event");
        // Value lookup on the logged map.
        match logged.data.get("k") {
            Some(Value::Integer(42)) => {}
            other => panic!("expected Integer(42) in logged data, got {:?}", other),
        }
    }

    #[test]
    fn dimension_event_log_after_complete_and_fail_round153() {
        // complete() and fail() each push one
        // event to event_log — pin both
        // entries so a future refactor can't
        // accidentally drop one of them.
        let mut dim_c = make_dim_with_objectives("complete_log", vec![]);
        let mut ctx_c = make_ctx();
        dim_c.complete(&mut ctx_c);
        assert_eq!(dim_c.event_log.len(), 1);
        assert_eq!(dim_c.event_log[0].event_type, "dimension_complete");
        assert_eq!(dim_c.state, DimensionState::Completed);

        let mut dim_f = make_dim_with_objectives("fail_log", vec![]);
        let mut ctx_f = make_ctx();
        dim_f.fail(&mut ctx_f, "out_of_time");
        assert_eq!(dim_f.event_log.len(), 1);
        assert_eq!(dim_f.event_log[0].event_type, "dimension_fail");
        match dim_f.event_log[0].data.get("reason") {
            Some(Value::String(s)) if s == "out_of_time" => {}
            other => panic!("expected String(\"out_of_time\"), got {:?}", other),
        }
        assert_eq!(dim_f.state, DimensionState::Failed);
    }

    // -----------------------------------------------------------------
    // DimensionProgress formula + DimensionRunner no-active paths.
    // -----------------------------------------------------------------

    #[test]
    fn dimension_progress_time_remaining_formula_round153() {
        // time_remaining = time_limit_secs - elapsed_time.
        // With no time_limit_secs the field is
        // None. Pin both branches.
        let mut dim = make_dim_with_objectives("time_remaining", vec![]);
        // No elapsed_time yet, full budget.
        let p0 = dim.get_progress();
        assert_eq!(p0.time_remaining, Some(60.0));
        // Simulate 10s of elapsed_time.
        dim.elapsed_time = 10.0;
        let p1 = dim.get_progress();
        assert_eq!(p1.time_remaining, Some(50.0));
        // Now flip time_limit_secs to None —
        // time_remaining becomes None.
        dim.config.time_limit_secs = None;
        let p2 = dim.get_progress();
        assert_eq!(p2.time_remaining, None);
    }

    #[test]
    fn dimension_runner_is_running_with_no_active_dim_round153() {
        // A fresh runner has no active dimension;
        // is_running() must return false.
        let runner = make_runner();
        assert!(!runner.is_running());
        assert!(runner.get_active_dimension().is_none());
        assert!(runner.get_progress().is_none());
    }
}
