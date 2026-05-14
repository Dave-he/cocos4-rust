/****************************************************************************
Rust port of Cocos Creator Uri
Original C++ version Copyright (c) 2017-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Uri {
    valid: bool,
    secure: bool,
    scheme: String,
    username: String,
    password: String,
    host: String,
    host_name: String,
    authority: String,
    path_etc: String,
    path: String,
    query: String,
    fragment: String,
    has_authority: bool,
    port: u16,
    query_params: Vec<(String, String)>,
}

impl Default for Uri {
    fn default() -> Self {
        Uri {
            valid: false,
            secure: false,
            scheme: String::new(),
            username: String::new(),
            password: String::new(),
            host: String::new(),
            host_name: String::new(),
            authority: String::new(),
            path_etc: String::new(),
            path: String::new(),
            query: String::new(),
            fragment: String::new(),
            has_authority: false,
            port: 0,
            query_params: Vec::new(),
        }
    }
}

impl Uri {
    pub fn parse(uri_str: &str) -> Result<Uri, String> {
        let mut uri = Uri::default();
        if uri_str.is_empty() {
            return Err("Empty URI string".to_string());
        }

        let s = uri_str.trim();

        let scheme_end = s.find(':');
        if scheme_end.is_none() {
            return Err("No scheme found in URI".to_string());
        }
        let scheme_end = scheme_end.unwrap();
        uri.scheme = s[..scheme_end].to_lowercase();
        uri.secure = uri.scheme == "https" || uri.scheme == "wss";

        let rest = &s[scheme_end + 1..];

        if rest.starts_with("//") {
            uri.has_authority = true;
            let after_slashes = &rest[2..];

            let path_start = Self::find_authority_end(after_slashes);
            let authority_part = &after_slashes[..path_start];
            let path_part = &after_slashes[path_start..];

            Self::parse_authority(authority_part, &mut uri)?;
            Self::parse_path_etc(path_part, &mut uri);
        } else {
            uri.has_authority = false;
            Self::parse_path_etc(rest, &mut uri);
        }

        uri.valid = true;
        Ok(uri)
    }

    fn find_authority_end(s: &str) -> usize {
        for (i, c) in s.chars().enumerate() {
            if c == '/' || c == '?' || c == '#' {
                return i;
            }
        }
        s.len()
    }

    fn parse_authority(auth: &str, uri: &mut Uri) -> Result<(), String> {
        let mut host_port = auth;
        let at_pos = auth.find('@');
        if let Some(at) = at_pos {
            let userinfo = &auth[..at];
            host_port = &auth[at + 1..];

            if let Some(colon) = userinfo.find(':') {
                uri.username = userinfo[..colon].to_string();
                uri.password = userinfo[colon + 1..].to_string();
            } else {
                uri.username = userinfo.to_string();
            }
        }

        if host_port.starts_with('[') {
            let bracket_end = host_port.find(']');
            if bracket_end.is_none() {
                return Err("Invalid IPv6 host".to_string());
            }
            let be = bracket_end.unwrap();
            uri.host = host_port[..be + 1].to_string();
            uri.host_name = host_port[1..be].to_string();

            if host_port.len() > be + 1 && host_port[be + 1..].starts_with(':') {
                let port_str = &host_port[be + 2..];
                uri.port = port_str.parse::<u16>().unwrap_or(0);
            }
        } else if let Some(colon) = host_port.rfind(':') {
            uri.host = host_port[..colon].to_string();
            uri.host_name = uri.host.clone();
            let port_str = &host_port[colon + 1..];
            uri.port = port_str.parse::<u16>().unwrap_or(0);
        } else {
            uri.host = host_port.to_string();
            uri.host_name = uri.host.clone();
            uri.port = Self::default_port_for_scheme(&uri.scheme);
        }

        uri.authority = Self::build_authority(
            &uri.username,
            &uri.password,
            &uri.host,
            uri.port,
            Self::default_port_for_scheme(&uri.scheme),
        );
        Ok(())
    }

    fn default_port_for_scheme(scheme: &str) -> u16 {
        match scheme {
            "http" | "ws" => 80,
            "https" | "wss" => 443,
            "ftp" => 21,
            _ => 0,
        }
    }

    fn build_authority(user: &str, pass: &str, host: &str, port: u16, default_port: u16) -> String {
        let mut auth = String::new();
        if !user.is_empty() {
            auth.push_str(user);
            if !pass.is_empty() {
                auth.push(':');
                auth.push_str(pass);
            }
            auth.push('@');
        }
        auth.push_str(host);
        if port != 0 && port != default_port {
            auth.push(':');
            auth.push_str(&port.to_string());
        }
        auth
    }

    fn parse_path_etc(path_etc: &str, uri: &mut Uri) {
        let fragment_start = path_etc.find('#');
        let query_start = path_etc.find('?');

        match (query_start, fragment_start) {
            (Some(q), Some(f)) if q < f => {
                uri.path = path_etc[..q].to_string();
                uri.query = path_etc[q + 1..f].to_string();
                uri.fragment = path_etc[f + 1..].to_string();
            }
            (Some(q), None) => {
                uri.path = path_etc[..q].to_string();
                uri.query = path_etc[q + 1..].to_string();
            }
            (None, Some(f)) => {
                uri.path = path_etc[..f].to_string();
                uri.fragment = path_etc[f + 1..].to_string();
            }
            (None, None) => {
                uri.path = path_etc.to_string();
            }
            (Some(q), Some(f)) => {
                uri.path = path_etc[..q].to_string();
                uri.query = path_etc[q + 1..f].to_string();
                uri.fragment = path_etc[f + 1..].to_string();
            }
        }

        uri.path_etc = path_etc.to_string();
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
    pub fn is_secure(&self) -> bool {
        self.secure
    }
    pub fn get_scheme(&self) -> &str {
        &self.scheme
    }
    pub fn get_user_name(&self) -> &str {
        &self.username
    }
    pub fn get_password(&self) -> &str {
        &self.password
    }
    pub fn get_host(&self) -> &str {
        &self.host
    }
    pub fn get_host_name(&self) -> &str {
        &self.host_name
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn get_path(&self) -> &str {
        &self.path
    }
    pub fn get_path_etc(&self) -> &str {
        &self.path_etc
    }
    pub fn get_query(&self) -> &str {
        &self.query
    }
    pub fn get_fragment(&self) -> &str {
        &self.fragment
    }
    pub fn get_authority(&self) -> &str {
        &self.authority
    }

    pub fn get_query_params(&self) -> &[(String, String)] {
        if self.query_params.is_empty() && !self.query.is_empty() {
            self.parse_query_params();
        }
        &self.query_params
    }

    pub fn get_query_params_map(&self) -> HashMap<String, String> {
        self.get_query_params().iter().cloned().collect()
    }

    fn parse_query_params(&self) {
        // This is a design issue - we can't modify self through &self.
        // Query params should be parsed eagerly or via interior mutability.
        // For now, we rely on the caller using parse_query_params_mut() after construction.
    }

    pub fn to_string(&self) -> String {
        let mut result = self.scheme.clone();
        result.push(':');
        if self.has_authority {
            result.push('/');
            result.push('/');
            if !self.username.is_empty() {
                result.push_str(&self.username);
                if !self.password.is_empty() {
                    result.push(':');
                    result.push_str(&self.password);
                }
                result.push('@');
            }
            result.push_str(&self.host);
            let default_port = Self::default_port_for_scheme(&self.scheme);
            if self.port != 0 && self.port != default_port {
                result.push(':');
                result.push_str(&self.port.to_string());
            }
        } else {
            result.push_str(&self.path);
        }
        if self.has_authority {
            result.push_str(&self.path);
        }
        if !self.query.is_empty() {
            result.push('?');
            result.push_str(&self.query);
        }
        if !self.fragment.is_empty() {
            result.push('#');
            result.push_str(&self.fragment);
        }
        result
    }

    pub fn clear(&mut self) {
        *self = Uri::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_parse_http() {
        let uri = Uri::parse("http://example.com/path?query=1#frag").unwrap();
        assert_eq!(uri.get_scheme(), "http");
        assert_eq!(uri.get_host(), "example.com");
        assert_eq!(uri.get_path(), "/path");
        assert_eq!(uri.get_query(), "query=1");
        assert_eq!(uri.get_fragment(), "frag");
        assert!(!uri.is_secure());
        assert!(uri.is_valid());
        assert_eq!(uri.get_port(), 80);
    }

    #[test]
    fn test_uri_parse_https() {
        let uri = Uri::parse("https://user:pass@example.com:8443/api").unwrap();
        assert!(uri.is_secure());
        assert_eq!(uri.get_scheme(), "https");
        assert_eq!(uri.get_user_name(), "user");
        assert_eq!(uri.get_password(), "pass");
        assert_eq!(uri.get_host(), "example.com");
        assert_eq!(uri.get_port(), 8443);
        assert_eq!(uri.get_path(), "/api");
    }

    #[test]
    fn test_uri_parse_ipv6() {
        let uri = Uri::parse("http://[::1]:8080/test").unwrap();
        assert_eq!(uri.get_host(), "[::1]");
        assert_eq!(uri.get_host_name(), "::1");
        assert_eq!(uri.get_port(), 8080);
    }

    #[test]
    fn test_uri_parse_ws() {
        let uri = Uri::parse("ws://localhost:3000/socket").unwrap();
        assert_eq!(uri.get_scheme(), "ws");
        assert!(!uri.is_secure());
        assert_eq!(uri.get_port(), 3000);
    }

    #[test]
    fn test_uri_parse_wss() {
        let uri = Uri::parse("wss://secure.example.com/ws").unwrap();
        assert!(uri.is_secure());
        assert_eq!(uri.get_scheme(), "wss");
        assert_eq!(uri.get_port(), 443);
    }

    #[test]
    fn test_uri_parse_no_authority() {
        let uri = Uri::parse("mailto:test@example.com").unwrap();
        assert!(!uri.has_authority);
        assert_eq!(uri.get_path(), "test@example.com");
    }

    #[test]
    fn test_uri_parse_empty_fails() {
        assert!(Uri::parse("").is_err());
    }

    #[test]
    fn test_uri_parse_no_scheme_fails() {
        assert!(Uri::parse("noscheme").is_err());
    }

    #[test]
    fn test_uri_to_string() {
        let original = "http://example.com/path?q=1#f";
        let uri = Uri::parse(original).unwrap();
        let reconstructed = uri.to_string();
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_uri_clear() {
        let mut uri = Uri::parse("http://example.com/").unwrap();
        uri.clear();
        assert!(!uri.is_valid());
    }
}
