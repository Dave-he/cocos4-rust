use std::collections::HashMap;
use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Cell(i32, i32, i32);

impl Cell {
    fn from_pos(pos: Vec3, cell_size: f32) -> Self {
        Cell(
            (pos.x / cell_size).floor() as i32,
            (pos.y / cell_size).floor() as i32,
            (pos.z / cell_size).floor() as i32,
        )
    }
}

pub struct SpatialGrid<T: Clone> {
    cell_size: f32,
    cells: HashMap<Cell, Vec<(Vec3, T)>>,
    count: usize,
}

impl<T: Clone> SpatialGrid<T> {
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "cell_size must be > 0");
        SpatialGrid {
            cell_size,
            cells: HashMap::new(),
            count: 0,
        }
    }

    pub fn insert(&mut self, pos: Vec3, value: T) {
        let cell = Cell::from_pos(pos, self.cell_size);
        self.cells.entry(cell).or_default().push((pos, value));
        self.count += 1;
    }

    pub fn remove_at(&mut self, pos: Vec3) -> usize {
        let cell = Cell::from_pos(pos, self.cell_size);
        let removed = if let Some(entries) = self.cells.get_mut(&cell) {
            let before = entries.len();
            entries.retain(|(p, _)| {
                let dx = p.x - pos.x;
                let dy = p.y - pos.y;
                let dz = p.z - pos.z;
                dx * dx + dy * dy + dz * dz > 1e-10
            });
            before - entries.len()
        } else {
            0
        };
        self.count -= removed;
        if let Some(entries) = self.cells.get(&cell) {
            if entries.is_empty() {
                self.cells.remove(&cell);
            }
        }
        removed
    }

    pub fn query_radius(&self, center: Vec3, radius: f32) -> Vec<(Vec3, &T)> {
        let r2 = radius * radius;
        let cell_radius = (radius / self.cell_size).ceil() as i32;
        let origin = Cell::from_pos(center, self.cell_size);
        let mut results = Vec::new();

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                for dz in -cell_radius..=cell_radius {
                    let cell = Cell(origin.0 + dx, origin.1 + dy, origin.2 + dz);
                    if let Some(entries) = self.cells.get(&cell) {
                        for (pos, val) in entries {
                            let ex = pos.x - center.x;
                            let ey = pos.y - center.y;
                            let ez = pos.z - center.z;
                            if ex * ex + ey * ey + ez * ez <= r2 {
                                results.push((*pos, val));
                            }
                        }
                    }
                }
            }
        }
        results
    }

    pub fn query_aabb(&self, min: Vec3, max: Vec3) -> Vec<(Vec3, &T)> {
        let cell_min = Cell::from_pos(min, self.cell_size);
        let cell_max = Cell::from_pos(max, self.cell_size);
        let mut results = Vec::new();

        for cx in cell_min.0..=cell_max.0 {
            for cy in cell_min.1..=cell_max.1 {
                for cz in cell_min.2..=cell_max.2 {
                    let cell = Cell(cx, cy, cz);
                    if let Some(entries) = self.cells.get(&cell) {
                        for (pos, val) in entries {
                            if pos.x >= min.x && pos.x <= max.x
                                && pos.y >= min.y && pos.y <= max.y
                                && pos.z >= min.z && pos.z <= max.z
                            {
                                results.push((*pos, val));
                            }
                        }
                    }
                }
            }
        }
        results
    }

    pub fn query_nearest(&self, center: Vec3, max_radius: f32) -> Option<(Vec3, &T)> {
        let candidates = self.query_radius(center, max_radius);
        candidates.into_iter().min_by(|(pa, _), (pb, _)| {
            let da = {
                let dx = pa.x - center.x;
                let dy = pa.y - center.y;
                let dz = pa.z - center.z;
                dx * dx + dy * dy + dz * dz
            };
            let db = {
                let dx = pb.x - center.x;
                let dy = pb.y - center.y;
                let dz = pb.z - center.z;
                dx * dx + dy * dy + dz * dz
            };
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.count = 0;
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn get_cell_size(&self) -> f32 {
        self.cell_size
    }
}

impl<T: Clone> Default for SpatialGrid<T> {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3::new(x, y, z)
    }

    #[test]
    fn test_grid_new() {
        let g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        assert!(g.is_empty());
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_insert_and_len() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(0.0, 0.0, 0.0), 1);
        g.insert(v(1.0, 1.0, 1.0), 2);
        assert_eq!(g.len(), 2);
    }

    #[test]
    fn test_query_radius_basic() {
        let mut g: SpatialGrid<&str> = SpatialGrid::new(1.0);
        g.insert(v(0.0, 0.0, 0.0), "origin");
        g.insert(v(0.5, 0.0, 0.0), "close");
        g.insert(v(5.0, 0.0, 0.0), "far");
        let found = g.query_radius(v(0.0, 0.0, 0.0), 1.0);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_query_radius_empty() {
        let g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        let r = g.query_radius(v(0.0, 0.0, 0.0), 100.0);
        assert!(r.is_empty());
    }

    #[test]
    fn test_query_radius_exact_boundary() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(1.0, 0.0, 0.0), 1);
        let r = g.query_radius(v(0.0, 0.0, 0.0), 1.0);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_query_aabb() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(0.5, 0.5, 0.5), 1);
        g.insert(v(1.5, 1.5, 1.5), 2);
        g.insert(v(3.0, 3.0, 3.0), 3);
        let r = g.query_aabb(v(0.0, 0.0, 0.0), v(2.0, 2.0, 2.0));
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn test_query_nearest() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(1.0, 0.0, 0.0), 1);
        g.insert(v(2.0, 0.0, 0.0), 2);
        g.insert(v(0.5, 0.0, 0.0), 3);
        let nearest = g.query_nearest(v(0.0, 0.0, 0.0), 10.0);
        assert!(nearest.is_some());
        assert_eq!(*nearest.unwrap().1, 3);
    }

    #[test]
    fn test_query_nearest_none_in_range() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(100.0, 0.0, 0.0), 1);
        let r = g.query_nearest(v(0.0, 0.0, 0.0), 1.0);
        assert!(r.is_none());
    }

    #[test]
    fn test_remove_at() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(0.0, 0.0, 0.0), 42);
        assert_eq!(g.len(), 1);
        let removed = g.remove_at(v(0.0, 0.0, 0.0));
        assert_eq!(removed, 1);
        assert_eq!(g.len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(0.0, 0.0, 0.0), 1);
        g.insert(v(1.0, 0.0, 0.0), 2);
        g.clear();
        assert!(g.is_empty());
        assert_eq!(g.cell_count(), 0);
    }

    #[test]
    fn test_large_cell_size() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(100.0);
        g.insert(v(10.0, 0.0, 0.0), 1);
        g.insert(v(50.0, 0.0, 0.0), 2);
        g.insert(v(99.0, 0.0, 0.0), 3);
        assert_eq!(g.cell_count(), 1);
        let r = g.query_radius(v(50.0, 0.0, 0.0), 60.0);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_many_items_same_cell() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(10.0);
        for i in 0..50 {
            g.insert(v(i as f32 * 0.1, 0.0, 0.0), i);
        }
        assert_eq!(g.cell_count(), 1);
        assert_eq!(g.len(), 50);
        let r = g.query_radius(v(2.5, 0.0, 0.0), 3.0);
        assert!(!r.is_empty());
    }

    #[test]
    fn test_3d_spread() {
        let mut g: SpatialGrid<u32> = SpatialGrid::new(1.0);
        g.insert(v(0.0, 0.0, 0.0), 0);
        g.insert(v(0.0, 0.0, 5.0), 1);
        g.insert(v(0.0, 5.0, 0.0), 2);
        g.insert(v(5.0, 0.0, 0.0), 3);
        let near = g.query_radius(v(0.0, 0.0, 0.0), 1.0);
        assert_eq!(near.len(), 1);
    }
}
