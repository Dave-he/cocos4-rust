use std::collections::HashMap;

pub mod common;

pub use common::*;

/// XR feature flag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XRFeatureFlag {
    HandTracking = 1 << 0,
    EyeTracking = 1 << 1,
    Passthrough = 1 << 2,
    SpatialAnchor = 1 << 3,
    PlaneDetection = 1 << 4,
    ImageTracking = 1 << 5,
    HitTesting = 1 << 6,
    SpatialMeshing = 1 << 7,
    FoveatedRendering = 1 << 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRSessionState {
    Idle,
    Ready,
    Running,
    Paused,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone, Default)]
pub struct XRViewport {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct XRView {
    pub eye: XREye,
    pub viewport: XRViewport,
    pub fov: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub position: [f32; 3],
    pub orientation: [f32; 4],
}

impl Default for XRView {
    fn default() -> Self {
        XRView {
            eye: XREye::Left,
            viewport: XRViewport::default(),
            fov: 90.0_f32.to_radians(),
            near_clip: 0.1,
            far_clip: 1000.0,
            position: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug, Clone)]
pub struct XRFrameInfo {
    pub predicted_display_time: f64,
    pub views: Vec<XRView>,
    pub swapchains: Vec<XRSwapchain>,
}

impl Default for XRFrameInfo {
    fn default() -> Self {
        XRFrameInfo {
            predicted_display_time: 0.0,
            views: Vec::new(),
            swapchains: Vec::new(),
        }
    }
}

pub trait XRInterface: Send + Sync {
    fn initialize(&mut self, config: &XRInterfaceConfig) -> bool;
    fn destroy(&mut self);
    fn begin_session(&mut self) -> bool;
    fn end_session(&mut self);
    fn wait_frame(&mut self) -> Option<XRFrameInfo>;
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn get_session_state(&self) -> XRSessionState;
    fn is_running(&self) -> bool;
    fn get_supported_features(&self) -> u32;
    fn is_feature_supported(&self, feature: XRFeatureFlag) -> bool;
    fn enable_feature(&mut self, feature: XRFeatureFlag) -> bool;
    fn get_config(&self, key: XRConfigKey) -> Option<XRConfigValue>;
    fn set_config(&mut self, key: XRConfigKey, value: XRConfigValue);
    fn get_vendor(&self) -> XRVendor;
}

#[derive(Debug, Clone)]
pub struct XRInterfaceConfig {
    pub vendor: XRVendor,
    pub render_width: u32,
    pub render_height: u32,
    pub refresh_rate: f32,
    pub enable_passthrough: bool,
    pub runtime_version: u32,
}

impl Default for XRInterfaceConfig {
    fn default() -> Self {
        XRInterfaceConfig {
            vendor: XRVendor::Monado,
            render_width: 1920,
            render_height: 1080,
            refresh_rate: 72.0,
            enable_passthrough: false,
            runtime_version: XR_INTERFACE_RUNTIME_VERSION_1_0,
        }
    }
}

pub struct XRSession {
    config: XRInterfaceConfig,
    state: XRSessionState,
    configs: HashMap<XRConfigKey, XRConfigValue>,
    supported_features: u32,
    enabled_features: u32,
    frame_index: u64,
}

impl XRSession {
    pub fn new(config: XRInterfaceConfig) -> Self {
        let mut configs = HashMap::new();
        configs.insert(XRConfigKey::RuntimeVersion, XRConfigValue::Int(config.runtime_version as i32));
        configs.insert(XRConfigKey::SwapchainWidth, XRConfigValue::Int(config.render_width as i32));
        configs.insert(XRConfigKey::SwapchainHeight, XRConfigValue::Int(config.render_height as i32));
        configs.insert(XRConfigKey::DisplayRefreshRate, XRConfigValue::Float(config.refresh_rate));
        configs.insert(XRConfigKey::DeviceVendor, XRConfigValue::Int(0));

        let mut supported = 0u32;
        supported |= XRFeatureFlag::Passthrough as u32;
        supported |= XRFeatureFlag::SpatialAnchor as u32;
        supported |= XRFeatureFlag::HandTracking as u32;

        XRSession {
            config,
            state: XRSessionState::Idle,
            configs,
            supported_features: supported,
            enabled_features: 0,
            frame_index: 0,
        }
    }

