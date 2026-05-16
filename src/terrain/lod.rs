#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LODLevel {
    LOD0 = 0,
    LOD1 = 1,
    LOD2 = 2,
    LOD3 = 3,
}

impl LODLevel {
    pub fn from_distance(distance: f32) -> Self {
        if distance < 20.0 { LODLevel::LOD0 }
        else if distance < 50.0 { LODLevel::LOD1 }
        else if distance < 100.0 { LODLevel::LOD2 }
        else { LODLevel::LOD3 }
    }

    pub fn get_subdivision_factor(&self) -> u32 {
        match self {
            LODLevel::LOD0 => 1,
            LODLevel::LOD1 => 2,
            LODLevel::LOD2 => 4,
            LODLevel::LOD3 => 8,
        }
    }

    pub fn get_index_count_per_patch(&self) -> u32 {
        6
    }
}

#[derive(Debug, Clone)]
pub struct LODData {
    pub level: LODLevel,
    pub vertex_count: u32,
    pub index_count: u32,
    pub visible: bool,
}

impl LODData {
    pub fn new(level: LODLevel) -> Self {
        Self { level, vertex_count: 0, index_count: 0, visible: true }
    }
}

#[derive(Debug, Clone)]
pub struct LODManager {
    pub lod_levels: Vec<LODData>,
    pub active_level: usize,
    max_distance: f32,
}

impl LODManager {
    pub fn new() -> Self {
        let mut lod_levels = Vec::new();
        for i in 0..4 {
            let level = match i {
                0 => LODLevel::LOD0,
                1 => LODLevel::LOD1,
                2 => LODLevel::LOD2,
                _ => LODLevel::LOD3,
            };
            lod_levels.push(LODData::new(level));
        }
        Self { lod_levels, active_level: 0, max_distance: 100.0 }
    }

    pub fn update(&mut self, distance: f32) {
        self.active_level = LODLevel::from_distance(distance) as usize;
    }

    pub fn get_active_lod(&self) -> &LODData {
        &self.lod_levels[self.active_level]
    }

    pub fn get_level_count(&self) -> usize { self.lod_levels.len() }
}

impl Default for LODManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_from_distance() {
        assert_eq!(LODLevel::from_distance(10.0), LODLevel::LOD0);
        assert_eq!(LODLevel::from_distance(30.0), LODLevel::LOD1);
        assert_eq!(LODLevel::from_distance(75.0), LODLevel::LOD2);
        assert_eq!(LODLevel::from_distance(150.0), LODLevel::LOD3);
    }

    #[test]
    fn test_lod_manager_new() {
        let lm = LODManager::new();
        assert_eq!(lm.get_level_count(), 4);
        assert_eq!(lm.active_level, 0);
    }

    #[test]
    fn test_lod_manager_update() {
        let mut lm = LODManager::new();
        lm.update(80.0);
        assert_eq!(lm.active_level, 2);
    }

    #[test]
    fn test_lod_subdivision_factor() {
        assert_eq!(LODLevel::LOD0.get_subdivision_factor(), 1);
        assert_eq!(LODLevel::LOD1.get_subdivision_factor(), 2);
        assert_eq!(LODLevel::LOD3.get_subdivision_factor(), 8);
    }
}
