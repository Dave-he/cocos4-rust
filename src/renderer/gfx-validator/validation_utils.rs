/****************************************************************************
Rust port of Cocos Creator GFX Validation Utilities
Provides common validation helper types and functions.
****************************************************************************/

use std::collections::HashMap;

#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
    pub kind: ValidationErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorKind {
    Lifecycle,
    Format,
    Buffer,
    Texture,
    CommandBufferState,
    Descriptor,
    ResourceLeak,
    General,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.kind, self.message)
    }
}

impl std::fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationErrorKind::Lifecycle => write!(f, "Lifecycle"),
            ValidationErrorKind::Format => write!(f, "Format"),
            ValidationErrorKind::Buffer => write!(f, "Buffer"),
            ValidationErrorKind::Texture => write!(f, "Texture"),
            ValidationErrorKind::CommandBufferState => write!(f, "CmdBufferState"),
            ValidationErrorKind::Descriptor => write!(f, "Descriptor"),
            ValidationErrorKind::ResourceLeak => write!(f, "ResourceLeak"),
            ValidationErrorKind::General => write!(f, "General"),
        }
    }
}

pub struct ValidationLog {
    errors: Vec<ValidationError>,
    warnings: Vec<String>,
    enabled: bool,
}

impl ValidationLog {
    pub fn new() -> Self {
        ValidationLog {
            errors: Vec::new(),
            warnings: Vec::new(),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        ValidationLog { errors: Vec::new(), warnings: Vec::new(), enabled: false }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn error(&mut self, kind: ValidationErrorKind, message: &str) {
        if self.enabled {
            self.errors.push(ValidationError {
                message: message.to_string(),
                kind,
            });
            eprintln!("[GFX Validator ERROR] [{}] {}", kind, message);
        }
    }

    pub fn warn(&mut self, message: &str) {
        if self.enabled {
            self.warnings.push(message.to_string());
            eprintln!("[GFX Validator WARN] {}", message);
        }
    }

    pub fn assert_inited(&mut self, inited: bool, resource_type: &str, id: u32) -> bool {
        if !inited {
            self.error(
                ValidationErrorKind::Lifecycle,
                &format!("{} (id={}) used before initialization", resource_type, id),
            );
            false
        } else {
            true
        }
    }

    pub fn assert_destroyed(&mut self, inited: bool, resource_type: &str, id: u32) -> bool {
        if inited {
            self.error(
                ValidationErrorKind::Lifecycle,
                &format!("{} (id={}) destroyed while still initialized", resource_type, id),
            );
            false
        } else {
            true
        }
    }

    pub fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }

    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
}

impl Default for ValidationLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandBufferStateTracker {
    inside_render_pass: bool,
    inited: bool,
    commands_flushed: bool,
    is_primary: bool,
    bound_pipeline_id: Option<u32>,
    bound_descriptor_sets: HashMap<u32, u32>,
    bound_input_assembler_id: Option<u32>,
}

impl CommandBufferStateTracker {
    pub fn new(is_primary: bool) -> Self {
        CommandBufferStateTracker {
            is_primary,
            ..Default::default()
        }
    }

    pub fn on_begin(&mut self) {
        self.inited = true;
        self.inside_render_pass = false;
        self.commands_flushed = false;
        self.bound_pipeline_id = None;
        self.bound_descriptor_sets.clear();
        self.bound_input_assembler_id = None;
    }

    pub fn on_end(&mut self) {
        self.inited = true;
        self.inside_render_pass = false;
    }

    pub fn on_begin_render_pass(&mut self) {
        self.inside_render_pass = true;
    }

    pub fn on_end_render_pass(&mut self) {
        self.inside_render_pass = false;
    }

    pub fn on_bind_pipeline(&mut self, pipeline_id: u32) {
        self.bound_pipeline_id = Some(pipeline_id);
    }

    pub fn on_bind_descriptor_set(&mut self, set: u32, descriptor_set_id: u32) {
        self.bound_descriptor_sets.insert(set, descriptor_set_id);
    }

    pub fn on_bind_input_assembler(&mut self, ia_id: u32) {
        self.bound_input_assembler_id = Some(ia_id);
    }

    pub fn on_flush(&mut self) {
        self.commands_flushed = true;
    }

    pub fn is_inside_render_pass(&self) -> bool {
        self.inside_render_pass
    }

    pub fn is_inited(&self) -> bool {
        self.inited
    }

    pub fn is_commands_flushed(&self) -> bool {
        self.commands_flushed
    }

    pub fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub fn get_bound_pipeline(&self) -> Option<u32> {
        self.bound_pipeline_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError {
            message: "buffer used before init".to_string(),
            kind: ValidationErrorKind::Lifecycle,
        };
        assert_eq!(err.to_string(), "[Lifecycle] buffer used before init");
    }

    #[test]
    fn test_validation_log_error() {
        let mut log = ValidationLog::new();
        log.error(ValidationErrorKind::Buffer, "buffer size is zero");
        assert!(log.has_errors());
        assert_eq!(log.get_errors().len(), 1);
    }

    #[test]
    fn test_validation_log_disabled() {
        let mut log = ValidationLog::disabled();
        log.error(ValidationErrorKind::General, "should be ignored");
        assert!(!log.has_errors());
    }

    #[test]
    fn test_validation_log_assert_inited() {
        let mut log = ValidationLog::new();
        assert!(log.assert_inited(true, "Buffer", 1));
        assert!(!log.assert_inited(false, "Buffer", 2));
        assert!(log.has_errors());
    }

    #[test]
    fn test_cmd_buffer_state_tracker() {
        let mut tracker = CommandBufferStateTracker::new(true);
        tracker.on_begin();
        assert!(tracker.is_inited());
        assert!(!tracker.is_inside_render_pass());
        tracker.on_begin_render_pass();
        assert!(tracker.is_inside_render_pass());
        tracker.on_bind_pipeline(42);
        assert_eq!(tracker.get_bound_pipeline(), Some(42));
        tracker.on_end_render_pass();
        assert!(!tracker.is_inside_render_pass());
        tracker.on_end();
    }

    #[test]
    fn test_cmd_buffer_state_tracker_secondary() {
        let tracker = CommandBufferStateTracker::new(false);
        assert!(!tracker.is_primary());
    }

    #[test]
    fn test_cmd_buffer_state_tracker_flush() {
        let mut tracker = CommandBufferStateTracker::new(true);
        tracker.on_begin();
        assert!(!tracker.is_commands_flushed());
        tracker.on_flush();
        assert!(tracker.is_commands_flushed());
    }

    #[test]
    fn test_validation_log_clear() {
        let mut log = ValidationLog::new();
        log.error(ValidationErrorKind::General, "test error");
        log.warn("test warning");
        log.clear();
        assert!(!log.has_errors());
        assert!(log.get_warnings().is_empty());
    }
}
