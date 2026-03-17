use super::os_interface::OSInterface;
use crate::math::Mat4;
use crate::renderer::gfx_base::{Format, API};
use crate::xr::{XRConfigKey, XRConfigValue, XREye, XRSwapchain, XRVendor};

/// EGL surface type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EGLSurfaceType {
    None,
    Window,
    Pbuffer,
}

/// XR interface - handles engine and XR library integration
pub trait IXRInterface: OSInterface {
    /// Get XR device vendor
    fn get_vendor(&self) -> XRVendor;

    /// Get XR config parameter
    fn get_xr_config(&self, key: XRConfigKey) -> XRConfigValue;

    /// Set XR config parameter
    fn set_xr_config(&mut self, key: XRConfigKey, value: XRConfigValue);

    /// Get XR runtime version
    fn get_runtime_version(&self) -> u32;

    /// Initialize XR runtime environment
    fn initialize(&mut self, java_vm: *mut std::ffi::c_void, activity: *mut std::ffi::c_void);

    /// Call when render pause
    fn on_render_pause(&mut self);

    /// Call when render resume
    fn on_render_resume(&mut self);

    /// Call when render destroy
    fn on_render_destroy(&mut self);

    /// Call before GFX device initialize
    fn pre_gfx_device_initialize(&mut self, gfx_api: API);

    /// Call after GFX device initialize
    fn post_gfx_device_initialize(&mut self, gfx_api: API);

    /// Call when GFX device acquire
    fn do_gfx_device_acquire(&mut self, gfx_api: API) -> XRSwapchain;

    /// Check if GFX device needs present
    fn is_gfx_device_needs_present(&self, gfx_api: API) -> bool;

    /// Call after GFX device present
    fn post_gfx_device_present(&mut self, gfx_api: API);

    /// Call when creating GFX swapchain
    fn create_xr_swapchains(&mut self);

    /// Get XR swapchain list
    fn get_xr_swapchains(&self) -> Vec<XRSwapchain>;

    /// Get XR swapchain format
    fn get_xr_swapchain_format(&self) -> Format;

    /// Bind engine swapchain with XR swapchain
    fn update_xr_swapchain_typed_id(&mut self, typed_id: u32);

    /// Platform loop start
    fn platform_loop_start(&mut self) -> bool;

    /// Frame render begin
    fn begin_render_frame(&mut self) -> bool;

    /// Check if current frame allows rendering
    fn is_render_allowable(&self) -> bool;

    /// Single eye render begin
    fn begin_render_eye_frame(&mut self, eye: u32) -> bool;

    /// Single eye render end
    fn end_render_eye_frame(&mut self, eye: u32) -> bool;

    /// Frame render end
    fn end_render_frame(&mut self) -> bool;

    /// Platform loop end
    fn platform_loop_end(&mut self) -> bool;

    /// Get HMD view position data
    fn get_hmd_view_position(&self, eye: u32, tracking_type: i32) -> Vec<f32>;

    /// Get XR view projection data
    fn get_xr_view_projection_data(&self, eye: u32, near: f32, far: f32) -> Vec<f32>;

    /// Get XR eye's FOV
    fn get_xr_eye_fov(&self, eye: u32) -> Vec<f32>;

    /// Get render window's XREye type
    fn get_xr_eye_by_render_window(&self, window: *mut std::ffi::c_void) -> XREye;

    /// Bind render window with XR eye
    fn bind_xr_eye_with_render_window(&mut self, window: *mut std::ffi::c_void, eye: XREye);

    /// Handle app command
    fn handle_app_command(&mut self, app_cmd: i32);

    /// Adapt orthographic matrix
    fn adapt_orthographic_matrix(
        &mut self,
        camera: *mut std::ffi::c_void,
        pre_transform: &[f32; 4],
        proj: &mut Mat4,
        view: &mut Mat4,
    );
}
