/****************************************************************************
Rust port of Cocos Creator PhysX Filter Shader
Original C++ version Copyright (c) 2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterAction {
    Kill = 0,
    Suppress = 1,
    Callback = 2,
    Default = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct FilterData {
    pub word0: u32,
    pub word1: u32,
    pub word2: u32,
    pub word3: u32,
}

impl FilterData {
    pub fn new(word0: u32, word1: u32, word2: u32, word3: u32) -> Self {
        FilterData { word0, word1, word2, word3 }
    }

    pub fn from_group_and_mask(group: u32, mask: u32) -> Self {
        FilterData {
            word0: group,
            word1: mask,
            word2: 0,
            word3: 0,
        }
    }
}

impl Default for FilterData {
    fn default() -> Self {
        FilterData { word0: 1, word1: 0xFFFFFFFF, word2: 0, word3: 0 }
    }
}

pub fn default_filter_shader(
    filter0: FilterData,
    filter1: FilterData,
) -> FilterAction {
    if (filter0.word0 & filter1.word1) == 0 && (filter1.word0 & filter0.word1) == 0 {
        return FilterAction::Suppress;
    }
    FilterAction::Default
}

pub fn should_collide(group_a: u32, mask_a: u32, group_b: u32, mask_b: u32) -> bool {
    (group_a & mask_b) != 0 && (group_b & mask_a) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_data_group_mask() {
        let f = FilterData::from_group_and_mask(0b0001, 0b1111);
        assert_eq!(f.word0, 0b0001);
        assert_eq!(f.word1, 0b1111);
    }

    #[test]
    fn test_should_collide_true() {
        assert!(should_collide(0b0001, 0b1111, 0b0010, 0b1111));
    }

    #[test]
    fn test_should_collide_false() {
        assert!(!should_collide(0b0001, 0b0010, 0b0100, 0b1000));
    }

    #[test]
    fn test_default_filter_shader_suppress() {
        let f0 = FilterData { word0: 1, word1: 0, word2: 0, word3: 0 };
        let f1 = FilterData { word0: 2, word1: 0, word2: 0, word3: 0 };
        assert_eq!(default_filter_shader(f0, f1), FilterAction::Suppress);
    }

    #[test]
    fn test_default_filter_shader_pass() {
        let f0 = FilterData { word0: 1, word1: 0xFF, word2: 0, word3: 0 };
        let f1 = FilterData { word0: 2, word1: 0xFF, word2: 0, word3: 0 };
        assert_eq!(default_filter_shader(f0, f1), FilterAction::Default);
    }
}
