/// XR interface runtime version
pub const XR_INTERFACE_RUNTIME_VERSION_1_0: u32 = 1;

/// XR eye type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XREye {
    None = -1,
    Left = 0,
    Right = 1,
    Mono = 2,
}

/// XR device vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRVendor {
    Monado,
    Meta,
    Huaweivr,
    Pico,
    Rokid,
    Seed,
    Spacesxr,
    Gsxr,
    Yvr,
    Htc,
    Iqiyi,
    Skyworth,
    Ffalcon,
    Nreal,
    Inmo,
    Lenovo,
}

/// XR configuration key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum XRConfigKey {
    MultiSamples = 0,
    RenderScale = 1,
    SessionRunning = 2,
    InstanceCreated = 3,
    VkQueueFamilyIndex = 4,
    MetricsState = 5,
    ViewCount = 6,
    SwapchainWidth = 7,
    SwapchainHeight = 8,
    SwapchainFormat = 9,
    MultithreadMode = 10,
    LogicThreadId = 11,
    RenderThreadId = 12,
    DeviceVendor = 13,
    RuntimeVersion = 14,
    PresentEnable = 15,
    RenderEyeFrameLeft = 16,
    RenderEyeFrameRight = 17,
    FeaturePassthrough = 18,
    ImageTracking = 19,
    ImageTrackingCandidateImage = 20,
    ImageTrackingData = 21,
    ImageTrackingSupportStatus = 22,
    HitTesting = 23,
    HitTestingData = 24,
    HitTestingSupportStatus = 25,
    PlaneDetection = 26,
    PlaneDetectionData = 27,
    PlaneDetectionSupportStatus = 28,
    SpatialAnchor = 29,
    SpatialAnchorData = 30,
    SpatialAnchorSupportStatus = 31,
    HandTracking = 32,
    HandTrackingData = 33,
    HandTrackingSupportStatus = 34,
    ApplyHapticController = 35,
    StopHapticController = 36,
    DeviceIpd = 37,
    AppCommand = 38,
    AdbCommand = 39,
    ActivityLifecycle = 40,
    NativeWindow = 41,
    SplitArGlasses = 42,
    EngineVersion = 43,
    RecenterHmd = 44,
    RecenterController = 45,
    FoveationLevel = 46,
    DisplayRefreshRate = 47,
    CameraAccess = 48,
    CameraAccessData = 49,
    CameraAccessSupportStatus = 50,
    SpatialMeshing = 51,
    SpatialMeshingData = 52,
    SpatialMeshingSupportStatus = 53,
    EyeRenderJsCallback = 54,
    AsyncLoadAssetsImage = 55,
    AsyncLoadAssetsImageResults = 56,
    LeftControllerActive = 57,
    RightControllerActive = 58,
    TsEventCallback = 59,
}

/// XR configuration value type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRConfigValueType {
    Unknown,
    Int,
    Float,
    Bool,
    String,
    VoidPointer,
}

/// XR configuration value
#[derive(Debug, Clone)]
pub enum XRConfigValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(String),
    VoidPointer(*mut std::ffi::c_void),
}

