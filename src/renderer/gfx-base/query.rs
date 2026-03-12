/****************************************************************************
Rust port of Cocos Creator GFX Query
Original C++ version Copyright (c) 2019-2023 Xiamen Yaji Software Co., Ltd.
****************************************************************************/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Occlusion = 0,
    Timestamp = 1,
    PipelineStatistics = 2,
}

#[derive(Debug, Clone)]
pub struct QueryPoolInfo {
    pub query_type: QueryType,
    pub max_queries: u32,
    pub precise_occlusion: bool,
}

impl Default for QueryPoolInfo {
    fn default() -> Self {
        QueryPoolInfo {
            query_type: QueryType::Occlusion,
            max_queries: 64,
            precise_occlusion: false,
        }
    }
}

#[derive(Debug)]
pub struct GfxQueryPool {
    pub id: u32,
    pub info: QueryPoolInfo,
    pub results: Vec<u64>,
}

impl GfxQueryPool {
    pub fn new(id: u32, info: QueryPoolInfo) -> Self {
        let capacity = info.max_queries as usize;
        GfxQueryPool {
            id,
            info,
            results: vec![0u64; capacity],
        }
    }

    pub fn get_result(&self, index: u32) -> u64 {
        self.results.get(index as usize).copied().unwrap_or(0)
    }

    pub fn reset(&mut self) {
        for r in self.results.iter_mut() {
            *r = 0;
        }
    }
}
