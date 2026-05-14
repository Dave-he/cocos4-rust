use crate::core::geometry::AABB;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProbeType {
    #[default]
    Cube = 0,
    Planar = 1,
}

pub struct ReflectionProbe {
    pub probe_id: i32,
    pub probe_type: ProbeType,
    pub resolution: i32,
    pub visibility: i32,
    pub need_render: bool,
    pub bounding_size: Vec3,
    pub bounding_box: Option<AABB>,
    pub background_color: [f32; 4],
}

impl ReflectionProbe {
    pub fn new(id: i32) -> Self {
        ReflectionProbe {
            probe_id: id,
            probe_type: ProbeType::Cube,
            resolution: 256,
            visibility: 0,
            need_render: false,
            bounding_size: Vec3::ONE,
            bounding_box: None,
            background_color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn set_probe_type(&mut self, type_: ProbeType) {
        self.probe_type = type_;
    }

    pub fn get_probe_type(&self) -> ProbeType {
        self.probe_type
    }

    pub fn get_probe_id(&self) -> i32 {
        self.probe_id
    }

    pub fn set_resolution(&mut self, resolution: i32) {
        self.resolution = resolution;
    }

    pub fn get_resolution(&self) -> i32 {
        self.resolution
    }

    pub fn set_visibility(&mut self, val: i32) {
        self.visibility = val;
    }

    pub fn get_visibility(&self) -> i32 {
        self.visibility
    }

    pub fn set_bounding_size(&mut self, val: Vec3) {
        self.bounding_size = val;
    }

    pub fn get_bounding_size(&self) -> &Vec3 {
        &self.bounding_size
    }

    pub fn set_background_color(&mut self, val: [f32; 4]) {
        self.background_color = val;
    }

    pub fn get_background_color(&self) -> &[f32; 4] {
        &self.background_color
    }

    pub fn update_bounding_box(&mut self) {
        let half = self.bounding_size * 0.5;
        self.bounding_box = Some(AABB::new(0.0, 0.0, 0.0, half.x, half.y, half.z));
    }

    pub fn get_bounding_box(&self) -> Option<&AABB> {
        self.bounding_box.as_ref()
    }

    pub fn need_render(&self) -> bool {
        self.need_render
    }

    pub fn set_need_render(&mut self, b: bool) {
        self.need_render = b;
    }

    pub fn destroy(&mut self) {
        self.bounding_box = None;
        self.need_render = false;
    }

    pub fn enable(&mut self) {
        self.need_render = true;
    }

    pub fn disable(&mut self) {
        self.need_render = false;
    }

    pub fn reflect(point: Vec3, normal: Vec3, offset: f32) -> Vec3 {
        let dot = Vec3::dot(&point, &normal);
        point - normal * (2.0 * dot + offset)
    }

    pub fn switch_probe_type(&mut self, type_: i32) {
        self.probe_type = if type_ == 0 {
            ProbeType::Cube
        } else {
            ProbeType::Planar
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_probe_new() {
        let probe = ReflectionProbe::new(1);
        assert_eq!(probe.probe_id, 1);
        assert_eq!(probe.probe_type, ProbeType::Cube);
        assert_eq!(probe.resolution, 256);
    }

    #[test]
    fn test_reflection_probe_update_bbox() {
        let mut probe = ReflectionProbe::new(1);
        probe.set_bounding_size(Vec3::new(10.0, 10.0, 10.0));
        probe.update_bounding_box();
        assert!(probe.bounding_box.is_some());
    }

    #[test]
    fn test_reflection_probe_reflect() {
        let point = Vec3::new(1.0, 1.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let reflected = ReflectionProbe::reflect(point, normal, 0.0);
        assert!((reflected.y - (-1.0)).abs() < 1e-5);
    }
}
