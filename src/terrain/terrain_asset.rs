#[derive(Debug, Clone)]
pub struct TerrainAsset {
    pub name: String,
    pub version: u32,
    pub tile_size: u32,
    pub block_count: [u32; 2],
    pub height_data: Vec<f32>,
    pub weight_data: Vec<u8>,
    pub holes: Vec<u32>,
}

impl TerrainAsset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: 1,
            tile_size: 32,
            block_count: [1, 1],
            height_data: Vec::new(),
            weight_data: Vec::new(),
            holes: Vec::new(),
        }
    }

    pub fn set_size(&mut self, tile_size: u32, blocks_x: u32, blocks_z: u32) {
        self.tile_size = tile_size;
        self.block_count = [blocks_x, blocks_z];
        let vertex_count = ((tile_size + 1) * blocks_x + 1) * ((tile_size + 1) * blocks_z + 1);
        self.height_data.resize(vertex_count as usize, 0.0);
    }

    pub fn get_height(&self, index: usize) -> f32 {
        self.height_data.get(index).copied().unwrap_or(0.0)
    }

    pub fn set_height(&mut self, index: usize, value: f32) {
        if index < self.height_data.len() {
            self.height_data[index] = value;
        }
    }

    pub fn add_hole(&mut self, tile_index: u32) {
        if !self.holes.contains(&tile_index) {
            self.holes.push(tile_index);
        }
    }

    pub fn get_hole_count(&self) -> usize {
        self.holes.len()
    }

    pub fn get_height_data_len(&self) -> usize {
        self.height_data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_asset_new() {
        let asset = TerrainAsset::new("terrain");
        assert_eq!(asset.name, "terrain");
        assert_eq!(asset.version, 1);
    }

    #[test]
    fn test_terrain_asset_set_size() {
        let mut asset = TerrainAsset::new("test");
        asset.set_size(16, 2, 2);
        assert_eq!(asset.tile_size, 16);
        assert!(asset.get_height_data_len() > 0);
    }

    #[test]
    fn test_terrain_asset_set_get_height() {
        let mut asset = TerrainAsset::new("test");
        asset.set_size(8, 1, 1);
        asset.set_height(10, 50.0);
        assert_eq!(asset.get_height(10), 50.0);
    }

    #[test]
    fn test_terrain_asset_holes() {
        let mut asset = TerrainAsset::new("test");
        asset.add_hole(5);
        asset.add_hole(5);
        assert_eq!(asset.get_hole_count(), 1);
    }

    #[test]
    fn test_out_of_bounds() {
        let mut asset = TerrainAsset::new("test");
        asset.set_size(8, 1, 1);
        asset.set_height(99999, 100.0);
        let h = asset.get_height(99999);
        assert_eq!(h, 0.0);
    }
}