    pub fn get_state(&self) -> XRSessionState {
        self.state
    }

    pub fn is_running(&self) -> bool {
        self.state == XRSessionState::Running
    }

    pub fn begin(&mut self) -> bool {
        if self.state == XRSessionState::Ready || self.state == XRSessionState::Idle {
            self.state = XRSessionState::Running;
            return true;
        }
        false
    }

    pub fn end(&mut self) {
        if self.state == XRSessionState::Running || self.state == XRSessionState::Paused {
            self.state = XRSessionState::Stopping;
            self.state = XRSessionState::Stopped;
        }
    }

    pub fn pause(&mut self) {
        if self.state == XRSessionState::Running {
            self.state = XRSessionState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == XRSessionState::Paused {
            self.state = XRSessionState::Running;
        }
    }

    pub fn wait_frame(&mut self) -> XRFrameInfo {
        self.frame_index += 1;
        let time = self.frame_index as f64 / self.config.refresh_rate as f64;

        let left_view = XRView {
            eye: XREye::Left,
            viewport: XRViewport {
                x: 0, y: 0,
                width: self.config.render_width / 2,
                height: self.config.render_height,
            },
            ..Default::default()
        };
        let right_view = XRView {
            eye: XREye::Right,
            viewport: XRViewport {
                x: (self.config.render_width / 2) as i32, y: 0,
                width: self.config.render_width / 2,
                height: self.config.render_height,
            },
            ..Default::default()
        };

        XRFrameInfo {
            predicted_display_time: time,
            views: vec![left_view, right_view],
            swapchains: Vec::new(),
        }
    }

    pub fn get_config(&self, key: XRConfigKey) -> Option<XRConfigValue> {
        self.configs.get(&key).cloned()
    }

    pub fn set_config(&mut self, key: XRConfigKey, value: XRConfigValue) {
        self.configs.insert(key, value);
    }

    pub fn get_supported_features(&self) -> u32 {
        self.supported_features
    }

    pub fn is_feature_supported(&self, feature: XRFeatureFlag) -> bool {
        self.supported_features & (feature as u32) != 0
    }

    pub fn enable_feature(&mut self, feature: XRFeatureFlag) -> bool {
        if self.is_feature_supported(feature) {
            self.enabled_features |= feature as u32;
            true
        } else {
            false
        }
    }

    pub fn is_feature_enabled(&self, feature: XRFeatureFlag) -> bool {
        self.enabled_features & (feature as u32) != 0
    }

    pub fn get_vendor(&self) -> XRVendor {
        self.config.vendor
    }

    pub fn get_frame_index(&self) -> u64 {
        self.frame_index
    }
}

impl Default for XRSession {
    fn default() -> Self {
        Self::new(XRInterfaceConfig::default())
    }
}

#[allow(clippy::type_complexity)]
pub struct XRManager {
    sessions: Vec<XRSession>,
    active_index: Option<usize>,
    controller_callbacks: Vec<Box<dyn Fn(&XRControllerEvent) + Send + Sync>>,
    config_callbacks: Vec<Box<dyn Fn(XRConfigKey, XRConfigValue) + Send + Sync>>,
}

impl XRManager {
    pub fn new() -> Self {
        XRManager {
            sessions: Vec::new(),
            active_index: None,
            controller_callbacks: Vec::new(),
            config_callbacks: Vec::new(),
        }
    }

    pub fn create_session(&mut self, config: XRInterfaceConfig) -> usize {
        let idx = self.sessions.len();
        self.sessions.push(XRSession::new(config));
        idx
    }

    pub fn get_session(&self, idx: usize) -> Option<&XRSession> {
        self.sessions.get(idx)
    }

    pub fn get_session_mut(&mut self, idx: usize) -> Option<&mut XRSession> {
        self.sessions.get_mut(idx)
    }

    pub fn set_active_session(&mut self, idx: usize) -> bool {
        if idx < self.sessions.len() {
            self.active_index = Some(idx);
            true
        } else {
            false
        }
    }

