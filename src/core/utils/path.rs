/****************************************************************************
Rust port of Cocos Creator Path utilities
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

pub fn join(segments: &[&str]) -> String {
    let mut result = String::new();
    for seg in segments {
        if !result.is_empty() {
            result.push('/');
        }
        result.push_str(seg);
        remove_last_slash_mut(&mut result);
    }
    result
}

pub fn extname(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let new_path = strip_query_string(path);
    match new_path.rfind('.') {
        Some(i) => new_path[i..].to_string(),
        None => String::new(),
    }
}

pub fn main_filename(file_name: &str) -> String {
    match file_name.rfind('.') {
        Some(i) => file_name[..i].to_string(),
        None => file_name.to_string(),
    }
}

pub fn basename(path: &str, ext_name: &str) -> String {
    let new_path = strip_query_string(path);
    let cleaned = remove_last_slash(&new_path);
    let index = find_last_sep(&cleaned);
    let base = match index {
        Some(i) => cleaned[i + 1..].to_string(),
        None => cleaned,
    };
    if !ext_name.is_empty() && ext_name.len() < base.len() {
        if base[base.len() - ext_name.len()..].eq_ignore_ascii_case(ext_name) {
            return base[..base.len() - ext_name.len()].to_string();
        }
    }
    base
}

pub fn dirname(path: &str) -> String {
    match find_last_sep(path) {
        Some(i) => remove_last_slash(&path[..i]),
        None => String::new(),
    }
}

pub fn change_extname(path: &str, ext_name: &str) -> String {
    let (new_path, query) = split_query_string(path);
    match new_path.rfind('.') {
        Some(i) => format!("{}{}{}", &new_path[..i], ext_name, query),
        None => format!("{}{}{}", new_path, ext_name, query),
    }
}

pub fn change_basename(path: &str, base_name: &str, is_same_ext: bool) -> String {
    if base_name.starts_with('.') {
        return change_extname(path, base_name);
    }
    let (new_path, query) = split_query_string(path);
    let ext = if is_same_ext { extname(&new_path) } else { String::new() };
    let sep_index = find_last_sep(&new_path);
    let dir_part = match sep_index {
        Some(i) if i > 0 => new_path[..i + 1].to_string(),
        Some(_) => "/".to_string(),
        None => String::new(),
    };
    format!("{}{}{}{}", dir_part, base_name, ext, query)
}

pub fn normalize(url: &str) -> String {
    let mut new_url = url.to_string();
    loop {
        let old_len = new_url.len();
        let index = new_url.find("../").or_else(|| new_url.find("..\\"));
        if let Some(idx) = index {
            if idx > 0 {
                let prev_slash = find_last_sep(&new_url[..idx]);
                let prev_twice_slash = prev_slash.and_then(|ps| {
                    if ps > 0 { find_last_sep(&new_url[..ps]) } else { None }
                });
                match (prev_slash, prev_twice_slash) {
                    (_ps, Some(pts)) => {
                        new_url = format!("{}{}", &new_url[..pts], &new_url[idx + 3..]);
                        new_url.insert(pts, '/');
                    }
                    (_, None) => {
                        new_url = new_url[idx + 3..].to_string();
                    }
                }
            } else {
                new_url = new_url[3..].to_string();
            }
        }
        if new_url.len() == old_len {
            break;
        }
    }
    new_url
}

pub fn strip_sep(path: &str) -> String {
    remove_last_slash(path)
}

pub fn get_separator() -> char {
    '/'
}

fn find_last_sep(s: &str) -> Option<usize> {
    let pos_slash = s.rfind('/');
    let pos_backslash = s.rfind('\\');
    match (pos_slash, pos_backslash) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn remove_last_slash_mut(path: &mut String) {
    if !path.is_empty() {
        let last = path.chars().last().unwrap();
        if last == '/' || last == '\\' {
            path.pop();
        }
    }
}

fn remove_last_slash(path: &str) -> String {
    if !path.is_empty() {
        let last = path.chars().last().unwrap();
        if last == '/' || last == '\\' {
            return path[..path.len() - 1].to_string();
        }
    }
    path.to_string()
}

fn strip_query_string(path: &str) -> String {
    match path.find('?') {
        Some(i) if i > 0 => path[..i].to_string(),
        _ => path.to_string(),
    }
}

fn split_query_string(path: &str) -> (String, String) {
    match path.find('?') {
        Some(i) if i > 0 => (path[..i].to_string(), path[i..].to_string()),
        _ => (path.to_string(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() {
        assert_eq!(join(&["a", "b", "c"]), "a/b/c");
        assert_eq!(join(&["foo", "bar/"]), "foo/bar");
    }

    #[test]
    fn test_extname() {
        assert_eq!(extname("file.png"), ".png");
        assert_eq!(extname("path/to/file.txt?query"), ".txt");
        assert_eq!(extname("noext"), "");
        assert_eq!(extname(""), "");
    }

    #[test]
    fn test_main_filename() {
        assert_eq!(main_filename("sprite.png"), "sprite");
        assert_eq!(main_filename("noext"), "noext");
    }

    #[test]
    fn test_basename() {
        assert_eq!(basename("path/to/file.png", ""), "file.png");
        assert_eq!(basename("path/to/file.png", ".png"), "file");
        assert_eq!(basename("path/to/file.PNG", ".png"), "file");
    }

    #[test]
    fn test_dirname() {
        assert_eq!(dirname("path/to/file.png"), "path/to");
        assert_eq!(dirname("file.png"), "");
    }

    #[test]
    fn test_change_extname() {
        assert_eq!(change_extname("file.png", ".jpg"), "file.jpg");
        assert_eq!(change_extname("file.png?q=1", ".jpg"), "file.jpg?q=1");
        assert_eq!(change_extname("noext", ".png"), "noext.png");
    }

    #[test]
    fn test_change_basename() {
        assert_eq!(change_basename("path/to/old.png", "new", true), "path/to/new.png");
        assert_eq!(change_basename("path/to/old.png", "new", false), "path/to/new");
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("a/b/../c"), "a/c");
        assert_eq!(normalize("a/../b/../c"), "c");
    }

    #[test]
    fn test_strip_sep() {
        assert_eq!(strip_sep("path/to/"), "path/to");
        assert_eq!(strip_sep("path/to"), "path/to");
    }

    #[test]
    fn test_get_separator() {
        assert_eq!(get_separator(), '/');
    }
}
