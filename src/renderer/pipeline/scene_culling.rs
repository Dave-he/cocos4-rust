use crate::scene::Camera;
use crate::scene::Model;
use crate::scene::lights::{SpotLight, PointLight, SphereLight};
use crate::core::geometry::{Frustum, Sphere, AABB, aabb_plane, sphere_plane, PlaneIntersectResult};
use crate::math::Vec3;

#[derive(Debug, Default)]
pub struct SceneCulling {
    pub visible_models: Vec<u64>,
    pub visible_spot_lights: Vec<u32>,
    pub visible_point_lights: Vec<u32>,
    pub visible_sphere_lights: Vec<u32>,
}

fn aabb_in_frustum(frustum: &Frustum, aabb: &AABB) -> bool {
    for plane in &frustum.planes {
        if aabb_plane(aabb, plane) == PlaneIntersectResult::InsideBack {
            return false;
        }
    }
    true
}

fn sphere_in_frustum(frustum: &Frustum, sphere: &Sphere) -> bool {
    for plane in &frustum.planes {
        if sphere_plane(sphere, plane) == PlaneIntersectResult::InsideBack {
            return false;
        }
    }
    true
}

fn sphere_from_point_range(center: Vec3, range: f32) -> Sphere {
    Sphere::new(center, range)
}

impl SceneCulling {
    pub fn new() -> Self {
        SceneCulling {
            visible_models: Vec::new(),
            visible_spot_lights: Vec::new(),
            visible_point_lights: Vec::new(),
            visible_sphere_lights: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.visible_models.clear();
        self.visible_spot_lights.clear();
        self.visible_point_lights.clear();
        self.visible_sphere_lights.clear();
    }

    pub fn cull_models(&mut self, camera: &Camera, models: &[Model]) {
        self.visible_models.clear();
        let frustum = camera.get_frustum();

        for model in models {
            if !model.enabled {
                continue;
            }

            if model.visibility & camera.visibility == 0 {
                continue;
            }

            if !model.enable_bounding_box_culling {
                self.visible_models.push(model.model_id);
                continue;
            }

            let visible = if let Some(bounds) = &model.world_bounds {
                if let Some(frustum) = &frustum {
                    aabb_in_frustum(frustum, bounds)
                } else {
                    true
                }
            } else {
                true
            };

            if visible {
                self.visible_models.push(model.model_id);
            }
        }
    }

    pub fn cull_spot_lights(&mut self, camera: &Camera, lights: &[SpotLight]) {
        self.visible_spot_lights.clear();
        let frustum = camera.get_frustum();
        let Some(frustum) = &frustum else { return };

        for (i, light) in lights.iter().enumerate() {
            if (light.base.visibility & camera.visibility) == 0 {
                continue;
            }
            let sphere = sphere_from_point_range(light.position, light.range);
            if sphere_in_frustum(frustum, &sphere) {
                self.visible_spot_lights.push(i as u32);
            }
        }
    }

    pub fn cull_point_lights(&mut self, camera: &Camera, lights: &[PointLight]) {
        self.visible_point_lights.clear();
        let frustum = camera.get_frustum();
        let Some(frustum) = &frustum else { return };

        for (i, light) in lights.iter().enumerate() {
            if (light.base.visibility & camera.visibility) == 0 {
                continue;
            }
            let sphere = sphere_from_point_range(light.position, light.range);
            if sphere_in_frustum(frustum, &sphere) {
                self.visible_point_lights.push(i as u32);
            }
        }
    }

    pub fn cull_sphere_lights(&mut self, camera: &Camera, lights: &[SphereLight]) {
        self.visible_sphere_lights.clear();
        let frustum = camera.get_frustum();
        let Some(frustum) = &frustum else { return };

        for (i, light) in lights.iter().enumerate() {
            if (light.base.visibility & camera.visibility) == 0 {
                continue;
            }
            let sphere = sphere_from_point_range(light.position, light.range);
            if sphere_in_frustum(frustum, &sphere) {
                self.visible_sphere_lights.push(i as u32);
            }
        }
    }

    pub fn get_visible_model_count(&self) -> usize {
        self.visible_models.len()
    }

    pub fn is_visible(&self, model_id: u64) -> bool {
        self.visible_models.contains(&model_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Model;
    use crate::core::geometry::AABB;

    fn make_camera() -> Camera {
        let mut cam = Camera::new();
        cam.enabled = true;
        cam.visibility = 0xFFFFFFFF;
        cam
    }

    fn make_model(id: u64, enabled: bool) -> Model {
        let mut m = Model::new(id);
        m.enabled = enabled;
        m.visibility = 0xFFFFFFFF;
        m
    }

    #[test]
    fn test_scene_culling_new() {
        let c = SceneCulling::new();
        assert_eq!(c.get_visible_model_count(), 0);
    }

    #[test]
    fn test_scene_culling_clear() {
        let mut c = SceneCulling::new();
        c.visible_models.push(1);
        c.clear();
        assert_eq!(c.get_visible_model_count(), 0);
    }

    #[test]
    fn test_scene_culling_is_visible() {
        let mut c = SceneCulling::new();
        c.visible_models.push(42);
        assert!(c.is_visible(42));
        assert!(!c.is_visible(99));
    }

    #[test]
    fn test_cull_disabled_model() {
        let mut c = SceneCulling::new();
        let cam = make_camera();
        let models = vec![make_model(1, false)];
        c.cull_models(&cam, &models);
        assert_eq!(c.get_visible_model_count(), 0);
    }

    #[test]
    fn test_cull_visibility_mask() {
        let mut c = SceneCulling::new();
        let mut m = make_model(1, true);
        m.visibility = 0x00000001;
        let mut cam2 = make_camera();
        cam2.visibility = 0x00000002;
        let models = vec![m];
        c.cull_models(&cam2, &models);
        assert_eq!(c.get_visible_model_count(), 0);
    }

    #[test]
    fn test_cull_no_bounds_always_visible() {
        let mut c = SceneCulling::new();
        let cam = make_camera();
        let mut m = make_model(1, true);
        m.world_bounds = None;
        let models = vec![m];
        c.cull_models(&cam, &models);
        assert_eq!(c.get_visible_model_count(), 1);
    }

    #[test]
    fn test_cull_bounding_box_disabled() {
        let mut c = SceneCulling::new();
        let cam = make_camera();
        let mut m = make_model(1, true);
        m.enable_bounding_box_culling = false;
        m.world_bounds = Some(AABB::new(9999.0, 9999.0, 9999.0, 0.1, 0.1, 0.1));
        let models = vec![m];
        c.cull_models(&cam, &models);
        assert_eq!(c.get_visible_model_count(), 1);
    }

    #[test]
    fn test_cull_multiple_models() {
        let mut c = SceneCulling::new();
        let cam = make_camera();
        let models = vec![
            make_model(1, true),
            make_model(2, false),
            make_model(3, true),
        ];
        c.cull_models(&cam, &models);
        assert_eq!(c.get_visible_model_count(), 2);
        assert!(c.is_visible(1));
        assert!(!c.is_visible(2));
        assert!(c.is_visible(3));
    }
}
