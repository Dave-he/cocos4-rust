/****************************************************************************
Rust port of Cocos Creator PhysX Event Manager
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsEventType {
    TriggerEnter = 0,
    TriggerStay = 1,
    TriggerExit = 2,
    CollisionEnter = 3,
    CollisionStay = 4,
    CollisionExit = 5,
}

#[derive(Debug, Clone)]
pub struct ContactPoint {
    pub position: Vec3,
    pub normal: Vec3,
    pub separation: f32,
    pub impulse: Vec3,
}

impl ContactPoint {
    pub fn new(position: Vec3, normal: Vec3, separation: f32) -> Self {
        ContactPoint {
            position,
            normal,
            separation,
            impulse: Vec3::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsEvent {
    pub event_type: PhysicsEventType,
    pub body_a_uuid: String,
    pub body_b_uuid: String,
    pub contacts: Vec<ContactPoint>,
}

impl PhysicsEvent {
    pub fn new_trigger(event_type: PhysicsEventType, a: &str, b: &str) -> Self {
        PhysicsEvent {
            event_type,
            body_a_uuid: a.to_string(),
            body_b_uuid: b.to_string(),
            contacts: Vec::new(),
        }
    }

    pub fn new_collision(event_type: PhysicsEventType, a: &str, b: &str, contacts: Vec<ContactPoint>) -> Self {
        PhysicsEvent {
            event_type,
            body_a_uuid: a.to_string(),
            body_b_uuid: b.to_string(),
            contacts,
        }
    }
}

pub type EventCallback = Box<dyn Fn(&PhysicsEvent) + Send + Sync>;

pub struct PhysicsEventManager {
    pending_events: Vec<PhysicsEvent>,
    ref_count: RefCountedImpl,
}

impl std::fmt::Debug for PhysicsEventManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicsEventManager")
            .field("pending_count", &self.pending_events.len())
            .finish()
    }
}

impl PhysicsEventManager {
    pub fn new() -> Self {
        PhysicsEventManager {
            pending_events: Vec::new(),
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn push_trigger_enter(&mut self, a: &str, b: &str) {
        self.pending_events.push(PhysicsEvent::new_trigger(PhysicsEventType::TriggerEnter, a, b));
    }

    pub fn push_trigger_exit(&mut self, a: &str, b: &str) {
        self.pending_events.push(PhysicsEvent::new_trigger(PhysicsEventType::TriggerExit, a, b));
    }

    pub fn push_trigger_stay(&mut self, a: &str, b: &str) {
        self.pending_events.push(PhysicsEvent::new_trigger(PhysicsEventType::TriggerStay, a, b));
    }

    pub fn push_collision_enter(&mut self, a: &str, b: &str, contacts: Vec<ContactPoint>) {
        self.pending_events.push(PhysicsEvent::new_collision(PhysicsEventType::CollisionEnter, a, b, contacts));
    }

    pub fn push_collision_exit(&mut self, a: &str, b: &str) {
        self.pending_events.push(PhysicsEvent::new_collision(PhysicsEventType::CollisionExit, a, b, Vec::new()));
    }

    pub fn dispatch(&mut self, callback: &dyn Fn(&PhysicsEvent)) {
        for event in &self.pending_events {
            callback(event);
        }
        self.pending_events.clear();
    }

    pub fn clear(&mut self) {
        self.pending_events.clear();
    }

    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    pub fn get_events_of_type(&self, event_type: PhysicsEventType) -> Vec<&PhysicsEvent> {
        self.pending_events.iter().filter(|e| e.event_type == event_type).collect()
    }
}

impl Default for PhysicsEventManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RefCounted for PhysicsEventManager {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_manager_push() {
        let mut mgr = PhysicsEventManager::new();
        mgr.push_trigger_enter("a", "b");
        assert_eq!(mgr.pending_count(), 1);
    }

    #[test]
    fn test_event_manager_dispatch() {
        let mut mgr = PhysicsEventManager::new();
        mgr.push_trigger_enter("a", "b");
        mgr.push_collision_enter("c", "d", vec![
            ContactPoint::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), 0.01),
        ]);
        let mut count = 0;
        mgr.dispatch(&|_e| { count += 1; });
        assert_eq!(count, 2);
        assert_eq!(mgr.pending_count(), 0);
    }

    #[test]
    fn test_event_manager_filter_by_type() {
        let mut mgr = PhysicsEventManager::new();
        mgr.push_trigger_enter("a", "b");
        mgr.push_trigger_exit("c", "d");
        mgr.push_collision_enter("e", "f", Vec::new());
        let triggers = mgr.get_events_of_type(PhysicsEventType::TriggerEnter);
        assert_eq!(triggers.len(), 1);
    }

    #[test]
    fn test_event_manager_clear() {
        let mut mgr = PhysicsEventManager::new();
        mgr.push_trigger_enter("a", "b");
        mgr.clear();
        assert_eq!(mgr.pending_count(), 0);
    }

    #[test]
    fn test_contact_point() {
        let cp = ContactPoint::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.05);
        assert!((cp.separation - 0.05).abs() < 1e-6);
        assert_eq!(cp.impulse, Vec3::ZERO);
    }
}
