#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileMapOrientation {
    Orthogonal,
    Isometric,
    Hexagonal,
    Staggered,
}

impl Default for TileMapOrientation {
    fn default() -> Self {
        Self::Orthogonal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileRenderOrder {
    RightDown,
    RightUp,
    LeftDown,
    LeftUp,
}

impl Default for TileRenderOrder {
    fn default() -> Self {
        Self::RightDown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileLayerType {
    Tile,
    Object,
    Image,
    Group,
}

#[derive(Debug, Clone)]
pub struct TileData {
    pub gid: u32,
    pub flip_h: bool,
    pub flip_v: bool,
    pub flip_d: bool,
    pub rotation: u32,
}

impl TileData {
    pub fn new(gid: u32) -> Self {
        Self {
            gid,
            flip_h: false,
            flip_v: false,
            flip_d: false,
            rotation: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TiledObject {
    pub id: u32,
    pub name: String,
    pub obj_type: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
    pub visible: bool,
    pub properties: Vec<(String, String)>,
}

impl TiledObject {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            name: String::new(),
            obj_type: String::new(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            visible: true,
            properties: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TilesetInfo {
    pub first_gid: u32,
    pub name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_count: u32,
    pub columns: u32,
    pub image_source: String,
    pub image_width: u32,
    pub image_height: u32,
    pub spacing: u32,
    pub margin: u32,
}

impl TilesetInfo {
    pub fn new(first_gid: u32, name: &str) -> Self {
        Self {
            first_gid,
            name: name.to_string(),
            tile_width: 16,
            tile_height: 16,
            tile_count: 0,
            columns: 0,
            image_source: String::new(),
            image_width: 0,
            image_height: 0,
            spacing: 0,
            margin: 0,
        }
    }
}
