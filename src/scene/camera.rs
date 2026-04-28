use crate::math::{Mat4, Vec3, Vec4};
use crate::core::geometry::{Frustum, Plane, Ray};
use super::define::{
    CameraAperture, CameraFOVAxis, CameraISO, CameraProjection, CameraShutter,
    CameraType, CameraUsage, TrackingType,
};

const FSTOPS: &[f32] = &[
    1.8, 2.0, 2.2, 2.5, 2.8, 3.2, 3.5, 4.0, 4.5, 5.0, 5.6, 6.3,
    7.1, 8.0, 9.0, 10.0, 11.0, 13.0, 14.0, 16.0, 18.0, 20.0, 22.0,
];

const SHUTTERS: &[f32] = &[
    1.0, 1.0 / 2.0, 1.0 / 4.0, 1.0 / 8.0, 1.0 / 15.0, 1.0 / 30.0,
    1.0 / 60.0, 1.0 / 125.0, 1.0 / 250.0, 1.0 / 500.0, 1.0 / 1000.0,
    1.0 / 2000.0, 1.0 / 4000.0,
];

const ISOS: &[f32] = &[100.0, 200.0, 400.0, 800.0];

pub struct CameraInfo {
    pub name: String,
    pub node_uuid: Option<String>,
    pub projection: CameraProjection,
    pub priority: u32,
    pub camera_type: CameraType,
    pub tracking_type: TrackingType,
    pub usage: CameraUsage,
}

impl Default for CameraInfo {
    fn default() -> Self {
        CameraInfo {
            name: String::new(),
            node_uuid: None,
            projection: CameraProjection::Perspective,
            priority: 0,
            camera_type: CameraType::Default,
            tracking_type: TrackingType::NoTracking,
            usage: CameraUsage::Game,
        }
    }
}

pub struct Camera {
    pub name: String,
    pub node_uuid: Option<String>,
    pub enabled: bool,
    pub is_culling_enabled: bool,
    pub projection: CameraProjection,
    pub fov_axis: CameraFOVAxis,
    pub fov: f32,
    pub ortho_height: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub aspect: f32,
    pub width: u32,
    pub height: u32,
    pub clear_color: [f32; 4],
    pub clear_depth: f32,
    pub clear_stencil: u32,
    pub clear_flag: u32,
    pub viewport: Vec4,
    pub forward: Vec3,
    pub position: Vec3,
    pub priority: u32,
    pub aperture: CameraAperture,
    pub aperture_value: f32,
    pub shutter: CameraShutter,
    pub shutter_value: f32,
    pub iso: CameraISO,
    pub iso_value: f32,
    pub ec: f32,
    pub exposure: f32,
    pub mat_view: Mat4,
    pub mat_proj: Mat4,
    pub mat_proj_inv: Mat4,
    pub mat_view_proj: Mat4,
    pub mat_view_proj_inv: Mat4,
    pub is_window_size: bool,
    pub screen_scale: f32,
    pub visibility: u32,
    pub camera_type: CameraType,
    pub tracking_type: TrackingType,
    pub usage: CameraUsage,
    pub camera_id: u32,
    pub is_proj_dirty: bool,
}

impl Camera {
    pub const STANDARD_EXPOSURE_VALUE: f32 = 1.0 / 38400.0;
    pub const STANDARD_LIGHT_METER_SCALE: f32 = 10000.0;

    pub fn new() -> Self {
        let aperture = CameraAperture::F16_0;
        let shutter = CameraShutter::D125;
        let iso = CameraISO::Iso100;

        let aperture_value = FSTOPS[aperture as usize];
        let shutter_value = SHUTTERS[shutter as usize];
        let iso_value = ISOS[iso as usize];
        let exposure = (aperture_value * aperture_value) / (shutter_value * iso_value * 1.0);

        Camera {
            name: String::new(),
            node_uuid: None,
            enabled: false,
            is_culling_enabled: true,
            projection: CameraProjection::Perspective,
            fov_axis: CameraFOVAxis::Vertical,
            fov: 45.0_f32.to_radians(),
            ortho_height: 10.0,
            near_clip: 1.0,
            far_clip: 1000.0,
            aspect: 0.0,
            width: 0,
            height: 0,
            clear_color: [0.2, 0.2, 0.2, 1.0],
            clear_depth: 1.0,
            clear_stencil: 0,
            clear_flag: 0,
            viewport: Vec4::new(0.0, 0.0, 1.0, 1.0),
            forward: Vec3::ZERO,
            position: Vec3::ZERO,
            priority: 0,
            aperture,
            aperture_value,
            shutter,
            shutter_value,
            iso,
            iso_value,
            ec: 0.0,
            exposure,
            mat_view: Mat4::IDENTITY,
            mat_proj: Mat4::IDENTITY,
            mat_proj_inv: Mat4::IDENTITY,
            mat_view_proj: Mat4::IDENTITY,
            mat_view_proj_inv: Mat4::IDENTITY,
            is_window_size: true,
            screen_scale: 0.0,
            visibility: super::define::CAMERA_DEFAULT_MASK,
            camera_type: CameraType::Default,
            tracking_type: TrackingType::NoTracking,
            usage: CameraUsage::Game,
            camera_id: 0,
            is_proj_dirty: true,
        }
    }

