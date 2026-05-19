use super::types::JointType2D;

#[derive(Debug, Clone)]
pub struct Joint2D {
    pub id: u32,
    pub joint_type: JointType2D,
    pub body_a_id: u32,
    pub body_b_id: u32,
    pub anchor_a: [f32; 2],
    pub anchor_b: [f32; 2],
    pub max_force: f32,
    pub max_torque: f32,
    pub enabled: bool,
    pub collide_connected: bool,
}

impl Joint2D {
    pub fn new(id: u32, joint_type: JointType2D, body_a: u32, body_b: u32) -> Self {
        Self {
            id,
            joint_type,
            body_a_id: body_a,
            body_b_id: body_b,
            anchor_a: [0.0, 0.0],
            anchor_b: [0.0, 0.0],
            max_force: 0.0,
            max_torque: 0.0,
            enabled: true,
            collide_connected: false,
        }
    }

    pub fn set_anchors(&mut self, anchor_a: [f32; 2], anchor_b: [f32; 2]) {
        self.anchor_a = anchor_a;
        self.anchor_b = anchor_b;
    }

    #[allow(dead_code)]
    pub fn get_reaction_force(&self, _inv_dt: f32) -> [f32; 2] {
        [0.0, 0.0]
    }

    #[allow(dead_code)]
    pub fn get_reaction_torque(&self, _inv_dt: f32) -> f32 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_new() {
        let joint = Joint2D::new(1, JointType2D::Revolute, 1, 2);
        assert_eq!(joint.id, 1);
        assert_eq!(joint.body_a_id, 1);
        assert_eq!(joint.body_b_id, 2);
        assert!(joint.enabled);
    }

    #[test]
    fn test_set_anchors() {
        let mut joint = Joint2D::new(1, JointType2D::Distance, 1, 2);
        joint.set_anchors([1.0, 0.0], [-1.0, 0.0]);
        assert_eq!(joint.anchor_a, [1.0, 0.0]);
        assert_eq!(joint.anchor_b, [-1.0, 0.0]);
    }

    #[test]
    fn test_reaction_force() {
        let joint = Joint2D::new(1, JointType2D::Weld, 1, 2);
        let force = joint.get_reaction_force(1.0 / 60.0);
        assert_eq!(force, [0.0, 0.0]);
    }

    #[test]
    fn test_reaction_torque() {
        let joint = Joint2D::new(1, JointType2D::Motor, 1, 2);
        let torque = joint.get_reaction_torque(1.0 / 60.0);
        assert_eq!(torque, 0.0);
    }

    #[test]
    fn test_joint_types() {
        let types = [
            JointType2D::Distance, JointType2D::Spring, JointType2D::Wheel,
            JointType2D::Revolute, JointType2D::Prismatic, JointType2D::Rope,
            JointType2D::Weld, JointType2D::Motor, JointType2D::Mouse,
            JointType2D::Relative,
        ];
        for t in &types {
            let joint = Joint2D::new(1, *t, 1, 2);
            assert_eq!(joint.joint_type, *t);
        }
    }
}
