use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Weak};

use super::scene_graph::{BaseNode, NodePtr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedulerPhase {
    Load,
    Start,
    Update,
    LateUpdate,
    Destroy,
}

struct PendingOp {
    node: Weak<Mutex<BaseNode>>,
}

pub struct ComponentScheduler {
    pending_start: VecDeque<PendingOp>,
    pending_destroy: VecDeque<PendingOp>,
    started_nodes: Vec<Weak<Mutex<BaseNode>>>,
    total_components: usize,
    active_components: usize,
}

impl Default for ComponentScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentScheduler {
    pub fn new() -> Self {
        Self {
            pending_start: VecDeque::new(),
            pending_destroy: VecDeque::new(),
            started_nodes: Vec::new(),
            total_components: 0,
            active_components: 0,
        }
    }

    pub fn register_node(&mut self, node: &NodePtr) {
        let weak = Arc::downgrade(node);
        let mut node_guard = node.lock().unwrap();

        for (_, comp) in node_guard.components_iter_mut() {
            comp.on_load();
            self.total_components += 1;
            self.active_components += 1;

            let pending = PendingOp {
                node: Arc::downgrade(node),
            };
            self.pending_start.push_back(pending);
        }

        if node_guard.is_active() {
            self.started_nodes.push(weak);
        }
    }

    pub fn unregister_node(&mut self, node: &NodePtr) {
        let weak = Arc::downgrade(node);
        self.started_nodes.retain(|n| !Weak::ptr_eq(n, &weak));

        let mut node_guard = node.lock().unwrap();
        for (_, comp) in node_guard.components_iter_mut() {
            comp.on_destroy();
            self.active_components = self.active_components.saturating_sub(1);
        }

        let pending = PendingOp {
            node: Arc::downgrade(node),
        };
        self.pending_destroy.push_back(pending);
    }

    pub fn tick(&mut self, dt: f32, nodes: &[NodePtr]) {
        self.process_start_phase();

        self.process_update(dt, nodes);

        self.process_late_update(dt, nodes);

        self.process_destroy();
    }

    fn process_start_phase(&mut self) {
        while let Some(op) = self.pending_start.pop_front() {
            if let Some(node_arc) = op.node.upgrade() {
                let mut node = node_arc.lock().unwrap();
                if node.is_active() {
                    for (_, comp) in node.components_iter_mut() {
                        comp.start();
                    }
                }
            }
        }
    }

    fn process_update(&self, dt: f32, nodes: &[NodePtr]) {
        for node_ptr in nodes {
            if let Ok(mut node) = node_ptr.try_lock() {
                if !node.is_active() { continue; }
                for (_, comp) in node.components_iter_mut() {
                    comp.update(dt);
                }
            }
        }
    }

    fn process_late_update(&self, dt: f32, nodes: &[NodePtr]) {
        for node_ptr in nodes {
            if let Ok(mut node) = node_ptr.try_lock() {
                if !node.is_active() { continue; }
                for (_, comp) in node.components_iter_mut() {
                    comp.late_update(dt);
                }
            }
        }
    }

    fn process_destroy(&mut self) {
        while let Some(op) = self.pending_destroy.pop_front() {
            if let Some(node_arc) = op.node.upgrade() {
                let mut node = node_arc.lock().unwrap();
                for (_, comp) in node.components_iter_mut() {
                    comp.on_destroy();
                }
            }
        }
    }

    pub fn enable_node(&mut self, node: &NodePtr) {
        let mut n = node.lock().unwrap();
        n.set_active(true);
        for (_, comp) in n.components_iter_mut() {
            comp.on_enable();
        }
    }

    pub fn disable_node(&mut self, node: &NodePtr) {
        let mut n = node.lock().unwrap();
        n.set_active(false);
        for (_, comp) in n.components_iter_mut() {
            comp.on_disable();
        }
    }

    pub fn get_total_components(&self) -> usize { self.total_components }
    pub fn get_active_components(&self) -> usize { self.active_components }
    pub fn get_started_node_count(&self) -> usize { self.started_nodes.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::scene_graph::Component;
    use std::sync::atomic::{AtomicI32, Ordering};

    // Per-thread counters so parallel `cargo test` invocations cannot race.
    thread_local! {
        static UPDATE_COUNT: AtomicI32 = const { AtomicI32::new(0) };
        static START_COUNT: AtomicI32 = const { AtomicI32::new(0) };
        static LOAD_COUNT: AtomicI32 = const { AtomicI32::new(0) };
        static LATE_UPDATE_COUNT: AtomicI32 = const { AtomicI32::new(0) };
        static ENABLE_COUNT: AtomicI32 = const { AtomicI32::new(0) };
        static DISABLE_COUNT: AtomicI32 = const { AtomicI32::new(0) };
        static DESTROY_COUNT: AtomicI32 = const { AtomicI32::new(0) };
    }

    fn load_update() -> i32 { UPDATE_COUNT.with(|c| c.load(Ordering::SeqCst)) }
    fn load_start() -> i32 { START_COUNT.with(|c| c.load(Ordering::SeqCst)) }
    fn load_load() -> i32 { LOAD_COUNT.with(|c| c.load(Ordering::SeqCst)) }
    fn load_late_update() -> i32 { LATE_UPDATE_COUNT.with(|c| c.load(Ordering::SeqCst)) }
    fn load_enable() -> i32 { ENABLE_COUNT.with(|c| c.load(Ordering::SeqCst)) }
    fn load_disable() -> i32 { DISABLE_COUNT.with(|c| c.load(Ordering::SeqCst)) }
    fn load_destroy() -> i32 { DESTROY_COUNT.with(|c| c.load(Ordering::SeqCst)) }

    struct TestComponent {
        type_id: std::any::TypeId,
        value: f32,
    }

    impl TestComponent {
        fn new() -> Self {
            Self { type_id: std::any::TypeId::of::<Self>(), value: 0.0 }
        }
    }

    impl Component for TestComponent {
        fn get_type_id(&self) -> std::any::TypeId { self.type_id }
        fn as_any(&self) -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }

        fn on_load(&mut self) { LOAD_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst)); }
        fn start(&mut self) { START_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst)); }
        fn update(&mut self, dt: f32) {
            self.value += dt;
            UPDATE_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst));
        }
        fn late_update(&mut self, _dt: f32) { LATE_UPDATE_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst)); }
        fn on_enable(&mut self) { ENABLE_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst)); }
        fn on_disable(&mut self) { DISABLE_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst)); }
        fn on_destroy(&mut self) { DESTROY_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst)); }
    }

    fn reset_counters() {
        UPDATE_COUNT.with(|c| c.store(0, Ordering::SeqCst));
        START_COUNT.with(|c| c.store(0, Ordering::SeqCst));
        LOAD_COUNT.with(|c| c.store(0, Ordering::SeqCst));
        LATE_UPDATE_COUNT.with(|c| c.store(0, Ordering::SeqCst));
        ENABLE_COUNT.with(|c| c.store(0, Ordering::SeqCst));
        DISABLE_COUNT.with(|c| c.store(0, Ordering::SeqCst));
        DESTROY_COUNT.with(|c| c.store(0, Ordering::SeqCst));
    }

    #[test]
    fn test_scheduler_register_and_start() {
        reset_counters();
        let mut scheduler = ComponentScheduler::new();
        let node = Arc::new(Mutex::new(BaseNode::new("test_node")));
        node.lock().unwrap().add_component_boxed(Box::new(TestComponent::new()));

        scheduler.register_node(&node);

        assert_eq!(load_load(), 1);
        assert_eq!(scheduler.get_total_components(), 1);

        let nodes = vec![node.clone()];
        scheduler.tick(0.016, &nodes);

        assert_eq!(load_start(), 1);
        assert_eq!(load_update(), 1);
        assert_eq!(load_late_update(), 1);
    }

    #[test]
    fn test_scheduler_multiple_ticks() {
        reset_counters();
        let mut scheduler = ComponentScheduler::new();
        let node = Arc::new(Mutex::new(BaseNode::new("tick_node")));
        node.lock().unwrap().add_component_boxed(Box::new(TestComponent::new()));

        scheduler.register_node(&node);
        let nodes = vec![node.clone()];

        scheduler.tick(0.016, &nodes);
        scheduler.tick(0.016, &nodes);
        scheduler.tick(0.016, &nodes);

        assert_eq!(load_update(), 3);
    }

    #[test]
    fn test_scheduler_enable_disable() {
        reset_counters();
        let mut scheduler = ComponentScheduler::new();
        let node = Arc::new(Mutex::new(BaseNode::new("enable_node")));
        node.lock().unwrap().add_component_boxed(Box::new(TestComponent::new()));

        scheduler.register_node(&node);
        let nodes = vec![node.clone()];
        scheduler.tick(0.016, &nodes);

        scheduler.disable_node(&node);
        assert_eq!(load_disable(), 1);

        scheduler.tick(0.016, &nodes);
        assert_eq!(load_update(), 1);

        scheduler.enable_node(&node);
        assert_eq!(load_enable(), 1);

        scheduler.tick(0.016, &nodes);
        assert_eq!(load_update(), 2);
    }

    #[test]
    fn test_scheduler_destroy() {
        reset_counters();
        let mut scheduler = ComponentScheduler::new();
        let node = Arc::new(Mutex::new(BaseNode::new("destroy_node")));
        node.lock().unwrap().add_component_boxed(Box::new(TestComponent::new()));

        scheduler.register_node(&node);
        scheduler.unregister_node(&node);

        assert!(load_destroy() >= 1);
    }

    #[test]
    fn test_scheduler_inactive_node_not_updated() {
        reset_counters();
        let mut scheduler = ComponentScheduler::new();
        let node = Arc::new(Mutex::new(BaseNode::new("inactive")));
        node.lock().unwrap().set_active(false);
        node.lock().unwrap().add_component_boxed(Box::new(TestComponent::new()));

        scheduler.register_node(&node);
        let nodes = vec![node.clone()];
        scheduler.tick(0.016, &nodes);

        assert_eq!(load_update(), 0);
    }

    #[test]
    fn test_scheduler_multiple_components() {
        reset_counters();
        let mut scheduler = ComponentScheduler::new();
        let node = Arc::new(Mutex::new(BaseNode::new("multi")));

        struct TestComponentA { type_id: std::any::TypeId, value: f32 }
        impl TestComponentA {
            fn new() -> Self { Self { type_id: std::any::TypeId::of::<Self>(), value: 0.0 } }
        }
        impl Component for TestComponentA {
            fn get_type_id(&self) -> std::any::TypeId { self.type_id }
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
            fn update(&mut self, dt: f32) {
                self.value += dt;
                UPDATE_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst));
            }
        }

        struct TestComponentB { type_id: std::any::TypeId, value: f32 }
        impl TestComponentB {
            fn new() -> Self { Self { type_id: std::any::TypeId::of::<Self>(), value: 0.0 } }
        }
        impl Component for TestComponentB {
            fn get_type_id(&self) -> std::any::TypeId { self.type_id }
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
            fn update(&mut self, dt: f32) {
                self.value += dt;
                UPDATE_COUNT.with(|c| c.fetch_add(1, Ordering::SeqCst));
            }
        }

        node.lock().unwrap().add_component_boxed(Box::new(TestComponentA::new()));
        node.lock().unwrap().add_component_boxed(Box::new(TestComponentB::new()));

        scheduler.register_node(&node);
        assert_eq!(scheduler.get_total_components(), 2);

        let nodes = vec![node.clone()];
        scheduler.tick(0.016, &nodes);

        assert_eq!(load_update(), 2);
    }

    #[test]
    fn test_scheduler_empty() {
        let mut scheduler = ComponentScheduler::new();
        let nodes: Vec<NodePtr> = vec![];
        scheduler.tick(0.016, &nodes);
        assert_eq!(scheduler.get_total_components(), 0);
        assert_eq!(scheduler.get_active_components(), 0);
    }
}
