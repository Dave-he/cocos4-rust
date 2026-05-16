use super::skeleton::Skeleton;

pub struct SpineParser;

impl SpineParser {
    pub fn parse_version(data: &[u8]) -> String {
        if data.len() < 4 {
            return "3.8".to_string();
        }
        format!("{}.{}", data[0], data[1])
    }

    pub fn parse(data: &[u8]) -> Result<Skeleton, String> {
        if data.len() < 4 || data[0] != b's' || data[1] != b'p' {
            return Err("Invalid Spine format".to_string());
        }
        Ok(Skeleton::new("spine"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spine_parse_version() {
        let data = [3u8, 8, 0, 0];
        let version = SpineParser::parse_version(&data);
        assert_eq!(version, "3.8");
    }

    #[test]
    fn test_spine_parse_invalid() {
        let data = [0u8, 0, 0, 0];
        let result = SpineParser::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_spine_parse_valid() {
        let data = [b's', b'p', 0, 0];
        let result = SpineParser::parse(&data);
        assert!(result.is_ok());
    }
}
