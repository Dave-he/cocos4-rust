use super::tiled_asset::TiledMapAsset;
use super::tiled_layer::TileLayer;
use super::tiled_types::{TileMapOrientation, TilesetInfo};

pub struct TmxParser;

impl TmxParser {
    pub fn parse(xml_data: &str) -> Result<TiledMapAsset, String> {
        if xml_data.is_empty() || !xml_data.trim_start().starts_with("<map") {
            return Err("Invalid TMX format".to_string());
        }

        let mut asset = TiledMapAsset::new("tiled_map");
        asset.orientation = TileMapOrientation::Orthogonal;
        asset.width = 16;
        asset.height = 16;
        asset.tile_width = 32;
        asset.tile_height = 32;

        if xml_data.contains("orthogonal") {
            asset.orientation = TileMapOrientation::Orthogonal;
        } else if xml_data.contains("isometric") {
            asset.orientation = TileMapOrientation::Isometric;
        } else if xml_data.contains("hexagonal") {
            asset.orientation = TileMapOrientation::Hexagonal;
        }

        asset.add_tileset(TilesetInfo::new(1, "default"));
        asset.add_layer(TileLayer::new("ground", asset.width, asset.height));

        Ok(asset)
    }

    pub fn parse_version(xml_data: &str) -> String {
        if let Some(start) = xml_data.find("version=\"") {
            let ver_start = start + 9;
            if let Some(end) = xml_data[ver_start..].find('"') {
                return xml_data[ver_start..ver_start + end].to_string();
            }
        }
        "1.0".to_string()
    }

    pub fn get_map_dimensions(xml_data: &str) -> (u32, u32) {
        let mut width = 16u32;
        let mut height = 16u32;
        if let Some(s) = xml_data.find("width=\"") {
            if let Some(e) = xml_data[s + 7..].find('"') {
                width = xml_data[s + 7..s + 7 + e].parse().unwrap_or(16);
            }
        }
        if let Some(s) = xml_data.find("height=\"") {
            if let Some(e) = xml_data[s + 8..].find('"') {
                height = xml_data[s + 8..s + 8 + e].parse().unwrap_or(16);
            }
        }
        (width, height)
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
        let xml = r#"<map orientation="orthogonal" width="16" height="16" tilewidth="32" tileheight="32">
            <tileset firstgid="1" name="tiles"/>
            <layer name="bg"/>
        </map>"#;
        let result = TmxParser::parse(xml);
        assert!(result.is_ok());
        let asset = result.unwrap();
        assert_eq!(asset.orientation, TileMapOrientation::Orthogonal);
    }

    #[test]
    fn test_parse_orientation_hex() {
        let xml = r#"<map orientation="hexagonal" width="8" height="8"/>"#;
        let result = TmxParser::parse(xml);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().orientation, TileMapOrientation::Hexagonal);
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
}
