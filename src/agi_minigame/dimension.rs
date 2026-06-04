use std::collections::HashMap;
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
        let mandatory: Vec<_> = self.config.objectives.iter().filter(|o| !o.is_optional).collect();
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
        if let Some(ref mut dim) = self.active_dimension {
            let mut ctx = AtomContext::new(Arc::clone(&self.world_state));
            dim.update(dt, &mut ctx);
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref mut dim) = self.active_dimension {
            let mut ctx = AtomContext::new(Arc::clone(&self.world_state));
            dim.pause(&mut ctx);
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref mut dim) = self.active_dimension {
            let mut ctx = AtomContext::new(Arc::clone(&self.world_state));
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
    use crate::agi_minigame::atom::AtomMetadata;
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
