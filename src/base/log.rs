use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum LogType {
    Kernel,
    Script,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum LogLevel {
    Fatal,
    Err,
    Warn,
    Info,
    LevelDebug,
    Count,
}

static SLOG_LEVEL: Mutex<LogLevel> = Mutex::new(LogLevel::Info);
static SLOG_FILE: Mutex<Option<&'static str>> = Mutex::new(None);

pub struct Log;

impl Log {
    pub fn set_log_level(level: LogLevel) {
        let mut slog_level = SLOG_LEVEL.lock().unwrap();
        *slog_level = level;
    }

    pub fn get_log_level() -> LogLevel {
        let slog_level = SLOG_LEVEL.lock().unwrap();
        *slog_level
    }

    pub fn set_log_file(_filename: &str) {}

    pub fn get_log_file() -> Option<&'static str> {
        let slog_file = SLOG_FILE.lock().unwrap();
        *slog_file
    }

    pub fn close() {
        let mut slog_file = SLOG_FILE.lock().unwrap();
        *slog_file = None;
    }

    pub fn log_message(log_type: LogType, level: LogLevel, message: &str) {
        let prefix = match log_type {
            LogType::Kernel => "[KERNEL]",
            LogType::Script => "[SCRIPT]",
            LogType::Count => "[COUNT]",
        };

        let level_str = match level {
            LogLevel::Fatal => "FATAL",
            LogLevel::Err => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::LevelDebug => "DEBUG",
            LogLevel::Count => "COUNT",
        };

        eprintln!("{} [{}] {}", prefix, level_str, message);
    }

    pub fn log_remote(msg: &str) {
        eprintln!("[REMOTE] {}", msg);
    }
}

#[macro_export]
macro_rules! cc_log {
    ($($arg:expr),+) => {
        if $crate::base::log::Log::get_log_level() <= $crate::base::log::LogLevel::LevelDebug {
            let msg = format!($($arg),+);
            $crate::base::log::Log::log_message($crate::base::log::LogType::Kernel, $crate::base::log::LogLevel::LevelDebug, &msg);
        }
    };
}

#[macro_export]
macro_rules! cc_log_debug {
    ($($arg:expr),+) => {
        if $crate::base::log::Log::get_log_level() <= $crate::base::log::LogLevel::LevelDebug {
            let msg = format!($($arg),+);
            $crate::base::log::Log::log_message($crate::base::log::LogType::Kernel, $crate::base::log::LogLevel::LevelDebug, &msg);
        }
    };
}

#[macro_export]
macro_rules! cc_log_info {
    ($($arg:expr),+) => {
        if $crate::base::log::Log::get_log_level() <= $crate::base::log::LogLevel::Info {
            let msg = format!($($arg),+);
            $crate::base::log::Log::log_message($crate::base::log::LogType::Kernel, $crate::base::log::LogLevel::Info, &msg);
        }
    };
}

#[macro_export]
macro_rules! cc_log_warning {
    ($($arg:expr),+) => {
        if $crate::base::log::Log::get_log_level() <= $crate::base::log::LogLevel::Warn {
            let msg = format!($($arg),+);
            $crate::base::log::Log::log_message($crate::base::log::LogType::Kernel, $crate::base::log::LogLevel::Warn, &msg);
        }
    };
}

#[macro_export]
macro_rules! cc_log_error {
    ($($arg:expr),+) => {
        if $crate::base::log::Log::get_log_level() <= $crate::base::log::LogLevel::Err {
            let msg = format!("[ERROR] file {} line {} ", file!(), line!(), format!($($arg),+));
            $crate::base::log::Log::log_message($crate::base::log::LogType::Kernel, $crate::base::log::LogLevel::Err, &msg);
        }
    };
}

#[macro_export]
macro_rules! cc_log_fatal {
    ($($arg:expr),+) => {
        if $crate::base::log::Log::get_log_level() <= $crate::base::log::LogLevel::Fatal {
            let msg = format!($($arg),+);
            $crate::base::log::Log::log_message($crate::base::log::LogType::Kernel, $crate::base::log::LogLevel::Fatal, &msg);
        }
    };
}

#[macro_export]
macro_rules! cc_log_message {
    ($type:expr, $level:expr, $fmt:expr, $($arg:expr),+) => {
        if $crate::base::log::Log::get_log_level() <= $level {
            let msg = format!($fmt, $($arg),*);
            $crate::base::log::Log::log_message($type, $level, &msg);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level() {
        Log::set_log_level(LogLevel::LevelDebug);
        assert_eq!(Log::get_log_level(), LogLevel::LevelDebug);

        Log::set_log_level(LogLevel::Info);
        assert_eq!(Log::get_log_level(), LogLevel::Info);
    }

    #[test]
    fn test_log_message() {
        Log::set_log_level(LogLevel::Info);

        Log::log_message(LogType::Kernel, LogLevel::Info, "Test message");
    }
}
