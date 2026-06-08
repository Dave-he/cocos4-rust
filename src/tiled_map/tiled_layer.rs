use super::tiled_types::{TileData, TileLayerType, TiledObject, TilesetInfo};

#[derive(Debug, Clone)]
pub struct TileLayer {
    pub name: String,
    pub layer_type: TileLayerType,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TileData>,
    pub objects: Vec<TiledObject>,
    pub opacity: f32,
    pub visible: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub parallax_x: f32,
    pub parallax_y: f32,
    anim_frame_index: u32,
    anim_timer: f32,
}

impl TileLayer {
    pub fn new(name: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.to_string(),
            layer_type: TileLayerType::Tile,
            width,
            height,
            tiles: vec![TileData::new(0); (width * height) as usize],
            objects: Vec::new(),
            opacity: 1.0,
            visible: true,
            offset_x: 0.0,
            offset_y: 0.0,
            parallax_x: 1.0,
            parallax_y: 1.0,
            anim_frame_index: 0,
            anim_timer: 0.0,
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, gid: u32) {
        let idx = (y * self.width + x) as usize;
        if idx < self.tiles.len() {
            self.tiles[idx] = TileData::new(gid);
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&TileData> {
        let idx = (y * self.width + x) as usize;
        self.tiles.get(idx)
    }

    pub fn update_animation(&mut self, dt: f32, _tilesets: &[TilesetInfo]) {
        if self.layer_type != TileLayerType::Tile {
            return;
        }
        self.anim_timer += dt;
        if self.anim_timer >= 0.2 {
            self.anim_timer = 0.0;
            self.anim_frame_index = (self.anim_frame_index + 1) % 4;
        }
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
        self.objects.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_layer_new() {
        let layer = TileLayer::new("ground", 16, 16);
        assert_eq!(layer.name, "ground");
        assert_eq!(layer.tiles.len(), 256);
    }

    #[test]
    fn test_set_get_tile() {
        let mut layer = TileLayer::new("test", 10, 10);
        layer.set_tile(5, 3, 42);
        let tile = layer.get_tile(5, 3).unwrap();
        assert_eq!(tile.gid, 42);
    }

    #[test]
    fn test_animation() {
        let mut layer = TileLayer::new("anim", 8, 8);
        layer.update_animation(0.3, &[]);
        assert_eq!(layer.anim_frame_index, 1);
    }
}
