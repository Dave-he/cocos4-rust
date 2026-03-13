use crate::base::{RefCounted, RefCountedImpl};
use crate::core::scene_graph::BaseNode;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointType {
    Sphere = 0,
    Revolute = 1,
    Spherical = 2,
    Prismatic = 3,
    Fixed = 4,
    Distance = 5,
    D6Spring = 6,
    D6Joint = 7,
}

#[derive(Debug, Clone, Copy)]
pub struct JointMotionLimit {
    pub lower: f32,
    pub upper: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub restitution: f32,
    pub bounce_threshold: f32,
}

impl Default for JointMotionLimit {
    fn default() -> Self {
        JointMotionLimit {
            lower: -std::f32::consts::PI,
            upper: std::f32::consts::PI,
            stiffness: 0.0,
            damping: 0.0,
            restitution: 0.0,
            bounce_threshold: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointMotionFlag {
    Locked = 0,
    Limited = 1,
    Free = 2,
}

#[derive(Debug, Clone)]
pub struct D6JointConfig {
    pub x_motion: JointMotionFlag,
    pub y_motion: JointMotionFlag,
    pub z_motion: JointMotionFlag,
    pub twist_motion: JointMotionFlag,
    pub swing1_motion: JointMotionFlag,
    pub swing2_motion: JointMotionFlag,
    pub linear_limit_x: JointMotionLimit,
    pub linear_limit_y: JointMotionLimit,
    pub linear_limit_z: JointMotionLimit,
    pub angular_limit_twist: JointMotionLimit,
    pub angular_limit_swing: JointMotionLimit,
    pub drive_stiffness: f32,
    pub drive_damping: f32,
    pub drive_force_limit: f32,
}

impl Default for D6JointConfig {
    fn default() -> Self {
        D6JointConfig {
            x_motion: JointMotionFlag::Free,
            y_motion: JointMotionFlag::Free,
            z_motion: JointMotionFlag::Free,
            twist_motion: JointMotionFlag::Free,
            swing1_motion: JointMotionFlag::Free,
            swing2_motion: JointMotionFlag::Free,
            linear_limit_x: JointMotionLimit::default(),
            linear_limit_y: JointMotionLimit::default(),
            linear_limit_z: JointMotionLimit::default(),
            angular_limit_twist: JointMotionLimit::default(),
            angular_limit_swing: JointMotionLimit::default(),
            drive_stiffness: 0.0,
            drive_damping: 0.0,
            drive_force_limit: f32::MAX,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JointConfig {
    pub joint_type: JointType,
    pub connected_body_uuid: Option<String>,
    pub anchor: Vec3,
    pub connected_anchor: Vec3,
    pub enable_collision: bool,
    pub break_force: f32,
    pub break_torque: f32,
    pub d6: D6JointConfig,
}

impl Default for JointConfig {
    fn default() -> Self {
        JointConfig {
            joint_type: JointType::Fixed,
            connected_body_uuid: None,
            anchor: Vec3::ZERO,
            connected_anchor: Vec3::ZERO,
            enable_collision: false,
            break_force: f32::MAX,
            break_torque: f32::MAX,
            d6: D6JointConfig::default(),
        }
    }
}

#[derive(Debug)]
pub struct JointImpl {
    pub id: u32,
    pub config: JointConfig,
    pub enabled: bool,
    pub node_uuid: Option<String>,
    pub is_broken: bool,
    ref_count: RefCountedImpl,
}

impl JointImpl {
    pub fn new(id: u32, config: JointConfig) -> Self {
        JointImpl {
            id,
            config,
            enabled: true,
            node_uuid: None,
            is_broken: false,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn get_type(&self) -> JointType {
        self.config.joint_type
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_anchor(&mut self, anchor: Vec3) {
        self.config.anchor = anchor;
    }

    pub fn get_anchor(&self) -> Vec3 {
        self.config.anchor
    }

    pub fn set_connected_anchor(&mut self, anchor: Vec3) {
        self.config.connected_anchor = anchor;
    }

    pub fn get_connected_anchor(&self) -> Vec3 {
        self.config.connected_anchor
    }

    pub fn set_break_force(&mut self, force: f32) {
        self.config.break_force = force;
    }

    pub fn set_break_torque(&mut self, torque: f32) {
        self.config.break_torque = torque;
    }

    pub fn apply_force(&mut self, force: Vec3) {
        if force.length() > self.config.break_force {
            self.is_broken = true;
            self.enabled = false;
        }
    }

    pub fn apply_torque(&mut self, torque: Vec3) {
        if torque.length() > self.config.break_torque {
            self.is_broken = true;
            self.enabled = false;
        }
    }

    pub fn is_broken(&self) -> bool {
        self.is_broken
    }

    pub fn reset(&mut self) {
        self.is_broken = false;
        self.enabled = true;
    }

    pub fn set_connected_body(&mut self, uuid: Option<String>) {
        self.config.connected_body_uuid = uuid;
    }

    pub fn get_connected_body(&self) -> Option<&str> {
        self.config.connected_body_uuid.as_deref()
    }

    pub fn set_enable_collision(&mut self, enabled: bool) {
        self.config.enable_collision = enabled;
    }

    pub fn is_collision_enabled(&self) -> bool {
        self.config.enable_collision
    }

    pub fn set_d6_motion(
        &mut self,
        axis: &str,
        motion: JointMotionFlag,
    ) {
        match axis {
            "x" => self.config.d6.x_motion = motion,
            "y" => self.config.d6.y_motion = motion,
            "z" => self.config.d6.z_motion = motion,
            "twist" => self.config.d6.twist_motion = motion,
            "swing1" => self.config.d6.swing1_motion = motion,
            "swing2" => self.config.d6.swing2_motion = motion,
            _ => {}
        }
    }

    pub fn set_drive(&mut self, stiffness: f32, damping: f32, force_limit: f32) {
        self.config.d6.drive_stiffness = stiffness;
        self.config.d6.drive_damping = damping;
        self.config.d6.drive_force_limit = force_limit;
    }
}

impl RefCounted for JointImpl {
    fn add_ref(&self) { self.ref_count.add_ref(); }
    fn release(&self) { self.ref_count.release(); }
    fn get_ref_count(&self) -> u32 { self.ref_count.get_ref_count() }
    fn is_last_reference(&self) -> bool { self.ref_count.is_last_reference() }
}

pub trait Joint: RefCounted {
    fn get_type(&self) -> JointType;
    fn set_enabled(&mut self, enabled: bool);
    fn is_enabled(&self) -> bool;
}

impl Joint for JointImpl {
    fn get_type(&self) -> JointType { self.config.joint_type }
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn is_enabled(&self) -> bool { self.enabled }
}

pub trait RigidBody: RefCounted {}

pub trait World: RefCounted {}

pub struct JointManager {
    joints: std::collections::HashMap<u32, JointImpl>,
    next_id: u32,
}

impl JointManager {
    pub fn new() -> Self {
        JointManager {
            joints: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn create_joint(&mut self, config: JointConfig) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.joints.insert(id, JointImpl::new(id, config));
        id
    }

    pub fn destroy_joint(&mut self, id: u32) {
        self.joints.remove(&id);
    }

    pub fn get_joint(&self, id: u32) -> Option<&JointImpl> {
        self.joints.get(&id)
    }

    pub fn get_joint_mut(&mut self, id: u32) -> Option<&mut JointImpl> {
        self.joints.get_mut(&id)
    }

    pub fn get_joint_count(&self) -> usize {
        self.joints.len()
    }

    pub fn get_broken_joints(&self) -> Vec<u32> {
        self.joints.values()
            .filter(|j| j.is_broken())
            .map(|j| j.id)
            .collect()
    }
}

impl Default for JointManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_create() {
        let config = JointConfig {
            joint_type: JointType::Fixed,
            ..Default::default()
        };
        let joint = JointImpl::new(1, config);
        assert_eq!(joint.get_type(), JointType::Fixed);
        assert!(joint.is_enabled());
    }

    #[test]
    fn test_joint_break_force() {
        let mut joint = JointImpl::new(1, JointConfig::default());
        joint.set_break_force(10.0);
        joint.apply_force(Vec3::new(20.0, 0.0, 0.0));
        assert!(joint.is_broken());
        assert!(!joint.is_enabled());
    }

    #[test]
    fn test_joint_reset() {
        let mut joint = JointImpl::new(1, JointConfig::default());
        joint.set_break_force(1.0);
        joint.apply_force(Vec3::new(5.0, 0.0, 0.0));
        assert!(joint.is_broken());
        joint.reset();
        assert!(!joint.is_broken());
        assert!(joint.is_enabled());
    }

    #[test]
    fn test_joint_anchor() {
        let mut joint = JointImpl::new(1, JointConfig::default());
        joint.set_anchor(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(joint.get_anchor(), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_joint_manager_create_destroy() {
        let mut mgr = JointManager::new();
        let id = mgr.create_joint(JointConfig::default());
        assert_eq!(mgr.get_joint_count(), 1);
        mgr.destroy_joint(id);
        assert_eq!(mgr.get_joint_count(), 0);
    }

    #[test]
    fn test_joint_manager_broken() {
        let mut mgr = JointManager::new();
        let id = mgr.create_joint(JointConfig {
            break_force: 1.0,
            ..Default::default()
        });
        mgr.get_joint_mut(id).unwrap().apply_force(Vec3::new(5.0, 0.0, 0.0));
        let broken = mgr.get_broken_joints();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0], id);
    }

    #[test]
    fn test_d6_joint_motion() {
        let mut joint = JointImpl::new(1, JointConfig {
            joint_type: JointType::D6Joint,
            ..Default::default()
        });
        joint.set_d6_motion("x", JointMotionFlag::Limited);
        assert_eq!(joint.config.d6.x_motion, JointMotionFlag::Limited);
        joint.set_d6_motion("twist", JointMotionFlag::Locked);
        assert_eq!(joint.config.d6.twist_motion, JointMotionFlag::Locked);
    }

    #[test]
    fn test_d6_joint_drive() {
        let mut joint = JointImpl::new(1, JointConfig::default());
        joint.set_drive(100.0, 10.0, 1000.0);
        assert!((joint.config.d6.drive_stiffness - 100.0).abs() < 1e-6);
        assert!((joint.config.d6.drive_damping - 10.0).abs() < 1e-6);
    }
}
