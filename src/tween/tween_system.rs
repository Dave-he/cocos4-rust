use crate::tween::tween::{Tween, TweenState};

pub struct TweenSystem {
    tweens: Vec<Tween>,
}

impl TweenSystem {
    pub fn new() -> Self {
        TweenSystem { tweens: Vec::new() }
    }

    pub fn add(&mut self, tween: Tween) {
        self.tweens.push(tween);
    }

    pub fn update(&mut self, dt: f32) {
        for t in self.tweens.iter_mut() {
            if t.is_running() {
                t.update(dt);
            }
        }
        self.tweens.retain(|t| t.get_state() != TweenState::Finished);
    }

    pub fn stop_all(&mut self) {
        for t in self.tweens.iter_mut() {
            t.stop();
        }
        self.tweens.clear();
    }

    pub fn pause_all(&mut self) {
        for t in self.tweens.iter_mut() {
            t.pause();
        }
    }

    pub fn resume_all(&mut self) {
        for t in self.tweens.iter_mut() {
            t.resume();
        }
    }

    pub fn count(&self) -> usize {
        self.tweens.len()
    }

    pub fn stop_by_id(&mut self, id: u64) {
        self.tweens.retain(|t| t.get_id() != id);
    }
}

impl Default for TweenSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tween::tween::{tween, TweenState};
    use crate::tween::easing::EasingMethod;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_tween_system_update() {
        let mut sys = TweenSystem::new();
        let t = tween()
            .to_single(0.1, "x", 0.0, 1.0, EasingMethod::Linear)
            .start();
        sys.add(t);
        assert_eq!(sys.count(), 1);
        sys.update(1.0);
        assert_eq!(sys.count(), 0);
    }

    #[test]
    fn test_tween_system_stop_all() {
        let mut sys = TweenSystem::new();
        for _ in 0..3 {
            let t = tween()
                .to_single(10.0, "x", 0.0, 1.0, EasingMethod::Linear)
                .start();
            sys.add(t);
        }
        sys.stop_all();
        assert_eq!(sys.count(), 0);
    }

    #[test]
    fn test_tween_system_pause_resume() {
        let mut sys = TweenSystem::new();
        let t = tween()
            .to_single(1.0, "x", 0.0, 1.0, EasingMethod::Linear)
            .start();
        sys.add(t);
        sys.pause_all();
        sys.update(2.0);
        assert_eq!(sys.count(), 1);
        sys.resume_all();
        sys.update(2.0);
        assert_eq!(sys.count(), 0);
    }

    #[test]
    fn test_tween_system_stop_by_id() {
        let mut sys = TweenSystem::new();
        let t = tween()
            .to_single(10.0, "x", 0.0, 1.0, EasingMethod::Linear)
            .start();
        let id = t.get_id();
        sys.add(t);
        sys.stop_by_id(id);
        assert_eq!(sys.count(), 0);
    }

    #[test]
    fn test_tween_system_multi_tweens() {
        let count = Arc::new(Mutex::new(0u32));
        let mut sys = TweenSystem::new();
        for _ in 0..5 {
            let c = Arc::clone(&count);
            let t = tween()
                .to_single(0.1, "x", 0.0, 1.0, EasingMethod::Linear)
                .on_complete(move || { *c.lock().unwrap() += 1; })
                .start();
            sys.add(t);
        }
        sys.update(1.0);
        assert_eq!(*count.lock().unwrap(), 5);
        assert_eq!(sys.count(), 0);
    }
}
