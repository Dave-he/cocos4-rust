/****************************************************************************
Rust port of Cocos Creator PipelineUBO
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::defines::{UBOCamera, UBOGlobal, UBOShadow, UBOCSM};
use crate::math::{Mat4, Vec3, Vec4};

#[derive(Debug, Default)]
pub struct PipelineUBO {
    global_ubo: Vec<f32>,
    camera_ubo: Vec<f32>,
    shadow_ubo: Vec<f32>,
    csm_ubo: Vec<f32>,
    initialized: bool,
}

impl PipelineUBO {
    pub fn new() -> Self {
        PipelineUBO {
            global_ubo: vec![0.0; UBOGlobal::SIZE as usize / 4],
            camera_ubo: vec![0.0; UBOCamera::SIZE as usize / 4],
            shadow_ubo: vec![0.0; UBOShadow::SIZE as usize / 4],
            csm_ubo: vec![0.0; UBOCSM::SIZE as usize / 4],
            initialized: false,
        }
    }

    pub fn activate(&mut self) {
        self.initialized = true;
    }

    pub fn destroy(&mut self) {
        self.global_ubo.clear();
        self.camera_ubo.clear();
        self.shadow_ubo.clear();
        self.csm_ubo.clear();
        self.initialized = false;
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn update_global_ubo(
        &mut self,
        time: f32,
        frame_time: f32,
        frame_count: u32,
        width: f32,
        height: f32,
    ) {
        let fv = &mut self.global_ubo;
        let time_off = UBOGlobal::TIME_OFFSET as usize;
        fv[time_off] = time;
        fv[time_off + 1] = frame_time;
        fv[time_off + 2] = frame_count as f32;
        fv[time_off + 3] = time - frame_time.floor();

        let screen_off = UBOGlobal::SCREEN_SIZE_OFFSET as usize;
        fv[screen_off] = width;
        fv[screen_off + 1] = height;
        fv[screen_off + 2] = 1.0 / width;
        fv[screen_off + 3] = 1.0 / height;

        let native_off = UBOGlobal::NATIVE_SIZE_OFFSET as usize;
        fv[native_off] = width;
        fv[native_off + 1] = height;
        fv[native_off + 2] = 1.0 / width;
        fv[native_off + 3] = 1.0 / height;
    }

    pub fn update_camera_ubo(
        &mut self,
        mat_view: &Mat4,
        mat_proj: &Mat4,
        mat_view_proj: &Mat4,
        camera_pos: Vec3,
        exposure: f32,
        near: f32,
        far: f32,
    ) {
        let fv = &mut self.camera_ubo;
        let mat_view_off = UBOCamera::MAT_VIEW_OFFSET as usize;
        fv[mat_view_off..mat_view_off + 16].copy_from_slice(&mat_view.m);

        let mat_proj_off = UBOCamera::MAT_PROJ_OFFSET as usize;
        fv[mat_proj_off..mat_proj_off + 16].copy_from_slice(&mat_proj.m);

        let mat_vp_off = UBOCamera::MAT_VIEW_PROJ_OFFSET as usize;
        fv[mat_vp_off..mat_vp_off + 16].copy_from_slice(&mat_view_proj.m);

        let pos_off = UBOCamera::CAMERA_POS_OFFSET as usize;
        fv[pos_off] = camera_pos.x;
        fv[pos_off + 1] = camera_pos.y;
        fv[pos_off + 2] = camera_pos.z;
        fv[pos_off + 3] = 1.0;

        let exp_off = UBOCamera::EXPOSURE_OFFSET as usize;
        fv[exp_off] = exposure;
        fv[exp_off + 1] = 1.0 / exposure;

        let nf_off = UBOCamera::NEAR_FAR_OFFSET as usize;
        fv[nf_off] = near;
        fv[nf_off + 1] = far;
        fv[nf_off + 2] = 1.0 / near;
        fv[nf_off + 3] = 1.0 / far;
    }

    pub fn update_shadow_ubo(
        &mut self,
        mat_light_view_proj: &Mat4,
        shadow_info: Vec4,
        shadow_color: Vec4,
        planar_nd_info: Vec4,
    ) {
        let fv = &mut self.shadow_ubo;
        let mat_off = UBOShadow::MAT_LIGHT_VIEW_PROJ_OFFSET as usize;
        fv[mat_off..mat_off + 16].copy_from_slice(&mat_light_view_proj.m);

        let info_off = UBOShadow::SHADOW_NEAR_FAR_LINEAR_SATURATION_INFO_OFFSET as usize;
        fv[info_off] = shadow_info.x;
        fv[info_off + 1] = shadow_info.y;
        fv[info_off + 2] = shadow_info.z;
        fv[info_off + 3] = shadow_info.w;

        let color_off = UBOShadow::SHADOW_COLOR_OFFSET as usize;
        fv[color_off] = shadow_color.x;
        fv[color_off + 1] = shadow_color.y;
        fv[color_off + 2] = shadow_color.z;
        fv[color_off + 3] = shadow_color.w;

        let planar_off = UBOShadow::PLANAR_NORMAL_DISTANCE_INFO_OFFSET as usize;
        fv[planar_off] = planar_nd_info.x;
        fv[planar_off + 1] = planar_nd_info.y;
        fv[planar_off + 2] = planar_nd_info.z;
        fv[planar_off + 3] = planar_nd_info.w;
    }

    pub fn get_global_ubo(&self) -> &[f32] {
        &self.global_ubo
    }

    pub fn get_camera_ubo(&self) -> &[f32] {
        &self.camera_ubo
    }

    pub fn get_shadow_ubo(&self) -> &[f32] {
        &self.shadow_ubo
    }

    pub fn get_csm_ubo(&self) -> &[f32] {
        &self.csm_ubo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_ubo_new() {
        let ubo = PipelineUBO::new();
        assert!(!ubo.is_initialized());
    }

    #[test]
    fn test_pipeline_ubo_activate() {
        let mut ubo = PipelineUBO::new();
        ubo.activate();
        assert!(ubo.is_initialized());
    }

    #[test]
    fn test_pipeline_ubo_update_global() {
        let mut ubo = PipelineUBO::new();
        ubo.activate();
        ubo.update_global_ubo(1.0, 0.016, 100, 1920.0, 1080.0);
        let fv = ubo.get_global_ubo();
        assert_eq!(fv[UBOGlobal::TIME_OFFSET as usize], 1.0);
        assert_eq!(fv[UBOGlobal::SCREEN_SIZE_OFFSET as usize], 1920.0);
    }

    #[test]
    fn test_pipeline_ubo_destroy() {
        let mut ubo = PipelineUBO::new();
        ubo.activate();
        ubo.destroy();
        assert!(!ubo.is_initialized());
    }
}
