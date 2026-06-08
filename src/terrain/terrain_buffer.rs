#[derive(Debug, Clone)]
pub struct TerrainBuffer {
    pub vertex_buffer: Vec<f32>,
    pub index_buffer: Vec<u32>,
    pub normal_buffer: Vec<f32>,
    pub uv_buffer: Vec<f32>,
    vertex_count: u32,
    index_count: u32,
}

impl TerrainBuffer {
    pub fn new() -> Self {
        Self {
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
            normal_buffer: Vec::new(),
            uv_buffer: Vec::new(),
            vertex_count: 0,
            index_count: 0,
        }
    }

    pub fn clear(&mut self) {
        self.vertex_buffer.clear();
        self.index_buffer.clear();
        self.normal_buffer.clear();
        self.uv_buffer.clear();
        self.vertex_count = 0;
        self.index_count = 0;
    }

    pub fn get_vertex_count(&self) -> u32 {
        self.vertex_count
    }
    pub fn get_index_count(&self) -> u32 {
        self.index_count
    }

    pub fn get_vertex_size(&self) -> usize {
        self.vertex_buffer.len() * std::mem::size_of::<f32>()
    }
    pub fn get_index_size(&self) -> usize {
        self.index_buffer.len() * std::mem::size_of::<u32>()
    }
}

impl Default for TerrainBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_buffer_new() {
        let tb = TerrainBuffer::new();
        assert_eq!(tb.get_vertex_count(), 0);
    }

    #[test]
    fn test_terrain_buffer_clear() {
        let mut tb = TerrainBuffer::new();
        tb.clear();
        assert_eq!(tb.get_vertex_count(), 0);
    }

    #[test]
    fn test_terrain_buffer_size() {
        let tb = TerrainBuffer::new();
        assert_eq!(tb.get_vertex_size(), 0);
        assert_eq!(tb.get_index_size(), 0);
    }
}
