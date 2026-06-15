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

    pub fn create(&self, id: &str) -> Option<Box<dyn Atom>> {
        self.atoms.get(id).map(|(_, factory)| factory())
    }

    pub fn get_metadata(&self, id: &str) -> Option<&AtomMetadata> {
        self.atoms.get(id).map(|(metadata, _)| metadata)
    }

    pub fn list_all(&self) -> Vec<&AtomMetadata> {
        self.atoms.values().map(|(m, _)| m).collect()
    }

    pub fn has_atom(&self, id: &str) -> bool {
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
        let factory: AtomFactory = Box::new(|| Box::new(TestAtom::new()));
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
            factory,
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

// ---------------------------------------------------------------------------
// Round 130 — atom.rs helper-level unit tests.
// Mirrors the round-110b / 122 / 123 / 124 / 125 / 126 / 127 / 128 / 129
// pattern: pin the small public helpers' contracts
// (`AtomRegistry` new/default/register/has/get/create/list,
// `AtomContext` new/with_delta_time/get_world, `AtomPhase`
// PartialEq round-trip) so a refactor can't silently
// change the registration / context-builder behaviour.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round130_tests {
    use super::*;
    use crate::agi_minigame::player::PlayerProfile;

    fn make_metadata(id: &str) -> AtomMetadata {
        AtomMetadata {
            id: id.to_string(),
            name: format!("{id} name"),
            version: 1,
            gameplay_type: "puzzle".to_string(),
            description: format!("{id} desc"),
            tags: vec!["test".to_string()],
        }
    }

    fn make_factory() -> AtomFactory {
        Box::new(|| panic!("factory should not run during these tests"))
    }

    #[test]
    fn atom_registry_new_is_empty() {
        // A freshly-constructed registry has
        // no atoms. list_all() returns [],
        // has_atom() returns false for any id.
        let r = AtomRegistry::new();
        assert!(r.list_all().is_empty());
        assert!(!r.has_atom("any"));
        assert!(r.get_metadata("any").is_none());
        assert!(r.create("any").is_none());
    }

    #[test]
    fn atom_registry_default_matches_new() {
        // `Default::default()` should produce
        // the same empty state as `new()`.
        let r: AtomRegistry = Default::default();
        assert!(r.list_all().is_empty());
    }

    #[test]
    fn atom_registry_register_then_has_atom() {
        let mut r = AtomRegistry::new();
        r.register("a".to_string(), make_metadata("a"), make_factory());
        assert!(r.has_atom("a"));
        assert!(!r.has_atom("b"));
    }

    #[test]
    fn atom_registry_register_same_id_twice_overwrites() {
        // The doc says `register` uses
        // `HashMap::insert` semantics — a 2nd
        // register with the same id overwrites
        // the previous factory. This test pins
        // the overwrite contract so a refactor
        // can't silently make it append.
        let mut r = AtomRegistry::new();
        let m1 = AtomMetadata {
            id: "a".to_string(),
            name: "first".to_string(),
            version: 1,
            gameplay_type: "puzzle".to_string(),
            description: "first".to_string(),
            tags: vec![],
        };
        let m2 = AtomMetadata {
            id: "a".to_string(),
            name: "second".to_string(),
            version: 2,
            gameplay_type: "strategy".to_string(),
            description: "second".to_string(),
            tags: vec!["v2".to_string()],
        };
        r.register("a".to_string(), m1, make_factory());
        r.register("a".to_string(), m2.clone(), make_factory());
        // The 2nd register wins — get_metadata
        // returns m2, not m1.
        let got = r.get_metadata("a").unwrap();
        assert_eq!(got.name, "second");
        assert_eq!(got.version, 2);
    }

    #[test]
    fn atom_registry_get_metadata_for_unknown_id_returns_none() {
        // Defensive: a get_metadata call for an
        // id that was never registered returns
        // None (not a panic). The WASM bridge
        // relies on this Option-returning contract
        // to silently ignore typos in blueprint
        // atom_ids.
        let r = AtomRegistry::new();
        assert!(r.get_metadata("nonexistent").is_none());
        assert!(r.get_metadata("").is_none());
        assert!(r.get_metadata("with.dots.in.id").is_none());
    }

    #[test]
    fn atom_registry_create_for_unknown_id_returns_none() {
        // Same defensive contract for
        // `create`. The runtime never panics
        // when asked to instantiate an atom
        // that doesn't exist.
        let r = AtomRegistry::new();
        assert!(r.create("nonexistent").is_none());
    }

    #[test]
    fn atom_registry_list_all_returns_all_registered() {
        // Register 3 atoms → list_all returns
        // 3 entries (order is HashMap
        // iteration order, not insertion
        // order — don't pin a specific order).
        let mut r = AtomRegistry::new();
        r.register("a".to_string(), make_metadata("a"), make_factory());
        r.register("b".to_string(), make_metadata("b"), make_factory());
        r.register("c".to_string(), make_metadata("c"), make_factory());
        let all = r.list_all();
        assert_eq!(all.len(), 3);
        let ids: std::collections::HashSet<&str> = all.iter().map(|m| m.id.as_str()).collect();
        assert!(ids.contains("a"));
        assert!(ids.contains("b"));
        assert!(ids.contains("c"));
    }

    #[test]
    fn atom_context_new_has_default_delta_time_0_016() {
        // `AtomContext::new` should set
        // `delta_time` to a sane default
        // (16ms = ~60 FPS). A regression
        // that defaulted to 0.0 would break
        // `on_update` callers that compute
        // frame-dependent deltas.
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("ctx_test"))));
        let ctx = AtomContext::new(Arc::clone(&ws));
        assert!((ctx.delta_time - 0.016).abs() < 1e-6);
    }

    #[test]
    fn atom_context_with_delta_time_returns_modified_context() {
        // The builder method should
        // mutate the delta_time field
        // and return self for chaining.
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("ctx_test"))));
        let ctx = AtomContext::new(ws).with_delta_time(0.033);
        assert!((ctx.delta_time - 0.033).abs() < 1e-6);
    }

    #[test]
    fn atom_context_get_world_returns_same_arc() {
        // `get_world` should return an
        // Arc that points to the same
        // underlying UnifiedWorldState
        // (Arc::clone contract). A regression
        // that returned a fresh state would
        // break the runner's ability to
        // observe world mutations.
        let ws = Arc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("ctx_test"))));
        let ctx = AtomContext::new(Arc::clone(&ws));
        let got = ctx.get_world();
        assert!(Arc::ptr_eq(&ws, &got));
    }

    #[test]
    fn atom_phase_partial_eq_for_all_5_variants() {
        // Pin the `PartialEq` contract for
        // all 5 variants. A refactor that
        // accidentally derives `PartialEq`
        // from a non-canonical field (e.g.
        // a debug label) would break this.
        use AtomPhase::*;
        assert_eq!(Uninitialized, Uninitialized);
        assert_eq!(Initialized, Initialized);
        assert_eq!(Running, Running);
        assert_eq!(Paused, Paused);
        assert_eq!(Completed, Completed);
        assert_ne!(Uninitialized, Initialized);
        assert_ne!(Running, Paused);
    }
}
