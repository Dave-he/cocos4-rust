/****************************************************************************
Rust port of Cocos Creator Physics Character Controller
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerCollisionFlag {
    None = 0,
    Sides = 1,
    Above = 2,
    Below = 4,
}

impl ControllerCollisionFlag {
    pub fn from_bits(bits: u32) -> u32 { bits }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterControllerType {
    Box = 0,
    Capsule = 1,
}

#[derive(Debug, Clone)]
pub struct CharacterControllerConfig {
    pub controller_type: CharacterControllerType,
    pub slope_limit: f32,
    pub step_offset: f32,
    pub skin_width: f32,
    pub min_move_distance: f32,
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub half_side_extent: Vec3,
}

impl Default for CharacterControllerConfig {
    fn default() -> Self {
        CharacterControllerConfig {
            controller_type: CharacterControllerType::Capsule,
            slope_limit: 45.0_f32.to_radians(),
            step_offset: 0.3,
            skin_width: 0.08,
            min_move_distance: 0.0,
            center: Vec3::ZERO,
            radius: 0.5,
            height: 2.0,
            half_side_extent: Vec3::new(0.5, 1.0, 0.5),
        }
    }
}

#[derive(Debug)]
pub struct CharacterController {
    pub id: u32,
    pub config: CharacterControllerConfig,
    pub position: Vec3,
    pub velocity: Vec3,
    pub on_ground: bool,
    pub collision_flags: u32,
    pub node_uuid: Option<String>,
    pub enabled: bool,
    ref_count: RefCountedImpl,
}

impl CharacterController {
    pub fn new(id: u32, config: CharacterControllerConfig) -> Self {
        CharacterController {
            id,
            config,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            on_ground: false,
            collision_flags: 0,
            node_uuid: None,
            enabled: true,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn get_type(&self) -> CharacterControllerType {
        self.config.controller_type
    }

    pub fn move_controller(&mut self, displacement: Vec3, min_dist: f32, _elapsed_time: f32) -> u32 {
        if !self.enabled {
            return ControllerCollisionFlag::None as u32;
        }

        let mut flags = 0u32;

        let actual_move = if displacement.length() >= min_dist {
            displacement
        } else {
            Vec3::ZERO
        };

        self.position = self.position + actual_move;

        if actual_move.y < 0.0 && self.position.y <= 0.0 {
            self.position.y = 0.0;
            flags |= ControllerCollisionFlag::Below as u32;
            self.on_ground = true;
        } else {
            self.on_ground = (flags & ControllerCollisionFlag::Below as u32) != 0;
        }

        self.collision_flags = flags;
        flags
    }

    pub fn teleport(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.position = pos;
    }

    pub fn is_on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn get_collision_flags(&self) -> u32 {
        self.collision_flags
    }

    pub fn set_slope_limit(&mut self, degrees: f32) {
        self.config.slope_limit = degrees.to_radians();
    }

    pub fn get_slope_limit_degrees(&self) -> f32 {
        self.config.slope_limit.to_degrees()
    }

    pub fn set_step_offset(&mut self, offset: f32) {
        self.config.step_offset = offset.max(0.0);
    }

    pub fn get_step_offset(&self) -> f32 {
        self.config.step_offset
    }

    pub fn set_skin_width(&mut self, width: f32) {
        self.config.skin_width = width.max(0.0);
    }

    pub fn get_skin_width(&self) -> f32 {
        self.config.skin_width
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.config.radius = radius.max(0.0);
    }

    pub fn get_radius(&self) -> f32 {
        self.config.radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.config.height = height.max(0.0);
    }

    pub fn get_height(&self) -> f32 {
        self.config.height
    }

    pub fn set_center(&mut self, center: Vec3) {
        self.config.center = center;
    }

    pub fn get_center(&self) -> Vec3 {
        self.config.center
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_node(&mut self, uuid: Option<String>) {
        self.node_uuid = uuid;
    }
}

impl RefCounted for CharacterController {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

pub struct CharacterControllerManager {
    controllers: std::collections::HashMap<u32, CharacterController>,
    next_id: u32,
}

impl CharacterControllerManager {
    pub fn new() -> Self {
        CharacterControllerManager {
            controllers: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_controller(&mut self, config: CharacterControllerConfig) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.controllers.insert(id, CharacterController::new(id, config));
        id
    }

    pub fn destroy_controller(&mut self, id: u32) {
        self.controllers.remove(&id);
    }

    pub fn get_controller(&self, id: u32) -> Option<&CharacterController> {
        self.controllers.get(&id)
    }

    pub fn get_controller_mut(&mut self, id: u32) -> Option<&mut CharacterController> {
        self.controllers.get_mut(&id)
    }

    pub fn get_count(&self) -> usize {
        self.controllers.len()
    }

    pub fn update_all(&mut self, gravity: Vec3, dt: f32) {
        for ctrl in self.controllers.values_mut() {
            if ctrl.enabled && !ctrl.is_on_ground() {
                let grav_disp = gravity * dt;
                ctrl.move_controller(grav_disp, 0.0, dt);
            }
        }
    }
}

impl Default for CharacterControllerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_create() {
        let ctrl = CharacterController::new(1, CharacterControllerConfig::default());
        assert_eq!(ctrl.get_type(), CharacterControllerType::Capsule);
        assert!(ctrl.is_enabled());
        assert!(!ctrl.is_on_ground());
    }

    #[test]
    fn test_controller_move() {
        let mut ctrl = CharacterController::new(1, CharacterControllerConfig::default());
        ctrl.move_controller(Vec3::new(1.0, 0.0, 0.0), 0.0, 0.016);
        assert!((ctrl.get_position().x - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_controller_teleport() {
        let mut ctrl = CharacterController::new(1, CharacterControllerConfig::default());
        ctrl.teleport(Vec3::new(10.0, 5.0, 0.0));
        assert_eq!(ctrl.get_position(), Vec3::new(10.0, 5.0, 0.0));
    }

    #[test]
    fn test_controller_on_ground_when_at_zero() {
        let mut ctrl = CharacterController::new(1, CharacterControllerConfig::default());
        ctrl.move_controller(Vec3::new(0.0, -1.0, 0.0), 0.0, 0.016);
        assert!(ctrl.is_on_ground());
    }

    #[test]
    fn test_controller_properties() {
        let mut ctrl = CharacterController::new(1, CharacterControllerConfig::default());
        ctrl.set_radius(1.0);
        assert!((ctrl.get_radius() - 1.0).abs() < 1e-6);
        ctrl.set_height(3.0);
        assert!((ctrl.get_height() - 3.0).abs() < 1e-6);
        ctrl.set_step_offset(0.5);
        assert!((ctrl.get_step_offset() - 0.5).abs() < 1e-6);
        ctrl.set_slope_limit(30.0);
        assert!((ctrl.get_slope_limit_degrees() - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_controller_manager() {
        let mut mgr = CharacterControllerManager::new();
        let id = mgr.create_controller(CharacterControllerConfig::default());
        assert_eq!(mgr.get_count(), 1);
        let ctrl = mgr.get_controller(id);
        assert!(ctrl.is_some());
        mgr.destroy_controller(id);
        assert_eq!(mgr.get_count(), 0);
    }

    #[test]
    fn test_controller_manager_update_gravity() {
        let mut mgr = CharacterControllerManager::new();
        let id = mgr.create_controller(CharacterControllerConfig::default());
        mgr.get_controller_mut(id).unwrap().teleport(Vec3::new(0.0, 10.0, 0.0));
        mgr.update_all(Vec3::new(0.0, -10.0, 0.0), 0.1);
        let y = mgr.get_controller(id).unwrap().get_position().y;
        assert!(y < 10.0);
    }

    #[test]
    fn test_controller_disabled_no_move() {
        let mut ctrl = CharacterController::new(1, CharacterControllerConfig::default());
        ctrl.set_enabled(false);
        ctrl.move_controller(Vec3::new(1.0, 0.0, 0.0), 0.0, 0.016);
        assert!((ctrl.get_position().x).abs() < 1e-6);
    }
}
