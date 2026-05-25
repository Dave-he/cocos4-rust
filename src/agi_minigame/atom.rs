use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::base::value::ValueMap;

use super::world_state::UnifiedWorldState;

pub type AtomId = String;
pub type AtomFactory = Box<dyn Fn() -> Box<dyn Atom>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomPhase {
    Uninitialized,
    Initialized,
    Running,
    Paused,
    Completed,
}

#[derive(Debug, Clone)]
pub struct AtomMetadata {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub gameplay_type: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AtomContext {
    pub world_state: Arc<Mutex<UnifiedWorldState>>,
    pub delta_time: f32,
    pub shared_data: ValueMap,
}

impl AtomContext {
    pub fn new(world_state: Arc<Mutex<UnifiedWorldState>>) -> Self {
        Self {
            world_state,
            delta_time: 0.016,
            shared_data: ValueMap::new(),
        }
    }

    pub fn with_delta_time(mut self, dt: f32) -> Self {
        self.delta_time = dt;
        self
    }

    pub fn get_world(&self) -> Arc<Mutex<UnifiedWorldState>> {
        Arc::clone(&self.world_state)
    }
}

pub trait Atom: Send + Sync {
    fn atom_id(&self) -> AtomId;
    fn atom_name(&self) -> &str;
    
    fn on_init(&mut self, ctx: &mut AtomContext);
    fn on_enter(&mut self, ctx: &mut AtomContext);
    fn on_update(&mut self, ctx: &mut AtomContext);
    fn on_pause(&mut self, ctx: &mut AtomContext);
    fn on_resume(&mut self, ctx: &mut AtomContext);
    fn on_exit(&mut self, ctx: &mut AtomContext);
    fn on_destroy(&mut self);
    
    fn save_state(&self) -> ValueMap;
    fn load_state(&mut self, state: &ValueMap);
    
    fn handle_event(&mut self, event: &str, data: &ValueMap, ctx: &mut AtomContext);
    
    fn current_phase(&self) -> AtomPhase;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct AtomRegistry {
    atoms: HashMap<AtomId, (AtomMetadata, AtomFactory)>,
}

impl AtomRegistry {
    pub fn new() -> Self {
        Self {
            atoms: HashMap::new(),
        }
    }

    pub fn register(&mut self, id: AtomId, metadata: AtomMetadata, factory: AtomFactory) {
        self.atoms.insert(id, (metadata, factory));
    }

    pub fn create(&self, id: &AtomId) -> Option<Box<dyn Atom>> {
        self.atoms.get(id).map(|(_, factory)| factory())
    }

    pub fn get_metadata(&self, id: &AtomId) -> Option<&AtomMetadata> {
        self.atoms.get(id).map(|(metadata, _)| metadata)
    }

    pub fn list_all(&self) -> Vec<&AtomMetadata> {
        self.atoms.values().map(|(m, _)| m).collect()
    }

    pub fn has_atom(&self, id: &AtomId) -> bool {
        self.atoms.contains_key(id)
    }
}

impl Default for AtomRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AtomRunner {
    atom: Box<dyn Atom>,
    phase: AtomPhase,
}

impl AtomRunner {
    pub fn new(atom: Box<dyn Atom>) -> Self {
        Self {
            atom,
            phase: AtomPhase::Uninitialized,
        }
    }

    pub fn init(&mut self, ctx: &mut AtomContext) {
        self.atom.on_init(ctx);
        self.phase = AtomPhase::Initialized;
    }

    pub fn enter(&mut self, ctx: &mut AtomContext) {
        self.atom.on_enter(ctx);
        self.phase = AtomPhase::Running;
    }

    pub fn update(&mut self, ctx: &mut AtomContext) {
        if self.phase == AtomPhase::Running {
            self.atom.on_update(ctx);
        }
    }

    pub fn pause(&mut self, ctx: &mut AtomContext) {
        if self.phase == AtomPhase::Running {
            self.atom.on_pause(ctx);
            self.phase = AtomPhase::Paused;
        }
    }

    pub fn resume(&mut self, ctx: &mut AtomContext) {
        if self.phase == AtomPhase::Paused {
            self.atom.on_resume(ctx);
            self.phase = AtomPhase::Running;
        }
    }

    pub fn exit(&mut self, ctx: &mut AtomContext) {
        self.atom.on_exit(ctx);
        self.phase = AtomPhase::Completed;
    }

    pub fn handle_event(&mut self, event: &str, data: &ValueMap, ctx: &mut AtomContext) {
        self.atom.handle_event(event, data, ctx);
    }

    pub fn save_state(&self) -> ValueMap {
        self.atom.save_state()
    }

    pub fn load_state(&mut self, state: &ValueMap) {
        self.atom.load_state(state);
    }

    pub fn get_atom(&self) -> &dyn Atom {
        self.atom.as_ref()
    }

    pub fn get_atom_mut(&mut self) -> &mut dyn Atom {
        self.atom.as_mut()
    }

    pub fn get_phase(&self) -> AtomPhase {
        self.phase
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agi_minigame::player::PlayerProfile;

    struct TestAtom {
        phase: AtomPhase,
    }

    impl TestAtom {
        fn new() -> Self {
            Self {
                phase: AtomPhase::Uninitialized,
            }
        }
    }

    impl Atom for TestAtom {
        fn atom_id(&self) -> AtomId { "test".to_string() }
        fn atom_name(&self) -> &str { "Test" }
        fn on_init(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Initialized; }
        fn on_enter(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
        fn on_update(&mut self, _ctx: &mut AtomContext) {}
        fn on_pause(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Paused; }
        fn on_resume(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Running; }
        fn on_exit(&mut self, _ctx: &mut AtomContext) { self.phase = AtomPhase::Completed; }
        fn on_destroy(&mut self) { self.phase = AtomPhase::Uninitialized; }
        fn save_state(&self) -> ValueMap { ValueMap::new() }
        fn load_state(&mut self, _state: &ValueMap) {}
        fn handle_event(&mut self, _event: &str, _data: &ValueMap, _ctx: &mut AtomContext) {}
        fn current_phase(&self) -> AtomPhase { self.phase }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_atom_registry() {
        let mut registry = AtomRegistry::new();
        registry.register(
            "test".to_string(),
            AtomMetadata {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: 1,
                gameplay_type: "test".to_string(),
                description: "Test atom".to_string(),
                tags: vec!["test".to_string()],
            },
            || Box::new(TestAtom::new()),
        );

        assert!(registry.has_atom("test"));
        assert!(!registry.has_atom("nonexistent"));
        
        let metadata = registry.get_metadata("test").unwrap();
        assert_eq!(metadata.name, "Test");

        let atom = registry.create("test");
        assert!(atom.is_some());
    }

    #[test]
    fn test_atom_runner() {
        let mut runner = AtomRunner::new(Box::new(TestAtom::new()));
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("test"))));
        let mut ctx = AtomContext::new(ws);

        assert_eq!(runner.get_phase(), AtomPhase::Uninitialized);
        
        runner.init(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Initialized);
        
        runner.enter(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Running);
        
        runner.pause(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Paused);
        
        runner.resume(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Running);
        
        runner.exit(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Completed);
    }
}