    pub fn get_active_session(&self) -> Option<&XRSession> {
        self.active_index.and_then(|i| self.sessions.get(i))
    }

    pub fn get_active_session_mut(&mut self) -> Option<&mut XRSession> {
        self.active_index.and_then(|i| self.sessions.get_mut(i))
    }

    pub fn is_xr_active(&self) -> bool {
        self.get_active_session().is_some_and(|s| s.is_running())
    }

    pub fn on_controller_event<F>(&mut self, callback: F)
    where
        F: Fn(&XRControllerEvent) + Send + Sync + 'static,
    {
        self.controller_callbacks.push(Box::new(callback));
    }

    pub fn on_config_change<F>(&mut self, callback: F)
    where
        F: Fn(XRConfigKey, XRConfigValue) + Send + Sync + 'static,
    {
        self.config_callbacks.push(Box::new(callback));
    }

    pub fn dispatch_controller_event(&self, event: &XRControllerEvent) {
        for cb in &self.controller_callbacks {
            cb(event);
        }
    }

    pub fn dispatch_config_change(&self, key: XRConfigKey, value: XRConfigValue) {
        for cb in &self.config_callbacks {
            cb(key, value.clone());
        }
    }
}

impl Default for XRManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xr_session_lifecycle() {
        let mut session = XRSession::new(XRInterfaceConfig::default());
        assert_eq!(session.get_state(), XRSessionState::Idle);
        assert!(session.begin());
        assert!(session.is_running());
        session.end();
        assert_eq!(session.get_state(), XRSessionState::Stopped);
    }

    #[test]
    fn test_xr_session_pause_resume() {
        let mut session = XRSession::default();
        session.begin();
        session.pause();
        assert_eq!(session.get_state(), XRSessionState::Paused);
        session.resume();
        assert!(session.is_running());
    }

    #[test]
    fn test_xr_session_wait_frame() {
        let mut session = XRSession::default();
        session.begin();
        let frame = session.wait_frame();
        assert_eq!(frame.views.len(), 2);
        assert_eq!(frame.views[0].eye, XREye::Left);
        assert_eq!(frame.views[1].eye, XREye::Right);
    }

    #[test]
    fn test_xr_session_config() {
        let mut session = XRSession::default();
        session.set_config(XRConfigKey::MultiSamples, XRConfigValue::Int(4));
        let v = session.get_config(XRConfigKey::MultiSamples);
        assert!(v.is_some());
        assert_eq!(v.unwrap().get_int(), 4);
    }

    #[test]
    fn test_xr_session_feature() {
        let mut session = XRSession::default();
        assert!(session.is_feature_supported(XRFeatureFlag::Passthrough));
        assert!(session.enable_feature(XRFeatureFlag::Passthrough));
        assert!(session.is_feature_enabled(XRFeatureFlag::Passthrough));
        assert!(!session.is_feature_supported(XRFeatureFlag::EyeTracking));
        assert!(!session.enable_feature(XRFeatureFlag::EyeTracking));
    }

    #[test]
    fn test_xr_manager_create_session() {
        let mut mgr = XRManager::new();
        let idx = mgr.create_session(XRInterfaceConfig::default());
        assert_eq!(idx, 0);
        assert!(mgr.get_session(idx).is_some());
    }

    #[test]
    fn test_xr_manager_active_session() {
        let mut mgr = XRManager::new();
        let idx = mgr.create_session(XRInterfaceConfig::default());
        mgr.set_active_session(idx);
        assert!(mgr.get_active_session().is_some());
        assert!(!mgr.is_xr_active());
        mgr.get_active_session_mut().unwrap().begin();
        assert!(mgr.is_xr_active());
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_xr_config_value() {
        let v = XRConfigValue::Int(10);
        assert!(v.is_int());
        assert_eq!(v.get_int(), 10);

        let v = XRConfigValue::Float(3.14);
        assert!(v.is_float());
        assert!((v.get_float() - 3.14).abs() < 1e-5);

        let v = XRConfigValue::Bool(true);
        assert!(v.is_bool());
        assert!(v.get_bool());

        let v = XRConfigValue::String("hello".to_string());
        assert!(v.is_string());
        assert_eq!(v.get_string(), "hello");
    }
}
