#[derive(Debug, Clone)]
pub struct SpineBone {
    pub name: String,
    pub parent: Option<String>,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub shear_x: f32,
    pub shear_y: f32,
    pub world_x: f32,
    pub world_y: f32,
    pub world_rotation: f32,
    pub world_scale_x: f32,
    pub world_scale_y: f32,
    pub length: f32,
}

impl SpineBone {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            parent: None,
            x: 0.0, y: 0.0,
            rotation: 0.0,
            scale_x: 1.0, scale_y: 1.0,
            shear_x: 0.0, shear_y: 0.0,
            world_x: 0.0, world_y: 0.0,
            world_rotation: 0.0,
            world_scale_x: 1.0, world_scale_y: 1.0,
            length: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spine_bone_new() {
        let bone = SpineBone::new("root");
        assert_eq!(bone.name, "root");
        assert_eq!(bone.scale_x, 1.0);
    }
}
