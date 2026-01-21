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
