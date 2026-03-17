use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type ScheduleCallback = Arc<dyn Fn(f32) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedulePriority {
    SystemMin = 0,
    NonSystemMin = 1,
}

pub const CC_REPEAT_FOREVER: u32 = u32::MAX;

pub struct TimerInfo {
    pub key: String,
    pub elapsed: f32,
    pub run_forever: bool,
    pub use_delay: bool,
    pub times_executed: u32,
    pub repeat: u32,
    pub delay: f32,
    pub interval: f32,
    pub paused: bool,
    pub callback: Arc<dyn Fn(f32) + Send + Sync>,
}

impl TimerInfo {
    pub fn new(
        key: String,
        callback: Arc<dyn Fn(f32) + Send + Sync>,
        interval: f32,
        repeat: u32,
        delay: f32,
        paused: bool,
    ) -> Self {
        TimerInfo {
            key,
            elapsed: -1.0,
            run_forever: repeat == CC_REPEAT_FOREVER,
            use_delay: delay > 0.0,
            times_executed: 0,
            repeat,
            delay,
            interval,
            paused,
            callback,
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if self.paused {
            return true;
        }

        if self.elapsed < 0.0 {
            self.elapsed = 0.0;
        }

        if self.use_delay {
            self.elapsed += dt;
            if self.elapsed >= self.delay {
                self.use_delay = false;
                (self.callback)(self.elapsed);
                self.times_executed += 1;
                self.elapsed = 0.0;
            }
            return true;
        }

        if self.interval == 0.0 {
            (self.callback)(dt);
            self.times_executed += 1;
        } else {
            self.elapsed += dt;
            if self.elapsed >= self.interval {
                (self.callback)(self.elapsed);
                self.times_executed += 1;
                self.elapsed = 0.0;
            }
        }

        if self.run_forever {
            return true;
        }

        self.times_executed <= self.repeat
    }
}

pub struct UpdateEntry {
    pub callback: ScheduleCallback,
    pub priority: i32,
    pub paused: bool,
    pub marked_for_removal: bool,
}

pub struct Scheduler {
    timers: HashMap<String, TimerInfo>,
    update_handlers: Vec<UpdateEntry>,
    functions_to_perform: Vec<Box<dyn Fn() + Send + Sync>>,
    perform_mutex: Arc<Mutex<()>>,
    timers_to_add: Vec<TimerInfo>,
    timers_to_remove: Vec<String>,
    update_locked: bool,
    time_scale: f32,
    paused: bool,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            timers: HashMap::new(),
            update_handlers: Vec::new(),
            functions_to_perform: Vec::new(),
            perform_mutex: Arc::new(Mutex::new(())),
            timers_to_add: Vec::new(),
            timers_to_remove: Vec::new(),
            update_locked: false,
            time_scale: 1.0,
            paused: false,
        }
    }

    pub fn get_time_scale(&self) -> f32 {
        self.time_scale
    }

    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale;
    }

    pub fn pause_all(&mut self) {
        self.paused = true;
    }

    pub fn resume_all(&mut self) {
        self.paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause_all_targets(&mut self) {
        for timer in self.timers.values_mut() {
            timer.paused = true;
        }
        for handler in self.update_handlers.iter_mut() {
            handler.paused = true;
        }
    }

    pub fn resume_all_targets(&mut self) {
        for timer in self.timers.values_mut() {
            timer.paused = false;
        }
        for handler in self.update_handlers.iter_mut() {
            handler.paused = false;
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }
        let scaled_dt = dt * self.time_scale;
        self.update_locked = true;

        let keys: Vec<String> = self.timers.keys().cloned().collect();
        let mut expired: Vec<String> = Vec::new();

        for key in &keys {
            if let Some(timer) = self.timers.get_mut(key) {
                if !timer.update(scaled_dt) {
                    expired.push(key.clone());
                }
            }
        }

        for key in expired {
            self.timers.remove(&key);
        }

        let handlers: Vec<ScheduleCallback> = self
            .update_handlers
            .iter()
            .filter(|h| !h.paused && !h.marked_for_removal)
            .map(|h| Arc::clone(&h.callback))
            .collect();

        for cb in handlers {
            cb(scaled_dt);
        }

        self.update_handlers.retain(|h| !h.marked_for_removal);

        self.update_locked = false;

        let to_add: Vec<TimerInfo> = self.timers_to_add.drain(..).collect();
        for timer in to_add {
            self.timers.insert(timer.key.clone(), timer);
        }

        let to_remove: Vec<String> = self.timers_to_remove.drain(..).collect();
        for key in to_remove {
            self.timers.remove(&key);
        }

        self.run_functions_to_perform();
    }

    pub fn schedule(
        &mut self,
        callback: Arc<dyn Fn(f32) + Send + Sync>,
        interval: f32,
        repeat: u32,
        delay: f32,
        paused: bool,
        key: String,
    ) {
        let timer = TimerInfo::new(key.clone(), callback, interval, repeat, delay, paused);
        if self.update_locked {
            self.timers_to_add.push(timer);
        } else {
            self.timers.insert(key, timer);
        }
    }

    pub fn schedule_forever(
        &mut self,
        callback: Arc<dyn Fn(f32) + Send + Sync>,
        interval: f32,
        paused: bool,
        key: String,
    ) {
        self.schedule(callback, interval, CC_REPEAT_FOREVER, 0.0, paused, key);
    }

    pub fn schedule_once(
        &mut self,
        callback: Arc<dyn Fn(f32) + Send + Sync>,
        delay: f32,
        key: String,
    ) {
        self.schedule(callback, 0.0, 0, delay, false, key);
    }

    pub fn unschedule(&mut self, key: &str) {
        if self.update_locked {
            self.timers_to_remove.push(key.to_string());
        } else {
            self.timers.remove(key);
        }
    }

    pub fn unschedule_all(&mut self) {
        if self.update_locked {
            let keys: Vec<String> = self.timers.keys().cloned().collect();
            self.timers_to_remove.extend(keys);
        } else {
            self.timers.clear();
        }
    }

    pub fn is_scheduled(&self, key: &str) -> bool {
        self.timers.contains_key(key)
    }

    pub fn pause_target(&mut self, key: &str) {
        if let Some(timer) = self.timers.get_mut(key) {
            timer.paused = true;
        }
    }

    pub fn resume_target(&mut self, key: &str) {
        if let Some(timer) = self.timers.get_mut(key) {
            timer.paused = false;
        }
    }

    pub fn is_target_paused(&self, key: &str) -> bool {
        self.timers.get(key).map_or(false, |t| t.paused)
    }

    pub fn schedule_update(
        &mut self,
        callback: Arc<dyn Fn(f32) + Send + Sync>,
        priority: i32,
        paused: bool,
    ) {
        let pos = self
            .update_handlers
            .iter()
            .position(|h| h.priority > priority)
            .unwrap_or(self.update_handlers.len());

        self.update_handlers.insert(
            pos,
            UpdateEntry {
                callback,
                priority,
                paused,
                marked_for_removal: false,
            },
        );
    }

    pub fn unschedule_update(&mut self, priority: i32) {
        if let Some(entry) = self.update_handlers.iter_mut().find(|h| h.priority == priority) {
            if self.update_locked {
                entry.marked_for_removal = true;
            } else {
                self.update_handlers.retain(|h| h.priority != priority);
            }
        }
    }

    pub fn perform_function_in_cocos_thread(&mut self, function: Box<dyn Fn() + Send + Sync>) {
        let _lock = self.perform_mutex.lock().unwrap();
        self.functions_to_perform.push(function);
    }

    pub fn remove_all_functions_to_be_performed_in_cocos_thread(&mut self) {
        let _lock = self.perform_mutex.lock().unwrap();
        self.functions_to_perform.clear();
    }

    fn run_functions_to_perform(&mut self) {
        let functions: Vec<Box<dyn Fn() + Send + Sync>> = {
            let _lock = self.perform_mutex.lock().unwrap();
            self.functions_to_perform.drain(..).collect()
        };

        for function in functions {
            function();
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn test_scheduler_new() {
        let scheduler = Scheduler::new();
        assert!(!scheduler.update_locked);
    }

    #[test]
    fn test_schedule_and_unschedule() {
        let mut scheduler = Scheduler::new();
        let key = "test".to_string();

        scheduler.schedule(
            Arc::new(|_| {}),
            1.0,
            5,
            0.0,
            false,
            key.clone(),
        );
        assert!(scheduler.is_scheduled(&key));

        scheduler.unschedule(&key);
        assert!(!scheduler.is_scheduled(&key));
    }

    #[test]
    fn test_schedule_callback_executes() {
        let mut scheduler = Scheduler::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.schedule_forever(
            Arc::new(move |_dt| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            0.0,
            false,
            "tick".to_string(),
        );

        scheduler.update(0.016);
        scheduler.update(0.016);
        scheduler.update(0.016);

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_schedule_with_interval() {
        let mut scheduler = Scheduler::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.schedule_forever(
            Arc::new(move |_dt| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            0.1,
            false,
            "interval_test".to_string(),
        );

        scheduler.update(0.05);
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        scheduler.update(0.06);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_schedule_limited_repeat() {
        let mut scheduler = Scheduler::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.schedule(
            Arc::new(move |_dt| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            0.0,
            2,
            0.0,
            false,
            "limited".to_string(),
        );

        for _ in 0..5 {
            scheduler.update(0.016);
        }

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_pause_resume_target() {
        let mut scheduler = Scheduler::new();
        let key = "test".to_string();

        scheduler.schedule_forever(Arc::new(|_| {}), 0.0, false, key.clone());
        scheduler.pause_target(&key);
        assert!(scheduler.is_target_paused(&key));

        scheduler.resume_target(&key);
        assert!(!scheduler.is_target_paused(&key));
    }

    #[test]
    fn test_schedule_update() {
        let mut scheduler = Scheduler::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.schedule_update(
            Arc::new(move |_dt| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            0,
            false,
        );

        scheduler.update(0.016);
        scheduler.update(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_perform_function_in_cocos_thread() {
        let mut scheduler = Scheduler::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.perform_function_in_cocos_thread(Box::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }));

        scheduler.update(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_time_scale() {
        let mut scheduler = Scheduler::new();
        let elapsed = Arc::new(std::sync::Mutex::new(0.0f32));
        let elapsed_clone = Arc::clone(&elapsed);

        scheduler.schedule_forever(
            Arc::new(move |dt| {
                *elapsed_clone.lock().unwrap() += dt;
            }),
            0.0,
            false,
            "ts_test".to_string(),
        );

        scheduler.set_time_scale(2.0);
        assert!((scheduler.get_time_scale() - 2.0).abs() < 1e-6);
        scheduler.update(0.1);
        assert!((*elapsed.lock().unwrap() - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_pause_all_resume_all() {
        let mut scheduler = Scheduler::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.schedule_forever(
            Arc::new(move |_dt| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            0.0,
            false,
            "pause_test".to_string(),
        );

        scheduler.pause_all();
        assert!(scheduler.is_paused());
        scheduler.update(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        scheduler.resume_all();
        assert!(!scheduler.is_paused());
        scheduler.update(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_pause_all_targets_resume_all_targets() {
        let mut scheduler = Scheduler::new();
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);

        scheduler.schedule_forever(
            Arc::new(move |_dt| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }),
            0.0,
            false,
            "pat_test".to_string(),
        );

        scheduler.pause_all_targets();
        scheduler.update(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        scheduler.resume_all_targets();
        scheduler.update(0.016);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
