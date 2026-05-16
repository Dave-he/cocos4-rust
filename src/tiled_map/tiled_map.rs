use super::tiled_asset::TiledMapAsset;
use super::tiled_layer::TileLayer;

#[derive(Debug, Clone)]
pub struct TiledMap {
    pub tile_map: TiledMapAsset,
    pub is_loaded: bool,
    width_in_pixels: u32,
    height_in_pixels: u32,
}

impl TiledMap {
    pub fn new(name: &str) -> Self {
        let tile_map = TiledMapAsset::new(name);
        Self {
            tile_map,
            is_loaded: false,
            width_in_pixels: 0,
            height_in_pixels: 0,
        }
    }

    pub fn load(&mut self, asset: TiledMapAsset) {
        self.width_in_pixels = asset.width * asset.tile_width;
        self.height_in_pixels = asset.height * asset.tile_height;
        self.tile_map = asset;
        self.is_loaded = true;
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_loaded {
            self.tile_map.update_animation(dt);
        }
    }

    pub fn get_layer(&self, name: &str) -> Option<&TileLayer> {
        self.tile_map.get_layer(name)
    }

    pub fn get_tile_at(&self, layer_name: &str, x: u32, y: u32) -> Option<&super::tiled_types::TileData> {
        self.tile_map.get_layer(layer_name)?.get_tile(x, y)
    }

    pub fn get_width_in_pixels(&self) -> u32 { self.width_in_pixels }
    pub fn get_height_in_pixels(&self) -> u32 { self.height_in_pixels }
    pub fn get_layer_count(&self) -> usize { self.tile_map.get_layer_count() }
    pub fn is_loaded(&self) -> bool { self.is_loaded }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiled_map_new() {
        let map = TiledMap::new("dungeon");
        assert!(!map.is_loaded());
    }

    #[test]
    fn test_tiled_map_load() {
        let mut map = TiledMap::new("dungeon");
        let asset = TiledMapAsset::new("dungeon");
        map.load(asset);
        assert!(map.is_loaded());
        assert!(map.get_width_in_pixels() > 0);
    }

    #[test]
    fn test_tiled_map_update() {
        let mut map = TiledMap::new("dungeon");
        let mut asset = TiledMapAsset::new("dungeon");
        asset.add_layer(TileLayer::new("anim", 8, 8));
        map.load(asset);
        map.update(0.3);
    }
}
