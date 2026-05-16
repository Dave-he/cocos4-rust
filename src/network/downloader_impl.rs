use crate::network::http_impl::HttpClient;

pub struct DownloaderImpl {
    client: HttpClient,
    max_concurrent: u32,
    save_path: String,
}

impl DownloaderImpl {
    pub fn new() -> Self {
        Self { client: HttpClient::new(), max_concurrent: 3, save_path: ".".to_string() }
    }

    pub fn set_max_concurrent(&mut self, max: u32) {
        self.max_concurrent = max;
    }

    pub fn set_save_path(&mut self, path: &str) {
        self.save_path = path.to_string();
    }

    pub fn download(&self, url: &str, filename: &str) -> Result<u64, String> {
        let full_path = format!("{}/{}", self.save_path, filename);
        self.client.download_file(url, &full_path)
    }
}

impl Default for DownloaderImpl {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloader_new() {
        let dl = DownloaderImpl::new();
        assert_eq!(dl.max_concurrent, 3);
    }

    #[test]
    fn test_downloader_save_path() {
        let mut dl = DownloaderImpl::new();
        dl.set_save_path("/tmp/downloads");
        assert_eq!(dl.save_path, "/tmp/downloads");
    }

    #[test]
    fn test_downloader_download() {
        let dl = DownloaderImpl::new();
        let result = dl.download("http://test/path", "file.bin");
        assert!(result.is_ok());
    }
}
