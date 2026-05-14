/****************************************************************************
Rust port of Cocos Creator BufferAsset
Original C++ version Copyright (c) 2021-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/
// SPDX-License-Identifier: MIT

use super::asset::AssetBase;

#[derive(Debug)]
#[allow(clippy::derivable_impls)]
pub struct BufferAsset {
    pub base: AssetBase,
    buffer: Option<Vec<u8>>,
}

impl BufferAsset {
    pub fn new() -> Self {
        BufferAsset {
            base: AssetBase::default(),
            buffer: None,
        }
    }

    pub fn get_buffer(&self) -> Option<&Vec<u8>> {
        self.buffer.as_ref()
    }

    pub fn set_buffer(&mut self, data: Vec<u8>) {
        self.buffer = Some(data);
    }

    pub fn validate(&self) -> bool {
        self.buffer.is_some()
    }
}

impl Default for BufferAsset {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        BufferAsset {
            base: AssetBase::default(),
            buffer: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_asset_new() {
        let asset = BufferAsset::new();
        assert!(!asset.validate());
        assert!(asset.get_buffer().is_none());
    }

    #[test]
    fn test_buffer_asset_set_buffer() {
        let mut asset = BufferAsset::new();
        asset.set_buffer(vec![1, 2, 3, 4]);
        assert!(asset.validate());
        assert_eq!(asset.get_buffer().unwrap().len(), 4);
    }

    #[test]
    fn test_buffer_asset_default() {
        let asset = BufferAsset::default();
        assert!(!asset.validate());
    }
}