impl XRConfigValue {
    pub fn value_type(&self) -> XRConfigValueType {
        match self {
            XRConfigValue::Int(_) => XRConfigValueType::Int,
            XRConfigValue::Float(_) => XRConfigValueType::Float,
            XRConfigValue::Bool(_) => XRConfigValueType::Bool,
            XRConfigValue::String(_) => XRConfigValueType::String,
            XRConfigValue::VoidPointer(_) => XRConfigValueType::VoidPointer,
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, XRConfigValue::Int(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(self, XRConfigValue::Float(_))
    }
    pub fn is_bool(&self) -> bool {
        matches!(self, XRConfigValue::Bool(_))
    }
    pub fn is_pointer(&self) -> bool {
        matches!(self, XRConfigValue::VoidPointer(_))
    }
    pub fn is_string(&self) -> bool {
        matches!(self, XRConfigValue::String(_))
    }

    pub fn get_bool(&self) -> bool {
        if let XRConfigValue::Bool(v) = self {
            *v
        } else {
            false
        }
    }

    pub fn get_int(&self) -> i32 {
        if let XRConfigValue::Int(v) = self {
            *v
        } else {
            0
        }
    }

    pub fn get_float(&self) -> f32 {
        if let XRConfigValue::Float(v) = self {
            *v
        } else {
            0.0
        }
    }

    pub fn get_string(&self) -> &str {
        if let XRConfigValue::String(v) = self {
            v.as_str()
        } else {
            ""
        }
    }

    pub fn get_pointer(&self) -> *mut std::ffi::c_void {
        if let XRConfigValue::VoidPointer(v) = self {
            *v
        } else {
            std::ptr::null_mut()
        }
    }
}

impl From<i32> for XRConfigValue {
    fn from(v: i32) -> Self {
        XRConfigValue::Int(v)
    }
}

impl From<f32> for XRConfigValue {
    fn from(v: f32) -> Self {
        XRConfigValue::Float(v)
    }
}

impl From<bool> for XRConfigValue {
    fn from(v: bool) -> Self {
        XRConfigValue::Bool(v)
    }
}

impl From<String> for XRConfigValue {
    fn from(v: String) -> Self {
        XRConfigValue::String(v)
    }
}

impl From<&str> for XRConfigValue {
    fn from(v: &str) -> Self {
        XRConfigValue::String(v.to_string())
    }
}

/// XR event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XREventType {
    Click,
    Stick,
    Grab,
    Pose,
    Touch,
    Unknown,
}

/// XR controller info trait
pub trait XRControllerInfo {
    fn get_xr_event_type(&self) -> XREventType;
}

/// XR click event
#[derive(Debug, Clone)]
pub struct XRClick {
    pub is_press: bool,
    pub click_type: XRClickType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRClickType {
    TriggerLeft,
    ShoulderLeft,
    ThumbstickLeft,
    X,
    Y,
    Menu,
    TriggerRight,
    ShoulderRight,
    ThumbstickRight,
    A,
    B,
    Home,
    Back,
    Start,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    Unknown,
}

impl XRControllerInfo for XRClick {
    fn get_xr_event_type(&self) -> XREventType {
        XREventType::Click
    }
}

/// XR stick event
#[derive(Debug, Clone)]
pub struct XRStick {
    pub is_active: bool,
    pub x: f32,
    pub y: f32,
    pub stick_type: XRStickType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRStickType {
    StickLeft,
    StickRight,
    Unknown,
}

impl XRControllerInfo for XRStick {
    fn get_xr_event_type(&self) -> XREventType {
        XREventType::Stick
    }
}

/// XR grab event
#[derive(Debug, Clone)]
pub struct XRGrab {
    pub is_active: bool,
    pub value: f32,
    pub grab_type: XRGrabType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRGrabType {
    TriggerLeft,
    GripLeft,
    TriggerRight,
    GripRight,
    Unknown,
}

impl XRControllerInfo for XRGrab {
    fn get_xr_event_type(&self) -> XREventType {
        XREventType::Grab
    }
}

/// XR touch event
#[derive(Debug, Clone)]
pub struct XRTouch {
    pub is_active: bool,
    pub value: f32,
    pub touch_type: XRTouchType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRTouchType {
    TouchA,
    TouchB,
    TouchX,
    TouchY,
    TouchTriggerLeft,
    TouchTriggerRight,
    TouchThumbstickLeft,
    TouchThumbstickRight,
    Unknown,
}

impl XRControllerInfo for XRTouch {
    fn get_xr_event_type(&self) -> XREventType {
        XREventType::Touch
    }
}

/// XR pose event
#[derive(Debug, Clone)]
pub struct XRPose {
    pub is_active: bool,
    pub px: f32,
    pub py: f32,
    pub pz: f32,
    pub qx: f32,
    pub qy: f32,
    pub qz: f32,
    pub qw: f32,
    pub pose_type: XRPoseType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRPoseType {
    ViewLeft,
    HandLeft,
    AimLeft,
    ViewRight,
    HandRight,
    AimRight,
    HeadMiddle,
    ArMobile,
    Unknown,
}

impl XRControllerInfo for XRPose {
    fn get_xr_event_type(&self) -> XREventType {
        XREventType::Pose
    }
}

/// XR controller event
#[derive(Default)]
pub struct XRControllerEvent {
    pub xr_controller_infos: Vec<Box<dyn XRControllerInfo + Send + Sync>>,
}

impl std::fmt::Debug for XRControllerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XRControllerEvent")
            .field("xr_controller_infos_count", &self.xr_controller_infos.len())
            .finish()
    }
}

/// XR swapchain
#[derive(Debug, Clone, Default)]
pub struct XRSwapchain {
    pub xr_swapchain_handle: *mut std::ffi::c_void,
    pub width: u32,
    pub height: u32,
    pub gl_draw_framebuffer: u32,
    pub swapchain_image_index: u32,
    pub eye: u32,
}

/// XR tracking image data
#[derive(Debug, Clone, Default)]
pub struct XRTrackingImageData {
    pub friendly_name: String,
    pub id: u32,
    pub buffer: Vec<u8>,
    pub buffer_size: u32,
    pub physical_width: f32,
    pub physical_height: f32,
    pub pixel_size_width: u32,
    pub pixel_size_height: u32,
    pub pose_position: [f32; 3],
    pub pose_quaternion: [f32; 4],
}

/// Graphics API constants
pub const GRAPHICS_API_OPENGL_ES: &str = "OpenGLES";
pub const GRAPHICS_API_VULKAN_1_0: &str = "Vulkan1";
pub const GRAPHICS_API_VULKAN_1_1: &str = "Vulkan2";

/// XR activity lifecycle type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRActivityLifecycleType {
    Unknown,
    Created,
    Started,
    Resumed,
    Paused,
    Stopped,
    SaveInstanceState,
    Destroyed,
}

/// Type alias for GLES3W load function
pub type PFNGles3wLoadProc = extern "C" fn(*const std::ffi::c_char) -> *mut std::ffi::c_void;

/// Type alias for XR config change callback
pub type XRConfigChangeCallback = dyn Fn(XRConfigKey, XRConfigValue) + Send + Sync;

/// Type alias for XR controller callback
pub type XRControllerCallback = dyn Fn(&XRControllerEvent) + Send + Sync;
