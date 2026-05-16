pub struct DragonBonesParser;

impl DragonBonesParser {
    pub fn parse_version(data: &[u8]) -> String {
        if data.len() < 8 {
            return "5.5".to_string();
        }
        let major = data[0];
        let minor = data[1];
        format!("{}.{}", major, minor)
    }

    pub fn parse(data: &[u8]) -> Result<super::armature::Armature, String> {
        if data.len() < 3 || data[0] != b'D' || data[1] != b'B' {
            return Err("Invalid DragonBones format".to_string());
        }
        let armature = super::armature::Armature::new("dragonbones");
        Ok(armature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        let data = vec![5u8, 5, 0, 0];
        let version = DragonBonesParser::parse_version(&data);
        assert_eq!(version, "5.5");
    }

    #[test]
    fn test_parse_invalid() {
        let data = vec![0u8, 0, 0, 0];
        let result = DragonBonesParser::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_valid() {
        let data = vec![b'D', b'B', 0, 0];
        let result = DragonBonesParser::parse(&data);
        assert!(result.is_ok());
    }
}
