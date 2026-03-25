use std::collections::HashMap;

pub type EventKey = u64;
static LISTENER_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_listener_id() -> EventKey {
    LISTENER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

struct Listener<E> {
    id: EventKey,
    priority: i32,
    once: bool,
    callback: Box<dyn Fn(&E) + Send + Sync>,
}

pub struct EventTarget<E: Clone + PartialEq + 'static> {
    listeners: Vec<Listener<E>>,
    dirty: bool,
}

impl<E: Clone + PartialEq + 'static> EventTarget<E> {
    pub fn new() -> Self {
        EventTarget {
            listeners: Vec::new(),
            dirty: false,
        }
    }

    pub fn on<F: Fn(&E) + Send + Sync + 'static>(&mut self, callback: F) -> EventKey {
        self.on_with_priority(callback, 0)
    }

    pub fn on_with_priority<F: Fn(&E) + Send + Sync + 'static>(&mut self, callback: F, priority: i32) -> EventKey {
        let id = next_listener_id();
        self.listeners.push(Listener {
            id,
            priority,
            once: false,
            callback: Box::new(callback),
        });
        self.dirty = true;
        id
    }

    pub fn once<F: Fn(&E) + Send + Sync + 'static>(&mut self, callback: F) -> EventKey {
        let id = next_listener_id();
        self.listeners.push(Listener {
            id,
            priority: 0,
            once: true,
            callback: Box::new(callback),
        });
        self.dirty = true;
        id
    }

    pub fn off(&mut self, key: EventKey) {
        self.listeners.retain(|l| l.id != key);
    }

    pub fn off_all(&mut self) {
        self.listeners.clear();
        self.dirty = false;
    }

    pub fn emit(&mut self, event: &E) {
        if self.dirty {
            self.listeners.sort_by(|a, b| b.priority.cmp(&a.priority));
            self.dirty = false;
        }
        let mut to_remove: Vec<EventKey> = Vec::new();
        for listener in &self.listeners {
            (listener.callback)(event);
            if listener.once {
                to_remove.push(listener.id);
            }
        }
        for id in to_remove {
            self.listeners.retain(|l| l.id != id);
        }
    }

    pub fn has_listeners(&self) -> bool {
        !self.listeners.is_empty()
    }

    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
}

impl<E: Clone + PartialEq + 'static> Default for EventTarget<E> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TypedEventBus {
    targets: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl TypedEventBus {
    pub fn new() -> Self {
        TypedEventBus {
            targets: HashMap::new(),
        }
    }
}

impl Default for TypedEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, PartialEq, Debug)]
    enum TestEvent {
        Click,
        Hover,
        KeyPress(u32),
    }

    #[test]
    fn test_event_target_new() {
        let et: EventTarget<TestEvent> = EventTarget::new();
        assert!(!et.has_listeners());
        assert_eq!(et.listener_count(), 0);
    }

    #[test]
    fn test_on_and_emit() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        et.on(move |_| { *c.lock().unwrap() += 1; });
        et.emit(&TestEvent::Click);
        assert_eq!(*count.lock().unwrap(), 1);
        et.emit(&TestEvent::Click);
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[test]
    fn test_once_fires_only_once() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        et.once(move |_| { *c.lock().unwrap() += 1; });
        et.emit(&TestEvent::Click);
        et.emit(&TestEvent::Click);
        assert_eq!(*count.lock().unwrap(), 1);
        assert_eq!(et.listener_count(), 0);
    }

    #[test]
    fn test_off_by_key() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let key = et.on(move |_| { *c.lock().unwrap() += 1; });
        et.emit(&TestEvent::Click);
        et.off(key);
        et.emit(&TestEvent::Click);
        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_off_all() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        et.on(|_| {});
        et.on(|_| {});
        et.off_all();
        assert!(!et.has_listeners());
    }

    #[test]
    fn test_priority_order() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        let order = Arc::new(Mutex::new(Vec::<i32>::new()));

        let o1 = Arc::clone(&order);
        et.on_with_priority(move |_| { o1.lock().unwrap().push(1); }, 1);

        let o2 = Arc::clone(&order);
        et.on_with_priority(move |_| { o2.lock().unwrap().push(10); }, 10);

        let o3 = Arc::clone(&order);
        et.on_with_priority(move |_| { o3.lock().unwrap().push(5); }, 5);

        et.emit(&TestEvent::Click);
        let o = order.lock().unwrap();
        assert_eq!(*o, vec![10, 5, 1]);
    }

    #[test]
    fn test_event_data_passed() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        let received = Arc::new(Mutex::new(0u32));
        let r = Arc::clone(&received);
        et.on(move |e| {
            if let TestEvent::KeyPress(code) = e {
                *r.lock().unwrap() = *code;
            }
        });
        et.emit(&TestEvent::KeyPress(65));
        assert_eq!(*received.lock().unwrap(), 65);
    }

    #[test]
    fn test_multiple_listeners_all_fired() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        let count = Arc::new(Mutex::new(0u32));
        for _ in 0..5 {
            let c = Arc::clone(&count);
            et.on(move |_| { *c.lock().unwrap() += 1; });
        }
        et.emit(&TestEvent::Click);
        assert_eq!(*count.lock().unwrap(), 5);
    }

    #[test]
    fn test_listener_count() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        assert_eq!(et.listener_count(), 0);
        et.on(|_| {});
        et.on(|_| {});
        assert_eq!(et.listener_count(), 2);
    }

    #[test]
    fn test_off_nonexistent_key_no_panic() {
        let mut et: EventTarget<TestEvent> = EventTarget::new();
        et.off(99999);
    }

    #[test]
    fn test_default() {
        let et: EventTarget<TestEvent> = EventTarget::default();
        assert!(!et.has_listeners());
    }
}
