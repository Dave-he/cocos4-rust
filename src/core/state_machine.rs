use std::collections::HashMap;
use std::any::Any;

pub trait State: Any + Send + Sync {
    fn name(&self) -> &str;
    fn on_enter(&mut self, _prev: Option<&str>) {}
    fn on_exit(&mut self, _next: Option<&str>) {}
    fn on_update(&mut self, _dt: f32) {}
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionResult {
    Accepted,
    Rejected,
    NotFound,
}

pub struct Transition {
    pub from: String,
    pub to: String,
    pub condition: Box<dyn Fn() -> bool + Send + Sync>,
    pub priority: i32,
}

impl Transition {
    pub fn new<F: Fn() -> bool + Send + Sync + 'static>(from: &str, to: &str, cond: F) -> Self {
        Transition {
            from: from.to_string(),
            to: to.to_string(),
            condition: Box::new(cond),
            priority: 0,
        }
    }

    pub fn with_priority(mut self, p: i32) -> Self {
        self.priority = p;
        self
    }
}

type TransitionGuard = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;
type StateChangeCallback = Box<dyn Fn(&str, &str) + Send + Sync>;

pub struct StateMachine {
    states: HashMap<String, Box<dyn State>>,
    transitions: Vec<Transition>,
    current: Option<String>,
    previous: Option<String>,
    history: Vec<String>,
    global_guard: Option<TransitionGuard>,
    on_change: Vec<StateChangeCallback>,
    transition_count: u64,
}

impl StateMachine {
    pub fn new() -> Self {
        StateMachine {
            states: HashMap::new(),
            transitions: Vec::new(),
            current: None,
            previous: None,
            history: Vec::new(),
            global_guard: None,
            on_change: Vec::new(),
            transition_count: 0,
        }
    }

