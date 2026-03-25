type ToggleCallback = Box<dyn Fn(bool) + Send + Sync>;

pub struct Toggle {
    pub is_checked: bool,
    pub interactable: bool,
    listeners: Vec<ToggleCallback>,
}

impl Toggle {
    pub fn new() -> Self {
        Toggle {
            is_checked: false,
            interactable: true,
            listeners: Vec::new(),
        }
    }

    pub fn check(&mut self) {
        if !self.interactable { return; }
        if !self.is_checked {
            self.is_checked = true;
            self.emit(true);
        }
    }

    pub fn uncheck(&mut self) {
        if !self.interactable { return; }
        if self.is_checked {
            self.is_checked = false;
            self.emit(false);
        }
    }

    pub fn toggle(&mut self) {
        if !self.interactable { return; }
        self.is_checked = !self.is_checked;
        let v = self.is_checked;
        self.emit(v);
    }

    pub fn on_change<F: Fn(bool) + Send + Sync + 'static>(&mut self, f: F) {
        self.listeners.push(Box::new(f));
    }

    fn emit(&self, checked: bool) {
        for cb in &self.listeners {
            cb(checked);
        }
    }
}

impl Default for Toggle {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Toggle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toggle")
            .field("is_checked", &self.is_checked)
            .field("interactable", &self.interactable)
            .finish()
    }
}

pub struct ToggleContainer {
    pub allow_switch_off: bool,
    toggles: Vec<usize>,
    active_index: Option<usize>,
}

impl ToggleContainer {
    pub fn new() -> Self {
        ToggleContainer {
            allow_switch_off: false,
            toggles: Vec::new(),
            active_index: None,
        }
    }

    pub fn add_toggle(&mut self, id: usize) {
        self.toggles.push(id);
    }

    pub fn set_active(&mut self, index: usize) -> bool {
        if index < self.toggles.len() {
            self.active_index = Some(index);
            true
        } else {
            false
        }
    }

    pub fn get_active_index(&self) -> Option<usize> {
        self.active_index
    }

    pub fn deactivate_all(&mut self) {
        if self.allow_switch_off {
            self.active_index = None;
        }
    }

    pub fn toggle_count(&self) -> usize {
        self.toggles.len()
    }
}

impl Default for ToggleContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ToggleContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToggleContainer")
            .field("allow_switch_off", &self.allow_switch_off)
            .field("toggle_count", &self.toggles.len())
            .field("active_index", &self.active_index)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_toggle_new() {
        let t = Toggle::new();
        assert!(!t.is_checked);
    }

    #[test]
    fn test_toggle_check_uncheck() {
        let mut t = Toggle::new();
        t.check();
        assert!(t.is_checked);
        t.uncheck();
        assert!(!t.is_checked);
    }

    #[test]
    fn test_toggle_toggle() {
        let mut t = Toggle::new();
        t.toggle();
        assert!(t.is_checked);
        t.toggle();
        assert!(!t.is_checked);
    }

    #[test]
    fn test_toggle_callback() {
        let mut t = Toggle::new();
        let val = Arc::new(Mutex::new(false));
        let v = Arc::clone(&val);
        t.on_change(move |checked| { *v.lock().unwrap() = checked; });
        t.check();
        assert!(*val.lock().unwrap());
        t.uncheck();
        assert!(!*val.lock().unwrap());
    }

    #[test]
    fn test_toggle_not_interactable() {
        let mut t = Toggle::new();
        t.interactable = false;
        t.toggle();
        assert!(!t.is_checked);
    }

    #[test]
    fn test_toggle_container_set_active() {
        let mut tc = ToggleContainer::new();
        tc.add_toggle(0);
        tc.add_toggle(1);
        tc.add_toggle(2);
        tc.set_active(1);
        assert_eq!(tc.get_active_index(), Some(1));
    }

    #[test]
    fn test_toggle_container_deactivate() {
        let mut tc = ToggleContainer::new();
        tc.allow_switch_off = true;
        tc.add_toggle(0);
        tc.set_active(0);
        tc.deactivate_all();
        assert_eq!(tc.get_active_index(), None);
    }

    #[test]
    fn test_toggle_container_no_switch_off() {
        let mut tc = ToggleContainer::new();
        tc.allow_switch_off = false;
        tc.add_toggle(0);
        tc.set_active(0);
        tc.deactivate_all();
        assert_eq!(tc.get_active_index(), Some(0));
    }
}