    pub fn initialize(&mut self, info: CameraInfo) -> bool {
        self.name = info.name;
        self.node_uuid = info.node_uuid;
        self.projection = info.projection;
        self.priority = info.priority;
        self.camera_type = info.camera_type;
        self.tracking_type = info.tracking_type;
        self.usage = info.usage;
        true
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.aspect = width as f32 / height as f32;
        self.is_proj_dirty = true;
    }

    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.is_proj_dirty = true;
    }

    pub fn set_near_clip(&mut self, near: f32) {
        self.near_clip = near;
        self.is_proj_dirty = true;
    }

    pub fn set_far_clip(&mut self, far: f32) {
        self.far_clip = far;
        self.is_proj_dirty = true;
    }

    pub fn set_ortho_height(&mut self, height: f32) {
        self.ortho_height = height;
        self.is_proj_dirty = true;
    }

    pub fn set_projection_type(&mut self, proj: CameraProjection) {
        self.projection = proj;
        self.is_proj_dirty = true;
    }

    pub fn set_fov_axis(&mut self, axis: CameraFOVAxis) {
        self.fov_axis = axis;
        self.is_proj_dirty = true;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_visibility(&mut self, vis: u32) {
        self.visibility = vis;
    }

    pub fn set_priority(&mut self, priority: u32) {
        self.priority = priority;
    }

    pub fn set_aperture(&mut self, aperture: CameraAperture) {
        self.aperture = aperture;
        self.aperture_value = FSTOPS[aperture as usize];
        self.update_exposure();
    }

    pub fn set_shutter(&mut self, shutter: CameraShutter) {
        self.shutter = shutter;
        self.shutter_value = SHUTTERS[shutter as usize];
        self.update_exposure();
    }

    pub fn set_iso(&mut self, iso: CameraISO) {
        self.iso = iso;
        self.iso_value = ISOS[iso as usize];
        self.update_exposure();
    }

    pub fn set_ec(&mut self, ec: f32) {
        self.ec = ec;
    }

    fn update_exposure(&mut self) {
        let ev100 = (self.aperture_value * self.aperture_value)
            .log2()
            - self.shutter_value.log2()
            - (self.iso_value / 100.0).log2()
            + self.ec;
        self.exposure = 0.833333 / (2.0_f32.powf(ev100));
    }

    pub fn update_matrices(&mut self) {
        if !self.is_proj_dirty {
            return;
        }
        self.is_proj_dirty = false;

        if self.projection == CameraProjection::Perspective {
            let aspect = if self.aspect > 0.0 {
                self.aspect
            } else {
                1.0
            };
            let fov = match self.fov_axis {
                CameraFOVAxis::Vertical => self.fov,
                CameraFOVAxis::Horizontal => 2.0 * ((self.fov * 0.5).tan() / aspect).atan(),
            };
            self.mat_proj = Mat4::perspective(fov, aspect, self.near_clip, self.far_clip);
        } else {
            let half_w = self.ortho_height * self.aspect * 0.5;
            let half_h = self.ortho_height * 0.5;
            self.mat_proj = Mat4::orthographic(
                -half_w, half_w,
                -half_h, half_h,
                self.near_clip, self.far_clip,
            );
        }

        let mut proj_inv = self.mat_proj;
        proj_inv.invert();
        self.mat_proj_inv = proj_inv;
    }

    pub fn world_to_screen(&self, world_pos: &Vec3) -> Vec3 {
        let ndc = world_pos.transform_mat4(&self.mat_view_proj);
        Vec3::new(
            (ndc.x * 0.5 + 0.5) * self.width as f32,
            ((1.0 - ndc.y) * 0.5) * self.height as f32,
            ndc.z * 0.5 + 0.5,
        )
    }

    pub fn screen_to_world(&self, screen_pos: &Vec3) -> Vec3 {
        let ndx = screen_pos.x / self.width as f32 * 2.0 - 1.0;
        let ndy = 1.0 - screen_pos.y / self.height as f32 * 2.0;
        let ndz = screen_pos.z * 2.0 - 1.0;
        let clip = Vec3::new(ndx, ndy, ndz);
        clip.transform_mat4(&self.mat_view_proj_inv)
    }

    pub fn screen_point_to_ray(&self, x: f32, y: f32) -> Ray {
        let near_world = self.screen_to_world(&Vec3::new(x, y, 0.0));
        let far_world = self.screen_to_world(&Vec3::new(x, y, 1.0));
        Ray::from_points(&near_world, &far_world)
    }

    pub fn get_view_frustum_planes(&self) -> [[f32; 4]; 6] {
        let m = &self.mat_view_proj;
        let left = [
            m.m[3] + m.m[0], m.m[7] + m.m[4],
            m.m[11] + m.m[8], m.m[15] + m.m[12],
        ];
        let right = [
            m.m[3] - m.m[0], m.m[7] - m.m[4],
            m.m[11] - m.m[8], m.m[15] - m.m[12],
        ];
        let bottom = [
            m.m[3] + m.m[1], m.m[7] + m.m[5],
            m.m[11] + m.m[9], m.m[15] + m.m[13],
        ];
        let top = [
            m.m[3] - m.m[1], m.m[7] - m.m[5],
            m.m[11] - m.m[9], m.m[15] - m.m[13],
        ];
        let near = [
            m.m[3] + m.m[2], m.m[7] + m.m[6],
            m.m[11] + m.m[10], m.m[15] + m.m[14],
        ];
        let far = [
            m.m[3] - m.m[2], m.m[7] - m.m[6],
            m.m[11] - m.m[10], m.m[15] - m.m[14],
        ];
        [left, right, bottom, top, near, far]
    }

    pub fn update_view_matrix(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.position = eye;
        let mut forward = target - eye;
        forward.normalize();
        self.forward = forward;
        self.mat_view = Mat4::look_at(&eye, &target, &up);
        self.mat_view_proj = Mat4::multiply_mat4(&self.mat_proj, &self.mat_view);
        let mut vp_inv = self.mat_view_proj;
        vp_inv.invert();
        self.mat_view_proj_inv = vp_inv;
    }

    pub fn get_frustum(&self) -> Option<Frustum> {
        if !self.enabled {
            return None;
        }
        let m = &self.mat_view_proj;
        let make_plane = |nx: f32, ny: f32, nz: f32, d: f32| -> Plane {
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len < 1e-6 {
                Plane::default()
            } else {
                Plane::new(Vec3::new(nx / len, ny / len, nz / len), -d / len)
            }
        };

        let mut f = Frustum::new();
        f.planes[0] = make_plane(m.m[3]+m.m[0], m.m[7]+m.m[4], m.m[11]+m.m[8],  m.m[15]+m.m[12]);
        f.planes[1] = make_plane(m.m[3]-m.m[0], m.m[7]-m.m[4], m.m[11]-m.m[8],  m.m[15]-m.m[12]);
        f.planes[2] = make_plane(m.m[3]+m.m[1], m.m[7]+m.m[5], m.m[11]+m.m[9],  m.m[15]+m.m[13]);
        f.planes[3] = make_plane(m.m[3]-m.m[1], m.m[7]-m.m[5], m.m[11]-m.m[9],  m.m[15]-m.m[13]);
        f.planes[4] = make_plane(m.m[3]+m.m[2], m.m[7]+m.m[6], m.m[11]+m.m[10], m.m[15]+m.m[14]);
        f.planes[5] = make_plane(m.m[3]-m.m[2], m.m[7]-m.m[6], m.m[11]-m.m[10], m.m[15]-m.m[14]);
        Some(f)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod camera_tests {
    use super::*;

    #[test]
    fn test_camera_new() {
        let cam = Camera::new();
        assert!(!cam.enabled);
        assert!(cam.is_culling_enabled);
        assert_eq!(cam.projection, CameraProjection::Perspective);
    }

    #[test]
    fn test_camera_resize() {
        let mut cam = Camera::new();
        cam.resize(1920, 1080);
        assert_eq!(cam.width, 1920);
        assert_eq!(cam.height, 1080);
        assert!((cam.aspect - 1920.0 / 1080.0).abs() < 1e-5);
        assert!(cam.is_proj_dirty);
    }

    #[test]
    fn test_camera_update_matrices_perspective() {
        let mut cam = Camera::new();
        cam.resize(800, 600);
        cam.update_matrices();
        assert!(!cam.is_proj_dirty);
    }

    #[test]
    fn test_camera_update_matrices_ortho() {
        let mut cam = Camera::new();
        cam.set_projection_type(CameraProjection::Ortho);
        cam.resize(800, 600);
        cam.update_matrices();
        assert!(!cam.is_proj_dirty);
    }

    #[test]
    fn test_camera_fov() {
        let mut cam = Camera::new();
        let new_fov = 60.0_f32.to_radians();
        cam.set_fov(new_fov);
        assert!((cam.fov - new_fov).abs() < 1e-6);
        assert!(cam.is_proj_dirty);
    }

    #[test]
    fn test_camera_exposure() {
        let cam = Camera::new();
        assert!(cam.exposure > 0.0);
    }

    #[test]
    fn test_camera_aperture() {
        let mut cam = Camera::new();
        let _old_exposure = cam.exposure;
        cam.set_aperture(CameraAperture::F1_8);
    }
}
