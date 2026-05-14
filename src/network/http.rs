/****************************************************************************
Rust port of Cocos Creator HttpRequest / HttpResponse / HttpClient
Original C++ version Copyright (c) 2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use crate::base::{RefCounted, RefCountedImpl};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpRequestType {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
    Unknown,
}

pub type HttpResponseCallback = Arc<dyn Fn(&HttpClient, &HttpResponse) + Send + Sync>;

pub struct HttpRequest {
    request_type: HttpRequestType,
    url: String,
    request_data: Vec<u8>,
    tag: String,
    headers: Vec<String>,
    timeout: f32,
    callback: Option<HttpResponseCallback>,
    ref_count: RefCountedImpl,
}

impl std::fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpRequest")
            .field("request_type", &self.request_type)
            .field("url", &self.url)
            .field("tag", &self.tag)
            .field("timeout", &self.timeout)
            .field("has_callback", &self.callback.is_some())
            .finish()
    }
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            request_type: HttpRequestType::Unknown,
            url: String::new(),
            request_data: Vec::new(),
            tag: String::new(),
            headers: Vec::new(),
            timeout: 10.0,
            callback: None,
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn set_request_type(&mut self, type_: HttpRequestType) {
        self.request_type = type_;
    }
    pub fn get_request_type(&self) -> HttpRequestType {
        self.request_type
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }
    pub fn get_url(&self) -> &str {
        &self.url
    }

    pub fn set_request_data(&mut self, data: &[u8]) {
        self.request_data = data.to_vec();
    }
    pub fn get_request_data(&self) -> &[u8] {
        &self.request_data
    }
    pub fn get_request_data_size(&self) -> usize {
        self.request_data.len()
    }

    pub fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string();
    }
    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    pub fn set_headers(&mut self, headers: Vec<String>) {
        self.headers = headers;
    }
    pub fn get_headers(&self) -> &[String] {
        &self.headers
    }

    pub fn set_timeout(&mut self, timeout: f32) {
        self.timeout = timeout;
    }
    pub fn get_timeout(&self) -> f32 {
        self.timeout
    }

    pub fn set_response_callback(&mut self, cb: HttpResponseCallback) {
        self.callback = Some(cb);
    }
    pub fn get_response_callback(&self) -> Option<&HttpResponseCallback> {
        self.callback.as_ref()
    }
}

impl RefCounted for HttpRequest {
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

impl Default for HttpRequest {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HttpResponse {
    request: Arc<Mutex<HttpRequest>>,
    succeed: bool,
    response_data: Vec<u8>,
    response_header: Vec<u8>,
    response_code: i64,
    error_buffer: String,
    response_data_string: String,
    ref_count: RefCountedImpl,
}

impl std::fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpResponse")
            .field("succeed", &self.succeed)
            .field("response_code", &self.response_code)
            .field("error_buffer", &self.error_buffer)
            .finish()
    }
}

impl HttpResponse {
    pub fn new(request: HttpRequest) -> Self {
        HttpResponse {
            request: Arc::new(Mutex::new(request)),
            succeed: false,
            response_data: Vec::new(),
            response_header: Vec::new(),
            response_code: 0,
            error_buffer: String::new(),
            response_data_string: String::new(),
            ref_count: RefCountedImpl::new(),
        }
    }

    pub fn get_http_request(&self) -> std::sync::MutexGuard<'_, HttpRequest> {
        self.request.lock().unwrap()
    }

    pub fn is_succeed(&self) -> bool {
        self.succeed
    }
    pub fn get_response_data(&self) -> &[u8] {
        &self.response_data
    }
    pub fn get_response_header(&self) -> &[u8] {
        &self.response_header
    }
    pub fn get_response_code(&self) -> i64 {
        self.response_code
    }
    pub fn get_error_buffer(&self) -> &str {
        &self.error_buffer
    }
    pub fn get_response_data_string(&self) -> &str {
        &self.response_data_string
    }

    pub fn set_succeed(&mut self, val: bool) {
        self.succeed = val;
    }
    pub fn set_response_data(&mut self, data: Vec<u8>) {
        self.response_data = data;
    }
    pub fn set_response_header(&mut self, data: Vec<u8>) {
        self.response_header = data;
    }
    pub fn set_response_code(&mut self, code: i64) {
        self.response_code = code;
    }
    pub fn set_error_buffer(&mut self, msg: &str) {
        self.error_buffer = msg.to_string();
    }
    pub fn set_response_data_string(&mut self, s: &str) {
        self.response_data_string = s.to_string();
    }
}

impl RefCounted for HttpResponse {
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

#[derive(Debug)]
pub struct CookiesInfo {
    domain: String,
    tail_match: bool,
    path: String,
    secure: bool,
    name: String,
    value: String,
    expires: String,
}

impl Default for CookiesInfo {
    fn default() -> CookiesInfo {
        CookiesInfo {
            domain: String::new(),
            tail_match: false,
            path: String::new(),
            secure: false,
            name: String::new(),
            value: String::new(),
            expires: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct HttpCookie {
    cookies: Vec<CookiesInfo>,
    cookie_file: String,
}

impl HttpCookie {
    pub fn new() -> Self {
        HttpCookie {
            cookies: Vec::new(),
            cookie_file: String::new(),
        }
    }

    pub fn set_cookie_file_name(&mut self, name: &str) {
        self.cookie_file = name.to_string();
    }
    pub fn get_cookies(&self) -> &[CookiesInfo] {
        &self.cookies
    }
    pub fn get_match_cookie(&self, url: &str) -> Option<&CookiesInfo> {
        self.cookies.iter().find(|c| url.contains(&c.domain))
    }
    pub fn update_or_add_cookie(&mut self, cookie: CookiesInfo) {
        if let Some(existing) = self
            .cookies
            .iter_mut()
            .find(|c| c.name == cookie.name && c.domain == cookie.domain)
        {
            *existing = cookie;
        } else {
            self.cookies.push(cookie);
        }
    }
}

impl Default for HttpCookie {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct HttpClient {
    cookie: HttpCookie,
    ssl_verification: String,
    timeout_for_connect: u32,
    timeout_for_read: u32,
    thread_count: u32,
}

impl HttpClient {
    pub fn new() -> Self {
        HttpClient {
            cookie: HttpCookie::new(),
            ssl_verification: String::new(),
            timeout_for_connect: 60,
            timeout_for_read: 60,
            thread_count: 4,
        }
    }

    pub fn enable_cookies(&mut self, file: &str) {
        self.cookie.set_cookie_file_name(file);
    }
    pub fn get_cookie_filename(&self) -> &str {
        &self.cookie.cookie_file
    }

    pub fn set_ssl_verification(&mut self, ca_file: &str) {
        self.ssl_verification = ca_file.to_string();
    }
    pub fn get_ssl_verification(&self) -> &str {
        &self.ssl_verification
    }

    pub fn set_timeout_for_connect(&mut self, timeout: u32) {
        self.timeout_for_connect = timeout;
    }
    pub fn get_timeout_for_connect(&self) -> u32 {
        self.timeout_for_connect
    }

    pub fn set_timeout_for_read(&mut self, timeout: u32) {
        self.timeout_for_read = timeout;
    }
    pub fn get_timeout_for_read(&self) -> u32 {
        self.timeout_for_read
    }

    pub fn set_thread_count(&mut self, count: u32) {
        self.thread_count = count;
    }
    pub fn get_thread_count(&self) -> u32 {
        self.thread_count
    }

    pub fn get_cookie(&self) -> &HttpCookie {
        &self.cookie
    }

    pub fn send(&self, _request: &HttpRequest) {
        // Async send placeholder - requires async runtime integration
    }

    pub fn send_immediate(&self, _request: &HttpRequest) {
        // Blocking send placeholder - requires HTTP client library integration
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_new() {
        let req = HttpRequest::new();
        assert_eq!(req.get_request_type(), HttpRequestType::Unknown);
        assert_eq!(req.get_url(), "");
        assert_eq!(req.get_request_data_size(), 0);
        assert_eq!(req.get_timeout(), 10.0);
    }

    #[test]
    fn test_http_request_set_fields() {
        let mut req = HttpRequest::new();
        req.set_request_type(HttpRequestType::Get);
        req.set_url("https://example.com/api");
        req.set_request_data(b"hello world");
        req.set_tag("test-tag");
        req.set_headers(vec!["Content-Type: application/json".to_string()]);
        req.set_timeout(5.0);

        assert_eq!(req.get_request_type(), HttpRequestType::Get);
        assert_eq!(req.get_url(), "https://example.com/api");
        assert_eq!(req.get_request_data(), b"hello world");
        assert_eq!(req.get_tag(), "test-tag");
        assert_eq!(req.get_headers().len(), 1);
        assert_eq!(req.get_timeout(), 5.0);
    }

    #[test]
    fn test_http_request_ref_count() {
        let req = HttpRequest::new();
        assert_eq!(req.get_ref_count(), 1);
        req.add_ref();
        assert_eq!(req.get_ref_count(), 2);
        req.release();
        assert_eq!(req.get_ref_count(), 1);
    }

    #[test]
    fn test_http_response_new() {
        let req = HttpRequest::new();
        let resp = HttpResponse::new(req);
        assert!(!resp.is_succeed());
        assert_eq!(resp.get_response_code(), 0);
        assert_eq!(resp.get_error_buffer(), "");
    }

    #[test]
    fn test_http_response_set_fields() {
        let req = HttpRequest::new();
        let mut resp = HttpResponse::new(req);
        resp.set_succeed(true);
        resp.set_response_code(200);
        resp.set_response_data(b"OK".to_vec());
        resp.set_error_buffer("");

        assert!(resp.is_succeed());
        assert_eq!(resp.get_response_code(), 200);
        assert_eq!(resp.get_response_data(), b"OK");
    }

    #[test]
    fn test_http_cookie() {
        let mut cookie = HttpCookie::new();
        cookie.update_or_add_cookie(CookiesInfo {
            domain: "example.com".to_string(),
            name: "session".to_string(),
            value: "abc123".to_string(),
            ..Default::default()
        });
        assert_eq!(cookie.get_cookies().len(), 1);

        let matched = cookie.get_match_cookie("https://example.com/api");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().value, "abc123");
    }

    #[test]
    fn test_http_client_new() {
        let client = HttpClient::new();
        assert_eq!(client.get_timeout_for_connect(), 60);
        assert_eq!(client.get_timeout_for_read(), 60);
        assert_eq!(client.get_thread_count(), 4);
    }

    #[test]
    fn test_http_client_config() {
        let mut client = HttpClient::new();
        client.enable_cookies("/tmp/cookies.txt");
        client.set_ssl_verification("/etc/ssl/certs/ca-bundle.crt");
        client.set_timeout_for_connect(30);
        client.set_timeout_for_read(30);
        client.set_thread_count(8);

        assert_eq!(client.get_cookie_filename(), "/tmp/cookies.txt");
        assert_eq!(
            client.get_ssl_verification(),
            "/etc/ssl/certs/ca-bundle.crt"
        );
        assert_eq!(client.get_timeout_for_connect(), 30);
        assert_eq!(client.get_timeout_for_read(), 30);
        assert_eq!(client.get_thread_count(), 8);
    }

    #[test]
    fn test_http_request_type_values() {
        assert_eq!(HttpRequestType::Get, HttpRequestType::Get);
        assert_ne!(HttpRequestType::Get, HttpRequestType::Post);
        let all_types = [
            HttpRequestType::Get,
            HttpRequestType::Post,
            HttpRequestType::Put,
            HttpRequestType::Delete,
            HttpRequestType::Head,
            HttpRequestType::Patch,
            HttpRequestType::Unknown,
        ];
        assert_eq!(all_types.len(), 7);
    }
}
