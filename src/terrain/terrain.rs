use super::height_field::HeightField;
use super::lod::LODManager;

pub struct Terrain {
    pub name: String,
    pub tile_size: u32,
    pub block_count: [u32; 2],
    pub height_field: HeightField,
    pub lod_manager: LODManager,
    pub layer_mask: u32,
    pub hole_count: u32,
    pub weight_map_size: u32,
    pub light_map_size: u32,
}

impl Terrain {
    pub fn new(name: &str, tile_size: u32, block_count: (u32, u32)) -> Self {
        let (bc_x, bc_z) = block_count;
        let vertex_count = (tile_size + 1) * (bc_x) + 1;
        Self {
            name: name.to_string(),
            tile_size,
            block_count: [bc_x, bc_z],
            height_field: HeightField::new(vertex_count, (tile_size + 1) * bc_z + 1),
            lod_manager: LODManager::new(),
            layer_mask: 0xFFFFFFFF,
            hole_count: 0,
            weight_map_size: 128,
            light_map_size: 128,
        }
    }

    pub fn get_tile_vertices(&self) -> u32 {
        self.tile_size + 1
    }

    pub fn get_block_count(&self) -> (u32, u32) {
        (self.block_count[0], self.block_count[1])
    }

    pub fn set_layer_mask(&mut self, mask: u32) {
        self.layer_mask = mask;
    }

    pub fn get_vertex_count(&self) -> usize {
        self.height_field.get_total_vertices()
    }

    pub fn get_height_at(&self, x: f32, z: f32) -> f32 {
        self.height_field.sample_bilinear(x, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_new() {
        let t = Terrain::new("terrain1", 32, (4, 4));
        assert_eq!(t.name, "terrain1");
        assert_eq!(t.tile_size, 32);
    }

    #[test]
    fn test_terrain_block_count() {
        let t = Terrain::new("test", 16, (2, 3));
        assert_eq!(t.get_block_count(), (2, 3));
    }

    #[test]
    fn test_terrain_layer_mask() {
        let mut t = Terrain::new("test", 32, (4, 4));
        t.set_layer_mask(0x0000FFFF);
        assert_eq!(t.layer_mask, 0x0000FFFF);
    }

    #[test]
    fn test_terrain_get_vertex_count() {
        let t = Terrain::new("test", 16, (2, 2));
        assert!(t.get_vertex_count() > 0);
    }
}
