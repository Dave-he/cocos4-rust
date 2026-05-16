
#[derive(Debug, Clone)]
pub enum SlotType {
    Image,
    Mesh,
    Mask,
}

#[derive(Debug, Clone)]
pub enum BlendMode {
    Normal,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub slot_type: SlotType,
    pub bone_name: String,
    pub blend_mode: BlendMode,
    pub color: [f32; 4],
    pub display_index: i32,
    pub visible: bool,
    pub z_order: i32,
}

impl Slot {
    pub fn new(name: &str, bone_name: &str) -> Self {
        Self {
            name: name.to_string(),
            slot_type: SlotType::Image,
            bone_name: bone_name.to_string(),
            blend_mode: BlendMode::Normal,
            color: [1.0, 1.0, 1.0, 1.0],
            display_index: 0,
            visible: true,
            z_order: 0,
        }
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color = [r, g, b, a];
    }

    pub fn set_display(&mut self, index: i32) {
        self.display_index = index;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_new() {
        let slot = Slot::new("head", "neck");
        assert_eq!(slot.name, "head");
        assert_eq!(slot.bone_name, "neck");
        assert!(slot.visible);
    }

    #[test]
    fn test_slot_color() {
        let mut slot = Slot::new("arm", "shoulder");
        slot.set_color(0.5, 0.5, 0.5, 0.8);
        assert_eq!(slot.color[3], 0.8);
    }

    #[test]
    fn test_slot_display() {
        let mut slot = Slot::new("weapon", "hand");
        slot.set_display(3);
        assert_eq!(slot.display_index, 3);
    }
}
