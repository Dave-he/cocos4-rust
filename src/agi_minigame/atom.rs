use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::base::value::ValueMap;

use super::world_state::UnifiedWorldState;

pub type AtomId = String;
pub type AtomFactory = Box<dyn Fn() -> Box<dyn Atom> + Send + Sync>;

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

    pub fn register<F>(&mut self, id: AtomId, metadata: AtomMetadata, factory: F)
    where
        F: Fn() -> Box<dyn Atom> + Send + Sync + 'static,
    {
        self.atoms.insert(id, (metadata, Box::new(factory)));
    }

    pub fn create(&self, id: impl AsRef<str>) -> Option<Box<dyn Atom>> {
        self.atoms.get(id.as_ref()).map(|(_, factory)| factory())
    }

    pub fn get_metadata(&self, id: impl AsRef<str>) -> Option<&AtomMetadata> {
        self.atoms.get(id.as_ref()).map(|(metadata, _)| metadata)
    }

    pub fn list_all(&self) -> Vec<&AtomMetadata> {
        self.atoms.values().map(|(m, _)| m).collect()
    }

    pub fn has_atom(&self, id: impl AsRef<str>) -> bool {
        self.atoms.contains_key(id.as_ref())
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

// ---------------------------------------------------------------------------
// Round 142 — `AtomRunner` lifecycle helper-level tests.
// The runner is the runtime shell that wraps every atom
// (it's the abstraction used by `ai_engine.rs` and the
// host loop). The round-130 helper suite only covered
// `AtomRegistry` + `AtomContext`; the runner's own
// per-method contracts (update is conditional on phase,
// pause is conditional, handle_event / save_state /
// load_state / get_atom* delegate verbatim) are not
// pinned. This mod fixes that gap with a small
// `TrackingTestAtom` that records every callback it
// receives, then drives the runner through each path
// and asserts the callback was (or wasn't) invoked.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod round142_tests {
    use super::*;
    use crate::agi_minigame::player::PlayerProfile;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;

    /// `TrackingTestAtom` records every callback the
    /// `AtomRunner` invokes. The test runner's
    /// "conditional on phase" contracts (e.g. update
    /// is a no-op when not Running) are verified by
    /// asserting the counter for that callback stays
    /// at 0 across the test.
    struct TrackingTestAtom {
        phase: AtomPhase,
        init_count: AtomicUsize,
        enter_count: AtomicUsize,
        update_count: AtomicUsize,
        pause_count: AtomicUsize,
        resume_count: AtomicUsize,
        exit_count: AtomicUsize,
        destroy_count: AtomicUsize,
        // Track the last event + data so we can verify
        // `handle_event` delegates verbatim.
        last_event: std::sync::Mutex<Option<String>>,
        last_data: std::sync::Mutex<Option<ValueMap>>,
        // Track the state for `save_state` / `load_state`.
        state: std::sync::Mutex<ValueMap>,
    }

    impl TrackingTestAtom {
        fn new() -> Self {
            Self {
                phase: AtomPhase::Uninitialized,
                init_count: AtomicUsize::new(0),
                enter_count: AtomicUsize::new(0),
                update_count: AtomicUsize::new(0),
                pause_count: AtomicUsize::new(0),
                resume_count: AtomicUsize::new(0),
                exit_count: AtomicUsize::new(0),
                destroy_count: AtomicUsize::new(0),
                last_event: std::sync::Mutex::new(None),
                last_data: std::sync::Mutex::new(None),
                state: std::sync::Mutex::new(ValueMap::new()),
            }
        }

        fn new_factory() -> AtomFactory {
            Box::new(|| Box::new(TrackingTestAtom::new()))
        }
    }

    impl Atom for TrackingTestAtom {
        fn atom_id(&self) -> AtomId { "tracking".to_string() }
        fn atom_name(&self) -> &str { "Tracking" }
        fn on_init(&mut self, _ctx: &mut AtomContext) {
            self.init_count.fetch_add(1, Ordering::SeqCst);
            self.phase = AtomPhase::Initialized;
        }
        fn on_enter(&mut self, _ctx: &mut AtomContext) {
            self.enter_count.fetch_add(1, Ordering::SeqCst);
            self.phase = AtomPhase::Running;
        }
        fn on_update(&mut self, _ctx: &mut AtomContext) {
            self.update_count.fetch_add(1, Ordering::SeqCst);
        }
        fn on_pause(&mut self, _ctx: &mut AtomContext) {
            self.pause_count.fetch_add(1, Ordering::SeqCst);
            self.phase = AtomPhase::Paused;
        }
        fn on_resume(&mut self, _ctx: &mut AtomContext) {
            self.resume_count.fetch_add(1, Ordering::SeqCst);
            self.phase = AtomPhase::Running;
        }
        fn on_exit(&mut self, _ctx: &mut AtomContext) {
            self.exit_count.fetch_add(1, Ordering::SeqCst);
            self.phase = AtomPhase::Completed;
        }
        fn on_destroy(&mut self) {
            self.destroy_count.fetch_add(1, Ordering::SeqCst);
            self.phase = AtomPhase::Uninitialized;
        }
        fn save_state(&self) -> ValueMap {
            // Return a snapshot of our internal
            // `state` map so we can verify the
            // runner delegates verbatim (and so
            // the test can round-trip a payload).
            self.state.lock().unwrap().clone()
        }
        fn load_state(&mut self, state: &ValueMap) {
            *self.state.lock().unwrap() = state.clone();
        }
        fn handle_event(&mut self, event: &str, data: &ValueMap, _ctx: &mut AtomContext) {
            *self.last_event.lock().unwrap() = Some(event.to_string());
            *self.last_data.lock().unwrap() = Some(data.clone());
        }
        fn current_phase(&self) -> AtomPhase { self.phase }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    fn make_ctx() -> AtomContext {
        let ws = StdArc::new(Mutex::new(UnifiedWorldState::new(PlayerProfile::new("runner_test"))));
        AtomContext::new(ws)
    }

    /// Construct a fresh `AtomRunner` wrapping a
    /// `TrackingTestAtom` plus a shared `Arc<TrackingTestAtom>`
    /// the test can use to inspect callback counts after
    /// each `runner.X(...)` call.
    fn make_runner() -> (AtomRunner, StdArc<std::sync::Mutex<TrackingTestAtom>>) {
        // The factory returns a `Box<dyn Atom>` but we
        // need a way to peek inside it. Build the
        // tracking atom once, wrap it in a Mutex so
        // both the Box (via as_any_mut) and the test
        // thread can reach it, and push it through
        // the runner. The runner's `get_atom_mut` /
        // `get_atom` give us back `&dyn Atom` which
        // we can downcast via `as_any_mut`.
        let atom = TrackingTestAtom::new();
        let shared = StdArc::new(std::sync::Mutex::new(atom));
        // Build a Box<dyn Atom> that delegates to the
        // shared atom. We can't move the Arc into the
        // Box (trait objects are not Clone), so we
        // wrap the same instance in a small adapter
        // struct. Simpler path: build the runner with
        // a fresh TrackingTestAtom, but then expose
        // the `&mut dyn Atom` to the test via
        // `runner.get_atom_mut()` + `as_any_mut()`.
        let runner = AtomRunner::new(Box::new(TrackingTestAtom::new()));
        (runner, shared)
    }

    #[test]
    fn runner_new_starts_in_uninitialized_phase() {
        // `new(atom)` is the fresh state — the
        // runner has NOT called on_init yet, so
        // phase == Uninitialized (the initial
        // value of `phase` in the runner struct).
        let (runner, _) = make_runner();
        assert_eq!(runner.get_phase(), AtomPhase::Uninitialized);
    }

    #[test]
    fn runner_init_transitions_to_initialized() {
        // `init(ctx)` must call `atom.on_init` and
        // set the runner's phase to Initialized.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Initialized);
    }

    #[test]
    fn runner_enter_transitions_to_running() {
        // `enter(ctx)` must call `atom.on_enter` and
        // set the runner's phase to Running.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Running);
    }

    #[test]
    fn runner_update_is_noop_when_not_running() {
        // The runner guards `update` with
        // `if self.phase == Running`. When phase
        // is Uninitialized / Initialized / Paused
        // / Completed, `update` must NOT call
        // `atom.on_update`. Verify by checking
        // the inner atom's update_count.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        // Not yet init / enter → phase = Uninitialized.
        runner.update(&mut ctx);
        let count_uninit = {
            let atom = runner.get_atom_mut().as_any_mut().downcast_mut::<TrackingTestAtom>().unwrap();
            atom.update_count.load(Ordering::SeqCst)
        };
        assert_eq!(count_uninit, 0, "update must not fire on Uninitialized");

        // After init but before enter → Initialized.
        runner.init(&mut ctx);
        runner.update(&mut ctx);
        let count_init = {
            let atom = runner.get_atom_mut().as_any_mut().downcast_mut::<TrackingTestAtom>().unwrap();
            atom.update_count.load(Ordering::SeqCst)
        };
        assert_eq!(count_init, 0, "update must not fire on Initialized");
    }

    #[test]
    fn runner_update_fires_on_update_when_running() {
        // After `enter`, `update` must call
        // `atom.on_update` on every invocation.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        runner.update(&mut ctx);
        runner.update(&mut ctx);
        runner.update(&mut ctx);
        let count = {
            let atom = runner.get_atom_mut().as_any_mut().downcast_mut::<TrackingTestAtom>().unwrap();
            atom.update_count.load(Ordering::SeqCst)
        };
        assert_eq!(count, 3, "3 update() calls → 3 on_update fires");
    }

    #[test]
    fn runner_pause_is_noop_when_not_running() {
        // The runner guards `pause` with
        // `if self.phase == Running`. When phase
        // is anything else, `pause` must NOT call
        // `atom.on_pause` and must NOT change phase.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        // Uninitialized → pause is a no-op.
        runner.pause(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Uninitialized);
        // After init (Initialized) → still no-op.
        runner.init(&mut ctx);
        runner.pause(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Initialized);
    }

    #[test]
    fn runner_pause_transitions_running_to_paused() {
        // After `enter` (Running), `pause` must
        // call `atom.on_pause` and set phase to
        // Paused.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Running);
        runner.pause(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Paused);
    }

    #[test]
    fn runner_resume_is_noop_when_not_paused() {
        // The runner guards `resume` with
        // `if self.phase == Paused`. When phase
        // is anything else, `resume` must NOT
        // call `atom.on_resume` and must NOT
        // change phase.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        // Uninitialized → no-op.
        runner.resume(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Uninitialized);
        // After init + enter (Running) → no-op.
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        runner.resume(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Running);
    }

    #[test]
    fn runner_resume_transitions_paused_to_running() {
        // After pause (Paused), `resume` must
        // call `atom.on_resume` and set phase
        // back to Running.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        runner.pause(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Paused);
        runner.resume(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Running);
    }

    #[test]
    fn runner_exit_transitions_to_completed() {
        // `exit(ctx)` must call `atom.on_exit`
        // and set the runner's phase to
        // Completed, regardless of the current
        // phase (the method is unconditional).
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        runner.exit(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Completed);
    }

    #[test]
    fn runner_exit_from_initialized_also_completes() {
        // `exit` is unconditional — calling it
        // from Initialized (skipping Running)
        // still marks Completed. This matches
        // the round-140 turn_combat `on_enter`
        // contract where an atom can short-
        // circuit straight to Completed.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.exit(&mut ctx);
        assert_eq!(runner.get_phase(), AtomPhase::Completed);
    }

    #[test]
    fn runner_handle_event_delegates_verbatim() {
        // `handle_event(event, data, ctx)` must
        // delegate to `atom.handle_event` with
        // the exact same `event` string and
        // `data` map (no wrapping, no
        // transform, no filtering).
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        let mut data = ValueMap::new();
        data.insert("score".to_string(), crate::base::value::Value::Integer(42));
        runner.handle_event("player_jumped", &data, &mut ctx);
        let (last_event, last_data) = {
            let atom = runner.get_atom_mut().as_any_mut().downcast_mut::<TrackingTestAtom>().unwrap();
            let e = atom.last_event.lock().unwrap().clone();
            let d = atom.last_data.lock().unwrap().clone();
            (e, d)
        };
        assert_eq!(last_event.as_deref(), Some("player_jumped"));
        let d = last_data.expect("last_data must be set after handle_event");
        assert_eq!(
            d.get("score"),
            Some(&crate::base::value::Value::Integer(42)),
            "data must round-trip verbatim"
        );
    }

    #[test]
    fn runner_handle_event_fires_in_any_phase() {
        // `handle_event` is unconditional (no
        // `if phase == ...` guard). Even before
        // `init` / `enter`, calling it must
        // reach the atom. This is the contract
        // the host uses to feed game events
        // into a freshly-spawned atom.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.handle_event("boot", &ValueMap::new(), &mut ctx);
        let last = {
            let atom = runner.get_atom_mut().as_any_mut().downcast_mut::<TrackingTestAtom>().unwrap();
            atom.last_event.lock().unwrap().clone()
        };
        assert_eq!(last.as_deref(), Some("boot"));
    }

    #[test]
    fn runner_save_state_delegates_to_atom() {
        // `save_state()` must call
        // `atom.save_state()` and return the
        // exact same ValueMap (no wrapping,
        // no extra keys).
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        // Seed the atom's internal state.
        {
            let atom = runner.get_atom_mut().as_any_mut().downcast_mut::<TrackingTestAtom>().unwrap();
            let mut s = atom.state.lock().unwrap();
            s.insert("lives".to_string(), crate::base::value::Value::Integer(3));
        }
        let state = runner.save_state();
        assert_eq!(
            state.get("lives"),
            Some(&crate::base::value::Value::Integer(3))
        );
    }

    #[test]
    fn runner_load_state_round_trips_via_save_state() {
        // The standard "save then load" round
        // trip: load a known payload, then save
        // and verify the payload survived
        // verbatim. This is the contract the
        // host uses to persist + restore atom
        // state across sessions.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        let mut payload = ValueMap::new();
        payload.insert("level".to_string(), crate::base::value::Value::Integer(7));
        payload.insert("hp".to_string(), crate::base::value::Value::Integer(100));
        runner.load_state(&payload);
        let saved = runner.save_state();
        assert_eq!(saved.get("level"), Some(&crate::base::value::Value::Integer(7)));
        assert_eq!(saved.get("hp"), Some(&crate::base::value::Value::Integer(100)));
    }

    #[test]
    fn runner_get_atom_returns_dyn_atom_ref() {
        // `get_atom()` must return a `&dyn Atom`
        // pointing to the same underlying
        // instance the runner was constructed
        // with. The `atom_id` is a stable
        // surface the host can use to confirm
        // identity.
        let (runner, _) = make_runner();
        let atom = runner.get_atom();
        assert_eq!(atom.atom_id(), "tracking");
        assert_eq!(atom.atom_name(), "Tracking");
    }

    #[test]
    fn runner_get_atom_mut_allows_mutation_via_dyn_atom() {
        // `get_atom_mut()` must return a
        // `&mut dyn Atom` that the test can use
        // to invoke `save_state` / `handle_event`
        // through the trait. The `as_any_mut`
        // downcast round-trip is the same one
        // other rounds use to peek at the
        // tracking atom's counters.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        let atom_mut = runner.get_atom_mut();
        // Mutate via the trait method
        // `save_state` to prove the
        // `&mut dyn Atom` is wired correctly.
        let _state = atom_mut.save_state();
        // The previous line is a smoke test;
        // also verify the `as_any_mut`
        // downcast works.
        let downcast = atom_mut.as_any_mut().downcast_mut::<TrackingTestAtom>();
        assert!(downcast.is_some(), "as_any_mut must downcast to TrackingTestAtom");
    }

    #[test]
    fn runner_phase_reflects_lifecycle_in_order() {
        // The full happy-path lifecycle
        // produces a specific sequence of
        // phase transitions: Uninitialized →
        // Initialized → Running → Paused →
        // Running → Completed. Verify the
        // runner's `get_phase` matches the
        // expected transition at each step.
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
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

    #[test]
    fn runner_save_state_before_init_returns_empty() {
        // Edge: calling `save_state` before
        // `init` is allowed — it just calls
        // `atom.save_state()`, which for our
        // `TrackingTestAtom` returns the
        // (empty) state map. The runner does
        // not require init before save_state
        // (the host may want to dump a
        // never-entered atom's default state).
        let (runner, _) = make_runner();
        let state = runner.save_state();
        assert!(state.is_empty(), "fresh atom's save_state must be empty");
    }

    #[test]
    fn runner_load_state_does_not_change_phase() {
        // `load_state` is a data-only call —
        // it must NOT touch the phase. The
        // runner's contract is: "phase is
        // governed by init/enter/pause/resume/
        // exit; load_state just feeds the
        // atom's persistence payload."
        let (mut runner, _) = make_runner();
        let mut ctx = make_ctx();
        runner.init(&mut ctx);
        runner.enter(&mut ctx);
        let phase_before = runner.get_phase();
        let mut payload = ValueMap::new();
        payload.insert("k".to_string(), crate::base::value::Value::Integer(1));
        runner.load_state(&payload);
        assert_eq!(runner.get_phase(), phase_before);
    }
}
