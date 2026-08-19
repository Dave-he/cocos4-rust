use super::tiled_asset::TiledMapAsset;
use super::tiled_layer::TileLayer;
use super::tiled_types::{TileMapOrientation, TileRenderOrder, TilesetInfo};

pub struct TmxParser;

impl TmxParser {
    pub fn parse(xml_data: &str) -> Result<TiledMapAsset, String> {
        if xml_data.is_empty() || !xml_data.trim_start().starts_with('<') {
            return Err("Invalid TMX format".to_string());
        }

        let map_start = xml_data.find("<map");
        if map_start.is_none() {
            return Err("No <map> element found".to_string());
        }

        let map_tag_end = xml_data[map_start.unwrap()..].find('>').map(|i| map_start.unwrap() + i);
        if map_tag_end.is_none() {
            return Err("Malformed <map> tag".to_string());
        }

        let map_tag = &xml_data[map_start.unwrap()..=map_tag_end.unwrap()];

        let mut asset = TiledMapAsset::new("tiled_map");

        asset.orientation = Self::parse_orientation(map_tag);
        asset.width = Self::parse_attr_u32(map_tag, "width").unwrap_or(16);
        asset.height = Self::parse_attr_u32(map_tag, "height").unwrap_or(16);
        asset.tile_width = Self::parse_attr_u32(map_tag, "tilewidth").unwrap_or(32);
        asset.tile_height = Self::parse_attr_u32(map_tag, "tileheight").unwrap_or(32);

        Self::parse_tilesets(xml_data, &mut asset);
        Self::parse_layers(xml_data, &mut asset);

        Ok(asset)
    }

    fn parse_orientation(map_tag: &str) -> TileMapOrientation {
        let orient = Self::parse_attr_str(map_tag, "orientation").unwrap_or_default();
        match orient.as_str() {
            "orthogonal" => TileMapOrientation::Orthogonal,
            "isometric" => TileMapOrientation::Isometric,
            "hexagonal" => TileMapOrientation::Hexagonal,
            "staggered" => TileMapOrientation::Staggered,
            _ => TileMapOrientation::Orthogonal,
        }
    }

    #[allow(dead_code)]
    fn parse_render_order(map_tag: &str) -> TileRenderOrder {
        let order = Self::parse_attr_str(map_tag, "renderorder").unwrap_or_default();
        match order.as_str() {
            "right-up" => TileRenderOrder::RightUp,
            "left-down" => TileRenderOrder::LeftDown,
            "left-up" => TileRenderOrder::LeftUp,
            _ => TileRenderOrder::RightDown,
        }
    }

    fn parse_tilesets(xml_data: &str, asset: &mut TiledMapAsset) {
        let mut search_pos = 0;
        while let Some(ts_start) = xml_data[search_pos..].find("<tileset") {
            let abs_start = search_pos + ts_start;
            if let Some(tag_end) = xml_data[abs_start..].find('>') {
                let tag = &xml_data[abs_start..=abs_start + tag_end];
                let first_gid = Self::parse_attr_u32(tag, "firstgid").unwrap_or(1);
                let name = Self::parse_attr_str(tag, "name").unwrap_or_else(|| format!("tileset_{}", first_gid));
                let tile_width = Self::parse_attr_u32(tag, "tilewidth").unwrap_or(16);
                let tile_height = Self::parse_attr_u32(tag, "tileheight").unwrap_or(16);
                let tile_count = Self::parse_attr_u32(tag, "tilecount").unwrap_or(0);
                let columns = Self::parse_attr_u32(tag, "columns").unwrap_or(0);
                let spacing = Self::parse_attr_u32(tag, "spacing").unwrap_or(0);
                let margin = Self::parse_attr_u32(tag, "margin").unwrap_or(0);

                let mut info = TilesetInfo::new(first_gid, &name);
                info.tile_width = tile_width;
                info.tile_height = tile_height;
                info.tile_count = tile_count;
                info.columns = columns;
                info.spacing = spacing;
                info.margin = margin;

                let img_end = xml_data[abs_start + tag_end..].find("</tileset>")
                    .map(|i| abs_start + tag_end + i)
                    .unwrap_or(abs_start + tag_end);
                let tileset_content = &xml_data[abs_start + tag_end..img_end];
                if let Some(img_start) = tileset_content.find("<image") {
                    if let Some(img_tag_end) = tileset_content[img_start..].find('>') {
                        let img_tag = &tileset_content[img_start..=img_start + img_tag_end];
                        info.image_source = Self::parse_attr_str(img_tag, "source").unwrap_or_default();
                        info.image_width = Self::parse_attr_u32(img_tag, "width").unwrap_or(0);
                        info.image_height = Self::parse_attr_u32(img_tag, "height").unwrap_or(0);
                    }
                }

                asset.add_tileset(info);
                search_pos = abs_start + tag_end + 1;
            } else {
                break;
            }
        }
    }

