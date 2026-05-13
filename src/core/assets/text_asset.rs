/****************************************************************************
Rust port of Cocos Creator TextAsset
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::asset::AssetBase;

#[derive(Debug)]
pub struct TextAsset {
    pub base: AssetBase,
    pub text: String,
}

impl TextAsset {
    pub fn new() -> Self {
        TextAsset {
            base: AssetBase::default(),
            text: String::new(),
        }
    }

    pub fn to_string(&self) -> &str {
        &self.text
    }
}

impl Default for TextAsset {
    fn default() -> Self {
        TextAsset {
            base: AssetBase::default(),
            text: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_asset_new() {
        let asset = TextAsset::new();
        assert_eq!(asset.text, "");
        assert_eq!(asset.to_string(), "");
    }

    #[test]
    fn test_text_asset_with_content() {
        let mut asset = TextAsset::new();
        asset.text = "Hello, World!".to_string();
        assert_eq!(asset.to_string(), "Hello, World!");
    }

    #[test]
    fn test_text_asset_default() {
        let asset = TextAsset::default();
        assert_eq!(asset.text, "");
    }
}