    pub fn add_state<S: State + 'static>(&mut self, state: S) {
        self.states.insert(state.name().to_string(), Box::new(state));
    }

    pub fn add_transition(&mut self, t: Transition) {
        self.transitions.push(t);
        self.transitions.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn add_any_transition<F: Fn() -> bool + Send + Sync + 'static>(
        &mut self,
        to: &str,
        cond: F,
        priority: i32,
    ) {
        self.transitions.push(Transition {
            from: "__any__".to_string(),
            to: to.to_string(),
            condition: Box::new(cond),
            priority,
        });
        self.transitions.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn set_global_guard<F: Fn(&str, &str) -> bool + Send + Sync + 'static>(&mut self, guard: F) {
        self.global_guard = Some(Box::new(guard));
    }

    pub fn on_state_change<F: Fn(&str, &str) + Send + Sync + 'static>(&mut self, cb: F) {
        self.on_change.push(Box::new(cb));
    }

    pub fn start(&mut self, initial: &str) -> bool {
        if !self.states.contains_key(initial) {
            return false;
        }
        let prev = self.current.take();
        self.current = Some(initial.to_string());
        if let Some(state) = self.states.get_mut(initial) {
            state.on_enter(prev.as_deref());
        }
        self.history.push(initial.to_string());
        true
    }

    pub fn transition_to(&mut self, target: &str) -> TransitionResult {
        if !self.states.contains_key(target) {
            return TransitionResult::NotFound;
        }
        let current = match &self.current {
            Some(c) => c.clone(),
            None => return TransitionResult::Rejected,
        };
        if current == target {
            return TransitionResult::Rejected;
        }
        if let Some(ref guard) = self.global_guard {
            if !guard(&current, target) {
                return TransitionResult::Rejected;
            }
        }
        if let Some(state) = self.states.get_mut(&current) {
            state.on_exit(Some(target));
        }
        for cb in &self.on_change {
            cb(&current, target);
        }
        self.previous = Some(current);
        self.current = Some(target.to_string());
        self.history.push(target.to_string());
        self.transition_count += 1;

        if let Some(state) = self.states.get_mut(target) {
            let prev = self.previous.as_deref();
            state.on_enter(prev);
        }
        TransitionResult::Accepted
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(current_name) = self.current.clone() {
            if let Some(state) = self.states.get_mut(&current_name) {
                state.on_update(dt);
            }
        }
        self.check_auto_transitions();
    }

    fn check_auto_transitions(&mut self) {
        let current = match self.current.clone() {
            Some(c) => c,
            None => return,
        };

        let target = self.transitions.iter().find_map(|t| {
            if (t.from == current || t.from == "__any__") && (t.condition)() {
                Some(t.to.clone())
            } else {
                None
            }
        });

        if let Some(target) = target {
            if target != current {
                self.transition_to(&target);
            }
        }
    }

    pub fn get_current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    pub fn get_previous(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    pub fn get_state<S: State + 'static>(&self, name: &str) -> Option<&S> {
        self.states.get(name)?.as_any().downcast_ref::<S>()
    }

    pub fn get_state_mut<S: State + 'static>(&mut self, name: &str) -> Option<&mut S> {
        self.states.get_mut(name)?.as_any_mut().downcast_mut::<S>()
    }

    pub fn is_in(&self, name: &str) -> bool {
        self.current.as_deref() == Some(name)
    }

    pub fn has_state(&self, name: &str) -> bool {
        self.states.contains_key(name)
    }

    pub fn get_transition_count(&self) -> u64 {
        self.transition_count
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn go_back(&mut self) -> bool {
        if self.history.len() < 2 {
            return false;
        }
        self.history.pop();
        let target = self.history.last().cloned();
        if let Some(target) = target {
            self.history.pop();
            self.transition_to(&target) == TransitionResult::Accepted
        } else {
            false
        }
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct IdleState {
        pub enter_count: u32,
        pub exit_count: u32,
        pub update_count: u32,
    }

    impl IdleState {
        fn new() -> Self { IdleState { enter_count: 0, exit_count: 0, update_count: 0 } }
    }

    impl State for IdleState {
        fn name(&self) -> &str { "Idle" }
        fn on_enter(&mut self, _: Option<&str>) { self.enter_count += 1; }
        fn on_exit(&mut self, _: Option<&str>) { self.exit_count += 1; }
        fn on_update(&mut self, _dt: f32) { self.update_count += 1; }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    struct RunState {
        pub speed: f32,
        pub enter_count: u32,
    }

    impl RunState {
        fn new(speed: f32) -> Self { RunState { speed, enter_count: 0 } }
    }

    impl State for RunState {
        fn name(&self) -> &str { "Run" }
        fn on_enter(&mut self, _: Option<&str>) { self.enter_count += 1; }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    struct AttackState;
    impl State for AttackState {
        fn name(&self) -> &str { "Attack" }
        fn as_any(&self) -> &dyn Any { self }
        fn as_any_mut(&mut self) -> &mut dyn Any { self }
    }

    fn make_sm() -> StateMachine {
        let mut sm = StateMachine::new();
        sm.add_state(IdleState::new());
        sm.add_state(RunState::new(5.0));
        sm.add_state(AttackState);
        sm
    }

    #[test]
    fn test_state_machine_new() {
        let sm = StateMachine::new();
        assert!(sm.get_current().is_none());
        assert_eq!(sm.get_transition_count(), 0);
    }

    #[test]
    fn test_start() {
        let mut sm = make_sm();
        assert!(sm.start("Idle"));
        assert!(sm.is_in("Idle"));
    }

    #[test]
    fn test_start_unknown_fails() {
        let mut sm = make_sm();
        assert!(!sm.start("Unknown"));
    }

    #[test]
    fn test_transition_to() {
        let mut sm = make_sm();
        sm.start("Idle");
        let r = sm.transition_to("Run");
        assert_eq!(r, TransitionResult::Accepted);
        assert!(sm.is_in("Run"));
        assert_eq!(sm.get_previous(), Some("Idle"));
    }

    #[test]
    fn test_transition_to_same_state() {
        let mut sm = make_sm();
        sm.start("Idle");
        let r = sm.transition_to("Idle");
        assert_eq!(r, TransitionResult::Rejected);
    }

    #[test]
    fn test_transition_to_unknown() {
        let mut sm = make_sm();
        sm.start("Idle");
        let r = sm.transition_to("Ghost");
        assert_eq!(r, TransitionResult::NotFound);
    }

    #[test]
    fn test_on_enter_exit_called() {
        let mut sm = make_sm();
        sm.start("Idle");
        sm.transition_to("Run");
        assert_eq!(sm.get_state::<IdleState>("Idle").unwrap().exit_count, 1);
        assert_eq!(sm.get_state::<RunState>("Run").unwrap().enter_count, 1);
    }

    #[test]
    fn test_on_state_change_callback() {
        let mut sm = make_sm();
        let log = Arc::new(Mutex::new(Vec::<String>::new()));
        let l = Arc::clone(&log);
        sm.on_state_change(move |from, to| {
            l.lock().unwrap().push(format!("{}->{}", from, to));
        });
        sm.start("Idle");
        sm.transition_to("Run");
        sm.transition_to("Attack");
        let v = log.lock().unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], "Idle->Run");
        assert_eq!(v[1], "Run->Attack");
    }

    #[test]
    fn test_update_calls_current_state() {
        let mut sm = make_sm();
        sm.start("Idle");
        sm.update(0.016);
        sm.update(0.016);
        assert_eq!(sm.get_state::<IdleState>("Idle").unwrap().update_count, 2);
    }

    #[test]
    fn test_auto_transition_condition() {
        let flag = Arc::new(Mutex::new(false));
        let f = Arc::clone(&flag);
        let mut sm = make_sm();
        sm.add_transition(Transition::new("Idle", "Run", move || *f.lock().unwrap()));
        sm.start("Idle");
        sm.update(0.0);
        assert!(sm.is_in("Idle"));
        *flag.lock().unwrap() = true;
        sm.update(0.0);
        assert!(sm.is_in("Run"));
    }

    #[test]
    fn test_global_guard_blocks_transition() {
        let mut sm = make_sm();
        sm.set_global_guard(|_from, to| to != "Attack");
        sm.start("Idle");
        let r = sm.transition_to("Attack");
        assert_eq!(r, TransitionResult::Rejected);
        assert!(sm.is_in("Idle"));
    }

    #[test]
    fn test_transition_count() {
        let mut sm = make_sm();
        sm.start("Idle");
        sm.transition_to("Run");
        sm.transition_to("Attack");
        assert_eq!(sm.get_transition_count(), 2);
    }

    #[test]
    fn test_history() {
        let mut sm = make_sm();
        sm.start("Idle");
        sm.transition_to("Run");
        sm.transition_to("Attack");
        assert_eq!(sm.get_history(), &["Idle", "Run", "Attack"]);
    }

    #[test]
    fn test_has_state() {
        let sm = make_sm();
        assert!(sm.has_state("Idle"));
        assert!(!sm.has_state("Ghost"));
    }

    #[test]
    fn test_any_transition() {
        let fired = Arc::new(Mutex::new(false));
        let f = Arc::clone(&fired);
        let mut sm = make_sm();
        sm.add_any_transition("Attack", move || *f.lock().unwrap(), 100);
        sm.start("Run");
        sm.update(0.0);
        assert!(sm.is_in("Run"));
        *fired.lock().unwrap() = true;
        sm.update(0.0);
        assert!(sm.is_in("Attack"));
    }

    #[test]
    fn test_get_state_downcast() {
        let mut sm = make_sm();
        sm.start("Run");
        let run = sm.get_state::<RunState>("Run");
        assert!(run.is_some());
        assert!((run.unwrap().speed - 5.0).abs() < 1e-6);
    }
}
