use std::collections::HashMap;
use crate::tween::easing::EasingMethod;

pub type ProgressFn = Box<dyn Fn(f32, f32, f32) -> f32 + Send + Sync>;

#[derive(Debug, Clone)]
pub struct PropValue {
    pub from: f32,
    pub to: f32,
}

pub struct TweenAction {
    pub duration: f32,
    pub elapsed: f32,
    pub easing: EasingMethod,
    pub relative: bool,
    props: HashMap<String, PropValue>,
    progress_fn: Option<ProgressFn>,
    on_update: Option<Box<dyn Fn(&HashMap<String, f32>) + Send + Sync>>,
    on_complete: Option<Box<dyn Fn() + Send + Sync>>,
}

impl TweenAction {
    pub fn new(duration: f32, easing: EasingMethod) -> Self {
        TweenAction {
            duration,
            elapsed: 0.0,
            easing,
            relative: false,
            props: HashMap::new(),
            progress_fn: None,
            on_update: None,
            on_complete: None,
        }
    }

    pub fn set_props(&mut self, props: HashMap<String, PropValue>) {
        self.props = props;
    }

    pub fn add_prop(&mut self, key: &str, from: f32, to: f32) {
        self.props.insert(key.to_string(), PropValue { from, to });
    }

    pub fn set_relative(&mut self, rel: bool) {
        self.relative = rel;
    }

    pub fn set_progress_fn(&mut self, f: ProgressFn) {
        self.progress_fn = Some(f);
    }

    pub fn on_update<F: Fn(&HashMap<String, f32>) + Send + Sync + 'static>(&mut self, f: F) {
        self.on_update = Some(Box::new(f));
    }

    pub fn on_complete<F: Fn() + Send + Sync + 'static>(&mut self, f: F) {
        self.on_complete = Some(Box::new(f));
    }

    pub fn is_done(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    pub fn update(&mut self, dt: f32) -> bool {
        self.elapsed += dt;
        let t = if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        };

        let eased = if let Some(ref pf) = self.progress_fn {
            pf(0.0, 1.0, t)
        } else {
            self.easing.apply(t)
        };

        let mut current: HashMap<String, f32> = HashMap::new();
        for (key, pv) in &self.props {
            let val = pv.from + (pv.to - pv.from) * eased;
            current.insert(key.clone(), val);
        }

        if let Some(ref cb) = self.on_update {
            cb(&current);
        }

        let done = self.is_done();
        if done {
            if let Some(ref cb) = self.on_complete {
                cb();
            }
        }
        done
    }

    pub fn get_current_values(&self) -> HashMap<String, f32> {
        let t = if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        };
        let eased = self.easing.apply(t);
        let mut result = HashMap::new();
        for (key, pv) in &self.props {
            result.insert(key.clone(), pv.from + (pv.to - pv.from) * eased);
        }
        result
    }
}

impl std::fmt::Debug for TweenAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TweenAction")
            .field("duration", &self.duration)
            .field("elapsed", &self.elapsed)
            .field("easing", &self.easing)
            .field("relative", &self.relative)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tween_action_new() {
        let action = TweenAction::new(1.0, EasingMethod::Linear);
        assert!(!action.is_done());
        assert!((action.duration - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_tween_action_update() {
        let mut action = TweenAction::new(1.0, EasingMethod::Linear);
        action.add_prop("x", 0.0, 100.0);
        action.update(0.5);
        let vals = action.get_current_values();
        assert!((vals["x"] - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_tween_action_done() {
        let mut action = TweenAction::new(0.5, EasingMethod::Linear);
        let done = action.update(1.0);
        assert!(done);
        assert!(action.is_done());
    }

    #[test]
    fn test_tween_action_on_complete() {
        use std::sync::{Arc, Mutex};
        let mut action = TweenAction::new(0.1, EasingMethod::Linear);
        let fired = Arc::new(Mutex::new(false));
        let f = Arc::clone(&fired);
        action.on_complete(move || { *f.lock().unwrap() = true; });
        action.update(1.0);
        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn test_tween_action_reset() {
        let mut action = TweenAction::new(1.0, EasingMethod::Linear);
        action.update(1.0);
        assert!(action.is_done());
        action.reset();
        assert!(!action.is_done());
    }
}
