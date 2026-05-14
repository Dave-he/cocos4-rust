/****************************************************************************
Rust port of Cocos Creator WebSocket / Downloader / SocketIO
Original C++ version Copyright (c) 2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketErrorCode {
    TimeOut,
    ConnectionFailure,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketState {
    Connecting,
    Open,
    Closing,
    Closed,
}

#[derive(Debug)]
pub struct WebSocketData {
    bytes: Vec<u8>,
    len: u32,
    issued: u32,
    is_binary: bool,
}

impl WebSocketData {
    pub fn new() -> Self {
        WebSocketData {
            bytes: Vec::new(),
            len: 0,
            issued: 0,
            is_binary: false,
        }
    }

    pub fn from_bytes(data: &[u8], is_binary: bool) -> Self {
        WebSocketData {
            bytes: data.to_vec(),
            len: data.len() as u32,
            issued: 0,
            is_binary,
        }
    }

    pub fn get_remain(&self) -> u32 {
        self.len.saturating_sub(self.issued)
    }

    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn get_len(&self) -> u32 {
        self.len
    }
    pub fn get_issued(&self) -> u32 {
        self.issued
    }
    pub fn is_binary(&self) -> bool {
        self.is_binary
    }
}

pub type WebSocketOnOpen = Arc<dyn Fn(&WebSocket) + Send + Sync>;
pub type WebSocketOnMessage = Arc<dyn Fn(&WebSocket, &WebSocketData) + Send + Sync>;
pub type WebSocketOnClose = Arc<dyn Fn(&WebSocket, u16, &str, bool) + Send + Sync>;
pub type WebSocketOnError = Arc<dyn Fn(&WebSocket, WebSocketErrorCode) + Send + Sync>;

pub struct WebSocket {
    state: Arc<Mutex<WebSocketState>>,
    url: String,
    protocol: String,
    extensions: String,
    buffered_amount: usize,
    on_open: Option<WebSocketOnOpen>,
    on_message: Option<WebSocketOnMessage>,
    on_close: Option<WebSocketOnClose>,
    on_error: Option<WebSocketOnError>,
    ref_count: RefCountedImpl,
}

impl std::fmt::Debug for WebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocket")
            .field("state", &*self.state.lock().unwrap())
            .field("url", &self.url)
            .field("protocol", &self.protocol)
            .field("buffered_amount", &self.buffered_amount)
            .finish()
    }
}

impl WebSocket {
    pub fn new() -> Self {
        WebSocket {
            state: Arc::new(Mutex::new(WebSocketState::Closed)),
            url: String::new(),
            protocol: String::new(),
            extensions: String::new(),
            buffered_amount: 0,
            on_open: None,
            on_message: None,
            on_close: None,
            on_error: None,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn init(
        &mut self,
        url: &str,
        _protocols: Option<&[String]>,
        _ca_file_path: Option<&str>,
    ) -> bool {
        self.url = url.to_string();
        *self.state.lock().unwrap() = WebSocketState::Connecting;
        true
    }

    pub fn send_text(&self, _message: &str) {
        // Placeholder - requires async WebSocket library
    }

    pub fn send_binary(&self, _data: &[u8]) {
        // Placeholder - requires async WebSocket library
    }

    pub fn close(&self) {
        *self.state.lock().unwrap() = WebSocketState::Closing;
    }

    pub fn close_async(&self) {
        *self.state.lock().unwrap() = WebSocketState::Closing;
    }

    pub fn close_async_with_code(&self, _code: i32, _reason: &str) {
        *self.state.lock().unwrap() = WebSocketState::Closing;
    }

    pub fn get_ready_state(&self) -> WebSocketState {
        *self.state.lock().unwrap()
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }
    pub fn get_buffered_amount(&self) -> usize {
        self.buffered_amount
    }
    pub fn get_extensions(&self) -> &str {
        &self.extensions
    }
    pub fn get_protocol(&self) -> &str {
        &self.protocol
    }

    pub fn set_on_open(&mut self, cb: WebSocketOnOpen) {
        self.on_open = Some(cb);
    }
    pub fn set_on_message(&mut self, cb: WebSocketOnMessage) {
        self.on_message = Some(cb);
    }
    pub fn set_on_close(&mut self, cb: WebSocketOnClose) {
        self.on_close = Some(cb);
    }
    pub fn set_on_error(&mut self, cb: WebSocketOnError) {
        self.on_error = Some(cb);
    }
}

impl RefCounted for WebSocket {
    fn add_ref(&self) {
        self.ref_count.add_ref();
    }
    fn release(&self) {
        self.ref_count.release();
    }
    fn get_ref_count(&self) -> u32 {
        self.ref_count.get_ref_count()
    }
    fn is_last_reference(&self) -> bool {
        self.ref_count.is_last_reference()
    }
}

impl Default for WebSocket {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DownloadTask {
    identifier: String,
    request_url: String,
    storage_path: String,
    header: std::collections::HashMap<String, String>,
}

impl DownloadTask {
    pub const ERROR_NO_ERROR: i32 = 0;
    pub const ERROR_INVALID_PARAMS: i32 = -1;
    pub const ERROR_FILE_OP_FAILED: i32 = -2;
    pub const ERROR_IMPL_INTERNAL: i32 = -3;
    pub const ERROR_ABORT: i32 = -4;

    pub fn new() -> Self {
        DownloadTask {
            identifier: String::new(),
            request_url: String::new(),
            storage_path: String::new(),
            header: std::collections::HashMap::new(),
        }
    }

    pub fn set_identifier(&mut self, id: &str) {
        self.identifier = id.to_string();
    }
    pub fn set_request_url(&mut self, url: &str) {
        self.request_url = url.to_string();
    }
    pub fn set_storage_path(&mut self, path: &str) {
        self.storage_path = path.to_string();
    }
    pub fn set_header(&mut self, key: &str, value: &str) {
        self.header.insert(key.to_string(), value.to_string());
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }
    pub fn get_request_url(&self) -> &str {
        &self.request_url
    }
    pub fn get_storage_path(&self) -> &str {
        &self.storage_path
    }
    pub fn get_header(&self) -> &std::collections::HashMap<String, String> {
        &self.header
    }
}

impl Default for DownloadTask {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DownloaderHints {
    #[allow(dead_code)]
    count_of_max_processing_tasks: u32,
    #[allow(dead_code)]
    timeout_in_seconds: u32,
    #[allow(dead_code)]
    temp_file_name_suffix: String,
}

impl Default for DownloaderHints {
    fn default() -> Self {
        DownloaderHints {
            count_of_max_processing_tasks: 6,
            timeout_in_seconds: 45,
            temp_file_name_suffix: ".tmp".to_string(),
        }
    }
}

pub type DownloaderOnDataTaskSuccess = Arc<dyn Fn(&DownloadTask, &[u8]) + Send + Sync>;
pub type DownloaderOnFileTaskSuccess = Arc<dyn Fn(&DownloadTask) + Send + Sync>;
pub type DownloaderOnTaskProgress = Arc<dyn Fn(&DownloadTask, u32, u32, u32) + Send + Sync>;
pub type DownloaderOnTaskError = Arc<dyn Fn(&DownloadTask, i32, i32, &str) + Send + Sync>;

#[derive(Default)]
pub struct Downloader {
    hints: DownloaderHints,
    on_data_task_success: Option<DownloaderOnDataTaskSuccess>,
    on_file_task_success: Option<DownloaderOnFileTaskSuccess>,
    on_task_progress: Option<DownloaderOnTaskProgress>,
    on_task_error: Option<DownloaderOnTaskError>,
}

impl std::fmt::Debug for Downloader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Downloader")
            .field("hints", &self.hints)
            .field("has_data_callback", &self.on_data_task_success.is_some())
            .field("has_file_callback", &self.on_file_task_success.is_some())
            .field("has_progress_callback", &self.on_task_progress.is_some())
            .field("has_error_callback", &self.on_task_error.is_some())
            .finish()
    }
}

impl Downloader {
    pub fn new() -> Self {
        Downloader::default()
    }
    pub fn with_hints(hints: DownloaderHints) -> Self {
        Downloader {
            hints,
            ..Default::default()
        }
    }

    pub fn create_data_task(&mut self, src_url: &str, identifier: &str) -> DownloadTask {
        let mut task = DownloadTask::new();
        task.set_request_url(src_url);
        task.set_identifier(identifier);
        task
    }

    pub fn create_download_task(
        &mut self,
        src_url: &str,
        storage_path: &str,
        identifier: &str,
    ) -> DownloadTask {
        let mut task = DownloadTask::new();
        task.set_request_url(src_url);
        task.set_storage_path(storage_path);
        task.set_identifier(identifier);
        task
    }

    pub fn abort(&self, _task: &DownloadTask) {
        // Placeholder - requires download implementation
    }

    pub fn set_on_data_task_success(&mut self, cb: DownloaderOnDataTaskSuccess) {
        self.on_data_task_success = Some(cb);
    }
    pub fn set_on_file_task_success(&mut self, cb: DownloaderOnFileTaskSuccess) {
        self.on_file_task_success = Some(cb);
    }
    pub fn set_on_task_progress(&mut self, cb: DownloaderOnTaskProgress) {
        self.on_task_progress = Some(cb);
    }
    pub fn set_on_task_error(&mut self, cb: DownloaderOnTaskError) {
        self.on_task_error = Some(cb);
    }
}

#[derive(Debug, Clone)]
pub struct SIOClient {
    #[allow(dead_code)]
    url: String,
    tag: String,
    instance_id: u32,
    connected: bool,
}

impl SIOClient {
    pub fn new(url: &str) -> Self {
        SIOClient {
            url: url.to_string(),
            tag: String::new(),
            instance_id: 0,
            connected: false,
        }
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
    }
    pub fn send(&self, _message: &str) {}
    pub fn emit(&self, _event: &str, _args: &str) {}
    pub fn on(&mut self, _event_name: &str, _handler: Box<dyn Fn(&str) + Send + Sync>) {}
    pub fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string();
    }
    pub fn get_tag(&self) -> &str {
        &self.tag
    }
    pub fn get_instance_id(&self) -> u32 {
        self.instance_id
    }
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_new() {
        let ws = WebSocket::new();
        assert_eq!(ws.get_ready_state(), WebSocketState::Closed);
        assert_eq!(ws.get_url(), "");
    }

    #[test]
    fn test_websocket_init() {
        let mut ws = WebSocket::new();
        ws.init("ws://localhost:8080", None, None);
        assert_eq!(ws.get_url(), "ws://localhost:8080");
        assert_eq!(ws.get_ready_state(), WebSocketState::Connecting);
    }

    #[test]
    fn test_websocket_close() {
        let ws = WebSocket::new();
        ws.close();
    }

    #[test]
    fn test_websocket_state_transitions() {
        let ws = WebSocket::new();
        assert_eq!(ws.get_ready_state(), WebSocketState::Closed);

        let state = ws.state.clone();
        *state.lock().unwrap() = WebSocketState::Open;
        assert_eq!(ws.get_ready_state(), WebSocketState::Open);

        ws.close_async();
        assert_eq!(ws.get_ready_state(), WebSocketState::Closing);
    }

    #[test]
    fn test_websocket_data() {
        let data = WebSocketData::from_bytes(b"hello", true);
        assert_eq!(data.get_len(), 5);
        assert!(data.is_binary());
        assert_eq!(data.get_remain(), 5);
    }

    #[test]
    fn test_websocket_data_remain() {
        let mut data = WebSocketData::from_bytes(b"hello", false);
        data.issued = 3;
        assert_eq!(data.get_remain(), 2);
    }

    #[test]
    fn test_websocket_callbacks() {
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        let mut ws = WebSocket::new();
        ws.set_on_open(Arc::new(move |_| {
            *c.lock().unwrap() += 1;
        }));
        assert!(ws.on_open.is_some());
    }

    #[test]
    fn test_download_task() {
        let mut task = DownloadTask::new();
        task.set_request_url("https://example.com/file.zip");
        task.set_storage_path("/tmp/file.zip");
        task.set_identifier("download-1");
        assert_eq!(task.get_request_url(), "https://example.com/file.zip");
        assert_eq!(task.get_storage_path(), "/tmp/file.zip");
        assert_eq!(task.get_identifier(), "download-1");
    }

    #[test]
    fn test_download_task_error_codes() {
        assert_eq!(DownloadTask::ERROR_NO_ERROR, 0);
        assert_eq!(DownloadTask::ERROR_INVALID_PARAMS, -1);
        assert_eq!(DownloadTask::ERROR_FILE_OP_FAILED, -2);
        assert_eq!(DownloadTask::ERROR_IMPL_INTERNAL, -3);
        assert_eq!(DownloadTask::ERROR_ABORT, -4);
    }

    #[test]
    fn test_downloader_hints_default() {
        let hints = DownloaderHints::default();
        assert_eq!(hints.count_of_max_processing_tasks, 6);
        assert_eq!(hints.timeout_in_seconds, 45);
        assert_eq!(hints.temp_file_name_suffix, ".tmp");
    }

    #[test]
    fn test_downloader_create_task() {
        let mut dl = Downloader::new();
        let task = dl.create_data_task("https://example.com/api", "task-1");
        assert_eq!(task.get_request_url(), "https://example.com/api");
        assert_eq!(task.get_identifier(), "task-1");
    }

    #[test]
    fn test_sio_client() {
        let mut client = SIOClient::new("http://localhost:3000");
        assert!(!client.is_connected());
        client.set_tag("game-client");
        assert_eq!(client.get_tag(), "game-client");
    }
}
