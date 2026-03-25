#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortingLayerInfo {
    pub id: u32,
    pub name: String,
}

pub struct SortingLayers {
    layers: Vec<SortingLayerInfo>,
    next_id: u32,
}

impl SortingLayers {
    pub fn new() -> Self {
        let mut sl = SortingLayers {
            layers: Vec::new(),
            next_id: 1,
        };
        sl.layers.push(SortingLayerInfo { id: 0, name: "Default".to_string() });
        sl
    }

    pub fn add_layer(&mut self, name: &str) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.layers.push(SortingLayerInfo { id, name: name.to_string() });
        id
    }

    pub fn remove_layer(&mut self, id: u32) -> bool {
        if id == 0 { return false; }
        let before = self.layers.len();
        self.layers.retain(|l| l.id != id);
        self.layers.len() < before
    }

    pub fn get_layer_id(&self, name: &str) -> Option<u32> {
        self.layers.iter().find(|l| l.name == name).map(|l| l.id)
    }

    pub fn get_layer_name(&self, id: u32) -> Option<&str> {
        self.layers.iter().find(|l| l.id == id).map(|l| l.name.as_str())
    }

    pub fn get_layer_index(&self, id: u32) -> Option<usize> {
        self.layers.iter().position(|l| l.id == id)
    }

    pub fn get_sorting_priority(&self, layer_id: u32, order: i32) -> i64 {
        let idx = self.get_layer_index(layer_id).unwrap_or(0) as i64;
        (idx << 16) + order as i64
    }

    pub fn get_all_layers(&self) -> &[SortingLayerInfo] {
        &self.layers
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

impl Default for SortingLayers {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SortingLayers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SortingLayers")
            .field("layer_count", &self.layers.len())
            .field("layers", &self.layers)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sorting_layers_new() {
        let sl = SortingLayers::new();
        assert_eq!(sl.layer_count(), 1);
        assert_eq!(sl.get_layer_name(0), Some("Default"));
    }

    #[test]
    fn test_add_layer() {
        let mut sl = SortingLayers::new();
        let id = sl.add_layer("Background");
        assert_eq!(sl.get_layer_name(id), Some("Background"));
        assert_eq!(sl.layer_count(), 2);
    }

    #[test]
    fn test_remove_layer() {
        let mut sl = SortingLayers::new();
        let id = sl.add_layer("Test");
        assert!(sl.remove_layer(id));
        assert_eq!(sl.layer_count(), 1);
    }

    #[test]
    fn test_cannot_remove_default() {
        let mut sl = SortingLayers::new();
        assert!(!sl.remove_layer(0));
    }

    #[test]
    fn test_get_layer_id() {
        let mut sl = SortingLayers::new();
        sl.add_layer("UI");
        assert!(sl.get_layer_id("UI").is_some());
        assert!(sl.get_layer_id("Nonexistent").is_none());
    }

    #[test]
    fn test_sorting_priority() {
        let mut sl = SortingLayers::new();
        let bg = sl.add_layer("Background");
        let ui = sl.add_layer("UI");
        let bg_prio = sl.get_sorting_priority(bg, 0);
        let ui_prio = sl.get_sorting_priority(ui, 0);
        assert!(ui_prio > bg_prio);
    }

    #[test]
    fn test_sorting_priority_order() {
        let sl = SortingLayers::new();
        let p1 = sl.get_sorting_priority(0, 5);
        let p2 = sl.get_sorting_priority(0, 10);
        assert!(p2 > p1);
    }
}
