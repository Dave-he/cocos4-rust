#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditBoxInputMode {
    Any,
    Email,
    Integer,
    Decimal,
    Phone,
    Url,
    SingleLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditBoxReturnType {
    Default,
    Done,
    Send,
    Search,
    Next,
    Go,
}

#[derive(Debug, Clone)]
pub struct EditBox {
    pub text: String,
    pub placeholder: String,
    pub input_mode: EditBoxInputMode,
    pub return_type: EditBoxReturnType,
    pub max_length: u32,
    pub tab_index: i32,
    pub focused: bool,
    pub input_enabled: bool,
    pub password_enabled: bool,
    pub multiline: bool,
}

impl EditBox {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::new(),
            input_mode: EditBoxInputMode::Any,
            return_type: EditBoxReturnType::Default,
            max_length: 255,
            tab_index: 0,
            focused: false,
            input_enabled: true,
            password_enabled: false,
            multiline: false,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.placeholder = placeholder.to_string();
    }

    pub fn set_input_mode(&mut self, mode: EditBoxInputMode) {
        self.input_mode = mode;
    }

    pub fn set_max_length(&mut self, max_length: u32) {
        self.max_length = max_length;
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn blur(&mut self) {
        self.focused = false;
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }
    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl Default for EditBox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_box_new() {
        let eb = EditBox::new();
        assert!(!eb.is_focused());
        assert_eq!(eb.max_length, 255);
    }

    #[test]
    fn test_edit_box_text() {
        let mut eb = EditBox::new();
        eb.set_text("hello");
        assert_eq!(eb.get_text(), "hello");
    }

    #[test]
    fn test_edit_box_placeholder() {
        let mut eb = EditBox::new();
        eb.set_placeholder("Enter name...");
        assert_eq!(eb.placeholder, "Enter name...");
    }

    #[test]
    fn test_edit_box_focus() {
        let mut eb = EditBox::new();
        eb.focus();
        assert!(eb.is_focused());
        eb.blur();
        assert!(!eb.is_focused());
    }

    #[test]
    fn test_edit_box_clear() {
        let mut eb = EditBox::new();
        eb.set_text("data");
        eb.clear();
        assert_eq!(eb.get_text(), "");
    }
}
