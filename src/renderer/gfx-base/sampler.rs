/****************************************************************************
Rust port of Cocos Creator GFX Sampler
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

use super::{Address, ComparisonFunc, Filter};

#[derive(Debug, Clone)]
pub struct SamplerInfo {
    pub min_filter: Filter,
    pub mag_filter: Filter,
    pub mip_filter: Filter,
    pub address_u: Address,
    pub address_v: Address,
    pub address_w: Address,
    pub max_anisotropy: u32,
    pub cmp_func: ComparisonFunc,
    pub mip_lod_bias: f32,
    pub min_lod: f32,
    pub max_lod: f32,
}

impl Default for SamplerInfo {
    fn default() -> Self {
        SamplerInfo {
            min_filter: Filter::Linear,
            mag_filter: Filter::Linear,
            mip_filter: Filter::None,
            address_u: Address::Wrap,
            address_v: Address::Wrap,
            address_w: Address::Wrap,
            max_anisotropy: 0,
            cmp_func: ComparisonFunc::Always,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 1000.0,
        }
    }
}

#[derive(Debug)]
pub struct GfxSampler {
    pub id: u32,
    pub info: SamplerInfo,
}

impl GfxSampler {
    pub fn new(id: u32, info: SamplerInfo) -> Self {
        GfxSampler { id, info }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_default() {
        let info = SamplerInfo::default();
        assert_eq!(info.min_filter, Filter::Linear);
        assert_eq!(info.mag_filter, Filter::Linear);
        assert_eq!(info.address_u, Address::Wrap);
    }

    #[test]
    fn test_sampler_new() {
        let info = SamplerInfo::default();
        let sampler = GfxSampler::new(1, info);
        assert_eq!(sampler.id, 1);
    }
}
