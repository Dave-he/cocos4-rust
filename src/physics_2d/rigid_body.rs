use super::types::RigidBodyType2D;

#[derive(Debug, Clone)]
pub struct RigidBody2D {
    pub id: u32,
    pub body_type: RigidBodyType2D,
    pub mass: f32,
    pub linear_velocity: [f32; 2],
    pub angular_velocity: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub gravity_scale: f32,
    pub fixed_rotation: bool,
    pub awake: bool,
    pub enabled: bool,
}

impl RigidBody2D {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            body_type: RigidBodyType2D::Dynamic,
            mass: 1.0,
            linear_velocity: [0.0, 0.0],
            angular_velocity: 0.0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            fixed_rotation: false,
            awake: true,
            enabled: true,
        }
    }

    pub fn set_type(&mut self, body_type: RigidBodyType2D) {
        self.body_type = body_type;
    }

    pub fn sleep(&mut self) {
        self.awake = false;
    }

    pub fn wake_up(&mut self) {
        self.awake = true;
    }

    pub fn is_awake(&self) -> bool {
        self.awake
    }

    pub fn apply_force(&mut self, force: [f32; 2]) {
        if self.body_type != RigidBodyType2D::Dynamic {
            return;
        }
        if self.mass > 0.0 {
            self.linear_velocity[0] += force[0] / self.mass;
            self.linear_velocity[1] += force[1] / self.mass;
            self.awake = true;
        }
    }

    pub fn apply_impulse(&mut self, impulse: [f32; 2]) {
        if self.body_type != RigidBodyType2D::Dynamic {
            return;
        }
        if self.mass > 0.0 {
            self.linear_velocity[0] += impulse[0] / self.mass;
            self.linear_velocity[1] += impulse[1] / self.mass;
            self.awake = true;
        }
    }

    pub fn get_mass(&self) -> f32 {
        self.mass
    }
    pub fn is_static(&self) -> bool {
        self.body_type == RigidBodyType2D::Static
    }
    pub fn is_kinematic(&self) -> bool {
        self.body_type == RigidBodyType2D::Kinematic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_new() {
        let body = RigidBody2D::new(1);
        assert_eq!(body.id, 1);
        assert_eq!(body.body_type, RigidBodyType2D::Dynamic);
        assert_eq!(body.mass, 1.0);
    }

    #[test]
    fn test_apply_force() {
        let mut body = RigidBody2D::new(1);
        body.apply_force([10.0, 0.0]);
        assert!((body.linear_velocity[0] - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_static_no_force() {
        let mut body = RigidBody2D::new(1);
        body.body_type = RigidBodyType2D::Static;
        body.apply_force([10.0, 0.0]);
        assert_eq!(body.linear_velocity[0], 0.0);
    }

    #[test]
    fn test_is_static_kinematic() {
        let mut body = RigidBody2D::new(1);
        assert!(!body.is_static());
        assert!(!body.is_kinematic());
        body.body_type = RigidBodyType2D::Static;
        assert!(body.is_static());
        body.body_type = RigidBodyType2D::Kinematic;
        assert!(body.is_kinematic());
    }

    #[test]
    fn test_apply_impulse() {
        let mut body = RigidBody2D::new(1);
        body.apply_impulse([0.0, 20.0]);
        assert!((body.linear_velocity[1] - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_sleep_wake_equivalent_case() {
        let mut body = RigidBody2D::new(1);
        assert!(body.is_awake());
        body.sleep();
        assert!(!body.is_awake());
        body.wake_up();
        assert!(body.is_awake());
    }
}
