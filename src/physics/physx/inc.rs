/****************************************************************************
Rust port of Cocos Creator PhysX Inc (common types)
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Vec3, Quaternion, Mat4};

#[derive(Debug, Clone, Copy)]
pub struct PxTransform {
    pub position: Vec3,
    pub rotation: Quaternion,
}

impl PxTransform {
    pub fn new(position: Vec3, rotation: Quaternion) -> Self {
        PxTransform { position, rotation }
    }

    pub fn identity() -> Self {
        PxTransform {
            position: Vec3::ZERO,
            rotation: Quaternion::IDENTITY,
        }
    }

    pub fn to_mat4(&self) -> Mat4 {
        let mut mat = Mat4::from_quat(self.rotation);
        mat.m[12] = self.position.x;
        mat.m[13] = self.position.y;
        mat.m[14] = self.position.z;
        mat
    }

    pub fn from_mat4(mat: &Mat4) -> Self {
        let position = Vec3::new(mat.m[12], mat.m[13], mat.m[14]);
        let rotation = Quaternion::from_mat4(mat);
        PxTransform { position, rotation }
    }
}

impl Default for PxTransform {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PxForceMode {
    Force = 0,
    Impulse = 1,
    VelocityChange = 2,
    Acceleration = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PxActorFlag {
    Visualization = 1 << 0,
    DisableGravity = 1 << 1,
    SendSleepNotifies = 1 << 2,
    DisableSimulation = 1 << 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PxRigidBodyFlag {
    Kinematic = 1 << 0,
    EnableCCD = 1 << 2,
    EnableCCDFriction = 1 << 3,
    EnablePoseIntegrationPreview = 1 << 4,
    EnableSpeculativeCCD = 1 << 5,
    EnableCCDMaxContactImpulse = 1 << 6,
    RetainAccelerations = 1 << 7,
}

#[derive(Debug, Clone, Copy)]
pub struct PxBounds3 {
    pub minimum: Vec3,
    pub maximum: Vec3,
}

impl PxBounds3 {
    pub fn new(minimum: Vec3, maximum: Vec3) -> Self {
        PxBounds3 { minimum, maximum }
    }

    pub fn empty() -> Self {
        PxBounds3 {
            minimum: Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            maximum: Vec3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.minimum.x + self.maximum.x) * 0.5,
            (self.minimum.y + self.maximum.y) * 0.5,
            (self.minimum.z + self.maximum.z) * 0.5,
        )
    }

    pub fn extents(&self) -> Vec3 {
        Vec3::new(
            (self.maximum.x - self.minimum.x) * 0.5,
            (self.maximum.y - self.minimum.y) * 0.5,
            (self.maximum.z - self.minimum.z) * 0.5,
        )
    }

    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.minimum.x && point.x <= self.maximum.x
            && point.y >= self.minimum.y && point.y <= self.maximum.y
            && point.z >= self.minimum.z && point.z <= self.maximum.z
    }
}

impl Default for PxBounds3 {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_px_transform_identity() {
        let t = PxTransform::identity();
        assert_eq!(t.position, Vec3::ZERO);
    }

    #[test]
    fn test_px_bounds_center() {
        let b = PxBounds3::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0));
        let c = b.center();
        assert!((c.x - 1.0).abs() < 1e-6);
        assert!((c.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_px_bounds_contains() {
        let b = PxBounds3::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));
        assert!(b.contains(Vec3::ZERO));
        assert!(!b.contains(Vec3::new(2.0, 0.0, 0.0)));
    }
}
