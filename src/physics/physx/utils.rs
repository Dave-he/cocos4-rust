/****************************************************************************
Rust port of Cocos Creator PhysX Utils
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::math::{Vec3, Quaternion, Mat4};
use super::inc::PxTransform;

pub fn vec3_to_px(v: Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

pub fn px_to_vec3(arr: [f32; 3]) -> Vec3 {
    Vec3::new(arr[0], arr[1], arr[2])
}

pub fn quat_to_px(q: Quaternion) -> [f32; 4] {
    [q.x, q.y, q.z, q.w]
}

pub fn px_to_quat(arr: [f32; 4]) -> Quaternion {
    Quaternion::new(arr[0], arr[1], arr[2], arr[3])
}

pub fn px_transform_to_mat4(t: &PxTransform) -> Mat4 {
    t.to_mat4()
}

pub fn mat4_to_px_transform(mat: &Mat4) -> PxTransform {
    PxTransform::from_mat4(mat)
}

pub fn combine_px_transforms(parent: &PxTransform, local: &PxTransform) -> PxTransform {
    let pos = parent.rotation.transform_point(local.position) + parent.position;
    let rot = parent.rotation * local.rotation;
    PxTransform::new(pos, rot)
}

pub fn invert_px_transform(t: &PxTransform) -> PxTransform {
    let inv_rot = t.rotation.conjugate();
    let inv_pos = inv_rot.transform_point(-t.position);
    PxTransform::new(inv_pos, inv_rot)
}

pub fn deg_to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / std::f32::consts::PI
}

pub fn compute_aabb_from_sphere(center: Vec3, radius: f32) -> (Vec3, Vec3) {
    let r = Vec3::new(radius, radius, radius);
    (center - r, center + r)
}

pub fn compute_aabb_from_box(center: Vec3, half_extents: Vec3) -> (Vec3, Vec3) {
    (center - half_extents, center + half_extents)
}

pub fn aabbs_overlap(min_a: Vec3, max_a: Vec3, min_b: Vec3, max_b: Vec3) -> bool {
    min_a.x <= max_b.x && max_a.x >= min_b.x
        && min_a.y <= max_b.y && max_a.y >= min_b.y
        && min_a.z <= max_b.z && max_a.z >= min_b.z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_px_roundtrip() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let px = vec3_to_px(v);
        let back = px_to_vec3(px);
        assert!((back.x - v.x).abs() < 1e-6);
        assert!((back.y - v.y).abs() < 1e-6);
    }

    #[test]
    fn test_deg_rad() {
        assert!((deg_to_rad(180.0) - std::f32::consts::PI).abs() < 1e-5);
        assert!((rad_to_deg(std::f32::consts::PI) - 180.0).abs() < 1e-4);
    }

    #[test]
    fn test_aabb_sphere() {
        let (min, max) = compute_aabb_from_sphere(Vec3::new(0.0, 1.0, 0.0), 1.0);
        assert!((min.x - (-1.0)).abs() < 1e-6);
        assert!((max.y - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_aabbs_overlap_true() {
        let (min_a, max_a) = compute_aabb_from_sphere(Vec3::ZERO, 1.0);
        let (min_b, max_b) = compute_aabb_from_sphere(Vec3::new(0.5, 0.0, 0.0), 1.0);
        assert!(aabbs_overlap(min_a, max_a, min_b, max_b));
    }

    #[test]
    fn test_aabbs_overlap_false() {
        let (min_a, max_a) = compute_aabb_from_sphere(Vec3::ZERO, 0.4);
        let (min_b, max_b) = compute_aabb_from_sphere(Vec3::new(2.0, 0.0, 0.0), 0.4);
        assert!(!aabbs_overlap(min_a, max_a, min_b, max_b));
    }

    #[test]
    fn test_invert_transform() {
        let t = PxTransform::new(Vec3::new(1.0, 2.0, 3.0), Quaternion::IDENTITY);
        let inv = invert_px_transform(&t);
        assert!((inv.position.x - (-1.0)).abs() < 1e-5);
        assert!((inv.position.y - (-2.0)).abs() < 1e-5);
    }
}
