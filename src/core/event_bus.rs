use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type EventBusKey = u64;
static BUS_LISTENER_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_bus_id() -> EventBusKey {
    BUS_LISTENER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

type BusCallback = Arc<dyn Fn(&dyn std::any::Any) + Send + Sync>;

struct BusEntry {
    id: EventBusKey,
    once: bool,
    callback: BusCallback,
}

pub struct EventBus {
    channels: HashMap<String, Vec<BusEntry>>,
    history: Vec<(String, u64)>,
    max_history: usize,
    emit_count: u64,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            channels: HashMap::new(),
            history: Vec::new(),
            max_history: 100,
            emit_count: 0,
        }
    }

    pub fn on<T: std::any::Any + Send + Sync + 'static, F: Fn(&T) + Send + Sync + 'static>(
        &mut self,
        event: &str,
        callback: F,
    ) -> EventBusKey {
        let id = next_bus_id();
        let cb: BusCallback = Arc::new(move |any| {
            if let Some(v) = any.downcast_ref::<T>() {
                callback(v);
            }
        });
        self.channels.entry(event.to_string()).or_default().push(BusEntry {
            id,
            once: false,
            callback: cb,
        });
        id
    }

    pub fn once<T: std::any::Any + Send + Sync + 'static, F: Fn(&T) + Send + Sync + 'static>(
        &mut self,
        event: &str,
        callback: F,
    ) -> EventBusKey {
        let id = next_bus_id();
        let cb: BusCallback = Arc::new(move |any| {
            if let Some(v) = any.downcast_ref::<T>() {
                callback(v);
            }
        });
        self.channels.entry(event.to_string()).or_default().push(BusEntry {
            id,
            once: true,
            callback: cb,
        });
        id
    }

    pub fn off(&mut self, event: &str, key: EventBusKey) {
        if let Some(entries) = self.channels.get_mut(event) {
            entries.retain(|e| e.id != key);
        }
    }

    pub fn off_all(&mut self, event: &str) {
        self.channels.remove(event);
    }

    pub fn emit<T: std::any::Any + Send + Sync + 'static>(&mut self, event: &str, data: T) {
        self.emit_count += 1;
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push((event.to_string(), self.emit_count));

        let any: Box<dyn std::any::Any> = Box::new(data);
        if let Some(entries) = self.channels.get(event) {
            let callbacks: Vec<(EventBusKey, bool, BusCallback)> = entries
                .iter()
                .map(|e| (e.id, e.once, Arc::clone(&e.callback)))
                .collect();
            for (_, _, cb) in &callbacks {
                cb(any.as_ref());
            }
            let once_ids: Vec<EventBusKey> = callbacks
                .iter()
                .filter(|(_, once, _)| *once)
                .map(|(id, _, _)| *id)
                .collect();
            if !once_ids.is_empty() {
                if let Some(entries) = self.channels.get_mut(event) {
                    entries.retain(|e| !once_ids.contains(&e.id));
                }
            }
        }
    }

    pub fn listener_count(&self, event: &str) -> usize {
        self.channels.get(event).map(|v| v.len()).unwrap_or(0)
    }

    pub fn has_listeners(&self, event: &str) -> bool {
        self.channels.get(event).map(|v| !v.is_empty()).unwrap_or(false)
    }

    pub fn get_emit_count(&self) -> u64 {
        self.emit_count
    }

    pub fn get_history(&self) -> &[(String, u64)] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
    }

    pub fn clear_all(&mut self) {
        self.channels.clear();
        self.history.clear();
        self.emit_count = 0;
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone)]
    struct HealthChanged(pub i32);

    #[derive(Debug, Clone)]
    struct LevelUp(pub u32);

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new();
        assert_eq!(bus.get_emit_count(), 0);
    }

    #[test]
    fn test_on_and_emit() {
        let mut bus = EventBus::new();
        let val = Arc::new(Mutex::new(0i32));
        let v = Arc::clone(&val);
        bus.on::<HealthChanged, _>("health", move |e| { *v.lock().unwrap() = e.0; });
        bus.emit("health", HealthChanged(50));
        assert_eq!(*val.lock().unwrap(), 50);
        assert_eq!(bus.get_emit_count(), 1);
    }

    #[test]
    fn test_once_fires_once() {
        let mut bus = EventBus::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        bus.once::<HealthChanged, _>("health", move |_| { *c.lock().unwrap() += 1; });
        bus.emit("health", HealthChanged(10));
        bus.emit("health", HealthChanged(20));
        assert_eq!(*count.lock().unwrap(), 1);
        assert_eq!(bus.listener_count("health"), 0);
    }

    #[test]
    fn test_off_removes_listener() {
        let mut bus = EventBus::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let key = bus.on::<HealthChanged, _>("health", move |_| { *c.lock().unwrap() += 1; });
        bus.emit("health", HealthChanged(1));
        bus.off("health", key);
        bus.emit("health", HealthChanged(2));
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_off_all() {
        let mut bus = EventBus::new();
        bus.on::<HealthChanged, _>("health", |_| {});
        bus.on::<HealthChanged, _>("health", |_| {});
        bus.off_all("health");
        assert_eq!(bus.listener_count("health"), 0);
    }

    #[test]
    fn test_multiple_listeners_same_event() {
        let mut bus = EventBus::new();
        let count = Arc::new(Mutex::new(0u32));
        for _ in 0..3 {
            let c = Arc::clone(&count);
            bus.on::<LevelUp, _>("level_up", move |_| { *c.lock().unwrap() += 1; });
        }
        bus.emit("level_up", LevelUp(2));
        assert_eq!(bus.get_history().last().unwrap().0.as_str(), "level_up");
        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn test_different_event_types() {
        let mut bus = EventBus::new();
        let hp = Arc::new(Mutex::new(0i32));
        let lv = Arc::new(Mutex::new(0u32));
        let hp2 = Arc::clone(&hp);
        let lv2 = Arc::clone(&lv);
        bus.on::<HealthChanged, _>("health", move |e| { *hp2.lock().unwrap() = e.0; });
        bus.on::<LevelUp, _>("level_up", move |e| { *lv2.lock().unwrap() = e.0; });
        bus.emit("health", HealthChanged(100));
        bus.emit("level_up", LevelUp(5));
        assert_eq!(*hp.lock().unwrap(), 100);
        assert_eq!(*lv.lock().unwrap(), 5);
    }

    #[test]
    fn test_emit_wrong_type_no_crash() {
        let mut bus = EventBus::new();
        let called = Arc::new(Mutex::new(false));
        let c = Arc::clone(&called);
        bus.on::<HealthChanged, _>("health", move |_| { *c.lock().unwrap() = true; });
        bus.emit("health", LevelUp(1));
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn test_history_tracks_events() {
        let mut bus = EventBus::new();
        bus.emit("a", 1u32);
        bus.emit("b", 2u32);
        bus.emit("c", 3u32);
        assert_eq!(bus.get_history().len(), 3);
        assert_eq!(bus.get_history()[0].0, "a");
        assert_eq!(bus.get_history()[2].0, "c");
    }

    #[test]
    fn test_clear_all() {
        let mut bus = EventBus::new();
        bus.on::<HealthChanged, _>("health", |_| {});
        bus.emit("health", HealthChanged(1));
        bus.clear_all();
        assert_eq!(bus.get_emit_count(), 0);
        assert_eq!(bus.listener_count("health"), 0);
        assert!(bus.get_history().is_empty());
    }

    #[test]
    fn test_has_listeners() {
        let mut bus = EventBus::new();
        assert!(!bus.has_listeners("evt"));
        bus.on::<u32, _>("evt", |_| {});
        assert!(bus.has_listeners("evt"));
    }

    #[test]
    fn test_history_max_size() {
        let mut bus = EventBus::new();
        bus.set_max_history(3);
        for i in 0..5u32 {
            bus.emit(&format!("e{}", i), i);
        }
        assert_eq!(bus.get_history().len(), 3);
    }
}
