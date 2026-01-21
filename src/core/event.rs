/****************************************************************************
 Rust port of Cocos Creator Event Bus system
 Original C++ version Copyright (c) 2022-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub trait EventListener: Send + Sync {
    fn on_event(&self, event: &dyn Any);
}

pub type EventCallback = Arc<RwLock<dyn Fn(&dyn Any) + Send + Sync>>;

pub struct EventBus {
    listeners: HashMap<TypeId, Vec<EventCallback>>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            listeners: HashMap::new(),
        }
    }

    pub fn add_listener<E: 'static>(&mut self, listener: Arc<dyn EventListener>) {
        let type_id = TypeId::of::<E>();
        self.listeners
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(Arc::new(RwLock::new(Box::new(listener))));
    }

    pub fn remove_listener<E: 'static>(&mut self, listener: &Arc<dyn EventListener>) {
        let type_id = TypeId::of::<E>();
        if let Some(listeners) = self.listeners.get_mut(&type_id) {
            listeners.retain(|cb| {
                let cb_read = cb.read().unwrap();
                if let Some(boxed) = cb_read.as_any().downcast_ref::<Box<dyn EventListener>>() {
                    Arc::ptr_eq(boxed, listener)
                } else {
                    false
                }
            });
        }
    }

    pub fn emit<E: 'static>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        if let Some(listeners) = self.listeners.get(&type_id) {
            let boxed_event: Box<dyn Any> = Box::new(event);

            for callback in listeners {
                let cb = callback.read().unwrap();
                cb(&boxed_event);
            }
        }
    }

    pub fn named_bus(name: &'static str) -> NamedEventBus {
        NamedEventBus {
            name,
            bus: EventBus::new(),
        }
    }
}

pub struct NamedEventBus {
    pub name: &'static str,
    pub bus: EventBus,
}

impl NamedEventBus {
    pub fn emit<E: 'static>(&self, event: E) {
        self.bus.emit(event);
    }

    pub fn add_listener<E: 'static>(&mut self, listener: Arc<dyn EventListener>) {
        self.bus.add_listener::<E>(listener);
    }

    pub fn remove_listener<E: 'static>(&mut self, listener: &Arc<dyn EventListener>) {
        self.bus.remove_listener::<E>(listener);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEvent {
        pub value: i32,
    }

    impl EventListener for TestObject {
        fn on_event(&mut self, event: &dyn Any) {
            if let Some(test_event) = event.downcast_ref::<TestEvent>() {
                self.value = test_event.value * 2;
            }
        }
    }

    struct TestObject {
        pub value: i32,
    }

    #[test]
    fn test_event_bus() {
        let bus = EventBus::new();
        let object = Arc::new(RwLock::new(TestObject { value: 10 }));
        let listener = Arc::clone(&object);

        bus.add_listener::<TestEvent>(listener);

        let event = TestEvent { value: 5 };
        bus.emit(event);

        let value = object.read().unwrap();
        assert_eq!(value.value, 5);
    }

    #[test]
    fn test_named_bus() {
        let bus = EventBus::named_bus("TestBus");
        let object = Arc::new(RwLock::new(TestObject { value: 20 }));
        let listener = Arc::clone(&object);

        bus.add_listener::<TestEvent>(listener);

        let event = TestEvent { value: 10 };
        bus.emit(event);

        let value = object.read().unwrap();
        assert_eq!(value.value, 10);
    }

    #[test]
    fn test_remove_listener() {
        let bus = EventBus::new();
        let object = Arc::new(RwLock::new(TestObject { value: 30 }));
        let listener = Arc::clone(&object);

        bus.add_listener::<TestEvent>(listener);
        bus.remove_listener::<TestEvent>(&listener);

        let event = TestEvent { value: 15 };
        bus.emit(event);

        let value = object.read().unwrap();
        assert_eq!(value.value, 30);
    }
}
