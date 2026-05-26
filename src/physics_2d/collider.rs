use super::types::{ColliderType2D, PhysicsMaterial2D, AABB2D};

#[derive(Debug, Clone)]
pub struct Collider2D {
    pub id: u32,
    pub collider_type: ColliderType2D,
    pub material: PhysicsMaterial2D,
    pub offset: [f32; 2],
    pub size: [f32; 2],
    pub radius: f32,
    pub is_trigger: bool,
    pub enabled: bool,
    pub group: u32,
    pub mask: u32,
}

impl Collider2D {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            collider_type: ColliderType2D::Box,
            material: PhysicsMaterial2D::default(),
            offset: [0.0, 0.0],
            size: [1.0, 1.0],
            radius: 0.5,
            is_trigger: false,
            enabled: true,
            group: 1,
            mask: 0xFFFFFFFF,
        }
    }

    pub fn set_as_box(&mut self, width: f32, height: f32) {
        self.collider_type = ColliderType2D::Box;
        self.size = [width, height];
    }

    pub fn set_as_circle(&mut self, radius: f32) {
        self.collider_type = ColliderType2D::Circle;
        self.radius = radius;
    }

    pub fn get_aabb(&self, position: [f32; 2]) -> AABB2D {
        match self.collider_type {
            ColliderType2D::Box => {
                AABB2D::new(
                    crate::math::Vec2::new(
                        position[0] + self.offset[0] - self.size[0] * 0.5,
                        position[1] + self.offset[1] - self.size[1] * 0.5,
                    ),
                    crate::math::Vec2::new(
                        position[0] + self.offset[0] + self.size[0] * 0.5,
                        position[1] + self.offset[1] + self.size[1] * 0.5,
                    ),
                )
            }
            ColliderType2D::Circle => {
                AABB2D::new(
                    crate::math::Vec2::new(
                        position[0] + self.offset[0] - self.radius,
                        position[1] + self.offset[1] - self.radius,
                    ),
                    crate::math::Vec2::new(
                        position[0] + self.offset[0] + self.radius,
                        position[1] + self.offset[1] + self.radius,
                    ),
                )
            }
            _ => AABB2D::new(
                crate::math::Vec2::new(position[0] - 1.0, position[1] - 1.0),
                crate::math::Vec2::new(position[0] + 1.0, position[1] + 1.0),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collider_new() {
        let collider = Collider2D::new(1);
        assert_eq!(collider.id, 1);
        assert_eq!(collider.collider_type, ColliderType2D::Box);
        assert!(collider.enabled);
    }

    #[test]
    fn test_set_as_box() {
        let mut collider = Collider2D::new(1);
        collider.set_as_box(2.0, 3.0);
        assert_eq!(collider.size, [2.0, 3.0]);
    }

    #[test]
    fn test_set_as_circle() {
        let mut collider = Collider2D::new(1);
        collider.set_as_circle(1.5);
        assert_eq!(collider.collider_type, ColliderType2D::Circle);
        assert_eq!(collider.radius, 1.5);
    }

    #[test]
    fn test_get_aabb_box() {
        let collider = Collider2D::new(1);
        let aabb = collider.get_aabb([0.0, 0.0]);
        assert!(aabb.min.x < aabb.max.x);
        assert!(aabb.min.y < aabb.max.y);
    }

    #[test]
    fn test_get_aabb_circle() {
        let mut collider = Collider2D::new(1);
        collider.set_as_circle(1.0);
        let aabb = collider.get_aabb([5.0, 5.0]);
        assert!(aabb.min.x < 5.0);
        assert!(aabb.max.x > 5.0);
    }

    #[test]
    fn test_collider_material_and_sensor_equivalent_case() {
        let mut collider = Collider2D::new(1);
        collider.material.density = 2.5;
        collider.material.friction = 0.3;
        collider.material.restitution = 0.8;
        collider.is_trigger = true;
        collider.group = 0b0010;
        collider.mask = 0b0100;

        assert!((collider.material.density - 2.5).abs() < 1e-6);
        assert!((collider.material.friction - 0.3).abs() < 1e-6);
        assert!((collider.material.restitution - 0.8).abs() < 1e-6);
        assert!(collider.is_trigger);
        assert_eq!(collider.group, 0b0010);
        assert_eq!(collider.mask, 0b0100);
    }
}
