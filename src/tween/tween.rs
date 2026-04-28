use std::collections::HashMap;
use crate::tween::tween_action::TweenAction;
use crate::tween::easing::EasingMethod;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenState {
    Idle,
    Running,
    Paused,
    Finished,
}

enum TweenStep {
    Action(TweenAction),
    Delay(f32, f32),
    Call(Box<dyn Fn() + Send + Sync>),
    #[allow(dead_code)]
    Sequence(Vec<TweenStep>),
    #[allow(dead_code)]
    Parallel(Vec<TweenStep>),
    Repeat(Box<TweenStep>, u32, u32),
    RepeatForever(Box<TweenStep>, ()),
}

impl std::fmt::Debug for TweenStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TweenStep::Action(_) => write!(f, "Action"),
            TweenStep::Delay(total, _) => write!(f, "Delay({})", total),
            TweenStep::Call(_) => write!(f, "Call"),
            TweenStep::Sequence(steps) => write!(f, "Sequence({})", steps.len()),
            TweenStep::Parallel(steps) => write!(f, "Parallel({})", steps.len()),
            TweenStep::Repeat(_, total, _) => write!(f, "Repeat({})", total),
            TweenStep::RepeatForever(_, _) => write!(f, "RepeatForever"),
        }
    }
}

impl TweenStep {
    fn update(&mut self, dt: f32) -> bool {
        match self {
            TweenStep::Action(action) => action.update(dt),
            TweenStep::Delay(total, elapsed) => {
                *elapsed += dt;
                *elapsed >= *total
            }
            TweenStep::Call(f) => {
                f();
                true
            }
            TweenStep::Sequence(steps) => {
                if steps.is_empty() { return true; }
                let done = steps[0].update(dt);
                if done {
                    steps.remove(0);
                }
                steps.is_empty()
            }
            TweenStep::Parallel(steps) => {
                let mut all_done = true;
                for step in steps.iter_mut() {
                    if !step.is_done() {
                        step.update(dt);
                    }
                    if !step.is_done() {
                        all_done = false;
                    }
                }
                all_done
            }
            TweenStep::Repeat(step, total, current) => {
                let done = step.update(dt);
                if done {
                    *current += 1;
                    if *current < *total {
                        step.reset();
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            TweenStep::RepeatForever(step, _) => {
                let done = step.update(dt);
                if done {
                    step.reset();
                }
                false
            }
        }
    }

    fn is_done(&self) -> bool {
        match self {
            TweenStep::Action(a) => a.is_done(),
            TweenStep::Delay(total, elapsed) => *elapsed >= *total,
            TweenStep::Call(_) => false,
            TweenStep::Sequence(steps) => steps.is_empty(),
            TweenStep::Parallel(steps) => steps.iter().all(|s| s.is_done()),
            TweenStep::Repeat(_, total, current) => *current >= *total,
            TweenStep::RepeatForever(_, _) => false,
        }
    }

    fn reset(&mut self) {
        match self {
            TweenStep::Action(a) => a.reset(),
            TweenStep::Delay(_, elapsed) => *elapsed = 0.0,
            TweenStep::Call(_) => {}
            TweenStep::Sequence(_) => {}
            TweenStep::Parallel(steps) => {
                for s in steps.iter_mut() { s.reset(); }
            }
            TweenStep::Repeat(step, _, current) => {
                *current = 0;
                step.reset();
            }
            TweenStep::RepeatForever(step, _) => step.reset(),
        }
    }
}

pub struct Tween {
    steps: Vec<TweenStep>,
    current_step: usize,
    state: TweenState,
    on_complete: Option<Box<dyn Fn() + Send + Sync>>,
    on_start: Option<Box<dyn Fn() + Send + Sync>>,
    id: u64,
}

static TWEEN_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Tween {
    pub fn new() -> Self {
        Tween {
            steps: Vec::new(),
            current_step: 0,
            state: TweenState::Idle,
            on_complete: None,
            on_start: None,
            id: TWEEN_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn to(mut self, duration: f32, props: HashMap<String, (f32, f32)>, easing: EasingMethod) -> Self {
        let mut action = TweenAction::new(duration, easing);
        for (key, (from, to)) in props {
            action.add_prop(&key, from, to);
        }
        self.steps.push(TweenStep::Action(action));
        self
    }

    pub fn to_single(mut self, duration: f32, key: &str, from: f32, to: f32, easing: EasingMethod) -> Self {
        let mut action = TweenAction::new(duration, easing);
        action.add_prop(key, from, to);
        self.steps.push(TweenStep::Action(action));
        self
    }

    pub fn by(mut self, duration: f32, props: HashMap<String, f32>, easing: EasingMethod) -> Self {
        let mut action = TweenAction::new(duration, easing);
        action.set_relative(true);
        for (key, delta) in props {
            action.add_prop(&key, 0.0, delta);
        }
        self.steps.push(TweenStep::Action(action));
        self
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.steps.push(TweenStep::Delay(seconds, 0.0));
        self
    }

    pub fn call<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.steps.push(TweenStep::Call(Box::new(f)));
        self
    }

    pub fn repeat(mut self, times: u32) -> Self {
        if let Some(last) = self.steps.pop() {
            self.steps.push(TweenStep::Repeat(Box::new(last), times, 0));
        }
        self
    }

    pub fn repeat_forever(mut self) -> Self {
        if let Some(last) = self.steps.pop() {
            self.steps.push(TweenStep::RepeatForever(Box::new(last), ()));
        }
        self
    }

    pub fn on_complete<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_complete = Some(Box::new(f));
        self
    }

    pub fn on_start<F: Fn() + Send + Sync + 'static>(mut self, f: F) -> Self {
        self.on_start = Some(Box::new(f));
        self
    }

    pub fn start(mut self) -> Self {
        self.state = TweenState::Running;
        if let Some(ref f) = self.on_start {
            f();
        }
        self
    }

    pub fn pause(&mut self) {
        if self.state == TweenState::Running {
            self.state = TweenState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == TweenState::Paused {
            self.state = TweenState::Running;
        }
    }

    pub fn stop(&mut self) {
        self.state = TweenState::Finished;
    }

    pub fn get_state(&self) -> TweenState {
        self.state
    }

    pub fn is_running(&self) -> bool {
        self.state == TweenState::Running
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if self.state != TweenState::Running {
            return self.state == TweenState::Finished;
        }

        while self.current_step < self.steps.len() {
            let done = self.steps[self.current_step].update(dt);
            if done {
                self.current_step += 1;
            } else {
                break;
            }
        }

        if self.current_step >= self.steps.len() {
            self.state = TweenState::Finished;
            if let Some(ref f) = self.on_complete {
                f();
            }
            return true;
        }
        false
    }
}

impl Default for Tween {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Tween {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tween")
            .field("id", &self.id)
            .field("state", &self.state)
            .field("steps", &self.steps.len())
            .field("current_step", &self.current_step)
            .finish()
    }
}

pub fn tween() -> Tween {
    Tween::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_tween_basic_to() {
        let mut t = tween()
            .to_single(1.0, "x", 0.0, 100.0, EasingMethod::Linear)
            .start();
        assert_eq!(t.get_state(), TweenState::Running);
        t.update(0.5);
        assert_eq!(t.get_state(), TweenState::Running);
        t.update(0.5);
        assert_eq!(t.get_state(), TweenState::Finished);
    }

    #[test]
    fn test_tween_delay() {
        let mut t = tween()
            .delay(0.5)
            .to_single(0.5, "x", 0.0, 10.0, EasingMethod::Linear)
            .start();
        t.update(0.4);
        assert_eq!(t.get_state(), TweenState::Running);
        t.update(0.2);
        assert_eq!(t.get_state(), TweenState::Running);
        t.update(0.5);
        assert_eq!(t.get_state(), TweenState::Finished);
    }

    #[test]
    fn test_tween_call() {
        let called = Arc::new(Mutex::new(false));
        let c = Arc::clone(&called);
        let mut t = tween()
            .call(move || { *c.lock().unwrap() = true; })
            .start();
        t.update(0.0);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_tween_on_complete() {
        let done = Arc::new(Mutex::new(false));
        let d = Arc::clone(&done);
        let mut t = tween()
            .to_single(0.1, "x", 0.0, 1.0, EasingMethod::Linear)
            .on_complete(move || { *d.lock().unwrap() = true; })
            .start();
        t.update(1.0);
        assert!(*done.lock().unwrap());
    }

    #[test]
    fn test_tween_pause_resume() {
        let mut t = tween()
            .to_single(1.0, "x", 0.0, 100.0, EasingMethod::Linear)
            .start();
        t.pause();
        assert_eq!(t.get_state(), TweenState::Paused);
        t.update(10.0);
        assert_eq!(t.get_state(), TweenState::Paused);
        t.resume();
        t.update(1.0);
        assert_eq!(t.get_state(), TweenState::Finished);
    }

    #[test]
    fn test_tween_stop() {
        let mut t = tween()
            .to_single(1.0, "x", 0.0, 100.0, EasingMethod::Linear)
            .start();
        t.stop();
        assert_eq!(t.get_state(), TweenState::Finished);
    }

    #[test]
    fn test_tween_on_start() {
        let started = Arc::new(Mutex::new(false));
        let s = Arc::clone(&started);
        let _t = tween()
            .to_single(1.0, "x", 0.0, 10.0, EasingMethod::Linear)
            .on_start(move || { *s.lock().unwrap() = true; })
            .start();
        assert!(*started.lock().unwrap());
    }

    #[test]
    fn test_tween_unique_ids() {
        let t1 = tween();
        let t2 = tween();
        assert_ne!(t1.get_id(), t2.get_id());
    }

    #[test]
    fn test_tween_multi_steps() {
        let steps_done = Arc::new(Mutex::new(0u32));
        let s = Arc::clone(&steps_done);
        let s2 = Arc::clone(&steps_done);
        let mut t = tween()
            .to_single(0.5, "x", 0.0, 50.0, EasingMethod::Linear)
            .call(move || { *s.lock().unwrap() += 1; })
            .to_single(0.5, "x", 50.0, 100.0, EasingMethod::Linear)
            .call(move || { *s2.lock().unwrap() += 1; })
            .start();
        t.update(2.0);
        assert_eq!(*steps_done.lock().unwrap(), 2);
        assert_eq!(t.get_state(), TweenState::Finished);
    }
}
