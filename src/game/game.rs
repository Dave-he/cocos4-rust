use crate::game::director::Director;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    Hide,
    Show,
    LowMemory,
    GameInited,
    EngineInited,
    RendererInited,
    Restart,
    Pause,
    Resume,
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub frame_rate: u32,
    pub show_fps: bool,
    pub debug_mode: u32,
    pub render_mode: u8,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            frame_rate: 60,
            show_fps: false,
            debug_mode: 0,
            render_mode: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameBootstrapContract {
    pub runtime_style: String,
    pub main_entry: Option<String>,
    pub main_entry_source: Option<String>,
    pub game_path: String,
    pub settings_path: Option<String>,
    pub settings_source: Option<String>,
    pub entry_candidates: Vec<String>,
}

impl GameBootstrapContract {
    pub fn resolve_style_for_native(&self) -> &'static str {
        match self.normalized_runtime_style().as_str() {
            "legacy-cocos2d-js" => "legacy-cocos2d-js",
            "modern-systemjs" => "modern-systemjs",
            "bootstrap-only" => "bootstrap-only",
            _ => "unknown",
        }
    }

    fn normalized_runtime_style(&self) -> String {
        let style = self.runtime_style.trim().to_lowercase();
        if style.is_empty() {
            return "bootstrap-only".to_string();
        }

        let normalized = match style.as_str() {
            "legacy-cocos2d-js" | "legacy-cocos2d-jsb" | "legacy-cocos2d" | "cocos2d-js"
            | "cocos2d-jsb" => "legacy-cocos2d-js",
            "modern-systemjs" | "modern-cocos2d-js" | "systemjs" | "cjs" | "esm" | "modern" => {
                "modern-systemjs"
            }
            "bootstrap-only" | "bootstrap" => "bootstrap-only",
            other => other,
        };

        normalized.to_string()
    }

    pub fn has_main_entry(&self) -> bool {
        match self.main_entry.as_ref() {
            Some(entry) => !entry.trim().is_empty(),
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
enum BootstrapRuntimeKind {
    LegacyCocos2dJs,
    ModernSystemJs,
    BootstrapOnly,
    Unsupported,
}

impl BootstrapRuntimeKind {
    fn from_contract(contract: &GameBootstrapContract) -> Self {
        match contract.resolve_style_for_native() {
            "legacy-cocos2d-js" => Self::LegacyCocos2dJs,
            "modern-systemjs" => Self::ModernSystemJs,
            "bootstrap-only" => Self::BootstrapOnly,
            _ => Self::Unsupported,
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameBootstrapError {
    MissingMainEntry {
        game_path: String,
        candidates: Vec<String>,
    },
    RuntimeUnavailable {
        game_path: String,
        runtime_style: String,
        main_entry: String,
        reason: String,
    },
    UnsupportedRuntime {
        runtime_style: String,
    },
}

impl GameBootstrapError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingMainEntry { .. } => "MISSING_MAIN_ENTRY",
            Self::RuntimeUnavailable { .. } => "RUNTIME_UNAVAILABLE",
            Self::UnsupportedRuntime { .. } => "UNSUPPORTED_RUNTIME",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::MissingMainEntry {
                game_path,
                candidates,
            } => {
                format!(
                    "No usable main entry was selected for game package '{}'. candidates={:?}",
                    game_path, candidates
                )
            }
            Self::RuntimeUnavailable {
                game_path,
                runtime_style,
                main_entry,
                reason,
            } => {
                format!(
                    "Runtime not available yet in this branch (style='{}', main_entry='{}', game_path='{}', reason='{}')",
                    runtime_style, main_entry, game_path, reason
                )
            }
            Self::UnsupportedRuntime { runtime_style } => {
                format!("Unsupported runtime style '{}'", runtime_style)
            }
        }
    }
}

type GameEventCallback = Box<dyn Fn(&GameEvent) + Send + Sync>;

const REASON_UNIMPLEMENTED_JS_RUNTIME: &str =
    "native JS runtime path is not implemented in this branch yet";
const REASON_JS_BOOTSTRAP_ENTRY_NOT_DETECTED: &str =
    "javascript bootstrap entrypoint could not be detected in main entry source";
const REASON_JS_SOURCE_SYNTAX_HEURISTIC_FAILED: &str =
    "javascript source syntax heuristic probe failed";
#[allow(dead_code)] // only referenced under js-runtime-mock / js-runtime-real cfg
const REASON_JS_RUNTIME_SIMULATOR_FAILED: &str =
    "javascript runtime simulator rejected bootstrap entrypoint";
#[allow(dead_code)] // only referenced under js-runtime-mock / js-runtime-real cfg
const REASON_JS_RUNTIME_EXECUTION_FAILED: &str =
    "javascript runtime rejected bootstrap entrypoint during execution";

pub struct Game {
    config: GameConfig,
    director: Arc<Mutex<Director>>,
    inited: bool,
    paused: bool,
    listeners: Vec<(GameEvent, GameEventCallback)>,
}

impl Game {
    fn build_runtime_unavailable_reason(
        &self,
        bootstrap: &GameBootstrapContract,
    ) -> Option<String> {
        if let Some(reason) = self.probe_bootstrap_entry(
            bootstrap.normalized_runtime_style().as_str(),
            bootstrap.main_entry_source.as_deref(),
        ) {
            return Some(reason);
        }

        if bootstrap.main_entry_source.is_none() {
            return Some("main entry source is not available in bootstrap contract".to_string());
        }

        if bootstrap
            .main_entry_source
            .as_ref()
            .map(|source| source.trim().is_empty())
            == Some(true)
        {
            return Some("main entry source is empty".to_string());
        }

        if !self.try_execute_bootstrap(bootstrap) {
            #[cfg(feature = "js-runtime-mock")]
            {
                return Some(REASON_JS_RUNTIME_SIMULATOR_FAILED.to_string());
            }
            #[cfg(feature = "js-runtime-real")]
            {
                return Some(REASON_JS_RUNTIME_EXECUTION_FAILED.to_string());
            }
            #[cfg(all(not(feature = "js-runtime-mock"), not(feature = "js-runtime-real")))]
            {
                return Some(REASON_UNIMPLEMENTED_JS_RUNTIME.to_string());
            }
        }

        None
    }

    #[cfg(feature = "js-runtime-probe")]
    fn js_probe_parse_state(&self, source: &str) -> bool {
        let mut brace_depth: i32 = 0;
        let mut paren_depth: i32 = 0;
        let mut bracket_depth: i32 = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut in_template = false;
        let mut in_line_comment = false;
        let mut in_block_comment = false;
        let bytes = source.as_bytes();
        let mut i = 0usize;

        while i < bytes.len() {
            let c = bytes[i];
            let next = if i + 1 < bytes.len() {
                bytes[i + 1]
            } else {
                b'\0'
            };

            if in_line_comment {
                if c == b'\n' {
                    in_line_comment = false;
                }
                i += 1;
                continue;
            }
            if in_block_comment {
                if c == b'*' && next == b'/' {
                    in_block_comment = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            if in_single_quote {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == b'\'' {
                    in_single_quote = false;
                }
                i += 1;
                continue;
            }
            if in_double_quote {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == b'"' {
                    in_double_quote = false;
                }
                i += 1;
                continue;
            }
            if in_template {
                if c == b'\\' {
                    i += 2;
                    continue;
                }
                if c == b'`' {
                    in_template = false;
                }
                i += 1;
                continue;
            }

            match c {
                b'/' if next == b'/' => {
                    in_line_comment = true;
                    i += 2;
                }
                b'/' if next == b'*' => {
                    in_block_comment = true;
                    i += 2;
                }
                b'\'' => {
                    in_single_quote = true;
                    i += 1;
                }
                b'"' => {
                    in_double_quote = true;
                    i += 1;
                }
                b'`' => {
                    in_template = true;
                    i += 1;
                }
                b'(' => {
                    paren_depth += 1;
                    i += 1;
                }
                b')' => {
                    paren_depth -= 1;
                    i += 1;
                }
                b'[' => {
                    bracket_depth += 1;
                    i += 1;
                }
                b']' => {
                    bracket_depth -= 1;
                    i += 1;
                }
                b'{' => {
                    brace_depth += 1;
                    i += 1;
                }
                b'}' => {
                    brace_depth -= 1;
                    i += 1;
                }
                _ => i += 1,
            }

            if paren_depth < 0 || brace_depth < 0 || bracket_depth < 0 {
                return false;
            }
        }

        !in_single_quote
            && !in_double_quote
            && !in_template
            && !in_line_comment
            && !in_block_comment
            && paren_depth == 0
            && brace_depth == 0
            && bracket_depth == 0
    }

    #[cfg(feature = "js-runtime-probe")]
    fn run_js_probe(&self, source: &str) -> bool {
        self.js_probe_parse_state(source)
    }

    #[cfg(not(feature = "js-runtime-probe"))]
    fn run_js_probe(&self, source: &str) -> bool {
        source.len() > 0
    }

    fn probe_bootstrap_entry_inner(&self, style_detected: &str, source: &str) -> Option<String> {
        let normalized = source.to_lowercase();
        match style_detected {
            "legacy-cocos2d-js"
                if normalized.contains("window.boot")
                    || normalized.contains("window[\"boot\"]") =>
            {
                None
            }
            "legacy-cocos2d-js" => Some(format!(
                "{}: {}",
                REASON_JS_BOOTSTRAP_ENTRY_NOT_DETECTED,
                "legacy style expects window.boot definition"
            )),
            "modern-systemjs"
                if normalized.contains("system.register")
                    || normalized.contains("system[\"register\"]")
                    || normalized.contains("window.__require")
                    || normalized.contains("__require")
                    || normalized.contains("__initapp")
                    || normalized.contains("system.import")
                    || normalized.contains("system.warmup")
                    || normalized.contains("first-screen")
                    || normalized.contains("firstscreen") =>
            {
                None
            }
            "modern-systemjs" => Some(format!(
                "{}: {}",
                REASON_JS_BOOTSTRAP_ENTRY_NOT_DETECTED,
                "modern style expects System.register, __require, or __initApp-style bootstrap"
            )),
            _ => None,
        }
    }

    fn probe_bootstrap_entry(
        &self,
        normalized_style: &str,
        source: Option<&str>,
    ) -> Option<String> {
        if cfg!(feature = "js-runtime-probe") {
            if let Some(source) = source {
                if !self.run_js_probe(source) {
                    return Some(format!(
                        "{}: {}",
                        REASON_JS_SOURCE_SYNTAX_HEURISTIC_FAILED,
                        "unbalanced braces/comments/strings"
                    ));
                }
            }
        }

        let source = source?;
        let style_detected = match normalized_style {
            "legacy-cocos2d-js" => "legacy-cocos2d-js",
            "modern-systemjs" => "modern-systemjs",
            _ => return None,
        };
        self.probe_bootstrap_entry_inner(style_detected, source)
    }

    #[allow(unused_variables)] // used only under js-runtime-* cfg branches
    fn try_execute_bootstrap(&self, bootstrap: &GameBootstrapContract) -> bool {
        #[cfg(feature = "js-runtime-real")]
        {
            return self.execute_bootstrap_real(bootstrap);
        }

        #[cfg(all(feature = "js-runtime-mock", not(feature = "js-runtime-real")))]
        {
            return self.execute_bootstrap_mock(bootstrap);
        }

        #[cfg(all(not(feature = "js-runtime-real"), not(feature = "js-runtime-mock")))]
        {
            return false;
        }
    }

    #[cfg(feature = "js-runtime-mock")]
    fn execute_bootstrap_mock(&self, bootstrap: &GameBootstrapContract) -> bool {
        let source = bootstrap
            .main_entry_source
            .as_deref()
            .map(str::trim)
            .unwrap_or("");

        if source.is_empty() {
            return false;
        }

        let normalized_style = bootstrap.normalized_runtime_style();
        let source_lower = source.to_lowercase();
        match normalized_style.as_str() {
            "legacy-cocos2d-js" => {
                source_lower.contains("window.boot") && source_lower.contains("cc.game")
            }
            "modern-systemjs" => {
                let marker = source_lower.contains("system.register")
                    || source_lower.contains("system[\"register\"]")
                    || source_lower.contains("__require")
                    || source_lower.contains("__initapp")
                    || source_lower.contains("system.import")
                    || source_lower.contains("system.warmup")
                    || source_lower.contains("first-screen")
                    || source_lower.contains("firstscreen");
                marker && (source_lower.contains("cc") || source_lower.contains("__initapp"))
            }
            _ => false,
        }
    }

    #[cfg(feature = "js-runtime-real")]
    fn execute_bootstrap_real(&self, bootstrap: &GameBootstrapContract) -> bool {
        let source = bootstrap
            .main_entry_source
            .as_deref()
            .map(str::trim)
            .unwrap_or("");
        let settings_source = bootstrap
            .settings_source
            .as_deref()
            .map(str::trim)
            .unwrap_or("");
        let settings_path = bootstrap.settings_path.as_deref().unwrap_or("");
        let is_settings_json = settings_path
            .to_ascii_lowercase()
            .ends_with("settings.json");

        if source.is_empty() {
            return false;
        }

        if self
            .probe_bootstrap_entry_inner(bootstrap.normalized_runtime_style().as_str(), source)
            .is_some()
        {
            return false;
        }

        use boa_engine::{Context, Source};

        let normalized_style = bootstrap.normalized_runtime_style();
        let call_bootstrap = normalized_style == "legacy-cocos2d-js";
        let mut run_eval =
            |context: &mut Context, source: &str| context.eval(Source::from_bytes(source)).is_ok();
        let prelude = r###"
var window = typeof window === "undefined" ? (typeof globalThis === "undefined" ? this : globalThis) : window;
if (typeof console === "undefined") {
    var console = { log() {}, warn() {}, error() {}, info() {}, debug() {}, trace() {} };
}
if (typeof window.cc === "undefined") {
    window.cc = {};
}
var cc = window.cc;
if (typeof window.GameGlobal === "undefined") {
    window.GameGlobal = {};
}
if (typeof window.ks === "undefined") {
    window.ks = {
        canIUse: function () {
            return true;
        }
    };
}
if (typeof window.ks.onShow !== "function") {
    window.ks.onShow = function () {};
}
if (typeof window.ks.onHide !== "function") {
    window.ks.onHide = function () {};
}
if (typeof window.ks.onError !== "function") {
    window.ks.onError = function () {};
}
if (typeof window.ks.offError !== "function") {
    window.ks.offError = function () {};
}
if (typeof window.ks.getOpenDataContext !== "function") {
    window.ks.getOpenDataContext = function () {
        return {};
    };
}
if (typeof window.canvas === "undefined") {
    window.canvas = {
        width: 960,
        height: 540,
        id: "GameCanvas",
        addEventListener: function () {},
        removeEventListener: function () {}
    };
}
if (typeof window.canvas.id === "undefined") {
    window.canvas.id = "GameCanvas";
}
if (typeof window.WXWebAssembly === "undefined") {
    window.WXWebAssembly = window.KSWebAssembly || (typeof WebAssembly === "undefined" ? {} : WebAssembly);
}
if (typeof window.KSWebAssembly === "undefined") {
    window.KSWebAssembly = window.WXWebAssembly || (typeof WebAssembly === "undefined" ? {} : WebAssembly);
}
var __mockSystemInfo = {
    platform: "android",
    system: "android",
    windowWidth: 960,
    windowHeight: 540,
    screenWidth: 960,
    screenHeight: 540,
    pixelRatio: 1,
    language: "en",
    version: "1.0.0"
};
if (typeof window.devicePixelRatio === "undefined") {
    window.devicePixelRatio = __mockSystemInfo.pixelRatio;
}
if (typeof window.GameGlobal.wx === "undefined") {
    window.GameGlobal.wx = window.ks;
}
if (typeof window.screen === "undefined") {
    window.screen = {
        width: __mockSystemInfo.windowWidth,
        height: __mockSystemInfo.windowHeight
    };
}
if (typeof window.wx === "undefined") {
    window.wx = {
        env: {
            USER_DATA_PATH: "",
            appName: "cocos-runtime-mock",
            platform: "android",
            VERSION: {
                sdkVersion: "0.0.0",
                nativeVersion: ""
            }
        },
        onMessage: function () {},
        offMessage: function () {},
        onShow: function () {},
        offShow: function () {},
        onHide: function () {},
        offHide: function () {},
        onError: function () {},
        offError: function () {},
        onNetworkStatusChange: function () {},
        offNetworkStatusChange: function () {},
        getSystemInfoSync: function () {
            return __mockSystemInfo;
        },
        getSystemInfo: function (_success, _fail, _complete) {
            var result = __mockSystemInfo;
            if (typeof _success === "function") {
                _success(result);
            }
            if (typeof _complete === "function") {
                _complete(result);
            }
            return result;
        },
        loadFont: function () {
            return null;
        },
        setPreferredFramesPerSecond: function () {},
        loadSubpackage: function () {
            return null;
        },
        getStorageInfoSync: function () {
            var keys = [];
            if (window.localStorage && window.localStorage.__data) {
                keys = Object.keys(window.localStorage.__data);
            }
            return {
                keys: keys
            };
        },
        getStorageSync: function (_key) {
            if (!window.localStorage) {
                return null;
            }
            return window.localStorage.getItem(_key);
        },
        setStorageSync: function (_key, _value) {
            if (!window.localStorage) {
                return;
            }
            window.localStorage.setItem(_key, _value);
        },
        removeStorageSync: function (_key) {
            if (!window.localStorage) {
                return;
            }
            window.localStorage.removeItem(_key);
        },
        clearStorageSync: function () {
            if (!window.localStorage) {
                return;
            }
            window.localStorage.clear();
        },
        getFileSystemManager: function () {
            return {
                readFile: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({ data: "", filePath: options.filePath || "" });
                    }
                },
                readFileSync: function () {
                    return "";
                },
                writeFile: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({});
                    }
                },
                writeFileSync: function () {},
                mkdir: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({});
                    }
                },
                mkdirSync: function () {},
                rmdir: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({});
                    }
                },
                rmdirSync: function () {},
                readdir: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({ files: [] });
                    }
                },
                access: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({});
                    }
                },
                accessSync: function () {
                    return null;
                },
                copyFile: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({});
                    }
                },
                unlink: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({});
                    }
                },
                unzip: function (options) {
                    if (!options || typeof options !== "object") {
                        return;
                    }
                    if (typeof options.success === "function") {
                        options.success({});
                    }
                }
            };
        },
        getSharedCanvas: function () {
            return window.canvas;
        },
        connectSocket: function () {
            var listeners = {};
            return {
                onClose: function (fn) {
                    if (typeof fn === "function") {
                        listeners.close = fn;
                    }
                    return this;
                },
                onMessage: function (fn) {
                    if (typeof fn === "function") {
                        listeners.message = fn;
                    }
                    return this;
                },
                onOpen: function (fn) {
                    if (typeof fn === "function") {
                        listeners.open = fn;
                        fn();
                    }
                    return this;
                },
                onError: function (fn) {
                    if (typeof fn === "function") {
                        listeners.error = fn;
                    }
                    return this;
                },
                close: function () {
                    if (typeof listeners.close === "function") {
                        listeners.close({});
                    }
                },
                send: function (_data) {
                    if (typeof listeners.message === "function") {
                        listeners.message({ data: _data });
                    }
                }
            };
        },
        request: function () {
            return {};
        },
        createCanvas: function () {
            return window.document && typeof window.document.createElement === "function"
                ? window.document.createElement("canvas")
                : {
                    width: __mockSystemInfo.windowWidth,
                    height: __mockSystemInfo.windowHeight
                };
        },
        createImage: function () {
            return {
                width: 0,
                height: 0,
                onload: null,
                onerror: null,
                set src(value) {
                    this._src = value;
                    if (typeof this.onload === "function") {
                        this.onload({});
                    }
                },
                get src() {
                    return this._src || "";
                }
            };
        },
        downloadFile: function (options) {
            if (typeof options === "object" && typeof options.success === "function") {
                options.success({ statusCode: 404 });
            }
            return {
                onProgressUpdate: function () {
                    return {
                        abort: function () {}
                    };
                }
            };
        },
        saveFile: function (options) {
            if (typeof options === "object" && typeof options.fail === "function") {
                options.fail({ errMsg: "mock not implemented" });
            }
            return {};
        },
        getOpenDataContext: function () {
            return {};
        },
        createInnerAudioContext: function () {
            return {
                autoplay: false,
                loop: false,
                startTime: 0,
                volume: 1,
                onPlay: function () {},
                onPause: function () {},
                onStop: function () {},
                onEnded: function () {},
                onError: function () {},
                onSeeked: function () {},
                onSeeking: function () {},
                onTimeUpdate: function () {},
                onCanPlay: function () {},
                play: function () {},
                pause: function () {},
                stop: function () {},
                seek: function () {},
                destroy: function () {}
            };
        },
        createVideo: function () {
            return {
                autoplay: false,
                src: "",
                onPlay: function () {},
                onPause: function () {},
                onEnded: function () {},
                onError: function () {},
                onTimeUpdate: function () {},
                pause: function () {},
                play: function () {},
                stop: function () {},
                seek: function () {},
                destroy: function () {}
            };
        },
        onTouchStart: function () {},
        offTouchStart: function () {},
        onTouchMove: function () {},
        offTouchMove: function () {},
        onTouchEnd: function () {},
        offTouchEnd: function () {},
        onTouchCancel: function () {},
        offTouchCancel: function () {},
        onMessage: function () {},
        offMessage: function () {},
        onShow: function () {},
        offShow: function () {},
        onHide: function () {},
        offHide: function () {},
        onError: function () {},
        offError: function () {},
        onNetworkStatusChange: function () {},
        offNetworkStatusChange: function () {},
        onAccelerometerChange: function () {},
        offAccelerometerChange: function () {},
        onDeviceOrientationChange: function () {},
        offDeviceOrientationChange: function () {},
        onCompassChange: function () {},
        offCompassChange: function () {},
        startAccelerometer: function () {},
        stopAccelerometer: function () {},
        startCompass: function () {},
        stopCompass: function () {},
        onAudioInterruptionBegin: function () {},
        onAudioInterruptionEnd: function () {},
        offAudioInterruptionBegin: function () {},
        offAudioInterruptionEnd: function () {},
        setPreferredFramesPerSecond: function () {},
        showKeyboard: function () {},
        hideKeyboard: function () {},
        updateKeyboard: function () {},
        onKeyboardInput: function () {},
        onKeyboardConfirm: function () {},
        onKeyboardComplete: function () {},
        offKeyboardInput: function () {},
        offKeyboardConfirm: function () {},
        offKeyboardComplete: function () {}
    };
}
if (typeof window.wx.env !== "object" || window.wx.env === null) {
    window.wx.env = {};
}
if (typeof window.wx.env.USER_DATA_PATH === "undefined") {
    window.wx.env.USER_DATA_PATH = "";
}
if (typeof window.wx.getSharedCanvas !== "function") {
    window.wx.getSharedCanvas = function () {
        return window.canvas;
    };
}
if (typeof window.wx.createInnerAudioContext !== "function") {
    window.wx.createInnerAudioContext = function () {
        return window.__globalAdapter && typeof window.__globalAdapter.createInnerAudioContext === "function"
            ? window.__globalAdapter.createInnerAudioContext()
            : {};
    };
}
if (typeof window.wx.createVideo !== "function") {
    window.wx.createVideo = function () {
        return window.__globalAdapter && typeof window.__globalAdapter.createVideo === "function"
            ? window.__globalAdapter.createVideo()
            : {};
    };
}
if (typeof window.wx.onKeyboardInput !== "function") {
    window.wx.onKeyboardInput = function () {};
}
if (typeof window.wx.offKeyboardInput !== "function") {
    window.wx.offKeyboardInput = function () {};
}
if (typeof window.wx.onKeyboardConfirm !== "function") {
    window.wx.onKeyboardConfirm = function () {};
}
if (typeof window.wx.offKeyboardConfirm !== "function") {
    window.wx.offKeyboardConfirm = function () {};
}
if (typeof window.wx.onKeyboardComplete !== "function") {
    window.wx.onKeyboardComplete = function () {};
}
if (typeof window.wx.offKeyboardComplete !== "function") {
    window.wx.offKeyboardComplete = function () {};
}
if (typeof window.wx.onTouchStart !== "function") {
    window.wx.onTouchStart = function () {};
}
if (typeof window.wx.offTouchStart !== "function") {
    window.wx.offTouchStart = function () {};
}
if (typeof window.wx.onTouchMove !== "function") {
    window.wx.onTouchMove = function () {};
}
if (typeof window.wx.offTouchMove !== "function") {
    window.wx.offTouchMove = function () {};
}
if (typeof window.wx.onTouchEnd !== "function") {
    window.wx.onTouchEnd = function () {};
}
if (typeof window.wx.offTouchEnd !== "function") {
    window.wx.offTouchEnd = function () {};
}
if (typeof window.wx.onTouchCancel !== "function") {
    window.wx.onTouchCancel = function () {};
}
if (typeof window.wx.offTouchCancel !== "function") {
    window.wx.offTouchCancel = function () {};
}
if (typeof window.wx.onAccelerometerChange !== "function") {
    window.wx.onAccelerometerChange = function () {};
}
if (typeof window.wx.offAccelerometerChange !== "function") {
    window.wx.offAccelerometerChange = function () {};
}
if (typeof window.wx.onCompassChange !== "function") {
    window.wx.onCompassChange = function () {};
}
if (typeof window.wx.offCompassChange !== "function") {
    window.wx.offCompassChange = function () {};
}
if (typeof window.wx.startAccelerometer !== "function") {
    window.wx.startAccelerometer = function () {};
}
if (typeof window.wx.stopAccelerometer !== "function") {
    window.wx.stopAccelerometer = function () {};
}
if (typeof window.wx.startCompass !== "function") {
    window.wx.startCompass = function () {};
}
if (typeof window.wx.stopCompass !== "function") {
    window.wx.stopCompass = function () {};
}
if (typeof window.wx.onDeviceOrientationChange !== "function") {
    window.wx.onDeviceOrientationChange = function () {};
}
if (typeof window.wx.offDeviceOrientationChange !== "function") {
    window.wx.offDeviceOrientationChange = function () {};
}
if (typeof window.wx.onAudioInterruptionBegin !== "function") {
    window.wx.onAudioInterruptionBegin = function () {};
}
if (typeof window.wx.onAudioInterruptionEnd !== "function") {
    window.wx.onAudioInterruptionEnd = function () {};
}
if (typeof window.wx.offAudioInterruptionBegin !== "function") {
    window.wx.offAudioInterruptionBegin = function () {};
}
if (typeof window.wx.offAudioInterruptionEnd !== "function") {
    window.wx.offAudioInterruptionEnd = function () {};
}
if (typeof window.__globalAdapter === "undefined") {
    window.__globalAdapter = {
        init: function () {},
        adaptEngine: function () {},
        getSystemInfoSync: function () {
            return window.wx.getSystemInfoSync ? window.wx.getSystemInfoSync() : __mockSystemInfo;
        },
        createInnerAudioContext: function () {
            return window.wx.createInnerAudioContext ? window.wx.createInnerAudioContext() : {};
        },
        createVideo: function () {
            return window.wx.createVideo ? window.wx.createVideo() : {};
        },
        onShow: function () {},
        onHide: function () {},
        onMessage: function () {},
        onNetworkStatusChange: function () {},
        onError: function () {},
        offError: function () {},
        onKeyboardInput: function () {},
        onKeyboardConfirm: function () {},
        onKeyboardComplete: function () {},
        offKeyboardInput: function () {},
        offKeyboardConfirm: function () {},
        offKeyboardComplete: function () {},
        showKeyboard: function () {},
        hideKeyboard: function () {},
        updateKeyboard: function () {},
        getMenuButtonBoundingClientRect: function () {
            return {
                width: 0,
                height: 0,
                left: 0,
                right: 0,
                top: 0,
                bottom: 0
            };
        },
        onTouchStart: function () {},
        offTouchStart: function () {},
        onTouchMove: function () {},
        offTouchMove: function () {},
        onTouchEnd: function () {},
        offTouchEnd: function () {},
        onTouchCancel: function () {},
        offTouchCancel: function () {},
        onAccelerometerChange: function () {},
        offAccelerometerChange: function () {},
        onDeviceOrientationChange: function () {},
        offDeviceOrientationChange: function () {},
        onCompassChange: function () {},
        offCompassChange: function () {},
        startAccelerometer: function () {},
        stopAccelerometer: function () {},
        startCompass: function () {},
        stopCompass: function () {},
        onAudioInterruptionBegin: function () {},
        onAudioInterruptionEnd: function () {},
        offAudioInterruptionBegin: function () {},
        offAudioInterruptionEnd: function () {}
    };
}
if (typeof window.__globalAdapter.init !== "function") {
    window.__globalAdapter.init = function () {};
}
if (typeof window.__globalAdapter.adaptEngine !== "function") {
    window.__globalAdapter.adaptEngine = function () {};
}
if (typeof window.__globalAdapter.getSystemInfoSync !== "function") {
    window.__globalAdapter.getSystemInfoSync = function () {
        return window.wx.getSystemInfoSync ? window.wx.getSystemInfoSync() : __mockSystemInfo;
    };
}
if (typeof window.__globalAdapter.setPreferredFramesPerSecond !== "function") {
    window.__globalAdapter.setPreferredFramesPerSecond = function () {};
}
if (typeof window.__globalAdapter.createInnerAudioContext !== "function") {
    window.__globalAdapter.createInnerAudioContext = function () {
        return {};
    };
}
if (typeof window.__globalAdapter.createVideo !== "function") {
    window.__globalAdapter.createVideo = function () {
        return {};
    };
}
if (typeof window.__globalAdapter.getMenuButtonBoundingClientRect !== "function") {
    window.__globalAdapter.getMenuButtonBoundingClientRect = function () {
        return {
            width: 0,
            height: 0,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0
        };
    };
}
if (typeof window.__globalAdapter.onShow !== "function") {
    window.__globalAdapter.onShow = function () {};
}
if (typeof window.__globalAdapter.offShow !== "function") {
    window.__globalAdapter.offShow = function () {};
}
if (typeof window.__globalAdapter.onMessage !== "function") {
    window.__globalAdapter.onMessage = function () {};
}
if (typeof window.__globalAdapter.offMessage !== "function") {
    window.__globalAdapter.offMessage = function () {};
}
if (typeof window.__globalAdapter.onHide !== "function") {
    window.__globalAdapter.onHide = function () {};
}
if (typeof window.__globalAdapter.offHide !== "function") {
    window.__globalAdapter.offHide = function () {};
}
if (typeof window.__globalAdapter.onNetworkStatusChange !== "function") {
    window.__globalAdapter.onNetworkStatusChange = function () {};
}
if (typeof window.__globalAdapter.offNetworkStatusChange !== "function") {
    window.__globalAdapter.offNetworkStatusChange = function () {};
}
if (typeof window.__globalAdapter.onError !== "function") {
    window.__globalAdapter.onError = function () {};
}
if (typeof window.__globalAdapter.offError !== "function") {
    window.__globalAdapter.offError = function () {};
}
if (typeof window.__globalAdapter.onKeyboardInput !== "function") {
    window.__globalAdapter.onKeyboardInput = function () {};
}
if (typeof window.__globalAdapter.offKeyboardInput !== "function") {
    window.__globalAdapter.offKeyboardInput = function () {};
}
if (typeof window.__globalAdapter.onKeyboardConfirm !== "function") {
    window.__globalAdapter.onKeyboardConfirm = function () {};
}
if (typeof window.__globalAdapter.offKeyboardConfirm !== "function") {
    window.__globalAdapter.offKeyboardConfirm = function () {};
}
if (typeof window.__globalAdapter.onKeyboardComplete !== "function") {
    window.__globalAdapter.onKeyboardComplete = function () {};
}
if (typeof window.__globalAdapter.offKeyboardComplete !== "function") {
    window.__globalAdapter.offKeyboardComplete = function () {};
}
if (typeof window.__globalAdapter.showKeyboard !== "function") {
    window.__globalAdapter.showKeyboard = function () {};
}
if (typeof window.__globalAdapter.hideKeyboard !== "function") {
    window.__globalAdapter.hideKeyboard = function () {};
}
if (typeof window.__globalAdapter.updateKeyboard !== "function") {
    window.__globalAdapter.updateKeyboard = function () {};
}
if (typeof window.__globalAdapter.onTouchStart !== "function") {
    window.__globalAdapter.onTouchStart = function () {};
}
if (typeof window.__globalAdapter.offTouchStart !== "function") {
    window.__globalAdapter.offTouchStart = function () {};
}
if (typeof window.__globalAdapter.onTouchMove !== "function") {
    window.__globalAdapter.onTouchMove = function () {};
}
if (typeof window.__globalAdapter.offTouchMove !== "function") {
    window.__globalAdapter.offTouchMove = function () {};
}
if (typeof window.__globalAdapter.onTouchEnd !== "function") {
    window.__globalAdapter.onTouchEnd = function () {};
}
if (typeof window.__globalAdapter.offTouchEnd !== "function") {
    window.__globalAdapter.offTouchEnd = function () {};
}
if (typeof window.__globalAdapter.onTouchCancel !== "function") {
    window.__globalAdapter.onTouchCancel = function () {};
}
if (typeof window.__globalAdapter.offTouchCancel !== "function") {
    window.__globalAdapter.offTouchCancel = function () {};
}
if (typeof window.__globalAdapter.onAccelerometerChange !== "function") {
    window.__globalAdapter.onAccelerometerChange = function () {};
}
if (typeof window.__globalAdapter.offAccelerometerChange !== "function") {
    window.__globalAdapter.offAccelerometerChange = function () {};
}
if (typeof window.__globalAdapter.onDeviceOrientationChange !== "function") {
    window.__globalAdapter.onDeviceOrientationChange = function () {};
}
if (typeof window.__globalAdapter.offDeviceOrientationChange !== "function") {
    window.__globalAdapter.offDeviceOrientationChange = function () {};
}
if (typeof window.__globalAdapter.onCompassChange !== "function") {
    window.__globalAdapter.onCompassChange = function () {};
}
if (typeof window.__globalAdapter.offCompassChange !== "function") {
    window.__globalAdapter.offCompassChange = function () {};
}
if (typeof window.__globalAdapter.startAccelerometer !== "function") {
    window.__globalAdapter.startAccelerometer = function () {};
}
if (typeof window.__globalAdapter.stopAccelerometer !== "function") {
    window.__globalAdapter.stopAccelerometer = function () {};
}
if (typeof window.__globalAdapter.startCompass !== "function") {
    window.__globalAdapter.startCompass = function () {};
}
if (typeof window.__globalAdapter.stopCompass !== "function") {
    window.__globalAdapter.stopCompass = function () {};
}
if (typeof window.__globalAdapter.onAudioInterruptionBegin !== "function") {
    window.__globalAdapter.onAudioInterruptionBegin = function () {};
}
if (typeof window.__globalAdapter.offAudioInterruptionBegin !== "function") {
    window.__globalAdapter.offAudioInterruptionBegin = function () {};
}
if (typeof window.__globalAdapter.onAudioInterruptionEnd !== "function") {
    window.__globalAdapter.onAudioInterruptionEnd = function () {};
}
if (typeof window.__globalAdapter.offAudioInterruptionEnd !== "function") {
    window.__globalAdapter.offAudioInterruptionEnd = function () {};
}
if (typeof window.location === "undefined") {
    window.location = {
        href: "",
        protocol: "https:",
        host: "",
        hostname: "",
        port: "",
        pathname: "",
        search: "",
        hash: "",
        origin: "https://localhost",
        toString: function () {
            return this.href || this.origin;
        }
    };
}
if (typeof window.parent === "undefined") {
    window.parent = window;
}
if (typeof window.top === "undefined") {
    window.top = window.parent;
}
if (typeof window.innerWidth === "undefined") {
    window.innerWidth = __mockSystemInfo.windowWidth;
}
if (typeof window.innerHeight === "undefined") {
    window.innerHeight = __mockSystemInfo.windowHeight;
}
if (typeof window.outerWidth === "undefined") {
    window.outerWidth = __mockSystemInfo.windowWidth;
}
if (typeof window.outerHeight === "undefined") {
    window.outerHeight = __mockSystemInfo.windowHeight;
}
if (typeof window.screenY === "undefined") {
    window.screenY = 0;
}
if (typeof window.scrollY === "undefined") {
    window.scrollY = 0;
}
if (typeof window.__mockPerformanceStart === "undefined") {
    window.__mockPerformanceStart = typeof Date === "undefined" || typeof Date.now !== "function" ? 0 : Date.now();
}
if (typeof window.performance === "undefined") {
    window.performance = {
        now: function () {
            if (typeof Date === "undefined" || typeof Date.now !== "function") {
                return 0;
            }
            return Date.now() - window.__mockPerformanceStart;
        }
    };
}
if (typeof window.performance.now !== "function") {
    window.performance.now = function () {
        if (typeof Date === "undefined" || typeof Date.now !== "function") {
            return 0;
        }
        return Date.now() - window.__mockPerformanceStart;
    };
}
if (typeof window.eventListeners === "undefined") {
    window.eventListeners = {};
}
if (typeof window.addEventListener !== "function") {
    window.addEventListener = function (_type, _listener) {
        if (typeof _type !== "string" || typeof _listener !== "function") {
            return;
        }
        if (typeof window.eventListeners[_type] !== "object" || window.eventListeners[_type] === null) {
            window.eventListeners[_type] = [];
        }
        window.eventListeners[_type].push(_listener);
    };
}
if (typeof window.removeEventListener !== "function") {
    window.removeEventListener = function (_type, _listener) {
        if (typeof _type !== "string" || !window.eventListeners[_type]) {
            return;
        }
        if (typeof _listener !== "function") {
            window.eventListeners[_type] = [];
            return;
        }
        var listeners = window.eventListeners[_type];
        var remain = [];
        for (var i = 0; i < listeners.length; i += 1) {
            if (listeners[i] !== _listener) {
                remain.push(listeners[i]);
            }
        }
        window.eventListeners[_type] = remain;
    };
}
if (typeof window.dispatchEvent !== "function") {
    window.dispatchEvent = function (_event) {
        if (!_event || typeof _event.type !== "string") {
            return false;
        }
        var listeners = window.eventListeners[_event.type] || [];
        for (var i = 0; i < listeners.length; i += 1) {
            var fn = listeners[i];
            if (typeof fn === "function") {
                try {
                    fn(_event);
                } catch (_ignore) {}
            }
        }
        return true;
    };
}
if (typeof window.navigator === "undefined") {
    window.navigator = {};
}
if (typeof window.navigator.geolocation === "undefined") {
    window.navigator.geolocation = {
        getCurrentPosition: function () {},
        watchPosition: function () { return 0; },
        clearWatch: function () {}
    };
}
if (typeof window.navigator.userAgent === "undefined") {
    window.navigator.userAgent = "cocos-runtime-mock";
}
if (typeof window.navigator.language === "undefined") {
    window.navigator.language = "en-US";
}
if (typeof window.navigator.platform === "undefined") {
    window.navigator.platform = "Android";
}
if (typeof window.TouchEvent === "undefined") {
    window.TouchEvent = function (_type) {
        this.type = _type || "";
        this.touches = [];
        this.targetTouches = [];
        this.changedTouches = [];
        this.target = window.canvas;
        this.currentTarget = window.canvas;
    };
    window.TouchEvent.prototype.preventDefault = function () {};
    window.TouchEvent.prototype.stopPropagation = function () {};
}
if (typeof window.MouseEvent === "undefined") {
    window.MouseEvent = function () {};
}
if (typeof window.DeviceMotionEvent === "undefined") {
    window.DeviceMotionEvent = function () {};
}
if (typeof window.localStorage === "undefined") {
    window.localStorage = {
        __data: {},
        get length() {
            return Object.keys(this.__data).length;
        },
        key: function (_index) {
            var keys = Object.keys(this.__data);
            return keys[Number(_index)] || null;
        },
        getItem: function (_key) {
            return Object.prototype.hasOwnProperty.call(this.__data, _key) ? this.__data[_key] : null;
        },
        setItem: function (_key, _value) {
            this.__data[_key] = String(_value);
        },
        removeItem: function (_key) {
            delete this.__data[_key];
        },
        clear: function () {
            this.__data = {};
        }
    };
}
if (typeof window.sessionStorage === "undefined") {
    window.sessionStorage = {
        __data: {},
        getItem: function (_key) {
            return Object.prototype.hasOwnProperty.call(this.__data, _key) ? this.__data[_key] : null;
        },
        setItem: function (_key, _value) {
            this.__data[_key] = String(_value);
        },
        removeItem: function (_key) {
            delete this.__data[_key];
        },
        clear: function () {
            this.__data = {};
        }
    };
}
if (typeof window.document === "undefined") {
    window.canvas = window.canvas || {};
    window.document = {
        readyState: "complete",
        hidden: false,
        visibilityState: "visible",
        documentElement: {
            appendChild: function () {}
        },
        head: {
            appendChild: function () {}
        },
        body: {
            appendChild: function (_node) {
                return _node;
            }
        },
        addEventListener: function () {},
        removeEventListener: function () {},
        getElementById: function (_id) {
            if (_id === "GameCanvas" || _id === "gameCanvas") {
                return window.canvas;
            }
            return null;
        },
        getElementsByTagName: function (_name) {
            if (typeof _name === "string" && _name.toLowerCase() === "head") {
                return [window.document.head];
            }
            if (typeof _name === "string" && _name.toLowerCase() === "body") {
                return [window.document.body];
            }
            if (typeof _name === "string" && _name.toLowerCase() === "canvas") {
                return [window.canvas];
            }
            return [];
        },
        getElementsByName: function (_name) {
            if (_name === "head") {
                return [window.document.head];
            }
            if (_name === "body") {
                return [window.document.body];
            }
            if (typeof _name === "string" && _name.toLowerCase() === "canvas") {
                return [window.canvas];
            }
            return [];
        },
        querySelector: function (_selector) {
            if (_selector === "head") {
                return window.document.head;
            }
            if (_selector === "documentElement" || _selector === ":root" || _selector === "html") {
                return window.document.documentElement;
            }
            if (_selector === "body") {
                return window.document.body;
            }
            if (_selector === "#GameCanvas" || _selector === "#gameCanvas" || _selector === "canvas") {
                return window.canvas;
            }
            return null;
        },
        querySelectorAll: function (_selector) {
            var target = this.querySelector(_selector);
            if (target) {
                return [target];
            }
            return [];
        }
    };
}
if (typeof window.document.head === "undefined") {
    window.document.head = {
        appendChild: function () {}
    };
}
if (typeof window.document.documentElement === "undefined") {
    window.document.documentElement = {
        appendChild: function () {}
    };
}
if (typeof window.document.getElementsByName !== "function") {
    window.document.getElementsByName = function (_name) {
        if (typeof window.document.querySelectorAll !== "function") {
            return [];
        }
        var result = window.document.querySelectorAll(_name);
        return Array.isArray(result) ? result : [];
    };
}
if (typeof window.document.querySelectorAll !== "function") {
    window.document.querySelectorAll = function (_selector) {
        var target = this.querySelector(_selector);
        if (target) {
            return [target];
        }
        return [];
    };
}
if (typeof window.document.createElement !== "function") {
    window.document.createElement = function (_tagName) {
        var tag = String(_tagName || "").toLowerCase();
        if (tag === "canvas") {
            return {
                tagName: tag,
                style: {},
                width: 960,
                height: 540,
                children: [],
                appendChild: function () {},
                getContext: function (_ctx) {
                    if (_ctx === "webgl" || _ctx === "webgl2" || _ctx === "2d") {
                        return {
                            canvas: this,
                            getExtension: function (_name) {
                                if (_name === "WEBGL_debug_renderer_info") {
                                    return {};
                                }
                                return null;
                            },
                            getContextAttributes: function () {
                                return {};
                            }
                        };
                    }
                    return {};
                },
                addEventListener: function () {},
                removeEventListener: function () {},
                getBoundingClientRect: function () {
                    return {
                        width: this.width || 0,
                        height: this.height || 0,
                        left: 0,
                        top: 0,
                        right: this.width || 0,
                        bottom: this.height || 0
                    };
                }
            };
        }
        return {
            tagName: tag,
            style: {},
            children: [],
            appendChild: function () {},
            getContext: function () {
                return {};
            },
            addEventListener: function () {},
            removeEventListener: function () {}
        };
    };
}
if (typeof window.document.querySelector !== "function") {
    window.document.querySelector = function (_selector) {
        return null;
    };
}
if (typeof window.requirePlugin !== "function") {
    window.requirePlugin = function () {};
}
if (typeof window.requestAnimationFrame !== "function") {
    window.requestAnimationFrame = function (callback) {
        if (typeof callback === "function") {
            callback();
        }
        return 0;
    };
}
if (typeof window.cancelAnimationFrame !== "function") {
    window.cancelAnimationFrame = function () {};
}
if (typeof window.GameGlobal.requestAnimationFrame !== "function") {
    window.GameGlobal.requestAnimationFrame = window.requestAnimationFrame;
}
if (typeof window.GameGlobal.cancelAnimationFrame !== "function") {
    window.GameGlobal.cancelAnimationFrame = window.cancelAnimationFrame;
}
var ks = window.ks;
var wx = window.wx;
var canvas = window.canvas;
var GameGlobal = window.GameGlobal;
if (typeof window.setTimeout === "undefined") {
    window.setTimeout = function (callback) {
        if (typeof callback === "function") {
            callback();
        }
        return 0;
    };
}
if (typeof window.clearTimeout === "undefined") {
    window.clearTimeout = function () {};
}
if (typeof window.setInterval === "undefined") {
    window.setInterval = function (callback) {
        if (typeof callback === "function") {
            return callback();
        }
        return 0;
    };
}
if (typeof window.clearInterval === "undefined") {
    window.clearInterval = function () {};
}
if (typeof cc._CCSettings === "undefined") {
    cc._CCSettings = {};
}
if (typeof _CCSettings === "undefined") {
    var _CCSettings = cc._CCSettings;
}
if (typeof _CCSettings === "object" && _CCSettings === null) {
    _CCSettings = cc._CCSettings;
}
if (typeof cc._CCSettings.jsList === "undefined") {
    cc._CCSettings.jsList = [];
}
if (typeof cc._CCSettings.bundleVers === "undefined") {
    cc._CCSettings.bundleVers = {};
}
if (typeof window._CCSettings === "undefined") {
    window._CCSettings = cc._CCSettings;
}
if (typeof cc._RF === "undefined") {
    cc._RF = {
        push: function () {},
        pop: function () {},
        remap: function (id) { return id; }
    };
}
if (typeof cc.macro === "undefined") {
    cc.macro = {};
}
if (typeof cc.macro.CLEANUP_IMAGE_CACHE === "undefined") {
    cc.macro.CLEANUP_IMAGE_CACHE = true;
}
if (typeof cc.macro.CLEANUP_MATERIAL_CACHE === "undefined") {
    cc.macro.CLEANUP_MATERIAL_CACHE = false;
}
if (typeof cc.macro.ENABLE_MULTI_TOUCH === "undefined") {
    cc.macro.ENABLE_MULTI_TOUCH = true;
}
if (typeof cc.AssetManager === "undefined") {
    cc.AssetManager = {
        BuiltinBundleName: {
            INTERNAL: "internal",
            RESOURCES: "resources",
            MAIN: "main",
            START_SCENE: "start-scene"
        }
    };
}
if (typeof cc.AssetManager.BuiltinBundleName !== "object") {
    cc.AssetManager.BuiltinBundleName = {
        INTERNAL: "internal",
        RESOURCES: "resources",
        MAIN: "main",
        START_SCENE: "start-scene"
    };
}
if (typeof cc.path === "undefined") {
    cc.path = {};
}
if (typeof cc.path.basename !== "function") {
    cc.path.basename = function (path) {
        var normalized = String(path);
        var idx = normalized.lastIndexOf("/");
        if (idx < 0) idx = normalized.lastIndexOf("\\");
        return idx < 0 ? normalized : normalized.slice(idx + 1);
    };
}
if (typeof cc.path.extname !== "function") {
    cc.path.extname = function (path) {
        var normalized = String(path);
        var idx = normalized.lastIndexOf(".");
        return idx < 0 ? "" : normalized.slice(idx);
    };
}
if (typeof cc.path.dirname !== "function") {
    cc.path.dirname = function (path) {
        var normalized = String(path);
        var idx = normalized.lastIndexOf("/");
        if (idx < 0) idx = normalized.lastIndexOf("\\");
        return idx < 0 ? "" : normalized.slice(0, idx);
    };
}
if (typeof cc.path.mainFileName !== "function") {
    cc.path.mainFileName = function (path) {
        var name = cc.path.basename ? cc.path.basename(path) : String(path);
        var idx = name.lastIndexOf(".");
        return idx < 0 ? name : name.slice(0, idx);
    };
}
if (typeof cc.path.join !== "function") {
    cc.path.join = function () {
        var args = Array.prototype.slice.call(arguments);
        var out = [];
        for (var i = 0; i < args.length; i++) {
            var part = args[i];
            if (part === null || part === undefined) continue;
            part = String(part);
            if (part.length === 0) continue;
            if (out.length === 0) {
                out.push(part);
            } else {
                if (out[out.length - 1].slice(-1) === "/" || part.charAt(0) === "/") {
                    out.push(part);
                } else {
                    out.push("/" + part);
                }
            }
        }
        return out.join("");
    };
}
if (typeof cc.game === "undefined") {
    cc.game = {};
}
if (typeof cc._createCocosDelegate !== "function") {
    cc._createCocosDelegate = function () {
        return {
            _callbacks: [],
            _fired: false,
            add: function (callback) {
                if (typeof callback !== "function") {
                    return;
                }
                if (this._fired) {
                    try {
                        callback();
                    } catch (_e) {}
                    return;
                }
                this._callbacks.push(callback);
            },
            fire: function () {
                if (this._fired) {
                    return;
                }
                this._fired = true;
                for (var i = 0; i < this._callbacks.length; i++) {
                    var cb = this._callbacks[i];
                    if (typeof cb === "function") {
                        try {
                            cb();
                        } catch (_e) {}
                    }
                }
            }
        };
    };
}
if (typeof cc.DebugMode === "undefined") {
    if (typeof cc.debug !== "undefined" && typeof cc.debug.DebugMode !== "undefined") {
        cc.DebugMode = cc.debug.DebugMode;
    } else {
        cc.DebugMode = {
            NONE: 0,
            VERBOSE: 1,
            INFO: 2,
            WARN: 3,
            ERROR: 4,
            INFO_FOR_WEB_PAGE: 5,
            WARN_FOR_WEB_PAGE: 6,
            ERROR_FOR_WEB_PAGE: 7
        };
    }
}
if (typeof cc.game.run !== "function") {
    cc.game.run = function (_options, _callback) {
        var runCallback = _callback;
        if (typeof _options === "function" && typeof _callback !== "function") {
            runCallback = _options;
        }
        if (typeof cc.game.onStart === "function") {
            try {
                cc.game.onStart();
            } catch (err) {
                // legacy bootstrap may depend on many APIs; we keep execution tolerant here.
            }
        }
        if (typeof runCallback === "function") {
            try {
                runCallback();
            } catch (_err) {}
        }
        cc.game.__running = true;
        cc.game.__lastRunOptions = _options;
        return true;
    };
}
if (typeof cc.game.init !== "function") {
    cc.game.init = function (_options) {
        cc.game.__config = _options || {};
        if (typeof cc.game.onPostBaseInitDelegate !== "undefined" &&
            typeof cc.game.onPostBaseInitDelegate.fire === "function") {
            cc.game.onPostBaseInitDelegate.fire();
        }
        if (typeof cc.game.onPostSubsystemInitDelegate !== "undefined" &&
            typeof cc.game.onPostSubsystemInitDelegate.fire === "function") {
            cc.game.onPostSubsystemInitDelegate.fire();
        }
        if (typeof window.__mockThenable === "function") {
            return window.__mockThenable(window);
        }
        return {
            then: function (callback) {
                if (typeof callback === "function") {
                    callback(null, _options || {});
                }
                return {
                    then: function () {},
                    catch: function () {}
                };
            }
        };
    };
}
if (typeof cc.game.runPromise !== "function") {
    cc.game.runPromise = function () {
        if (typeof cc.game.run === "function") {
            if (typeof window.__mockThenable === "function") {
                return window.__mockThenable(cc.game.run());
            }
            return cc.game.run();
        }
        if (typeof window.__mockThenable === "function") {
            return window.__mockThenable();
        }
        return {
            then: function () {
                return {
                    then: function () {},
                    catch: function () {}
                };
            },
            catch: function () {}
        };
    };
}
if (typeof cc.game.emit !== "function") {
    cc.game.emit = function (_event) {
        if (!this.__eventMap || typeof _event !== "string") {
            return;
        }
        var args = Array.prototype.slice.call(arguments, 1);
        var list = this.__eventMap[_event];
        if (!list || !list.length) {
            return;
        }
        for (var i = 0; i < list.length; i++) {
            var fn = list[i];
            if (typeof fn === "function") {
                try {
                    fn.apply(this, args);
                } catch (_e) {}
            }
        }
    };
}
if (typeof cc.game.on !== "function") {
    cc.game.on = function (_event, callback) {
        if (!this.__eventMap) {
            this.__eventMap = {};
        }
        if (typeof _event !== "string" || typeof callback !== "function") {
            return;
        }
        if (!this.__eventMap[_event]) {
            this.__eventMap[_event] = [];
        }
        this.__eventMap[_event].push(callback);
    };
}
if (typeof cc.game.off !== "function") {
    cc.game.off = function (_event, callback) {
        if (!this.__eventMap || typeof _event !== "string") {
            return;
        }
        if (typeof callback !== "function") {
            this.__eventMap[_event] = [];
            return;
        }
        var list = this.__eventMap[_event];
        if (!list || !list.length) {
            return;
        }
        var next = [];
        for (var i = 0; i < list.length; i++) {
            if (list[i] !== callback) {
                next.push(list[i]);
            }
        }
        this.__eventMap[_event] = next;
    };
}
if (typeof cc.game.once !== "function") {
    cc.game.once = function (_event, callback) {
        if (typeof callback !== "function") {
            return;
        }
        var self = this;
        var wrapper = function () {
            self.off(_event, wrapper);
            callback.apply(this, arguments);
        };
        cc.game.on(_event, wrapper);
    };
}
if (typeof cc.game.onPostBaseInitDelegate === "undefined") {
    cc.game.onPostBaseInitDelegate = cc._createCocosDelegate();
}
if (typeof cc.game.onPostSubsystemInitDelegate === "undefined") {
    cc.game.onPostSubsystemInitDelegate = cc._createCocosDelegate();
}
if (typeof cc.game.onStart === "undefined") {
    cc.game.onStart = null;
}
if (typeof cc.director === "undefined") {
    cc.director = {};
}
if (typeof cc.director.loadScene !== "function") {
    cc.director.loadScene = function (_sceneName, _arg1, callback) {
        if (typeof callback === "function") callback(null);
        return null;
    };
}
if (typeof cc.director.runScene !== "function") {
    cc.director.runScene = function () {};
}
if (typeof cc.director.runSceneImmediate !== "function") {
    cc.director.runSceneImmediate = function () {};
}
if (typeof cc.director.getRunningScene !== "function") {
    cc.director.getRunningScene = function () {
        if (typeof cc.director.getScene === "function") {
            return cc.director.getScene();
        }
        return null;
    };
}
if (typeof cc.director.pushScene !== "function") {
    cc.director.pushScene = function () {};
}
if (typeof cc.director.popScene !== "function") {
    cc.director.popScene = function () {
        return null;
    };
}
if (typeof cc.director.replaceScene !== "function") {
    cc.director.replaceScene = function () {};
}
if (typeof cc.director.pause !== "function") {
    cc.director.pause = function () {};
}
if (typeof cc.director.resume !== "function") {
    cc.director.resume = function () {};
}
if (typeof cc.director.on !== "function") {
    cc.director.on = function (type, callback) {
        if (!this.__eventMap) {
            this.__eventMap = {};
        }
        if (typeof type !== "string" || typeof callback !== "function") {
            return;
        }
        if (!this.__eventMap[type]) {
            this.__eventMap[type] = [];
        }
        this.__eventMap[type].push(callback);
    };
}
if (typeof cc.director.off !== "function") {
    cc.director.off = function (type, callback) {
        if (!this.__eventMap || typeof type !== "string") {
            return;
        }
        if (typeof callback !== "function") {
            this.__eventMap[type] = [];
            return;
        }
        var list = this.__eventMap[type];
        if (!list || !list.length) {
            return;
        }
        var next = [];
        for (var i = 0; i < list.length; i += 1) {
            if (list[i] !== callback) {
                next.push(list[i]);
            }
        }
        this.__eventMap[type] = next;
    };
}
if (typeof cc.director.emit !== "function") {
    cc.director.emit = function (type) {
        if (!this.__eventMap || typeof type !== "string") {
            return;
        }
        var args = Array.prototype.slice.call(arguments, 1);
        var list = this.__eventMap[type] || [];
        for (var i = 0; i < list.length; i += 1) {
            var callback = list[i];
            if (typeof callback === "function") {
                try {
                    callback.apply(this, args);
                } catch (_e) {}
            }
        }
    };
}
if (typeof cc.director.preloadScene !== "function") {
    cc.director.preloadScene = function (_sceneName, _resources, _options, callback) {
        if (typeof callback === "function") callback(null);
        return null;
    };
}
if (typeof cc.director.getScene !== "function") {
    cc.director.getScene = function () { return null; };
}
if (typeof cc.director.setDisplayStats !== "function") {
    cc.director.setDisplayStats = function () {};
}
if (typeof cc.LoaderScene === "undefined") {
    cc.LoaderScene = {};
}
if (typeof cc.LoaderScene.preload !== "function") {
    cc.LoaderScene.preload = function (_resources, _onProgress, callback) {
        if (typeof callback === "function") callback(null);
        return null;
    };
}
if (typeof cc.assetManager === "undefined") {
    cc.assetManager = {};
}
if (typeof cc.assetManager.bundles === "undefined") {
    cc.assetManager.bundles = [];
}
if (typeof cc.assetManager.downloader === "undefined") {
    cc.assetManager.downloader = {};
}
if (typeof cc.assetManager.downloader.bundleVers === "undefined") {
    cc.assetManager.downloader.bundleVers = {};
}
if (typeof cc.assetManager.downloader.remoteBundles === "undefined") {
    cc.assetManager.downloader.remoteBundles = [];
}
if (typeof cc.assetManager.loadScript !== "function") {
    cc.assetManager.loadScript = function (_url, _options, callback) {
        if (typeof _options === "function") callback = _options;
        if (typeof callback === "function") callback(null);
        return true;
    };
}
if (typeof cc.assetManager.loadBundle !== "function") {
    cc.assetManager.loadBundle = function (_nameOrUrl, _options, _arg2, _callback) {
        var callback = _callback;
        if (typeof _options === "function") {
            callback = _options;
        }
        if (typeof callback === "function") callback(null, { name: _nameOrUrl });
        return true;
    };
}
if (typeof cc.assetManager.init !== "function") {
    cc.assetManager.init = function (_options, callback) {
        var complete = callback;
        if (typeof _options === "function") {
            complete = _options;
        }
        if (typeof complete === "function") {
            complete(null);
        }
        return true;
    };
}
if (typeof cc.sys === "undefined") {
    cc.sys = {
        isBrowser: false,
        isMobile: false,
        language: "en",
        platform: "unknown"
    };
}
if (typeof cc.sys.isBrowser === "undefined") {
    cc.sys.isBrowser = false;
}
if (typeof cc.sys.Platform === "undefined") {
    cc.sys.Platform = {
        UNKNOWN: "UNKNOWN",
        EDITOR_PAGE: "EDITOR_PAGE",
        EDITOR_CORE: "EDITOR_CORE",
        NODEJS_PAGE: "NODEJS_PAGE",
        MOBILE_BROWSER: "MOBILE_BROWSER",
        DESKTOP_BROWSER: "DESKTOP_BROWSER",
        WIN32: "WIN32",
        ANDROID: "ANDROID",
        IOS: "IOS",
        MACOS: "MACOS",
        OHOS: "OHOS",
        OPENHARMONY: "OPENHARMONY",
        WECHAT_GAME: "WECHAT_GAME",
        WECHAT_MINI_PROGRAM: "WECHAT_MINI_PROGRAM",
        XIAOMI_QUICK_GAME: "XIAOMI_QUICK_GAME",
        ALIPAY_MINI_GAME: "ALIPAY_MINI_GAME",
        BYTEDANCE_MINI_GAME: "BYTEDANCE_MINI_GAME",
        OPPO_MINI_GAME: "OPPO_MINI_GAME",
        VIVO_MINI_GAME: "VIVO_MINI_GAME",
        HUAWEI_QUICK_GAME: "HUAWEI_QUICK_GAME",
        MIGU_MINI_GAME: "MIGU_MINI_GAME",
        HONOR_MINI_GAME: "HONOR_MINI_GAME",
        SUD_MINI_GAME: "SUD_MINI_GAME",
        SUDV2_MINI_GAME: "SUDV2_MINI_GAME"
    };
}
if (typeof cc.sys.WECHAT_GAME === "undefined") {
    cc.sys.WECHAT_GAME = "WECHAT_GAME";
}
if (typeof cc.sys.WECHAT_GAME_SUB === "undefined") {
    cc.sys.WECHAT_GAME_SUB = "WECHAT_MINI_PROGRAM";
}
if (typeof cc.sys.WECHAT_GAME === "string" && typeof cc.sys.platform === "undefined") {
    cc.sys.platform = cc.sys.WECHAT_GAME;
}
if (typeof cc.log !== "function") {
    cc.log = function () {
        if (typeof console !== "undefined" && typeof console.log === "function") {
            console.log.apply(console, arguments);
        }
    };
}
if (typeof cc.warn !== "function") {
    cc.warn = function () {
        if (typeof console !== "undefined" && typeof console.warn === "function") {
            console.warn.apply(console, arguments);
        }
    };
}
if (typeof cc.error !== "function") {
    cc.error = function () {
        if (typeof console !== "undefined" && typeof console.error === "function") {
            console.error.apply(console, arguments);
        }
    };
}
if (typeof cc.debug === "undefined") {
    cc.debug = {};
}
if (typeof cc.debug.DebugMode === "undefined") {
    cc.debug.DebugMode = {
        NONE: 0,
        VERBOSE: 1,
        INFO: 2,
        WARN: 3,
        ERROR: 4,
        INFO_FOR_WEB_PAGE: 5,
        WARN_FOR_WEB_PAGE: 6,
        ERROR_FOR_WEB_PAGE: 7
    };
}
if (typeof cc.debug.setDisplayStats !== "function") {
    cc.debug.setDisplayStats = function () {};
}
if (typeof cc.loader === "undefined") {
    cc.loader = {};
}
if (typeof cc.loader.loadJs !== "function") {
    cc.loader.loadJs = function (_url, _options, callback) {
        if (Object.prototype.toString.call(_url) === "[object Array]") {
            if (typeof callback === "function") {
                callback(null);
            }
            return true;
        }
        if (typeof _options === "function") callback = _options;
        if (typeof callback === "function") callback(null);
        return true;
    };
}
if (typeof cc.path.normalize !== "function") {
    cc.path.normalize = function (path) { return String(path); };
}
if (typeof cc.resources === "undefined") {
    cc.resources = {};
}
if (typeof cc.resources.load !== "function") {
    cc.resources.load = function (path, onProgress, onComplete) {
        var callback = onComplete;
        if (typeof onProgress === "function" && typeof onComplete !== "function") {
            callback = onProgress;
        }
        var result = {
            path: String(path || ""),
            loaded: true
        };
        if (typeof callback === "function") {
            callback(null, result);
        }
        if (typeof window.__mockThenable === "function") {
            return window.__mockThenable(result);
        }
        return {
            then: function (resolve) {
                if (typeof resolve === "function") {
                    resolve(result);
                }
                return this;
            },
            catch: function () { return this; }
        };
    };
}
if (typeof cc.resources.loadDir !== "function") {
    cc.resources.loadDir = function (path, onProgress, onComplete) {
        var callback = onComplete;
        if (typeof onProgress === "function" && typeof onComplete !== "function") {
            callback = onProgress;
        }
        var result = {
            path: String(path || ""),
            loaded: true
        };
        if (typeof callback === "function") {
            callback(null, [result]);
        }
        if (typeof window.__mockThenable === "function") {
            return window.__mockThenable([result]);
        }
        return {
            then: function (resolve) {
                if (typeof resolve === "function") {
                    resolve([result]);
                }
                return this;
            },
            catch: function () { return this; }
        };
    };
}
if (typeof cc.resources.loadRemote !== "function") {
    cc.resources.loadRemote = function (path, options, onComplete) {
        var callback = onComplete;
        if (typeof options === "function" && typeof onComplete !== "function") {
            callback = options;
        }
        if (typeof callback === "function") {
            callback(null, { path: String(path || ""), options: options || null });
        }
        return {
            then: function (resolve) {
                if (typeof resolve === "function") {
                    resolve({ path: String(path || ""), options: options || null });
                }
                return this;
            },
            catch: function () { return this; }
        };
    };
}
if (typeof cc.resources.preload !== "function") {
    cc.resources.preload = function (paths, options, onComplete) {
        var callback = onComplete;
        if (typeof options === "function" && typeof onComplete !== "function") {
            callback = options;
        }
        if (typeof callback === "function") {
            callback(null);
        }
        return true;
    };
}
if (typeof cc.resources.getAssetInfo !== "function") {
    cc.resources.getAssetInfo = function (path) {
        return {
            path: String(path || ""),
            exists: true
        };
    };
}
if (typeof cc.resources.release !== "function") {
    cc.resources.release = function (_asset) {
        return true;
    };
}
if (typeof cc.find !== "function") {
    cc.find = function (_name) {
        return null;
    };
}
if (typeof cc.instantiate !== "function") {
    cc.instantiate = function (source) {
        if (source && typeof source.clone === "function") {
            return source.clone();
        }
        return {};
    };
}
if (typeof cc.Vec2 === "undefined") {
    cc.Vec2 = function (x, y) {
        if (x && typeof x === "object" && typeof x.x === "number" && typeof x.y === "number") {
            y = x.y;
            x = x.x;
        }
        this.x = Number(x) || 0;
        this.y = Number(y) || 0;
    };
    cc.Vec2.prototype = {
        constructor: cc.Vec2,
        clone: function () {
            return new cc.Vec2(this.x, this.y);
        },
        equals: function (other) {
            return !!other && this.x === other.x && this.y === other.y;
        }
    };
}
if (typeof cc.Vec3 === "undefined") {
    cc.Vec3 = function (x, y, z) {
        if (x && typeof x === "object" && typeof x.x === "number" && typeof x.y === "number" && typeof x.z === "number") {
            y = x.y;
            z = x.z;
            x = x.x;
        }
        this.x = Number(x) || 0;
        this.y = Number(y) || 0;
        this.z = Number(z) || 0;
    };
    cc.Vec3.prototype = {
        constructor: cc.Vec3,
        clone: function () {
            return new cc.Vec3(this.x, this.y, this.z);
        },
        equals: function (other) {
            return !!other && this.x === other.x && this.y === other.y && this.z === other.z;
        }
    };
}
if (typeof cc.v2 !== "function") {
    cc.v2 = function (x, y) {
        return new cc.Vec2(x, y);
    };
}
if (typeof cc.v3 !== "function") {
    cc.v3 = function (x, y, z) {
        return new cc.Vec3(x, y, z);
    };
}
if (typeof cc.color !== "function") {
    cc.color = function (r, g, b, a) {
        if (typeof r === "object" && r !== null && typeof r.r === "number") {
            var source = r;
            return {
                r: Number(source.r) || 0,
                g: Number(source.g) || 0,
                b: Number(source.b) || 0,
                a: source.a === undefined ? 255 : Number(source.a)
            };
        }
        return {
            r: Number(r) || 0,
            g: Number(g) || 0,
            b: Number(b) || 0,
            a: a === undefined ? 255 : Number(a)
        };
    };
}
if (typeof cc.Color === "undefined") {
    cc.Color = function (r, g, b, a) {
        var value = cc.color(r, g, b, a);
        this.r = value.r;
        this.g = value.g;
        this.b = value.b;
        this.a = value.a;
    };
    cc.Color.WHITE = {
        r: 255,
        g: 255,
        b: 255,
        a: 255
    };
    cc.Color.BLACK = {
        r: 0,
        g: 0,
        b: 0,
        a: 255
    };
    cc.Color.RED = {
        r: 255,
        g: 0,
        b: 0,
        a: 255
    };
    cc.Color.GREEN = {
        r: 0,
        g: 255,
        b: 0,
        a: 255
    };
    cc.Color.BLUE = {
        r: 0,
        g: 0,
        b: 255,
        a: 255
    };
    cc.Color.prototype = {
        constructor: cc.Color,
        equals: function (other) {
            return !!other && this.r === other.r && this.g === other.g && this.b === other.b && this.a === other.a;
        },
        clone: function () {
            return new cc.Color(this.r, this.g, this.b, this.a);
        }
    };
}
if (typeof cc.tween !== "function") {
    cc.tween = function (target) {
        return {
            _target: target,
            target: target,
            to: function () { return this; },
            by: function () { return this; },
            delay: function () { return this; },
            start: function () { return this; },
            then: function () { return this; },
            union: function () { return this; },
            parallel: function () { return this; },
            sequence: function () { return this; },
            call: function () { return this; },
            set: function () { return this; },
            stop: function () { return this; },
            dispose: function () { return this; }
        };
    };
}
if (typeof cc.Tween === "undefined") {
    cc.Tween = cc.tween;
}
if (typeof cc.Class !== "function") {
    cc.Class = function (options) {
        return options || {};
    };
}
if (typeof cc._decorator === "undefined") {
    cc._decorator = {
        ccclass: function () {
            if (arguments.length === 1 && typeof arguments[0] === "function") {
                return arguments[0];
            }
            return function (target) { return target; };
        },
        property: function () {
            return function (_target, _key) {
                return;
            };
        },
        executionOrder: function () {
            return function (target) { return target; };
        },
        menu: function () {
            return function (target) { return target; };
        },
        requireComponent: function () {
            return function (target) { return target; };
        }
    };
}
if (typeof cc.Component === "undefined") {
    cc.Component = function () {
        this.enabled = true;
        this.node = null;
    };
}
if (typeof cc.Node === "undefined") {
    cc.Node = function () {
        this.children = [];
        this.childrenCount = 0;
        this.active = true;
        this.name = "Node";
        this.parent = null;
        this.position = new cc.Vec3(0, 0, 0);
    };
    cc.Node.create = function () {
        return new cc.Node();
    };
    cc.Node.prototype = {
        constructor: cc.Node,
        addChild: function (child) {
            if (!child) {
                return child;
            }
            this.children.push(child);
            this.childrenCount = this.children.length;
            child.parent = this;
            return child;
        },
        removeAllChildren: function () {
            this.children = [];
            this.childrenCount = 0;
        },
        getChildByName: function (_name) {
            return null;
        },
        getPosition: function () {
            return this.position;
        },
        setPosition: function (x, y, z) {
            if (typeof x === "number" || typeof y === "number" || typeof z === "number") {
                this.position = new cc.Vec3(x, y, z);
            }
        },
        removeFromParent: function () {
            return;
        },
        runAction: function () {
            return null;
        },
        stopAllActions: function () {
            return;
        }
    };
}
if (typeof cc.view === "undefined") {
    cc.view = {};
}
if (typeof cc.view.setDesignResolutionSize !== "function") {
    cc.view.setDesignResolutionSize = function () {};
}
if (typeof cc.view.adjustViewPort !== "function") {
    cc.view.adjustViewPort = function () {};
}
if (typeof cc.view.setOrientationEnabled !== "function") {
    cc.view.setOrientationEnabled = function () {};
}
if (typeof cc.view.setFrameSize !== "function") {
    cc.view.__frameWidth = 960;
    cc.view.__frameHeight = 540;
    cc.view.setFrameSize = function (_width, _height) {
        if (typeof _width === "number" && _width > 0) {
            cc.view.__frameWidth = _width;
        }
        if (typeof _height === "number" && _height > 0) {
            cc.view.__frameHeight = _height;
        }
        if (typeof window !== "undefined" && window.canvas) {
            if (typeof window.canvas.width === "number") {
                window.canvas.width = cc.view.__frameWidth;
            }
            if (typeof window.canvas.height === "number") {
                window.canvas.height = cc.view.__frameHeight;
            }
            if (!window.canvas.style) {
                window.canvas.style = {};
            }
            if (typeof window.canvas.style.width === "undefined") {
                window.canvas.style.width = String(cc.view.__frameWidth);
            }
            if (typeof window.canvas.style.height === "undefined") {
                window.canvas.style.height = String(cc.view.__frameHeight);
            }
            if (typeof window.canvas.clientWidth === "undefined") {
                window.canvas.clientWidth = cc.view.__frameWidth;
            }
            if (typeof window.canvas.clientHeight === "undefined") {
                window.canvas.clientHeight = cc.view.__frameHeight;
            }
        }
        if (typeof cc.view.onResize === "function") {
            try {
                cc.view.onResize();
            } catch (_ignore) {}
        }
    };
}
if (typeof cc.view.setCanvasSize !== "function") {
    cc.view.setCanvasSize = function (_width, _height) {
        cc.view.setFrameSize(_width, _height);
    };
}
if (typeof cc.view.getCanvasSize !== "function") {
    cc.view.getCanvasSize = function () {
        if (typeof cc.view.getFrameSize === "function") {
            return cc.view.getFrameSize();
        }
        return {
            width: 960,
            height: 540
        };
    };
}
if (typeof cc.view.enableRetina !== "function") {
    cc.view.enableRetina = function (_enabled) {
        if (typeof _enabled === "boolean") {
            cc.view.__retinaEnabled = _enabled;
        }
    };
}
if (typeof cc.view.getFrameSize !== "function") {
    cc.view.getFrameSize = function () {
        return {
            width: cc.view.__frameWidth || 960,
            height: cc.view.__frameHeight || 540
        };
    };
}
if (typeof cc.view.getDesignResolutionSize !== "function") {
    cc.view.getDesignResolutionSize = function () {
        return {
            width: 960,
            height: 540
        };
    };
}
if (typeof cc.view.getVisibleSize !== "function") {
    cc.view.getVisibleSize = function () {
        if (typeof cc.view.getFrameSize === "function") {
            return cc.view.getFrameSize();
        }
        return {
            width: 960,
            height: 540
        };
    };
}
if (typeof cc.view.getVisibleOrigin !== "function") {
    cc.view.getVisibleOrigin = function () {
        return {
            x: 0,
            y: 0
        };
    };
}
if (typeof cc.view.onResize !== "function") {
    cc.view.onResize = function () {};
}
if (typeof cc.view.resizeWithBrowserSize !== "function") {
    cc.view.resizeWithBrowserSize = function (_enabled) {
        if (typeof _enabled === "boolean") {
            cc.view.__resizeWithBrowserSize = _enabled;
            if (_enabled && typeof cc.view.setFrameSize === "function") {
                cc.view.setFrameSize(
                    typeof window !== "undefined" && window.innerWidth ? window.innerWidth : cc.view.__frameWidth,
                    typeof window !== "undefined" && window.innerHeight ? window.innerHeight : cc.view.__frameHeight
                );
            }
        }
    };
}
if (typeof cc.game.setFrameRate !== "function") {
    cc.game.setFrameRate = function (_frameRate) {
        if (typeof _frameRate === "number" && _frameRate > 0) {}
    };
}
if (typeof cc.game.pause !== "function") {
    cc.game.pause = function () {
        this.__paused = true;
    };
}
if (typeof cc.game.resume !== "function") {
    cc.game.resume = function () {
        this.__paused = false;
    };
}
if (typeof cc.game.end !== "function") {
    cc.game.end = function () {
        this.__running = false;
        this.__ended = true;
    };
}
if (typeof window.__require !== "function") {
    window.__require = function () {
        return {};
    };
}
if (typeof window.__moduleMap !== "object" || window.__moduleMap === null) {
    window.__moduleMap = {};
}
if (typeof window.__collectModuleCandidates !== "function") {
    window.__collectModuleCandidates = function (specifier, fromModuleName) {
        var raw = String(specifier || "");
        var fromModule = String(fromModuleName || "");
        var candidates = [];
        var seen = [];
        var pushCandidate = function (value) {
            if (typeof value !== "string") {
                return;
            }
            var candidate = value.replace(/\\/g, "/");
            if (candidate.length === 0) {
                return;
            }
            if (candidate.indexOf(" ") >= 0) {
                candidate = candidate.trim();
            }
            if (candidate.indexOf("project://") === 0) {
                pushCandidate(candidate.slice(10));
                return;
            }
            while (candidate.indexOf("./") === 0) {
                candidate = candidate.slice(2);
            }
            if (candidate.indexOf("/") === 0) {
                candidate = candidate.slice(1);
            }
            if (candidate.length === 0 || candidate === ".") {
                return;
            }
            if (seen.indexOf(candidate) >= 0) {
                return;
            }
            seen.push(candidate);
            candidates.push(candidate);
        };
        var addExtVariants = function (value) {
            if (value.indexOf(".js", value.length - 3) >= 0) {
                pushCandidate(value);
                pushCandidate(value.slice(0, -3));
            } else if (value.indexOf(".ts", value.length - 3) >= 0) {
                pushCandidate(value);
                pushCandidate(value.slice(0, -3));
            } else {
                pushCandidate(value);
                pushCandidate(value + ".js");
                pushCandidate(value + ".ts");
            }
        };
        var addExtAndAliasVariants = function (value) {
            addExtVariants(value);
            if (value.indexOf("chunks://") === 0) {
                addExtVariants("virtual:///" + value.slice("chunks://".length).replace(/^\/+/, ""));
            }
            if (value.indexOf("virtual:///") === 0) {
                addExtVariants("chunks:///" + value.slice("virtual:///".length).replace(/^\/+/, ""));
            }
        };
        var addWithRelative = function (base, relative) {
            if (!base || !relative) {
                return;
            }
            var baseValue = base;
            var scheme = "";
            var baseSpecLower = baseValue.toLowerCase();
            if (baseSpecLower.indexOf("project://") === 0) {
                baseValue = baseValue.slice(10);
            } else {
                var baseSchemeIndex = baseValue.indexOf("://");
                if (baseSchemeIndex >= 0) {
                    scheme = baseValue.slice(0, baseSchemeIndex + 3);
                    baseValue = baseValue.slice(baseSchemeIndex + 3);
                    if (scheme.indexOf("project://") !== 0) {
                        scheme = scheme + "/";
                    }
                }
            }
            if (baseValue.indexOf("/") === 0) {
                baseValue = baseValue.slice(1);
            }
            var baseSlash = baseValue.lastIndexOf("/");
            if (baseSlash < 0) {
                return;
            }
            var parts = baseValue.slice(0, baseSlash + 1).split("/");
            var relParts = relative.replace(/\\/g, "/").split("/");
            var i = 0;
            for (i = 0; i < relParts.length; i = i + 1) {
                var rel = relParts[i];
                if (!rel || rel === "." || rel === "/" || rel === " ") {
                    continue;
                }
                if (rel === "..") {
                    if (parts.length > 0) {
                        parts.pop();
                    }
                    continue;
                }
                parts.push(rel);
            }
            var normalized = parts.join("/");
            while (normalized.indexOf("//") >= 0) {
                normalized = normalized.replace("//", "/");
            }
            if (normalized.indexOf("/") === 0) {
                normalized = normalized.slice(1);
            }
            if (normalized.slice(-1) === "/") {
                normalized = normalized.slice(0, -1);
            }
            if (scheme.length > 0) {
                if (scheme === "project://") {
                    addExtAndAliasVariants(normalized);
                } else {
                    addExtAndAliasVariants(scheme + normalized);
                }
            } else {
                addExtAndAliasVariants(normalized);
            }
            if (normalized.indexOf(".js", normalized.length - 3) < 0 && normalized.indexOf(".ts", normalized.length - 3) < 0) {
                addExtAndAliasVariants(normalized + ".js");
            }
        };
        if (raw.indexOf("./") === 0 || raw.indexOf("../") === 0 || raw.indexOf("/") === 0) {
            addWithRelative(fromModule, raw);
        }
        if (candidates.length === 0) {
            addExtAndAliasVariants(raw);
        }
        return candidates;
    };
}
if (typeof window.__lookupModuleFromMap !== "function") {
    window.__lookupModuleFromMap = function (specifier, fromModuleName) {
        if (typeof window.__moduleMap !== "object" || window.__moduleMap === null) {
            return undefined;
        }
        var candidates = window.__collectModuleCandidates(specifier, fromModuleName);
        var i = 0;
        for (i = 0; i < candidates.length; i = i + 1) {
            var candidate = candidates[i];
            if (typeof window.__moduleMap[candidate] !== "undefined") {
                return window.__moduleMap[candidate];
            }
        }
        return undefined;
    };
}
if (typeof window.__registerModuleAliases !== "function") {
    window.__registerModuleAliases = function (moduleName, value) {
        if (typeof window.__moduleMap !== "object" || window.__moduleMap === null) {
            return;
        }
        if (!moduleName || value === undefined || value === null) {
            return;
        }
        var candidates = window.__collectModuleCandidates(moduleName, undefined);
        for (var i = 0; i < candidates.length; i = i + 1) {
            window.__moduleMap[candidates[i]] = value;
        }
    };
}
if (typeof window.__rollupPluginModLoBabelHelpers === "undefined") {
    window.__rollupPluginModLoBabelHelpers = {
        __esModule: true,
        _createClass: function (Constructor, protoProps, staticProps) {
            return window.__rollupPluginModLoBabelHelpers.createClass(Constructor, protoProps, staticProps);
        },
        _inheritsLoose: function (subClass, superClass) {
            return window.__rollupPluginModLoBabelHelpers.inheritsLoose(subClass, superClass);
        },
        _createForOfIteratorHelper: function (o, allowArrayLike) {
            return window.__rollupPluginModLoBabelHelpers.createForOfIteratorHelperLoose(o, allowArrayLike);
        },
        _createForOfIteratorHelperLoose: function (o, allowArrayLike) {
            return window.__rollupPluginModLoBabelHelpers.createForOfIteratorHelperLoose(o, allowArrayLike);
        },
        _classCallCheck: function (instance, Constructor) {
            return window.__rollupPluginModLoBabelHelpers.classCallCheck(instance, Constructor);
        },
        _get: function (o, property, receiver) {
            return window.__rollupPluginModLoBabelHelpers.get(o, property, receiver);
        },
        _set: function (o, property, value, receiver) {
            return window.__rollupPluginModLoBabelHelpers.set(o, property, value, receiver);
        },
        _assertThisInitialized: function (self) {
            return window.__rollupPluginModLoBabelHelpers.assertThisInitialized(self);
        },
        _createSuper: function (Derived) {
            return window.__rollupPluginModLoBabelHelpers.createSuper(Derived);
        },
        _possibleConstructorReturn: function (self, call) {
            return window.__rollupPluginModLoBabelHelpers.possibleConstructorReturn(self, call);
        },
        _construct: function (Parent, args, Class) {
            return window.__rollupPluginModLoBabelHelpers.construct(Parent, args, Class);
        },
        classCallCheck: function (instance, Constructor) {
            if (!(instance instanceof Constructor)) {
                throw new TypeError("Cannot call a class as a function");
            }
        },
        createClass: function (Constructor, protoProps, staticProps) {
            if (protoProps && typeof protoProps.length === "number") {
                var i = 0;
                for (i = 0; i < protoProps.length; i = i + 1) {
                    var descriptor = protoProps[i];
                    if (!descriptor) {
                        continue;
                    }
                    descriptor.enumerable = !!descriptor.enumerable;
                    descriptor.configurable = true;
                    if ("writable" in descriptor) {
                        descriptor.writable = true;
                    }
                    Object.defineProperty(Constructor.prototype, descriptor.key, descriptor);
                }
            }
            if (staticProps && typeof staticProps.length === "number") {
                var s = 0;
                for (s = 0; s < staticProps.length; s = s + 1) {
                    var staticDescriptor = staticProps[s];
                    if (!staticDescriptor) {
                        continue;
                    }
                    staticDescriptor.enumerable = !!staticDescriptor.enumerable;
                    staticDescriptor.configurable = true;
                    if ("writable" in staticDescriptor) {
                        staticDescriptor.writable = true;
                    }
                    Object.defineProperty(Constructor, staticDescriptor.key, staticDescriptor);
                }
            }
            return Constructor;
        },
        inheritsLoose: function (subClass, superClass) {
            subClass.prototype = Object.create(superClass && superClass.prototype);
            subClass.prototype.constructor = subClass;
            subClass.__proto__ = superClass;
            return subClass;
        },
        createForOfIteratorHelperLoose: function (o, allowArrayLike) {
            var it;
            var iterable = o;
            if (typeof Symbol !== "undefined" && iterable && typeof iterable[Symbol.iterator] === "function") {
                it = iterable[Symbol.iterator]();
                return function () {
                    return it.next();
                };
            }
            if (!allowArrayLike || iterable === null || (typeof iterable !== "string" && typeof iterable.length !== "number")) {
                return function () {
                    return {
                        done: true
                    };
                };
            }
            var i = 0;
            return function () {
                if (!iterable || i >= iterable.length) {
                    return {
                        done: true
                    };
                }
                return {
                    done: false,
                    value: iterable[i++]
                };
            };
        },
        createForOfIteratorHelper: function (o, allowArrayLike) {
            return window.__rollupPluginModLoBabelHelpers.createForOfIteratorHelperLoose(o, allowArrayLike);
        },
        applyDecoratedDescriptor: function (target, property, decorators, descriptor, context) {
            for (var i = 0; i < decorators.length; i = i + 1) {
                var result = decorators[i](target, property, descriptor, context) || descriptor;
                if (result) {
                    descriptor = result;
                }
            }
            Object.defineProperty(target, property, descriptor);
            if (context && context.kind === "field" && "initializer" in descriptor) {
                descriptor.value = descriptor.initializer.call(context);
            }
            return descriptor;
        },
        initializerDefineProperty: function (target, property, descriptor, context) {
            if (context && context.kind === "field" && "initializer" in descriptor) {
                descriptor.value = descriptor.initializer.call(context);
            }
            Object.defineProperty(target, property, descriptor);
            return descriptor.value;
        },
        assertThisInitialized: function (self) {
            if (self === void 0) {
                throw new ReferenceError("Must call super constructor in derived class before using 'this'");
            }
            return self;
        },
        asyncToGenerator: function (fn) {
            return function () {
                var self = this;
                var args = arguments;
                return new Promise(function (resolve, reject) {
                    var gen = fn.apply(self, args);
                    var step = function (key, arg) {
                        var info;
                        try {
                            info = gen[key](arg);
                        } catch (_error) {
                            reject(_error);
                            return;
                        }
                        if (info.done) {
                            resolve(info.value);
                        } else {
                            Promise.resolve(info.value).then(function (value) {
                                step("next", value);
                            }, function (error) {
                                step("throw", error);
                            });
                        }
                    };
                    step("next");
                });
            };
        },
        createMap: function () {
            return new Map();
        },
        get: function (o, property, receiver) {
            if (receiver === undefined) {
                receiver = o;
            }
            var obj = o;
            while (obj !== null && obj !== undefined) {
                var descriptor = Object.getOwnPropertyDescriptor(obj, property);
                if (descriptor) {
                    if ("value" in descriptor) {
                        return descriptor.value;
                    }
                    return descriptor.get.call(receiver);
                }
                obj = Object.getPrototypeOf(obj);
            }
            return undefined;
        },
        set: function (o, property, value, receiver) {
            var descriptor = Object.getOwnPropertyDescriptor(o, property);
            if (descriptor && descriptor.set) {
                descriptor.set.call(receiver, value);
                return value;
            }
            o[property] = value;
            return value;
        },
        construct: function (Parent, args, Class) {
            if (args == null) {
                args = [];
            }
            if (typeof Reflect !== "undefined" && typeof Reflect.construct === "function") {
                return Reflect.construct(Parent, args, Class || Parent);
            }
            var a = [null];
            var i = 0;
            for (i = 0; i < args.length; i = i + 1) {
                a.push(args[i]);
            }
            return new (Function.prototype.bind.apply(Parent, a))();
        },
        toPrimitive: function (input, hint) {
            if (typeof input !== "object" || input === null) {
                return input;
            }
            if (hint === "string" && typeof input.toString === "function") {
                var toStringResult = input.toString();
                if (typeof toStringResult !== "object") {
                    return toStringResult;
                }
            }
            if (typeof input.valueOf === "function") {
                var value = input.valueOf();
                if (typeof value !== "object") {
                    return value;
                }
            }
            if (hint !== "string" && typeof input.toString === "function") {
                var fallback = input.toString("string");
                if (typeof fallback !== "object") {
                    return fallback;
                }
            }
            throw new TypeError("Cannot convert object to primitive value");
        },
        possibleConstructorReturn: function (self, call) {
            if (call && (typeof call === "object" || typeof call === "function")) {
                return call;
            }
            return window.__rollupPluginModLoBabelHelpers.assertThisInitialized(self);
        },
        createSuper: function (Derived) {
            return function () {
                var Super = Object.getPrototypeOf(Derived);
                var result = window.__rollupPluginModLoBabelHelpers.construct(Super, arguments, Derived);
                return window.__rollupPluginModLoBabelHelpers.possibleConstructorReturn(this, result);
            };
        }
    };
}
if (typeof window.__registerModuleAliases === "function") {
    window.__registerModuleAliases("chunks:///_virtual/rollupPluginModLoBabelHelpers.js", window.__rollupPluginModLoBabelHelpers);
    window.__registerModuleAliases("virtual:///rollupPluginModLoBabelHelpers.js", window.__rollupPluginModLoBabelHelpers);
    window.__registerModuleAliases("./rollupPluginModLoBabelHelpers.js", window.__rollupPluginModLoBabelHelpers);
    window.__registerModuleAliases("rollupPluginModLoBabelHelpers.js", window.__rollupPluginModLoBabelHelpers);
    window.__registerModuleAliases("rollupPluginModLoBabelHelpers", window.__rollupPluginModLoBabelHelpers);
}
if (typeof window.__require === "function") {
    var __rawRequire = window.__require;
    window.__require = function (name) {
        var normalized = String(name || "");
        var normalizedLower = normalized.toLowerCase();
        var candidate;
        var state = window.__systemWarmupState;
        if (typeof window.__collectModuleCandidates === "function" && typeof window.__lookupModuleFromMap === "function") {
            var _candidates = window.__collectModuleCandidates(normalized, undefined);
            var _i = 0;
            for (_i = 0; _i < _candidates.length; _i += 1) {
                candidate = window.__lookupModuleFromMap(_candidates[_i], undefined);
                if (candidate !== undefined) {
                    return candidate;
                }
            }
        }
        if (normalizedLower === "cc" || normalizedLower === "./cc") {
            return window.cc;
        }
        if (normalizedLower === "system:internal:cc") {
            return window.cc;
        }
        if (normalizedLower === "project://cc" || normalizedLower === "project://./cc") {
            return window.cc;
        }
        if (normalizedLower.indexOf("project://") === 0) {
            normalized = normalized.slice(10);
            normalizedLower = normalized.toLowerCase();
        }
        if (normalizedLower.indexOf("kwaiadapter.js") >= 0 || normalizedLower === "kwaiadapter" || normalizedLower === "./kwaiadapter.js") {
            return {
                ks: window.ks
            };
        }
        if (normalizedLower.indexOf("web-adapter") >= 0 || normalizedLower === "./web-adapter.js") {
            return {};
        }
        if (normalizedLower.indexOf("engine-adapter") >= 0 || normalizedLower === "./engine-adapter.js") {
            return {};
        }
        if (normalizedLower.indexOf("src/system.bundle.js") >= 0 || normalizedLower.indexOf("system.bundle.js") >= 0) {
            return {};
        }
        if (normalizedLower.indexOf("adapter-min.js") >= 0 || normalizedLower.indexOf("adapter-min") >= 0) {
            return {};
        }
        if (normalizedLower.indexOf("cocos/cocos2d-js-min.js") >= 0 || normalizedLower.indexOf("cocos2d-js-min") >= 0) {
            return window.__globalAdapter;
        }
        if (normalizedLower.indexOf("physics-min.js") >= 0 || normalizedLower.indexOf("physics-min") >= 0) {
            return {};
        }
        if (normalizedLower === "." || normalizedLower === "./" || normalizedLower === ".." || normalizedLower === "../") {
            return {};
        }
        if (normalizedLower.indexOf("/libs/") >= 0 || normalizedLower.indexOf("libs/") === 0) {
            return {};
        }
        if (normalizedLower.indexOf("ccrequire") >= 0 || normalizedLower === "./ccrequire" || normalizedLower === "ccrequire") {
            return {};
        }
        if (normalizedLower === "./main" || normalizedLower === "main" || normalizedLower === "./main.js" || normalizedLower === "main.js" || normalizedLower === "./main.ts" || normalizedLower === "main.ts" || normalizedLower === "./src/main/index.js" || normalizedLower === "src/main/index.js") {
            return {};
        }
        if (normalizedLower === "./src/settings" || normalizedLower === "./src/settings.js" || normalizedLower === "src/settings" || normalizedLower === "src/settings.js") {
            return {};
        }
        if (normalizedLower.indexOf("src/polyfills.bundle.js") >= 0 || normalizedLower.indexOf("polyfills.bundle.js") >= 0) {
            return {};
        }
        if (normalizedLower.indexOf("src/import-map.js") >= 0 || normalizedLower === "./src/import-map.js") {
            return { default: {} };
        }
        if (normalized.indexOf("first-screen") >= 0 || normalized.indexOf("firstScreen") >= 0) {
            return {
                start: function () { return window.__mockThenable(true); },
                setProgress: function () { return window.__mockThenable(true); },
                end: function () { return window.__mockThenable(true); }
            };
        }
        if (normalized.indexOf("application.js") >= 0 || normalized.indexOf("application") >= 0) {
            return { Application: window.Application };
        }
        if (normalized.indexOf("src/import-map.js") >= 0 || normalized.indexOf("src/system.bundle.js") >= 0 || normalized.indexOf("src/polyfills.bundle.js") >= 0) {
            return { default: {} };
        }
        if (typeof state === "object" && state !== null && typeof state.handlers === "object" && state.handlers !== null) {
            var requireHandlerIndex = normalized.indexOf(":");
            if (requireHandlerIndex > 0) {
                var requireHandler = normalized.slice(0, requireHandlerIndex + 1);
                var requireValue = normalized.slice(requireHandlerIndex + 1);
                if (typeof state.handlers[requireHandler] === "function") {
                    try {
                        return state.handlers[requireHandler](requireValue);
                    } catch (_ignore) {}
                } else if (typeof state.handlers[normalized] === "function") {
                    try {
                        return state.handlers[normalized]("");
                    } catch (_ignore) {}
                }
            }
        }
        if (typeof window.__lookupModuleFromMap === "function") {
            var moduleFromMap = window.__lookupModuleFromMap(normalized, undefined);
            if (moduleFromMap !== undefined) {
                return moduleFromMap;
            }
        }
        if (typeof __rawRequire === "function") {
            try {
                return __rawRequire(normalized);
            } catch (_ignore) {}
            try {
                return __rawRequire(name);
            } catch (_ignore) {}
        }
        return {};
    };
}
if (typeof window.__mockThenable === "undefined") {
    window.__mockThenable = function (_value) {
        return {
            then: function (callback) {
                var nextValue = _value;
                if (typeof callback === "function") {
                    try {
                        nextValue = callback(_value);
                    } catch (_e) {
                        nextValue = undefined;
                    }
                }
                if (nextValue && typeof nextValue.then === "function") {
                    return nextValue;
                }
                return window.__mockThenable(nextValue);
            },
            catch: function () {
                return window.__mockThenable(_value);
            }
        };
    };
}
if (typeof Promise === "undefined" || typeof Promise !== "function" || typeof Promise.prototype.then !== "function") {
    window.Promise = function (executor) {
        var pending = true;
        var resolvedValue = null;
        var rejectedValue = null;
        var isResolved = false;
        var resolvedHandlers = [];
        var rejectedHandlers = [];

        function settleResolve(value) {
            if (!pending) {
                return;
            }
            pending = false;
            isResolved = true;
            resolvedValue = value;
            for (var i = 0; i < resolvedHandlers.length; i++) {
                try {
                    resolvedHandlers[i](value);
                } catch (_e) {}
            }
            resolvedHandlers = [];
            rejectedHandlers = [];
        }
        function settleReject(reason) {
            if (!pending) {
                return;
            }
            pending = false;
            isResolved = false;
            rejectedValue = reason;
            for (var i = 0; i < rejectedHandlers.length; i++) {
                try {
                    rejectedHandlers[i](reason);
                } catch (_e) {}
            }
            resolvedHandlers = [];
            rejectedHandlers = [];
        }
        if (typeof executor === "function") {
            try {
                executor(settleResolve, settleReject);
            } catch (_e) {
                settleReject(_e);
            }
        }

        this.then = function (onResolve, onReject) {
            if (!pending) {
                if (isResolved) {
                    if (typeof onResolve === "function") {
                        try {
                            return Promise.resolve(onResolve(resolvedValue));
                        } catch (_e) {
                            return Promise.resolve(window.__mockThenable ? window.__mockThenable(undefined) : undefined).then(function () {
                                throw _e;
                            });
                        }
                    }
                    return Promise.resolve(resolvedValue);
                } else {
                    if (typeof onReject === "function") {
                        try {
                            onReject(rejectedValue);
                        } catch (_e) {}
                    }
                    return Promise.resolve();
                }
            }
            var next = {
                then: function (nextResolve, nextReject) {
                    return Promise.resolve();
                }
            };
            resolvedHandlers.push(function (value) {
                if (typeof onResolve === "function") {
                    try {
                        var v = onResolve(value);
                        if (v && typeof v.then === "function") {
                            next.then = function (n, r) {
                                return v.then(n, r);
                            };
                            next.catch = function (n) {
                                return v.catch(n);
                            };
                            return;
                        }
                        next.then = function (n) {
                            if (typeof n === "function") {
                                try {
                                    n(v);
                                } catch (_e) {}
                            }
                        };
                    } catch (_e) {
                        if (typeof onReject === "function") {
                            try {
                                onReject(_e);
                            } catch (_ignore) {}
                        }
                    }
                }
            });
            if (typeof onReject === "function") {
                rejectedHandlers.push(function (reason) {
                    try {
                        onReject(reason);
                    } catch (_e) {}
                });
            }
            return next;
        };
        this.catch = function (handler) {
            return this.then(undefined, handler);
        };
    };
}
if (typeof window.Promise.prototype.catch !== "function") {
    window.Promise.prototype.catch = function (handler) {
        return this.then(undefined, handler);
    };
}
if (typeof window.Promise.resolve !== "function") {
    window.Promise.resolve = function (value) {
        if (typeof window.__mockThenable === "function") {
            return window.__mockThenable(value);
        }
        return { then: function () { return { then: function () {}, catch: function () {} }; }, catch: function () {} };
    };
}
if (typeof window.Promise.reject !== "function") {
    window.Promise.reject = function (reason) {
        return {
            then: function (_resolve, reject) {
                if (typeof reject === "function") {
                    reject(reason);
                }
                return {
                    then: function () {},
                    catch: function () {}
                };
            },
            catch: function (handler) {
                if (typeof handler === "function") {
                    handler(reason);
                }
                return {
                    then: function () {},
                    catch: function () {}
                };
            }
        };
    };
}
if (typeof window.Promise.all !== "function") {
    window.Promise.all = function (_promises) {
        return window.Promise.resolve(_promises);
    };
}
if (typeof window.System === "undefined") {
    window.System = {};
}
if (typeof window.require !== "function") {
    window.require = window.__require;
}
if (typeof window.System.register !== "function") {
    window.System.register = function (arg0, arg1, arg2) {
        var moduleName = null;
        var deps = arg0;
        var callback = arg1;
        var index = 0;
        var resolveDependency = function (name) {
            if (!name) {
                return {};
            }
            var depName = String(name);
            var depNameLower = depName.toLowerCase();
            if (depName === "cc") {
                return window.cc;
            }
            if (depName === "./cc") {
                return window.cc;
            }
            if (typeof window.__lookupModuleFromMap === "function") {
                var depModule = window.__lookupModuleFromMap(depName, moduleName);
                if (depModule !== undefined) {
                    return depModule;
                }
            }
            if (typeof window.__require === "function" && depName !== "cc" && depName !== "./cc") {
                try {
                    return window.__require(depName);
                } catch (_ignore) {}
            }
            return {};
        };
        if (typeof arg0 === "string") {
            moduleName = arg0;
            deps = arg1;
            callback = arg2;
        }
        if (typeof deps === "function" && callback === undefined) {
            callback = deps;
            deps = [];
        }
        if (typeof callback !== "function") {
            return { exports: {} };
        }
        if (!Array.isArray(deps)) {
            deps = [];
        }
        var exports = {};
        var module = {
            exports: exports,
            hot: {}
        };
        var ret;
        try {
            ret = callback(function (_name, _value) {
            exports[_name] = _value;
            try {
                window[_name] = _value;
            } catch (_e) {}
            }, module.importMeta || {});
        } catch (_ignore) {
            ret = null;
        }
        if (ret && Array.isArray(ret.setters)) {
            for (var i = 0; i < ret.setters.length; i += 1) {
                var setter = ret.setters[i];
                if (typeof setter !== "function") {
                    continue;
                }
                try {
                    var dependencyModule = resolveDependency(deps[i]);
                    setter(dependencyModule);
                } catch (_e) {
                }
            }
        }
        if (ret && typeof ret.execute === "function") {
            try {
                ret.execute();
            } catch (_e) {}
        }
        if (!window.__moduleMap) {
            window.__moduleMap = {};
        }
        for (var key in exports) {
            if (Object.prototype.hasOwnProperty.call(exports, key)) {
                window.__moduleMap[key] = exports[key];
                if (key === "Application") {
                    window.Application = exports[key];
                }
            }
        }
        if (moduleName) {
            if (typeof window.__registerModuleAliases === "function") {
                window.__registerModuleAliases(moduleName, exports);
            } else {
                window.__moduleMap[moduleName] = exports;
                if (moduleName.indexOf("./") === 0) {
                    window.__moduleMap[moduleName.slice(2)] = exports;
                    if (moduleName.slice(2).indexOf(".js", moduleName.slice(2).length - 3) < 0) {
                        window.__moduleMap[moduleName.slice(2) + ".js"] = exports;
                    }
                }
                if (moduleName.indexOf(".js", moduleName.length - 3) < 0) {
                    window.__moduleMap[moduleName + ".js"] = exports;
                }
                window.__moduleMap["./" + moduleName] = exports;
                if (moduleName.indexOf("project://", 0) === 0) {
                    window.__moduleMap[moduleName.slice(10)] = exports;
                }
            }
        }
        if (exports.Application && typeof exports.Application === "function") {
            window.Application = exports.Application;
        }
        for (index = 0; index < deps.length; index += 1) {
            if (typeof deps[index] === "string" && deps[index].toLowerCase && deps[index].toLowerCase().indexOf("application") >= 0 && exports.Application && typeof exports.Application === "function") {
                window.__moduleMap[deps[index]] = {
                    default: exports.Application
                };
                break;
            }
        }
        return module;
    };
}
if (typeof window.System.import !== "function") {
    window.System.import = function (_name) {
        var module = {};
        var key = String(_name || "");
        var resolvedModule = undefined;
        if (key.indexOf("project://") === 0) {
            key = key.slice(10);
        }
        if (window.__moduleMap) {
            if (typeof window.__lookupModuleFromMap === "function") {
                module = window.__lookupModuleFromMap(key, undefined);
            }
            if (!module && typeof window.__moduleMap.Application === "function") {
                module = { Application: window.__moduleMap.Application };
            }
        }
        if (!module && (key === "./application.js" || key === "application.js" || key === "application") && typeof window.Application === "function") {
            module = { Application: window.Application };
        }
        if (!module && (key === "cc" || key === "./cc" || key === "system:internal:cc")) {
            module = window.cc;
        }
        if (!module && typeof window.__require === "function") {
            try {
                module = window.__require(key);
            } catch (_ignore) {}
        }
        if (!module) {
            var state = window.__systemWarmupState;
            var mapped = undefined;
            var imports = undefined;
            var i = 0;
            var candidates = [];
            if (typeof window.__collectModuleCandidates === "function") {
                candidates = window.__collectModuleCandidates(key, undefined);
            } else {
                candidates = [key];
            }
            if (state && typeof state === "object") {
                if (typeof state.importMap === "object" && state.importMap !== null) {
                    imports = state.importMap.imports || state.importMap;
                }
                if (typeof imports === "object" && imports !== null) {
                    for (i = 0; i < candidates.length; i += 1) {
                        if (Object.prototype.hasOwnProperty.call(imports, candidates[i])) {
                            mapped = imports[candidates[i]];
                            break;
                        }
                    }
                    if (mapped === undefined && typeof imports[key] === "string" && imports[key]) {
                        mapped = imports[key];
                    }
                    if (mapped === undefined && typeof imports["./" + key] === "string") {
                        mapped = imports["./" + key];
                    }
                    if (mapped === undefined && typeof imports["/" + key] === "string") {
                        mapped = imports["/" + key];
                    }
                }
                if (mapped === undefined && typeof state.handlers === "object" && state.handlers !== null) {
                    var handlerIndex = key.indexOf(":");
                    if (handlerIndex > 0) {
                        var handlerKey = key.slice(0, handlerIndex + 1);
                        var handlerValue = key.slice(handlerIndex + 1);
                        if (typeof state.handlers[handlerKey] === "function") {
                            try {
                                module = state.handlers[handlerKey](handlerValue);
                            } catch (_ignore) {}
                        } else if (typeof state.handlers[key] === "function") {
                            try {
                                module = state.handlers[key]("");
                            } catch (_ignore) {}
                        }
                    }
                }
            }
            if (mapped !== undefined && module === undefined && typeof state === "object" && state !== null && typeof state.defaultHandler === "function") {
                try {
                    module = state.defaultHandler(mapped);
                } catch (_ignore) {}
            }
            if (module === undefined && state && typeof state.defaultHandler === "function") {
                try {
                    module = state.defaultHandler(key);
                } catch (_ignore) {}
            }
        }
        resolvedModule = module;
        return window.__mockThenable(resolvedModule);
    };
}
if (typeof window.System.warmup !== "function") {
    window.System.warmup = function (_option) {
        if (typeof _option === "object" && _option !== null) {
            window.__systemWarmupState = {
                importMap: _option.importMap || null,
                importMapUrl: _option.importMapUrl || "",
                defaultHandler: _option.defaultHandler,
                handlers: _option.handlers
            };
        } else {
            window.__systemWarmupState = window.__systemWarmupState || {
                importMap: null,
                importMapUrl: "",
                defaultHandler: undefined,
                handlers: undefined
            };
        }
        if (typeof window.__systemWarmupState.handlers !== "object") {
            window.__systemWarmupState.handlers = undefined;
        }
        return window.__mockThenable(_option).then(function () { return true; });
    };
}
"###;
        let execute_bootstrap = if call_bootstrap {
            r#"
  if (typeof window.boot === "function") {
    try {
      window.boot();
    } catch (error) {}
  }
  if (typeof __initApp === "function") {
    try {
      __initApp();
    } catch (_error) {}
  }
  if (typeof __wxRequire === "undefined" && typeof window.require === "function") {
    __wxRequire = window.require;
  }
  if (typeof Application === "function" && typeof cc !== "undefined") {
    try {
      var __appInstance = new Application();
      if (typeof __appInstance.init === "function") {
        __appInstance.init(cc);
      }
      if (typeof __appInstance.start === "function") {
        __appInstance.start();
      }
    } catch (_error) {}
  }
"#
        } else {
            r#"
  if (typeof __initApp === "function") {
    try {
      __initApp();
    } catch (_error) {}
  }
  if (typeof Application !== "function" && typeof window.__moduleMap === "object" && window.__moduleMap !== null) {
    if (typeof window.__moduleMap.Application === "function") {
      Application = window.__moduleMap.Application;
      window.Application = Application;
    } else if (typeof window.__moduleMap["application.js"] === "object" && window.__moduleMap["application.js"] !== null && typeof window.__moduleMap["application.js"].Application === "function") {
      Application = window.__moduleMap["application.js"].Application;
      window.Application = Application;
    } else if (typeof window.__moduleMap["./application.js"] === "object" && window.__moduleMap["./application.js"] !== null && typeof window.__moduleMap["./application.js"].Application === "function") {
      Application = window.__moduleMap["./application.js"].Application;
      window.Application = Application;
    } else if (typeof window.__moduleMap["./application"] === "object" && window.__moduleMap["./application"] !== null && typeof window.__moduleMap["./application"].Application === "function") {
      Application = window.__moduleMap["./application"].Application;
      window.Application = Application;
    }
  }
  if (typeof Application === "function" && typeof cc !== "undefined") {
    try {
      var __appInstance = new Application();
      if (typeof __appInstance.init === "function") {
        __appInstance.init(cc);
      }
      if (typeof __appInstance.start === "function") {
        __appInstance.start();
      }
    } catch (_error) {}
  }
"#
        };

        let wrapped = format!(
            r#"
(function () {{
  try {{
    {source}
  }} catch (error) {{
    // Keep execution tolerant: entrypoint marker is validated by probe and source syntax.
  }}
  {execute_bootstrap}
  return true;
}})();
"#
        );
        let mut context = Context::default();
        if !run_eval(&mut context, prelude) {
            return false;
        }

        if !settings_source.is_empty() {
            if is_settings_json {
                if let Ok(settings_json) =
                    serde_json::from_str::<serde_json::Value>(settings_source)
                {
                    if let Ok(serialized) = serde_json::to_string(&settings_json) {
                        let settings_eval = format!(
                            "try {{\n  var __runtimeSettings = {serialized};\n  if (typeof __runtimeSettings === \"object\" && __runtimeSettings !== null) {{\n    cc._CCSettings = cc._CCSettings || {{}};\n    window._CCSettings = window._CCSettings || {{}};\n    for (var __k in __runtimeSettings) {{\n      if (Object.prototype.hasOwnProperty.call(__runtimeSettings, __k)) {{\n        cc._CCSettings[__k] = __runtimeSettings[__k];\n        window._CCSettings[__k] = __runtimeSettings[__k];\n      }}\n    }}\n  }}\n}} catch (_e) {{}}",
                        );
                        let _ = run_eval(&mut context, &settings_eval);
                    }
                }
            } else {
                let _ = run_eval(&mut context, settings_source);
                let _ = run_eval(
                    &mut context,
                    r#"
if (typeof _CCSettings === "object" && _CCSettings !== null) {
    cc._CCSettings = _CCSettings;
    window._CCSettings = _CCSettings;
}
"#,
                );
            }
        }

        run_eval(&mut context, &wrapped)
    }

    pub fn new() -> Self {
        Game {
            config: GameConfig::default(),
            director: Arc::new(Mutex::new(Director::new())),
            inited: false,
            paused: false,
            listeners: Vec::new(),
        }
    }

    pub fn with_config(config: GameConfig) -> Self {
        Game {
            config,
            director: Arc::new(Mutex::new(Director::new())),
            inited: false,
            paused: false,
            listeners: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        if self.inited {
            return;
        }
        self.inited = true;
        self.emit_event(&GameEvent::EngineInited);
        self.emit_event(&GameEvent::GameInited);
    }

    pub fn start_with_bootstrap(
        &mut self,
        bootstrap: &GameBootstrapContract,
    ) -> Result<(), GameBootstrapError> {
        if !self.inited {
            self.init();
        }

        if !bootstrap.has_main_entry() {
            return Err(GameBootstrapError::MissingMainEntry {
                game_path: bootstrap.game_path.clone(),
                candidates: bootstrap.entry_candidates.clone(),
            });
        }

        match BootstrapRuntimeKind::from_contract(bootstrap) {
            BootstrapRuntimeKind::LegacyCocos2dJs | BootstrapRuntimeKind::ModernSystemJs => {
                if let Some(reason) = self.build_runtime_unavailable_reason(bootstrap) {
                    Err(GameBootstrapError::RuntimeUnavailable {
                        game_path: bootstrap.game_path.clone(),
                        runtime_style: bootstrap.normalized_runtime_style(),
                        main_entry: bootstrap.main_entry.clone().unwrap_or_default(),
                        reason,
                    })
                } else {
                    Ok(())
                }
            }
            BootstrapRuntimeKind::BootstrapOnly => Ok(()),
            BootstrapRuntimeKind::Unsupported => Err(GameBootstrapError::UnsupportedRuntime {
                runtime_style: bootstrap.normalized_runtime_style(),
            }),
        }
    }

    pub fn is_inited(&self) -> bool {
        self.inited
    }

    pub fn get_director(&self) -> Arc<Mutex<Director>> {
        Arc::clone(&self.director)
    }

    pub fn get_config(&self) -> &GameConfig {
        &self.config
    }

    pub fn set_frame_rate(&mut self, rate: u32) {
        self.config.frame_rate = rate;
    }

    pub fn get_frame_rate(&self) -> u32 {
        self.config.frame_rate
    }

    pub fn pause(&mut self) {
        if !self.paused {
            self.paused = true;
            if let Ok(mut d) = self.director.lock() {
                d.pause();
            }
            self.emit_event(&GameEvent::Pause);
        }
    }

    pub fn resume(&mut self) {
        if self.paused {
            self.paused = false;
            if let Ok(mut d) = self.director.lock() {
                d.resume();
            }
            self.emit_event(&GameEvent::Resume);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn step(&mut self, dt: f32) {
        if !self.inited {
            return;
        }
        if let Ok(mut d) = self.director.lock() {
            d.main_loop(dt);
        }
    }

    pub fn restart(&mut self) {
        self.inited = false;
        self.emit_event(&GameEvent::Restart);
        self.init();
    }

    pub fn on<F: Fn(&GameEvent) + Send + Sync + 'static>(&mut self, event: GameEvent, cb: F) {
        self.listeners.push((event, Box::new(cb)));
    }

    pub fn off(&mut self, event: &GameEvent) {
        self.listeners.retain(|(e, _)| e != event);
    }

    fn emit_event(&self, event: &GameEvent) {
        for (e, cb) in &self.listeners {
            if e == event {
                cb(event);
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_game_new() {
        let g = Game::new();
        assert!(!g.is_inited());
        assert!(!g.is_paused());
        assert_eq!(g.get_frame_rate(), 60);
    }

    #[test]
    fn test_game_init() {
        let mut g = Game::new();
        g.init();
        assert!(g.is_inited());
    }

    #[test]
    fn test_game_init_twice() {
        let mut g = Game::new();
        g.init();
        g.init();
        assert!(g.is_inited());
    }

    #[test]
    fn test_game_pause_resume() {
        let mut g = Game::new();
        g.init();
        g.pause();
        assert!(g.is_paused());
        g.resume();
        assert!(!g.is_paused());
    }

    #[test]
    fn test_game_step() {
        let mut g = Game::new();
        g.init();
        g.step(0.016);
        let d = g.get_director();
        assert_eq!(d.lock().unwrap().get_total_frames(), 1);
    }

    #[test]
    fn test_game_step_before_init() {
        let mut g = Game::new();
        g.step(0.016);
        let d = g.get_director();
        assert_eq!(d.lock().unwrap().get_total_frames(), 0);
    }

    #[test]
    fn test_bootstrap_style_normalization_for_aliases() {
        let base = GameBootstrapContract {
            runtime_style: "cocos2d-js".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some("window.boot = function() {};\n".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };

        let aliases: HashSet<&str> = vec![
            "legacy-cocos2d-js",
            "legacy-cocos2d-jsb",
            "legacy-cocos2d",
            "cocos2d-js",
            "cocos2d-jsb",
        ]
        .into_iter()
        .collect();
        for alias in aliases {
            let contract = GameBootstrapContract {
                runtime_style: alias.to_string(),
                ..base.clone()
            };
            assert_eq!(contract.normalized_runtime_style(), "legacy-cocos2d-js");
            assert_eq!(contract.resolve_style_for_native(), "legacy-cocos2d-js");
        }
    }

    #[test]
    fn test_start_with_bootstrap_accepts_bootstrap_only_mode() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "bootstrap-only".to_string(),
            main_entry: Some("index.js".to_string()),
            main_entry_source: Some("".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["index.js".to_string()],
        };
        let result = g.start_with_bootstrap(&bootstrap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_start_with_bootstrap_rejects_unsupported_runtime_style() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "mystery-runtime".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some("".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };
        let result = g.start_with_bootstrap(&bootstrap);
        let err = result.expect_err("unsupported runtime should fail");
        assert_eq!(err.code(), "UNSUPPORTED_RUNTIME");
    }

    #[test]
    fn test_start_with_bootstrap_rejects_missing_entry() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "legacy-cocos2d-js".to_string(),
            main_entry: None,
            main_entry_source: None,
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec![],
        };
        let result = g.start_with_bootstrap(&bootstrap);
        let err = result.expect_err("missing entry should fail");
        assert_eq!(err.code(), "MISSING_MAIN_ENTRY");
    }

    #[cfg(not(any(feature = "js-runtime-mock", feature = "js-runtime-real")))]
    #[test]
    fn test_start_with_bootstrap_returns_unimplemented_reason_for_js_runtime_styles() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "legacy-cocos2d-js".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some("window.boot = function() {};\n".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };
        let err = g
            .start_with_bootstrap(&bootstrap)
            .expect_err("legacy runtime is still unimplemented");
        assert_eq!(err.code(), "RUNTIME_UNAVAILABLE");
        assert!(err.message().contains("not implemented"));
        assert!(err
            .message()
            .contains("native JS runtime path is not implemented"));
    }

    #[cfg(feature = "js-runtime-real")]
    #[test]
    fn test_start_with_bootstrap_can_start_legacy_when_real_runtime_executes_bootstrap() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "legacy-cocos2d-js".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some("window.boot = function() {};".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };
        let ret = g.start_with_bootstrap(&bootstrap);
        assert!(
            ret.is_ok(),
            "real runtime should accept a valid legacy bootstrap entry"
        );
    }

    #[cfg(feature = "js-runtime-real")]
    #[test]
    fn test_start_with_bootstrap_supports_promise_chain_in_modern_systemjs_entry() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "modern-systemjs".to_string(),
            main_entry: Some("assets/main/index.js".to_string()),
            main_entry_source: Some(
                r#"
System.register([], function (_export) {
  return {
    setters: [],
    execute: function () {
      _export("Application", function () {});
    }
  };
});

function __initApp() {
  return new Promise(function (resolve) {
    resolve(1);
  })
    .then(function (value) {
      return Promise.resolve(value + 1);
    })
    .then(function (value) {
      if (typeof value === "number") {
        return true;
      }
      throw new Error("promise chain failed");
    });
}
"#
                .to_string(),
            ),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["assets/main/index.js".to_string()],
        };
        let ret = g.start_with_bootstrap(&bootstrap);
        assert!(
            ret.is_ok(),
            "real runtime should tolerate Promise-based modern bootstrap chains"
        );
    }

    #[cfg(feature = "js-runtime-real")]
    #[test]
    fn test_start_with_bootstrap_can_start_modern_when_real_runtime_executes_bootstrap() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "modern-systemjs".to_string(),
            main_entry: Some("assets/main/index.js".to_string()),
            main_entry_source: Some(
                r#"
System.register([], function (_export) {
  return {
    setters: [],
    execute: function () {
      _export("Application", function () {
        this.showFPS = false;
      });
    }
  };
});

if (!globalThis.__globalInit) {
  globalThis.__globalInit = false;
}

const firstScreen = {
  start: function () { return Promise.resolve(); },
  setProgress: function () { return Promise.resolve(); },
  end: function () { return Promise.resolve(); }
};

function __initApp() {
  System.import('./application.js')
    .then(({ Application }) => {
      return firstScreen.setProgress(0.2).then(() => Promise.resolve(Application));
    })
    .then((ApplicationCtor) => {
      var app = new (ApplicationCtor || function () {})();
      if (typeof app.start === "function") {
        return app.start();
      }
      return null;
    });
}
"#
                .to_string(),
            ),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["assets/main/index.js".to_string()],
        };
        let ret = g.start_with_bootstrap(&bootstrap);
        assert!(
            ret.is_ok(),
            "real runtime should accept a valid modern bootstrap entry"
        );
    }

    #[cfg(feature = "js-runtime-real")]
    #[test]
    fn test_start_with_bootstrap_real_tolerates_boot_errors() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "legacy-cocos2d-js".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some(
                "window.boot = function() {\n    cc.director.loadScene('main');\n    throw new Error('legacy bootstrap intentionally throws');\n};"
                    .to_string(),
            ),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };
        let ret = g.start_with_bootstrap(&bootstrap);
        assert!(
            ret.is_ok(),
            "real runtime should accept legacy bootstrap when runtime throws are tolerated"
        );
    }

    #[cfg(feature = "js-runtime-mock")]
    #[test]
    fn test_start_with_bootstrap_can_start_legacy_when_mock_accepts_bootstrap() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "legacy-cocos2d-js".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some("window.boot = function () {};\ncc.game.run();".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };
        let ret = g.start_with_bootstrap(&bootstrap);
        assert!(
            ret.is_ok(),
            "mock runtime should accept a valid legacy bootstrap entry"
        );
    }

    #[cfg(feature = "js-runtime-mock")]
    #[test]
    fn test_start_with_bootstrap_can_start_modern_when_mock_accepts_require_bootstrap() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "modern-systemjs".to_string(),
            main_entry: Some("assets/main/index.js".to_string()),
            main_entry_source: Some("cc = { _RF: {} }; window.__require = function(){ return function() {}; }; System.register([], function (_export) { var Application = function() {}; _export('Application', Application); };".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["assets/main/index.js".to_string()],
        };
        let ret = g.start_with_bootstrap(&bootstrap);
        assert!(
            ret.is_ok(),
            "mock runtime should accept a valid modern __require bootstrap entry"
        );
    }

    #[cfg(feature = "js-runtime-mock")]
    #[test]
    fn test_start_with_bootstrap_accepts_modern_systemjs_chain_from_application_import() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "modern-systemjs".to_string(),
            main_entry: Some("game.js".to_string()),
            main_entry_source: Some(
                r#"
System.register([], function (_export, _context) {
  var cc, Application;
  return {
    setters: [],
    execute: function () {
      _export("Application", Application = function () {
        this.inited = true;
      });
    }
  };
});

function __initApp() {
  const firstScreen = {
    start: function () { return Promise.resolve(); },
    setProgress: function () { return Promise.resolve(); },
    end: function () { return Promise.resolve(); }
  };

  System.import('./application.js').then(function (_module) {
    const Application = _module.Application;
    const app = new Application();
    return firstScreen.end().then(function () { return app; });
  });
}

__initApp();
"#
                .to_string(),
            ),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["game.js".to_string()],
        };
        let ret = g.start_with_bootstrap(&bootstrap);
        assert!(
            ret.is_ok(),
            "mock runtime should accept System.import/application.js chain"
        );
    }

    #[cfg(feature = "js-runtime-mock")]
    #[test]
    fn test_start_with_bootstrap_accepts_game_demo_chain_source() {
        use std::path::Path;
        use std::process::Command;

        let demo_paths = [
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("game-demo")
                .join("1000006_1.9.4.zip"),
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("game-demo")
                .join("1000007_1.9.4.zip"),
        ];

        let read_entry = |demo_path: &Path, entry: &str| -> Option<String> {
            let output = Command::new("unzip")
                .arg("-p")
                .arg(demo_path.to_string_lossy().as_ref())
                .arg(entry)
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let source = String::from_utf8(output.stdout).ok()?;
            if source.trim().is_empty() {
                return None;
            }
            Some(source)
        };

        for demo_path in demo_paths.iter() {
            if !demo_path.exists() {
                continue;
            }

            let game_entry = read_entry(demo_path, "game.js");
            let application_entry = read_entry(demo_path, "application.js");
            if game_entry.is_none() || application_entry.is_none() {
                continue;
            }

            let source = format!(
                "{}\n\n// [demo game-studio mock pre-check]\n{}",
                application_entry.clone().unwrap_or_default(),
                game_entry.clone().unwrap_or_default()
            );

            let mut g = Game::new();
            g.init();
            let bootstrap = GameBootstrapContract {
                runtime_style: "modern-systemjs".to_string(),
                main_entry: Some("game.js".to_string()),
                main_entry_source: Some(source),
                game_path: demo_path.to_string_lossy().to_string(),
                settings_path: Some("src/settings.json".to_string()),
                settings_source: None,
                entry_candidates: vec!["game.js".to_string()],
            };

            let result = g.start_with_bootstrap(&bootstrap);
            assert!(
                result.is_ok(),
                "mock runtime should accept the real demo game.js/application.js chain for {:?}",
                demo_path.file_name()
            );
        }
    }

    #[cfg(feature = "js-runtime-real")]
    #[test]
    fn test_start_with_bootstrap_can_start_game_demo_chain_source() {
        use std::path::Path;
        use std::process::Command;

        let demo_paths = [
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("game-demo")
                .join("1000006_1.9.4.zip"),
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("game-demo")
                .join("1000007_1.9.4.zip"),
        ];

        let read_entry = |demo_path: &Path, entry: &str| -> Option<String> {
            let output = Command::new("unzip")
                .arg("-p")
                .arg(demo_path.to_string_lossy().as_ref())
                .arg(entry)
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let source = String::from_utf8(output.stdout).ok()?;
            if source.trim().is_empty() {
                return None;
            }
            Some(source)
        };

        for demo_path in demo_paths.iter() {
            if !demo_path.exists() {
                continue;
            }

            let game_entry = read_entry(demo_path, "game.js");
            let application_entry = read_entry(demo_path, "application.js");
            if game_entry.is_none() || application_entry.is_none() {
                continue;
            }

            let source = format!(
                "{}\n\n// [demo game-studio real runtime pre-check]\n{}",
                application_entry.clone().unwrap_or_default(),
                game_entry.clone().unwrap_or_default()
            );

            let mut g = Game::new();
            g.init();
            let bootstrap = GameBootstrapContract {
                runtime_style: "modern-systemjs".to_string(),
                main_entry: Some("game.js".to_string()),
                main_entry_source: Some(source),
                game_path: demo_path.to_string_lossy().to_string(),
                settings_path: Some("src/settings.json".to_string()),
                settings_source: None,
                entry_candidates: vec!["game.js".to_string()],
            };

            let result = g.start_with_bootstrap(&bootstrap);
            assert!(
                result.is_ok(),
                "real runtime should accept the real demo game.js/application.js chain for {:?}",
                demo_path.file_name()
            );
        }
    }

    #[cfg(feature = "js-runtime-real")]
    #[test]
    fn test_start_with_bootstrap_modern_system_import_handlers_are_optional() {
        let mut g = Game::new();
        g.init();
        let source = r#"
System.warmup({
  handlers: {
    "plugin:": function (url) {
      return {
        kind: "plugin",
        name: url
      };
    },
    "project:": function (url) {
      return {
        kind: "project",
        name: url
      };
    }
  },
  defaultHandler: function (url) {
    return {
      kind: "default",
      name: url
    };
  }
});
System.import("plugin:abc").then(function (module) {
  return module.default;
});
System.import("project:def");
"#;

        let bootstrap = GameBootstrapContract {
            runtime_style: "modern-systemjs".to_string(),
            main_entry: Some("game.js".to_string()),
            main_entry_source: Some(source.to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["game.js".to_string()],
        };
        let result = g.start_with_bootstrap(&bootstrap);
        assert!(result.is_ok());
    }

    #[test]
    fn test_start_with_bootstrap_returns_source_missing_reason_for_js_runtime_styles() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "modern-systemjs".to_string(),
            main_entry: Some("application.js".to_string()),
            main_entry_source: None,
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["application.js".to_string()],
        };
        let err = g
            .start_with_bootstrap(&bootstrap)
            .expect_err("missing entry source should fail for js runtime");
        assert_eq!(err.code(), "RUNTIME_UNAVAILABLE");
        assert!(err.message().contains("main entry source is not available"));
    }

    #[test]
    fn test_start_with_bootstrap_validates_legacy_bootstrap_entrypoint() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "legacy-cocos2d-js".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some("console.log('hello');".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };
        let err = g
            .start_with_bootstrap(&bootstrap)
            .expect_err("legacy main entry without window.boot should fail preflight");
        assert_eq!(err.code(), "RUNTIME_UNAVAILABLE");
        assert!(err
            .message()
            .contains("legacy style expects window.boot definition"));
    }

    #[test]
    fn test_start_with_bootstrap_validates_modern_bootstrap_entrypoint() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "modern-systemjs".to_string(),
            main_entry: Some("application.js".to_string()),
            main_entry_source: Some("var x = 1;".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["application.js".to_string()],
        };
        let err = g
            .start_with_bootstrap(&bootstrap)
            .expect_err("modern application.js without System.register should fail preflight");
        assert_eq!(err.code(), "RUNTIME_UNAVAILABLE");
        assert!(err.message().contains("modern style expects"));
    }

    #[cfg(feature = "js-runtime-probe")]
    #[test]
    fn test_js_runtime_probe_fails_on_unbalanced_braces() {
        let mut g = Game::new();
        g.init();
        let bootstrap = GameBootstrapContract {
            runtime_style: "legacy-cocos2d-js".to_string(),
            main_entry: Some("main.js".to_string()),
            main_entry_source: Some("window.boot = function() {".to_string()),
            game_path: "test/path".to_string(),
            settings_path: None,
            settings_source: None,
            entry_candidates: vec!["main.js".to_string()],
        };
        let err = g
            .start_with_bootstrap(&bootstrap)
            .expect_err("unbalanced source should fail js runtime probe");
        assert_eq!(err.code(), "RUNTIME_UNAVAILABLE");
        assert!(err
            .message()
            .contains(REASON_JS_SOURCE_SYNTAX_HEURISTIC_FAILED));
    }

    #[test]
    fn test_game_event_callback() {
        let mut g = Game::new();
        let fired = Arc::new(Mutex::new(false));
        let f = Arc::clone(&fired);
        g.on(GameEvent::GameInited, move |_| {
            *f.lock().unwrap() = true;
        });
        g.init();
        assert!(*fired.lock().unwrap());
    }

    #[test]
    fn test_game_off_event() {
        let mut g = Game::new();
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        g.on(GameEvent::GameInited, move |_| {
            *c.lock().unwrap() += 1;
        });
        g.off(&GameEvent::GameInited);
        g.init();
        assert_eq!(*count.lock().unwrap(), 0);
    }

    #[test]
    fn test_game_set_frame_rate() {
        let mut g = Game::new();
        g.set_frame_rate(30);
        assert_eq!(g.get_frame_rate(), 30);
    }

    #[test]
    fn test_game_restart() {
        let mut g = Game::new();
        g.init();
        g.step(0.016);
        g.restart();
        assert!(g.is_inited());
    }

    #[test]
    fn test_game_config_default() {
        let cfg = GameConfig::default();
        assert_eq!(cfg.frame_rate, 60);
        assert!(!cfg.show_fps);
    }

    #[test]
    fn test_game_with_config() {
        let cfg = GameConfig {
            frame_rate: 30,
            show_fps: true,
            debug_mode: 1,
            render_mode: 2,
        };
        let g = Game::with_config(cfg);
        assert_eq!(g.get_frame_rate(), 30);
        assert!(g.get_config().show_fps);
    }
}
