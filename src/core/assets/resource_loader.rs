use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadStatus {
    Pending,
    Loading,
    Success,
    Failed,
}

pub struct LoadTask {
    pub url: String,
    pub status: LoadStatus,
    pub progress: f32,
    on_success: Option<Box<dyn Fn(&[u8]) + Send + Sync>>,
    on_error: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_progress: Option<Box<dyn Fn(f32) + Send + Sync>>,
    data: Option<Vec<u8>>,
    error: Option<String>,
}

impl LoadTask {
    fn new(url: &str) -> Self {
        LoadTask {
            url: url.to_string(),
            status: LoadStatus::Pending,
            progress: 0.0,
            on_success: None,
            on_error: None,
            on_progress: None,
            data: None,
            error: None,
        }
    }
}

pub type LoadHandle = u64;

static HANDLE_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_handle() -> LoadHandle {
    HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub type DataProvider = Box<dyn Fn(&str) -> Result<Vec<u8>, String> + Send + Sync>;

pub struct ResourceLoader {
    tasks: HashMap<LoadHandle, LoadTask>,
    providers: Vec<(String, DataProvider)>,
    cache: HashMap<String, Vec<u8>>,
    cache_enabled: bool,
    max_concurrent: usize,
    loading_count: usize,
    total_loaded: u64,
    total_failed: u64,
}

impl ResourceLoader {
    pub fn new() -> Self {
        ResourceLoader {
            tasks: HashMap::new(),
            providers: Vec::new(),
            cache: HashMap::new(),
            cache_enabled: true,
            max_concurrent: 8,
            loading_count: 0,
            total_loaded: 0,
            total_failed: 0,
        }
    }

    pub fn register_provider<F>(&mut self, prefix: &str, provider: F)
    where
        F: Fn(&str) -> Result<Vec<u8>, String> + Send + Sync + 'static,
    {
        self.providers.push((prefix.to_string(), Box::new(provider)));
    }

    pub fn load<S, E>(&mut self, url: &str, on_success: S, on_error: E) -> LoadHandle
    where
        S: Fn(&[u8]) + Send + Sync + 'static,
        E: Fn(&str) + Send + Sync + 'static,
    {
        if self.cache_enabled {
            if let Some(data) = self.cache.get(url) {
                on_success(data);
                let handle = next_handle();
                let mut task = LoadTask::new(url);
                task.status = LoadStatus::Success;
                task.progress = 1.0;
                task.data = Some(data.clone());
                self.tasks.insert(handle, task);
                return handle;
            }
        }

        let handle = next_handle();
        let mut task = LoadTask::new(url);
        task.on_success = Some(Box::new(on_success));
        task.on_error = Some(Box::new(on_error));
        task.status = LoadStatus::Pending;
        self.tasks.insert(handle, task);
        handle
    }

    pub fn load_with_progress<S, E, P>(
        &mut self,
        url: &str,
        on_success: S,
        on_error: E,
        on_progress: P,
    ) -> LoadHandle
    where
        S: Fn(&[u8]) + Send + Sync + 'static,
        E: Fn(&str) + Send + Sync + 'static,
        P: Fn(f32) + Send + Sync + 'static,
    {
        let handle = next_handle();
        let mut task = LoadTask::new(url);
        task.on_success = Some(Box::new(on_success));
        task.on_error = Some(Box::new(on_error));
        task.on_progress = Some(Box::new(on_progress));
        task.status = LoadStatus::Pending;
        self.tasks.insert(handle, task);
        handle
    }

    pub fn pump(&mut self) {
        let handles: Vec<LoadHandle> = self.tasks.keys().copied()
            .filter(|h| {
                self.tasks.get(h).map(|t| t.status == LoadStatus::Pending).unwrap_or(false)
            })
            .take(self.max_concurrent.saturating_sub(self.loading_count))
            .collect();

        for handle in handles {
            if let Some(task) = self.tasks.get_mut(&handle) {
                task.status = LoadStatus::Loading;
                self.loading_count += 1;
            }

            let url = self.tasks.get(&handle).map(|t| t.url.clone()).unwrap_or_default();
            let result = self.run_provider(&url);

            if let Some(task) = self.tasks.get_mut(&handle) {
                self.loading_count = self.loading_count.saturating_sub(1);
                match result {
                    Ok(data) => {
                        task.status = LoadStatus::Success;
                        task.progress = 1.0;
                        if let Some(ref cb) = task.on_progress {
                            cb(1.0);
                        }
                        if let Some(ref cb) = task.on_success {
                            cb(&data);
                        }
                        if self.cache_enabled {
                            self.cache.insert(url.clone(), data.clone());
                        }
                        task.data = Some(data);
                        self.total_loaded += 1;
                    }
                    Err(err) => {
                        task.status = LoadStatus::Failed;
                        task.error = Some(err.clone());
                        if let Some(ref cb) = task.on_error {
                            cb(&err);
                        }
                        self.total_failed += 1;
                    }
                }
            }
        }
    }

    fn run_provider(&self, url: &str) -> Result<Vec<u8>, String> {
        for (prefix, provider) in &self.providers {
            if url.starts_with(prefix.as_str()) || prefix.is_empty() {
                return provider(url);
            }
        }
        Err(format!("No provider for: {}", url))
    }

    pub fn cancel(&mut self, handle: LoadHandle) {
        if let Some(task) = self.tasks.get(&handle) {
            if task.status == LoadStatus::Pending {
                self.tasks.remove(&handle);
            }
        }
    }

    pub fn get_status(&self, handle: LoadHandle) -> Option<LoadStatus> {
        self.tasks.get(&handle).map(|t| t.status)
    }

    pub fn get_progress(&self, handle: LoadHandle) -> f32 {
        self.tasks.get(&handle).map(|t| t.progress).unwrap_or(0.0)
    }

    pub fn get_data(&self, handle: LoadHandle) -> Option<&[u8]> {
        self.tasks.get(&handle)?.data.as_deref()
    }

    pub fn get_error(&self, handle: LoadHandle) -> Option<&str> {
        self.tasks.get(&handle)?.error.as_deref()
    }

    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_cached(&self, url: &str) -> Option<&[u8]> {
        self.cache.get(url).map(|v| v.as_slice())
    }

    pub fn set_max_concurrent(&mut self, n: usize) {
        self.max_concurrent = n;
    }

    pub fn get_total_loaded(&self) -> u64 {
        self.total_loaded
    }

    pub fn get_total_failed(&self) -> u64 {
        self.total_failed
    }

    pub fn get_pending_count(&self) -> usize {
        self.tasks.values().filter(|t| t.status == LoadStatus::Pending).count()
    }
}

impl Default for ResourceLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn make_loader() -> ResourceLoader {
        let mut loader = ResourceLoader::new();
        loader.register_provider("", |url| {
            if url.ends_with(".fail") {
                Err(format!("not found: {}", url))
            } else {
                Ok(format!("data:{}", url).into_bytes())
            }
        });
        loader
    }

    #[test]
    fn test_loader_new() {
        let loader = ResourceLoader::new();
        assert_eq!(loader.get_total_loaded(), 0);
        assert_eq!(loader.get_pending_count(), 0);
    }

    #[test]
    fn test_load_success() {
        let mut loader = make_loader();
        let result = Arc::new(Mutex::new(None::<Vec<u8>>));
        let r = Arc::clone(&result);
        let h = loader.load("res/sprite.png", move |data| {
            *r.lock().unwrap() = Some(data.to_vec());
        }, |_| {});
        loader.pump();
        assert_eq!(loader.get_status(h), Some(LoadStatus::Success));
        assert!(result.lock().unwrap().is_some());
        assert_eq!(loader.get_total_loaded(), 1);
    }

    #[test]
    fn test_load_failure() {
        let mut loader = make_loader();
        let err_msg = Arc::new(Mutex::new(String::new()));
        let e = Arc::clone(&err_msg);
        let h = loader.load("res/missing.fail", |_| {}, move |err| {
            *e.lock().unwrap() = err.to_string();
        });
        loader.pump();
        assert_eq!(loader.get_status(h), Some(LoadStatus::Failed));
        assert!(!err_msg.lock().unwrap().is_empty());
        assert_eq!(loader.get_total_failed(), 1);
    }

    #[test]
    fn test_cache_hit() {
        let mut loader = make_loader();
        let count = Arc::new(Mutex::new(0u32));
        let c1 = Arc::clone(&count);
        let h1 = loader.load("cached/asset.png", move |_| { *c1.lock().unwrap() += 1; }, |_| {});
        loader.pump();
        let c2 = Arc::clone(&count);
        let h2 = loader.load("cached/asset.png", move |_| { *c2.lock().unwrap() += 1; }, |_| {});
        assert_eq!(*count.lock().unwrap(), 2);
        assert_eq!(loader.get_status(h2), Some(LoadStatus::Success));
    }

    #[test]
    fn test_cache_disabled() {
        let mut loader = make_loader();
        loader.set_cache_enabled(false);
        let count = Arc::new(Mutex::new(0u32));
        let c = Arc::clone(&count);
        loader.load("asset.dat", move |_| { *c.lock().unwrap() += 1; }, |_| {});
        loader.pump();
        let c2 = Arc::clone(&count);
        loader.load("asset.dat", move |_| { *c2.lock().unwrap() += 1; }, |_| {});
        loader.pump();
        assert_eq!(*count.lock().unwrap(), 2);
    }

    #[test]
    fn test_clear_cache() {
        let mut loader = make_loader();
        let h = loader.load("file.png", |_| {}, |_| {});
        loader.pump();
        assert!(loader.get_cached("file.png").is_some());
        loader.clear_cache();
        assert!(loader.get_cached("file.png").is_none());
    }

    #[test]
    fn test_cancel_pending() {
        let mut loader = make_loader();
        loader.set_max_concurrent(0);
        let h = loader.load("pending.png", |_| {}, |_| {});
        assert_eq!(loader.get_pending_count(), 1);
        loader.cancel(h);
        assert_eq!(loader.get_pending_count(), 0);
    }

    #[test]
    fn test_get_data() {
        let mut loader = make_loader();
        let h = loader.load("res/data.bin", |_| {}, |_| {});
        loader.pump();
        assert!(loader.get_data(h).is_some());
    }

    #[test]
    fn test_progress_callback() {
        let mut loader = make_loader();
        let progress = Arc::new(Mutex::new(0.0f32));
        let p = Arc::clone(&progress);
        let h = loader.load_with_progress(
            "file.png",
            |_| {},
            |_| {},
            move |v| { *p.lock().unwrap() = v; },
        );
        loader.pump();
        assert!((loader.get_progress(h) - 1.0).abs() < 1e-6);
        assert!(*progress.lock().unwrap() >= 1.0);
    }

    #[test]
    fn test_multiple_providers() {
        let mut loader = ResourceLoader::new();
        loader.register_provider("http://", |url| Ok(format!("remote:{}", url).into_bytes()));
        loader.register_provider("file://", |url| Ok(format!("local:{}", url).into_bytes()));
        let result = Arc::new(Mutex::new(String::new()));
        let r = Arc::clone(&result);
        let h = loader.load("http://example.com/img.png", move |data| {
            *r.lock().unwrap() = String::from_utf8_lossy(data).to_string();
        }, |_| {});
        loader.pump();
        assert!(result.lock().unwrap().starts_with("remote:"));
    }

    #[test]
    fn test_unique_handles() {
        let mut loader = make_loader();
        let h1 = loader.load("a.png", |_| {}, |_| {});
        let h2 = loader.load("b.png", |_| {}, |_| {});
        assert_ne!(h1, h2);
    }
}
