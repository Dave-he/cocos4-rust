use super::camera::Camera;
use super::directional_light::DirectionalLight;
use super::lights::{SpotLight, PointLight, SphereLight, RangedDirectionalLight};
use super::model::Model;

pub struct RenderSceneInfo {
    pub name: String,
}

pub struct RenderScene {
    pub name: String,
    model_id_counter: u64,
    pub main_light: Option<DirectionalLight>,
    pub directional_lights: Vec<DirectionalLight>,
    pub models: Vec<Model>,
    pub cameras: Vec<Camera>,
    pub spot_lights: Vec<SpotLight>,
    pub point_lights: Vec<PointLight>,
    pub sphere_lights: Vec<SphereLight>,
    pub ranged_dir_lights: Vec<RangedDirectionalLight>,
}

impl RenderScene {
    pub fn new() -> Self {
        RenderScene {
            name: String::new(),
            model_id_counter: 0,
            main_light: None,
            directional_lights: Vec::new(),
            models: Vec::new(),
            cameras: Vec::new(),
            spot_lights: Vec::new(),
            point_lights: Vec::new(),
            sphere_lights: Vec::new(),
            ranged_dir_lights: Vec::new(),
        }
    }

    pub fn initialize(&mut self, info: RenderSceneInfo) -> bool {
        self.name = info.name;
        true
    }

    pub fn destroy(&mut self) {
        self.remove_cameras();
        self.remove_models();
        self.remove_spot_lights();
        self.remove_point_lights();
        self.remove_sphere_lights();
        self.remove_ranged_dir_lights();
        self.directional_lights.clear();
        self.main_light = None;
    }

    pub fn update(&mut self, _stamp: u32) {}

    pub fn generate_model_id(&mut self) -> u64 {
        let id = self.model_id_counter;
        self.model_id_counter += 1;
        id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.cameras.push(camera);
    }

    pub fn remove_camera(&mut self, camera_id: u32) {
        self.cameras.retain(|c| c.camera_id != camera_id);
    }

    pub fn remove_cameras(&mut self) {
        self.cameras.clear();
    }

    pub fn get_cameras(&self) -> &[Camera] {
        &self.cameras
    }

    pub fn set_main_light(&mut self, light: Option<DirectionalLight>) {
        self.main_light = light;
    }

    pub fn get_main_light(&self) -> Option<&DirectionalLight> {
        self.main_light.as_ref()
    }

    pub fn add_directional_light(&mut self, light: DirectionalLight) {
        self.directional_lights.push(light);
    }

    pub fn remove_directional_light(&mut self, name: &str) {
        self.directional_lights.retain(|l| l.base.name != name);
    }

    pub fn add_sphere_light(&mut self, light: SphereLight) {
        self.sphere_lights.push(light);
    }

    pub fn remove_sphere_lights(&mut self) {
        self.sphere_lights.clear();
    }

    pub fn get_sphere_lights(&self) -> &[SphereLight] {
        &self.sphere_lights
    }

    pub fn add_spot_light(&mut self, light: SpotLight) {
        self.spot_lights.push(light);
    }

    pub fn remove_spot_lights(&mut self) {
        self.spot_lights.clear();
    }

    pub fn get_spot_lights(&self) -> &[SpotLight] {
        &self.spot_lights
    }

    pub fn add_point_light(&mut self, light: PointLight) {
        self.point_lights.push(light);
    }

    pub fn remove_point_lights(&mut self) {
        self.point_lights.clear();
    }

    pub fn get_point_lights(&self) -> &[PointLight] {
        &self.point_lights
    }

    pub fn add_ranged_dir_light(&mut self, light: RangedDirectionalLight) {
        self.ranged_dir_lights.push(light);
    }

    pub fn remove_ranged_dir_lights(&mut self) {
        self.ranged_dir_lights.clear();
    }

    pub fn get_ranged_dir_lights(&self) -> &[RangedDirectionalLight] {
        &self.ranged_dir_lights
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn remove_model(&mut self, model_id: u64) {
        self.models.retain(|m| m.model_id != model_id);
    }

    pub fn remove_models(&mut self) {
        self.models.clear();
    }

    pub fn get_models(&self) -> &[Model] {
        &self.models
    }

    pub fn get_models_mut(&mut self) -> &mut Vec<Model> {
        &mut self.models
    }

    pub fn create_model(&mut self) -> u64 {
        let id = self.generate_model_id();
        let model = Model::new(id);
        self.models.push(model);
        id
    }
}

impl Default for RenderScene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_scene_new() {
        let scene = RenderScene::new();
        assert_eq!(scene.name, "");
        assert!(scene.models.is_empty());
        assert!(scene.cameras.is_empty());
        assert!(scene.main_light.is_none());
    }

    #[test]
    fn test_render_scene_initialize() {
        let mut scene = RenderScene::new();
        let result = scene.initialize(RenderSceneInfo {
            name: "TestScene".to_string(),
        });
        assert!(result);
        assert_eq!(scene.name, "TestScene");
    }

    #[test]
    fn test_render_scene_models() {
        let mut scene = RenderScene::new();
        let id = scene.create_model();
        assert_eq!(scene.models.len(), 1);
        assert_eq!(scene.models[0].model_id, id);

        scene.remove_model(id);
        assert!(scene.models.is_empty());
    }

    #[test]
    fn test_render_scene_generate_id() {
        let mut scene = RenderScene::new();
        let id1 = scene.generate_model_id();
        let id2 = scene.generate_model_id();
        assert_ne!(id1, id2);
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_render_scene_lights() {
        let mut scene = RenderScene::new();
        scene.add_directional_light(DirectionalLight::new());
        scene.add_spot_light(SpotLight::new());
        scene.add_point_light(PointLight::new());
        scene.add_sphere_light(SphereLight::new());

        assert_eq!(scene.directional_lights.len(), 1);
        assert_eq!(scene.spot_lights.len(), 1);
        assert_eq!(scene.point_lights.len(), 1);
        assert_eq!(scene.sphere_lights.len(), 1);

        scene.remove_spot_lights();
        assert!(scene.spot_lights.is_empty());
    }

    #[test]
    fn test_render_scene_destroy() {
        let mut scene = RenderScene::new();
        scene.create_model();
        scene.add_point_light(PointLight::new());
        scene.destroy();
        assert!(scene.models.is_empty());
        assert!(scene.point_lights.is_empty());
    }
}
