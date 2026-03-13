/// Language type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageType {
    English = 0,
    Chinese,
    French,
    Italian,
    German,
    Spanish,
    Dutch,
    Russian,
    Korean,
    Japanese,
    Hungarian,
    Portuguese,
    Arabic,
    Norwegian,
    Polish,
    Turkish,
    Ukrainian,
    Romanian,
    Bulgarian,
    Hindi,
}

/// OS type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OSType {
    Windows,
    Linux,
    Mac,
    AndroidOS,
    IPhone,
    IPad,
    OhOS,
    OpenHarmony,
    Qnx,
}

/// System interface
pub trait ISystem {
    /// Get target system type
    fn get_os_type(&self) -> OSType;

    /// Get device model
    fn get_device_model(&self) -> String;

    /// Get current language
    fn get_current_language(&self) -> LanguageType;

    /// Get current language code (e.g., "en", "zh", "ja")
    fn get_current_language_code(&self) -> String;

    /// Get system version
    fn get_system_version(&self) -> String;

    /// Get current language as string
    fn get_current_language_to_string(&self) -> String;

    /// Copy text to clipboard
    fn copy_text_to_clipboard(&mut self, text: &str);

    /// Open URL in default browser
    /// Returns true if successful, false otherwise
    fn open_url(&mut self, url: &str) -> bool;
}

impl LanguageType {
    pub fn to_code(self) -> &'static str {
        match self {
            LanguageType::English => "en",
            LanguageType::Chinese => "zh",
            LanguageType::French => "fr",
            LanguageType::Italian => "it",
            LanguageType::German => "de",
            LanguageType::Spanish => "es",
            LanguageType::Dutch => "nl",
            LanguageType::Russian => "ru",
            LanguageType::Korean => "ko",
            LanguageType::Japanese => "ja",
            LanguageType::Hungarian => "hu",
            LanguageType::Portuguese => "pt",
            LanguageType::Arabic => "ar",
            LanguageType::Norwegian => "no",
            LanguageType::Polish => "pl",
            LanguageType::Turkish => "tr",
            LanguageType::Ukrainian => "uk",
            LanguageType::Romanian => "ro",
            LanguageType::Bulgarian => "bg",
            LanguageType::Hindi => "hi",
        }
    }
}

#[derive(Debug)]
pub struct DefaultSystem {
    pub os_type: OSType,
    pub language: LanguageType,
    pub version: String,
    pub model: String,
    pub clipboard: String,
}

impl Default for DefaultSystem {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        let os = OSType::Windows;
        #[cfg(target_os = "macos")]
        let os = OSType::Mac;
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let os = OSType::Linux;
        DefaultSystem {
            os_type: os,
            language: LanguageType::English,
            version: "1.0.0".to_string(),
            model: "Desktop".to_string(),
            clipboard: String::new(),
        }
    }
}

impl ISystem for DefaultSystem {
    fn get_os_type(&self) -> OSType { self.os_type }
    fn get_device_model(&self) -> String { self.model.clone() }
    fn get_current_language(&self) -> LanguageType { self.language }
    fn get_current_language_code(&self) -> String { self.language.to_code().to_string() }
    fn get_system_version(&self) -> String { self.version.clone() }
    fn get_current_language_to_string(&self) -> String { format!("{:?}", self.language) }
    fn copy_text_to_clipboard(&mut self, text: &str) { self.clipboard = text.to_string(); }
    fn open_url(&mut self, _url: &str) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_to_code() {
        assert_eq!(LanguageType::English.to_code(), "en");
        assert_eq!(LanguageType::Chinese.to_code(), "zh");
        assert_eq!(LanguageType::Japanese.to_code(), "ja");
    }

    #[test]
    fn test_default_system() {
        let sys = DefaultSystem::default();
        assert_eq!(sys.get_current_language(), LanguageType::English);
        assert_eq!(sys.get_current_language_code(), "en");
    }

    #[test]
    fn test_clipboard() {
        let mut sys = DefaultSystem::default();
        sys.copy_text_to_clipboard("hello world");
        assert_eq!(sys.clipboard, "hello world");
    }

    #[test]
    fn test_open_url_returns_false() {
        let mut sys = DefaultSystem::default();
        assert!(!sys.open_url("https://example.com"));
    }
}
