use crate::math::Vec3;

pub struct LODData {
    pub screen_usage_percentage: f32,
    pub models: Vec<u64>,
}

impl LODData {
    pub fn new() -> Self {
        LODData {
            screen_usage_percentage: 1.0,
            models: Vec::new(),
        }
    }

    pub fn get_screen_usage_percentage(&self) -> f32 {
        self.screen_usage_percentage
    }

    pub fn set_screen_usage_percentage(&mut self, val: f32) {
        self.screen_usage_percentage = val;
    }

    pub fn add_model(&mut self, model_id: u64) {
        self.models.push(model_id);
    }

    pub fn clear_models(&mut self) {
        self.models.clear();
    }

    pub fn erase_model(&mut self, model_id: u64) {
        self.models.retain(|id| *id != model_id);
    }

    pub fn get_models(&self) -> &[u64] {
        &self.models
    }
}

impl Default for LODData {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LODGroup {
    pub enabled: bool,
    pub object_size: f32,
    pub local_boundary_center: Vec3,
    pub lod_data_array: Vec<LODData>,
    pub locked_lod_levels: Vec<u8>,
    pub is_lock_level_changed: bool,
}

impl LODGroup {
    pub fn new() -> Self {
        LODGroup {
            enabled: true,
            object_size: 1.0,
            local_boundary_center: Vec3::ZERO,
            lod_data_array: Vec::new(),
            locked_lod_levels: Vec::new(),
            is_lock_level_changed: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, val: bool) {
        self.enabled = val;
    }

    pub fn get_object_size(&self) -> f32 {
        self.object_size
    }

    pub fn set_object_size(&mut self, val: f32) {
        self.object_size = val;
    }

    pub fn get_local_boundary_center(&self) -> &Vec3 {
        &self.local_boundary_center
    }

    pub fn set_local_boundary_center(&mut self, val: Vec3) {
        self.local_boundary_center = val;
    }

    pub fn get_lod_data_array(&self) -> &[LODData] {
        &self.lod_data_array
    }

    pub fn get_visible_lod_level(&self, camera_screen_usage: f32) -> i8 {
        if !self.enabled {
            return 0;
        }
        if self.lod_data_array.is_empty() {
            return -1;
        }
        for (i, lod) in self.lod_data_array.iter().enumerate() {
            if camera_screen_usage >= lod.screen_usage_percentage {
                return i as i8;
            }
        }
        -1
    }

    pub fn get_locked_lod_levels(&self) -> &[u8] {
        &self.locked_lod_levels
    }

    pub fn lock_lod_levels(&mut self, levels: Vec<i32>) {
        self.locked_lod_levels = levels.iter().map(|l| *l as u8).collect();
        self.is_lock_level_changed = true;
    }

    pub fn is_lock_level_changed(&self) -> bool {
        self.is_lock_level_changed
    }

    pub fn reset_lock_change_flag(&mut self) {
        self.is_lock_level_changed = false;
    }

    pub fn get_lod_count(&self) -> u8 {
        self.lod_data_array.len() as u8
    }

    pub fn clear_lods(&mut self) {
        self.lod_data_array.clear();
    }

    pub fn insert_lod(&mut self, index: u8, data: LODData) {
        let idx = index as usize;
        if idx <= self.lod_data_array.len() {
            self.lod_data_array.insert(idx, data);
        }
    }

    pub fn update_lod(&mut self, index: u8, data: LODData) {
        let idx = index as usize;
        if idx < self.lod_data_array.len() {
            self.lod_data_array[idx] = data;
        }
    }

    pub fn erase_lod(&mut self, index: u8) {
        let idx = index as usize;
        if idx < self.lod_data_array.len() {
            self.lod_data_array.remove(idx);
        }
    }
}

impl Default for LODGroup {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_data_new() {
        let lod = LODData::new();
        assert_eq!(lod.screen_usage_percentage, 1.0);
    }

    #[test]
    fn test_lod_group_new() {
        let group = LODGroup::new();
        assert!(group.enabled);
        assert_eq!(group.object_size, 1.0);
    }

    #[test]
    fn test_lod_group_visible_level() {
        let mut group = LODGroup::new();
        group.insert_lod(0, LODData { screen_usage_percentage: 0.5, models: vec![] });
        group.insert_lod(1, LODData { screen_usage_percentage: 0.2, models: vec![] });
        group.insert_lod(2, LODData { screen_usage_percentage: 0.05, models: vec![] });
        assert_eq!(group.get_visible_lod_level(0.3), 1);
        assert_eq!(group.get_visible_lod_level(0.6), 0);
    }

    #[test]
    fn test_lod_group_lock_levels() {
        let mut group = LODGroup::new();
        group.lock_lod_levels(vec![0, 2]);
        assert!(group.is_lock_level_changed());
        assert_eq!(group.locked_lod_levels.len(), 2);
        group.reset_lock_change_flag();
        assert!(!group.is_lock_level_changed());
    }
}
