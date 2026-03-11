/****************************************************************************
 Rust port of Cocos Creator Event Bus system
 Original C++ version Copyright (c) 2022-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Event trait that all events must implement
pub trait Event: 'static + Send + Sync + Clone {
    fn as_any(&self) -> &dyn Any;
}

/// Type-erased event callback
pub type EventCallback = Box<dyn Fn(&dyn Any) + Send + Sync>;

/// Event bus for managing event subscriptions and emissions
pub struct EventBus {
    listeners: HashMap<TypeId, Vec<EventCallback>>,
}

impl EventBus {
    /// Creates a new event bus
    pub fn new() -> Self {
        EventBus {
            listeners: HashMap::new(),
        }
    }

    /// Registers a listener for a specific event type
    pub fn on<E, F>(&mut self, callback: F)
    where
        E: Event,
        F: Fn(&E) + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<E>();
        let wrapped_callback: EventCallback = Box::new(move |event| {
            if let Some(typed_event) = event.downcast_ref::<E>() {
                callback(typed_event);
            }
        });
        
        self.listeners
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(wrapped_callback);
    }

    /// Emits an event to all registered listeners
    pub fn emit<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        if let Some(listeners) = self.listeners.get(&type_id) {
            for callback in listeners {
                callback(&event);
            }
        }
    }

    /// Removes all listeners for a specific event type
    pub fn off<E: Event>(&mut self) {
        let type_id = TypeId::of::<E>();
        self.listeners.remove(&type_id);
    }

    /// Clears all listeners
    pub fn clear(&mut self) {
        self.listeners.clear();
    }

    /// Returns the number of listeners for a specific event type
    pub fn listener_count<E: Event>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.listeners.get(&type_id).map_or(0, |v| v.len())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe event bus using Arc and Mutex
pub struct SharedEventBus {
    inner: Arc<Mutex<EventBus>>,
}

impl SharedEventBus {
    /// Creates a new shared event bus
    pub fn new() -> Self {
        SharedEventBus {
            inner: Arc::new(Mutex::new(EventBus::new())),
        }
    }

    /// Registers a listener for a specific event type
    pub fn on<E, F>(&self, callback: F)
    where
        E: Event,
        F: Fn(&E) + Send + Sync + 'static,
    {
        if let Ok(mut bus) = self.inner.lock() {
            bus.on(callback);
        }
    }

    /// Emits an event to all registered listeners
    pub fn emit<E: Event>(&self, event: E) {
        if let Ok(bus) = self.inner.lock() {
            bus.emit(event);
        }
    }

    /// Removes all listeners for a specific event type
    pub fn off<E: Event>(&self) {
        if let Ok(mut bus) = self.inner.lock() {
            bus.off::<E>();
        }
    }

    /// Clears all listeners
    pub fn clear(&self) {
        if let Ok(mut bus) = self.inner.lock() {
            bus.clear();
        }
    }
}

impl Default for SharedEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SharedEventBus {
    fn clone(&self) -> Self {
        SharedEventBus {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Event target for DOM-like event handling
pub struct EventTarget {
    bus: EventBus,
}

impl EventTarget {
    /// Creates a new event target
    pub fn new() -> Self {
        EventTarget {
            bus: EventBus::new(),
        }
    }

    /// Adds an event listener
    pub fn add_event_listener<E, F>(&mut self, callback: F)
    where
        E: Event,
        F: Fn(&E) + Send + Sync + 'static,
    {
        self.bus.on(callback);
    }

    /// Dispatches an event
    pub fn dispatch_event<E: Event>(&self, event: E) {
        self.bus.emit(event);
    }

    /// Removes all event listeners for a specific event type
    pub fn remove_event_listeners<E: Event>(&mut self) {
        self.bus.off::<E>();
    }
}

impl Default for EventTarget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestEvent {
        pub value: i32,
    }

    impl Event for TestEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Debug, Clone)]
    struct AnotherEvent {
        pub message: String,
    }

    impl Event for AnotherEvent {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_event_bus_basic() {
        let mut bus = EventBus::new();
        let received = Arc::new(Mutex::new(0));
        let received_clone = Arc::clone(&received);

        bus.on::<TestEvent, _>(move |event| {
            let mut value = received_clone.lock().unwrap();
            *value = event.value;
        });

        bus.emit(TestEvent { value: 42 });

        assert_eq!(*received.lock().unwrap(), 42);
    }

    #[test]
    fn test_event_bus_multiple_listeners() {
        let mut bus = EventBus::new();
        let count = Arc::new(Mutex::new(0));

        for _ in 0..3 {
            let count_clone = Arc::clone(&count);
            bus.on::<TestEvent, _>(move |_| {
                let mut value = count_clone.lock().unwrap();
                *value += 1;
            });
        }

        bus.emit(TestEvent { value: 1 });

        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn test_event_bus_different_events() {
        let mut bus = EventBus::new();
        let test_received = Arc::new(Mutex::new(0));
        let another_received = Arc::new(Mutex::new(String::new()));

        let test_clone = Arc::clone(&test_received);
        bus.on::<TestEvent, _>(move |event| {
            let mut value = test_clone.lock().unwrap();
            *value = event.value;
        });

        let another_clone = Arc::clone(&another_received);
        bus.on::<AnotherEvent, _>(move |event| {
            let mut value = another_clone.lock().unwrap();
            *value = event.message.clone();
        });

        bus.emit(TestEvent { value: 100 });
        bus.emit(AnotherEvent {
            message: "Hello".to_string(),
        });

        assert_eq!(*test_received.lock().unwrap(), 100);
        assert_eq!(*another_received.lock().unwrap(), "Hello");
    }

    #[test]
    fn test_event_bus_off() {
        let mut bus = EventBus::new();
        let received = Arc::new(Mutex::new(0));
        let received_clone = Arc::clone(&received);

        bus.on::<TestEvent, _>(move |event| {
            let mut value = received_clone.lock().unwrap();
            *value = event.value;
        });

        bus.emit(TestEvent { value: 10 });
        assert_eq!(*received.lock().unwrap(), 10);

        bus.off::<TestEvent>();
        bus.emit(TestEvent { value: 20 });
        assert_eq!(*received.lock().unwrap(), 10); // Should still be 10
    }

    #[test]
    fn test_shared_event_bus() {
        let bus = SharedEventBus::new();
        let received = Arc::new(Mutex::new(0));
        let received_clone = Arc::clone(&received);

        bus.on::<TestEvent, _>(move |event| {
            let mut value = received_clone.lock().unwrap();
            *value = event.value;
        });

        bus.emit(TestEvent { value: 99 });

        assert_eq!(*received.lock().unwrap(), 99);
    }

    #[test]
    fn test_event_target() {
        let mut target = EventTarget::new();
        let received = Arc::new(Mutex::new(0));
        let received_clone = Arc::clone(&received);

        target.add_event_listener::<TestEvent, _>(move |event| {
            let mut value = received_clone.lock().unwrap();
            *value = event.value;
        });

        target.dispatch_event(TestEvent { value: 55 });

        assert_eq!(*received.lock().unwrap(), 55);
    }

    #[test]
    fn test_listener_count() {
        let mut bus = EventBus::new();
        assert_eq!(bus.listener_count::<TestEvent>(), 0);

        bus.on::<TestEvent, _>(|_| {});
        assert_eq!(bus.listener_count::<TestEvent>(), 1);

        bus.on::<TestEvent, _>(|_| {});
        assert_eq!(bus.listener_count::<TestEvent>(), 2);

        bus.off::<TestEvent>();
        assert_eq!(bus.listener_count::<TestEvent>(), 0);
    }
}
