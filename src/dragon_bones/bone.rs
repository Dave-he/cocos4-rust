#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoneType {
    Root,
    Joint,
    Limb,
}

#[derive(Debug, Clone)]
pub struct Bone {
    pub name: String,
    pub parent: Option<String>,
    pub bone_type: BoneType,
    pub position: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
    pub world_position: [f32; 2],
    pub world_rotation: f32,
    pub world_scale: [f32; 2],
    pub length: f32,
    pub inherit_rotation: bool,
    pub inherit_scale: bool,
}

impl Bone {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: None,
            bone_type: BoneType::Joint,
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            world_position: [0.0, 0.0],
            world_rotation: 0.0,
            world_scale: [1.0, 1.0],
            length: 0.0,
            inherit_rotation: true,
            inherit_scale: true,
        }
    }

    pub fn update_world_transform(
        &mut self,
        parent_wp: &[f32; 2],
        parent_rot: f32,
        parent_scale: &[f32; 2],
    ) {
        let cos = parent_rot.to_radians().cos();
        let sin = parent_rot.to_radians().sin();
        let rx = self.position[0] * cos - self.position[1] * sin;
        let ry = self.position[0] * sin + self.position[1] * cos;
        self.world_position = [
            parent_wp[0] + rx * parent_scale[0],
            parent_wp[1] + ry * parent_scale[1],
        ];
        self.world_rotation = parent_rot + self.rotation;
        self.world_scale = [
            parent_scale[0] * self.scale[0],
            parent_scale[1] * self.scale[1],
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_new() {
        let bone = Bone::new("root");
        assert_eq!(bone.name, "root");
        assert_eq!(bone.scale, [1.0, 1.0]);
    }

    #[test]
    fn test_bone_world_transform() {
        let mut child = Bone::new("child");
        child.position = [10.0, 0.0];
        child.rotation = 45.0;
        child.update_world_transform(&[0.0, 0.0], 0.0, &[1.0, 1.0]);
        assert!((child.world_position[0] - 10.0).abs() < 0.01);
        assert_eq!(child.world_rotation, 45.0);
    }

    #[test]
    fn test_bone_inheritance() {
        let bone = Bone::new("arm");
        assert!(bone.inherit_rotation);
        assert!(bone.inherit_scale);
    }
}