    fn parse_layers(xml_data: &str, asset: &mut TiledMapAsset) {
        let mut search_pos = 0;
        while let Some(layer_start) = xml_data[search_pos..].find("<layer") {
            let abs_start = search_pos + layer_start;
            if let Some(tag_end) = xml_data[abs_start..].find('>') {
                let tag = &xml_data[abs_start..=abs_start + tag_end];
                let name = Self::parse_attr_str(tag, "name").unwrap_or_else(|| "layer".to_string());
                let width = Self::parse_attr_u32(tag, "width").unwrap_or(asset.width);
                let height = Self::parse_attr_u32(tag, "height").unwrap_or(asset.height);

                let mut layer = TileLayer::new(&name, width, height);

                let layer_close = xml_data[abs_start + tag_end..].find("</layer>")
                    .map(|i| abs_start + tag_end + i)
                    .unwrap_or(abs_start + tag_end);
                let layer_content = &xml_data[abs_start + tag_end..layer_close];

                if let Some(data_start) = layer_content.find("<data") {
                    if let Some(data_content_start) = layer_content[data_start..].find('>') {
                        let dc_start = data_start + data_content_start + 1;
                        let data_end = layer_content[dc_start..].find("</data>")
                            .map(|i| dc_start + i)
                            .unwrap_or(layer_content.len());
                        let data_content = layer_content[dc_start..data_end].trim();

                        let encoding = Self::parse_attr_str(
                            &layer_content[data_start..data_start + data_content_start],
                            "encoding"
                        ).unwrap_or_default();

                        match encoding.as_str() {
                            "csv" => Self::parse_csv_data(data_content, &mut layer),
                            "base64" => {}
                            _ => Self::parse_xml_tiles(data_content, &mut layer),
                        }
                    }
                }

                asset.add_layer(layer);
                search_pos = layer_close + 7;
            } else {
                break;
            }
        }
    }

    fn parse_csv_data(data: &str, layer: &mut TileLayer) {
        let gids: Vec<u32> = data
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .collect();

        for (i, gid) in gids.iter().enumerate() {
            let x = (i as u32) % layer.width;
            let y = (i as u32) / layer.width;
            if y < layer.height {
                layer.set_tile(x, y, *gid);
            }
        }
    }

    fn parse_xml_tiles(data: &str, layer: &mut TileLayer) {
        let mut search_pos = 0;
        let mut idx = 0u32;
        while let Some(tile_start) = data[search_pos..].find("<tile") {
            let abs_start = search_pos + tile_start;
            if let Some(tag_end) = data[abs_start..].find('>') {
                let tag = &data[abs_start..=abs_start + tag_end];
                let gid = Self::parse_attr_u32(tag, "gid").unwrap_or(0);
                let x = idx % layer.width;
                let y = idx / layer.width;
                if y < layer.height {
                    layer.set_tile(x, y, gid);
                }
                idx += 1;
                search_pos = abs_start + tag_end + 1;
            } else {
                break;
            }
        }
    }

    fn parse_attr_str(tag: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = tag.find(&pattern) {
            let val_start = start + pattern.len();
            if let Some(end) = tag[val_start..].find('"') {
                return Some(tag[val_start..val_start + end].to_string());
            }
        }
        None
    }

