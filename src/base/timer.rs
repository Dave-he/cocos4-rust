use std::sync::{Arc, Mutex};
use crate::base::scheduler::{Scheduler, CC_REPEAT_FOREVER};

static TIMER_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_timer_id() -> u64 {
    TIMER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Running,
    Paused,
    Finished,
    Cancelled,
}

pub struct TimerHandle {
    pub id: u64,
    key: String,
    scheduler: Arc<Mutex<Scheduler>>,
}

impl TimerHandle {
    fn new(id: u64, key: String, scheduler: Arc<Mutex<Scheduler>>) -> Self {
        TimerHandle { id, key, scheduler }
    }

    pub fn cancel(&self) {
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.unschedule(&self.key);
        }
    }

    pub fn pause(&self) {
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.pause_target(&self.key);
        }
    }

    pub fn resume(&self) {
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.resume_target(&self.key);
        }
    }

    pub fn is_active(&self) -> bool {
        if let Ok(sched) = self.scheduler.lock() {
            sched.is_scheduled(&self.key)
        } else {
            false
        }
    }
}

pub struct TimerManager {
    scheduler: Arc<Mutex<Scheduler>>,
}

impl TimerManager {
    pub fn new(scheduler: Arc<Mutex<Scheduler>>) -> Self {
        TimerManager { scheduler }
    }

    pub fn schedule<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        interval: f32,
        callback: F,
    ) -> TimerHandle {
        let id = next_timer_id();
        let key = format!("timer-{}", id);
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.schedule_forever(Arc::new(callback), interval, false, key.clone());
        }
        TimerHandle::new(id, key, Arc::clone(&self.scheduler))
    }

    pub fn schedule_once<F: Fn() + Send + Sync + 'static>(
        &self,
        delay: f32,
        callback: F,
    ) -> TimerHandle {
        let id = next_timer_id();
        let key = format!("timer-{}", id);
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.schedule_once(
                Arc::new(move |_dt| callback()),
                delay,
                key.clone(),
            );
        }
        TimerHandle::new(id, key, Arc::clone(&self.scheduler))
    }

    pub fn schedule_repeat<F: Fn(u32) + Send + Sync + 'static>(
        &self,
        interval: f32,
        times: u32,
        callback: F,
    ) -> TimerHandle {
        let id = next_timer_id();
        let key = format!("timer-{}", id);
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let cb_counter = Arc::clone(&counter);
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.schedule(
                Arc::new(move |_dt| {
                    let n = cb_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    callback(n);
                }),
                interval,
                times.saturating_sub(1),
                0.0,
                false,
                key.clone(),
            );
        }
        TimerHandle::new(id, key, Arc::clone(&self.scheduler))
    }

    pub fn countdown<F: Fn(f32) + Send + Sync + 'static>(
        &self,
        total: f32,
        tick_interval: f32,
        callback: F,
    ) -> TimerHandle {
        let id = next_timer_id();
        let key = format!("timer-{}", id);
        let elapsed = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let elapsed_cb = Arc::clone(&elapsed);
        let total_ticks = (total / tick_interval).ceil() as u32;
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.schedule(
                Arc::new(move |_dt| {
                    let n = elapsed_cb.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    let remaining = total - (n as f32 * tick_interval);
                    callback(remaining.max(0.0));
                }),
                tick_interval,
                total_ticks.saturating_sub(1),
                0.0,
                false,
                key.clone(),
            );
        }
        TimerHandle::new(id, key, Arc::clone(&self.scheduler))
    }

    pub fn update(&self, dt: f32) {
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.update(dt);
        }
    }

    pub fn cancel_all(&self) {
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.unschedule_all();
        }
    }

    pub fn set_time_scale(&self, scale: f32) {
        if let Ok(mut sched) = self.scheduler.lock() {
            sched.set_time_scale(scale);
        }
    }

    pub fn get_time_scale(&self) -> f32 {
        if let Ok(sched) = self.scheduler.lock() {
            sched.get_time_scale()
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn make_timer_manager() -> (TimerManager, Arc<Mutex<Scheduler>>) {
        let sched = Arc::new(Mutex::new(Scheduler::new()));
        let tm = TimerManager::new(Arc::clone(&sched));
        (tm, sched)
    }

    #[test]
    fn test_schedule_fires_on_interval() {
        let (tm, _sched) = make_timer_manager();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let _handle = tm.schedule(0.1, move |_dt| {
            *c.lock().unwrap() += 1;
        });
        tm.update(0.1);
        tm.update(0.1);
        tm.update(0.1);
        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn test_schedule_once_fires_once() {
        let (tm, _sched) = make_timer_manager();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        tm.schedule_once(0.0, move || {
            *c.lock().unwrap() += 1;
        });
        tm.update(0.016);
        tm.update(0.016);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_cancel_handle() {
        let (tm, _sched) = make_timer_manager();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let handle = tm.schedule(0.1, move |_| {
            *c.lock().unwrap() += 1;
        });
        tm.update(0.1);
        assert_eq!(*count.lock().unwrap(), 1);
        handle.cancel();
        tm.update(0.1);
        tm.update(0.1);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_pause_resume_handle() {
        let (tm, _sched) = make_timer_manager();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let handle = tm.schedule(0.0, move |_| {
            *c.lock().unwrap() += 1;
        });
        handle.pause();
        tm.update(0.016);
        assert_eq!(*count.lock().unwrap(), 0);
        handle.resume();
        tm.update(0.016);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_handle_is_active() {
        let (tm, _sched) = make_timer_manager();
        let handle = tm.schedule(0.0, |_| {});
        assert!(handle.is_active());
        handle.cancel();
        tm.update(0.016);
        assert!(!handle.is_active());
    }

    #[test]
    fn test_schedule_repeat() {
        let (tm, _sched) = make_timer_manager();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let _handle = tm.schedule_repeat(0.0, 3, move |_n| {
            *c.lock().unwrap() += 1;
        });
        for _ in 0..5 {
            tm.update(0.016);
        }
        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn test_cancel_all() {
        let (tm, _sched) = make_timer_manager();
        let count = Arc::new(Mutex::new(0u32));
        for _ in 0..3 {
            let c = Arc::clone(&count);
            tm.schedule(0.0, move |_| {
                *c.lock().unwrap() += 1;
            });
        }
        tm.cancel_all();
        tm.update(0.016);
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn test_time_scale() {
        let (tm, _sched) = make_timer_manager();
        tm.set_time_scale(2.0);
        assert!((tm.get_time_scale() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_countdown() {
        let (tm, _sched) = make_timer_manager();
        let ticks = Arc::new(Mutex::new(Vec::<f32>::new()));
        let t = Arc::clone(&ticks);
        let _handle = tm.countdown(0.3, 0.1, move |remaining| {
            t.lock().unwrap().push(remaining);
        });
        tm.update(0.1);
        tm.update(0.1);
        tm.update(0.1);
        let v = ticks.lock().unwrap();
        assert!(v.len() >= 2);
    }

    #[test]
    fn test_unique_timer_ids() {
        let (tm, _sched) = make_timer_manager();
        let h1 = tm.schedule(0.1, |_| {});
        let h2 = tm.schedule(0.1, |_| {});
        assert_ne!(h1.id, h2.id);
    }
}
