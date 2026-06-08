use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType2D {
    Static,
    Dynamic,
    Kinematic,
    Animated,
}

impl Default for RigidBodyType2D {
    fn default() -> Self {
        Self::Dynamic
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointType2D {
    Distance,
    Spring,
    Wheel,
    Revolute,
    Prismatic,
    Rope,
    Weld,
    Motor,
    Mouse,
    Relative,
}

impl Default for JointType2D {
    fn default() -> Self {
        Self::Distance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColliderType2D {
    Box,
    Circle,
    Polygon,
    Capsule,
    Edge,
}

impl Default for ColliderType2D {
    fn default() -> Self {
        Self::Box
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsMaterial2D {
    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
}

impl Default for PhysicsMaterial2D {
    fn default() -> Self {
        Self {
            density: 1.0,
            friction: 0.5,
            restitution: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB2D {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB2D {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }
    pub fn contains(&self, point: &Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }
    pub fn overlaps(&self, other: &AABB2D) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCastResult2D {
    pub hit: bool,
    pub point: Vec2,
    pub normal: Vec2,
    pub fraction: f32,
}

impl Default for RayCastResult2D {
    fn default() -> Self {
        Self {
            hit: false,
            point: Vec2::ZERO,
            normal: Vec2::ZERO,
            fraction: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactPoint2D {
    pub point: Vec2,
    pub normal: Vec2,
    pub impulse: f32,
    pub separation: f32,
}
