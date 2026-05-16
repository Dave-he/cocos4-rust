use super::tiled_types::TilesetInfo;
use super::tiled_layer::TileLayer;

#[derive(Debug, Clone)]
pub struct TiledMapAsset {
    pub name: String,
    pub orientation: super::tiled_types::TileMapOrientation,
    pub render_order: super::tiled_types::TileRenderOrder,
    pub width: u32,
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tilesets: Vec<TilesetInfo>,
    pub layers: Vec<TileLayer>,
    pub bg_color: [u8; 4],
    pub version: String,
}

impl TiledMapAsset {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            orientation: super::tiled_types::TileMapOrientation::Orthogonal,
            render_order: super::tiled_types::TileRenderOrder::RightDown,
            width: 16, height: 16, tile_width: 32, tile_height: 32,
            tilesets: Vec::new(), layers: Vec::new(),
            bg_color: [0, 0, 0, 0],
            version: "1.0".to_string(),
        }
    }

    pub fn add_tileset(&mut self, tileset: TilesetInfo) {
        self.tilesets.push(tileset);
    }

    pub fn add_layer(&mut self, layer: TileLayer) {
        self.layers.push(layer);
    }

    pub fn get_layer(&self, name: &str) -> Option<&TileLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut TileLayer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    pub fn update_animation(&mut self, dt: f32) {
        for layer in &mut self.layers {
            layer.update_animation(dt, &self.tilesets);
        }
    }

    pub fn get_layer_count(&self) -> usize { self.layers.len() }
    pub fn get_tileset_count(&self) -> usize { self.tilesets.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiled_asset_new() {
        let asset = TiledMapAsset::new("map01");
        assert_eq!(asset.name, "map01");
        assert_eq!(asset.get_layer_count(), 0);
    }

    #[test]
    fn test_add_tileset_and_layer() {
        let mut asset = TiledMapAsset::new("map01");
        asset.add_tileset(TilesetInfo::new(1, "tileset1"));
        asset.add_layer(TileLayer::new("ground", 16, 16));
        assert_eq!(asset.get_tileset_count(), 1);
        assert_eq!(asset.get_layer_count(), 1);
    }

    #[test]
    fn test_animation_update() {
        let mut asset = TiledMapAsset::new("map01");
        asset.add_layer(TileLayer::new("anim", 8, 8));
        asset.update_animation(0.3);
    }
}