    fn parse_attr_u32(tag: &str, attr: &str) -> Option<u32> {
        Self::parse_attr_str(tag, attr).and_then(|s| s.parse().ok())
    }

    pub fn parse_version(xml_data: &str) -> String {
        Self::parse_attr_str(xml_data, "version").unwrap_or_else(|| "1.0".to_string())
    }

    pub fn get_map_dimensions(xml_data: &str) -> (u32, u32) {
        let map_start = xml_data.find("<map");
        if let Some(start) = map_start {
            let map_tag_end = xml_data[start..].find('>').map(|i| start + i);
            if let Some(end) = map_tag_end {
                let map_tag = &xml_data[start..=end];
                let width = Self::parse_attr_u32(map_tag, "width").unwrap_or(16);
                let height = Self::parse_attr_u32(map_tag, "height").unwrap_or(16);
                return (width, height);
            }
        }
        (16, 16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = TmxParser::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid() {
        let result = TmxParser::parse("<invalid_xml_without_map_tag/>");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.5" orientation="orthogonal" width="16" height="16" tilewidth="32" tileheight="32">
    <tileset firstgid="1" name="tiles" tilewidth="32" tileheight="32" tilecount="100" columns="10">
        <image source="tiles.png" width="320" height="320"/>
    </tileset>
    <layer name="bg" width="16" height="16">
        <data encoding="csv">0,1,2,3,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0</data>
    </layer>
</map>"#;
        let result = TmxParser::parse(xml);
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.orientation, TileMapOrientation::Orthogonal);
        assert_eq!(asset.width, 16);
        assert_eq!(asset.height, 16);
        assert_eq!(asset.tile_width, 32);
        assert_eq!(asset.tile_height, 32);
    }

    #[test]
    fn test_parse_orientation_hex() {
        let xml = r#"<map orientation="hexagonal" width="8" height="8" tilewidth="16" tileheight="16"/>"#;
        let result = TmxParser::parse(xml);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().orientation, TileMapOrientation::Hexagonal);
    }

    #[test]
    fn test_parse_orientation_iso() {
        let xml = r#"<map orientation="isometric" width="10" height="10" tilewidth="32" tileheight="32"/>"#;
        let result = TmxParser::parse(xml);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().orientation, TileMapOrientation::Isometric);
    }

    #[test]
    fn test_parse_tileset() {
        let xml = r#"<map orientation="orthogonal" width="16" height="16" tilewidth="32" tileheight="32">
            <tileset firstgid="1" name="tiles" tilewidth="32" tileheight="32">
                <image source="tiles.png" width="320" height="320"/>
            </tileset>
        </map>"#;
        let asset = TmxParser::parse(xml).unwrap();
        assert_eq!(asset.get_tileset_count(), 1);
    }

    #[test]
    fn test_parse_csv_data() {
        let xml = r#"<map orientation="orthogonal" width="4" height="2" tilewidth="16" tileheight="16">
            <layer name="bg" width="4" height="2">
                <data encoding="csv">1,2,3,4,5,6,7,8</data>
            </layer>
        </map>"#;
        let asset = TmxParser::parse(xml).unwrap();
        assert!(asset.get_layer_count() > 0);
    }

    #[test]
    fn test_parse_dimensions() {
        let xml = r#"<map width="32" height="24" tilewidth="16" tileheight="16"/>"#;
        let (w, h) = TmxParser::get_map_dimensions(xml);
        assert_eq!(w, 32);
        assert_eq!(h, 24);
    }

    #[test]
    fn test_parse_version() {
        let xml = r#"<map version="1.5" orientation="orthogonal"/>"#;
        let ver = TmxParser::parse_version(xml);
        assert_eq!(ver, "1.5");
    }

    #[test]
    fn test_parse_attr_u32() {
        let tag = r#"<map width="100" height="50"/>"#;
        assert_eq!(TmxParser::parse_attr_u32(tag, "width"), Some(100));
        assert_eq!(TmxParser::parse_attr_u32(tag, "height"), Some(50));
        assert_eq!(TmxParser::parse_attr_u32(tag, "nonexistent"), None);
    }
}
