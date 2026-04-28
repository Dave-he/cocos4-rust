use crate::math::{Vec3, Color};

fn pseudo_rand(seed: u64) -> f32 {
    let h = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let h = ((h >> 33) ^ h).wrapping_mul(0xff51afd7ed558ccd);
    let h = ((h >> 33) ^ h).wrapping_mul(0xc4ceb9fe1a85ec53);
    let h = (h >> 33) ^ h;
    (h as f32) / (u64::MAX as f32)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise_1d(x: f32) -> f32 {
    let xi = x.floor() as i64;
    let xf = x.fract();
    let a = pseudo_rand(xi as u64 ^ 0xdeadbeef);
    let b = pseudo_rand((xi + 1) as u64 ^ 0xdeadbeef);
    let t = smoothstep(xf);
    a + (b - a) * t
}

fn value_noise_3d(pos: Vec3, seed: u64) -> f32 {
    let nx = value_noise_1d(pos.x + seed as f32 * 1.1);
    let ny = value_noise_1d(pos.y + seed as f32 * 2.3);
    let nz = value_noise_1d(pos.z + seed as f32 * 3.7);
    (nx + ny + nz) / 3.0
}

#[derive(Debug, Clone)]
pub struct NoiseModule {
    pub enabled: bool,
    pub strength: f32,
    pub frequency: f32,
    pub scroll_speed: f32,
    pub damping: bool,
    pub octaves: u32,
    pub octave_multiplier: f32,
    pub octave_scale: f32,
    pub quality: u32,
    elapsed: f32,
}

impl NoiseModule {
    pub fn new() -> Self {
        NoiseModule {
            enabled: false,
            strength: 1.0,
            frequency: 0.5,
            scroll_speed: 0.2,
            damping: true,
            octaves: 1,
            octave_multiplier: 0.5,
            octave_scale: 2.0,
            quality: 1,
            elapsed: 0.0,
        }
    }

    pub fn update_time(&mut self, dt: f32) {
        self.elapsed += dt;
    }

    pub fn evaluate(&self, pos: Vec3, normalized_lifetime: f32) -> Vec3 {
        if !self.enabled { return Vec3::ZERO; }
        let scale = if self.damping { 1.0 - normalized_lifetime } else { 1.0 };
        let mut result = Vec3::ZERO;
        let mut amp = self.strength * scale;
        let mut freq = self.frequency;
        let t_offset = self.elapsed * self.scroll_speed;
        let sample_pos = Vec3::new(
            pos.x * freq + t_offset,
            pos.y * freq + t_offset,
            pos.z * freq + t_offset,
        );
        for oct in 0..self.octaves {
            let seed = oct as u64 * 7919 + 12345;
            let nx = value_noise_3d(Vec3::new(sample_pos.x * (freq / self.frequency), pos.y, pos.z), seed) * 2.0 - 1.0;
            let ny = value_noise_3d(Vec3::new(pos.x, sample_pos.y * (freq / self.frequency), pos.z), seed + 1) * 2.0 - 1.0;
            let nz = value_noise_3d(Vec3::new(pos.x, pos.y, sample_pos.z * (freq / self.frequency)), seed + 2) * 2.0 - 1.0;
            result.x += nx * amp;
            result.y += ny * amp;
            result.z += nz * amp;
            amp *= self.octave_multiplier;
            freq *= self.octave_scale;
        }
        result
    }
}

impl Default for NoiseModule {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct TrailPoint {
    pub position: Vec3,
    pub color: Color,
    pub width: f32,
    pub lifetime: f32,
    pub elapsed: f32,
}

impl TrailPoint {
    fn new(pos: Vec3, color: Color, width: f32, lifetime: f32) -> Self {
        TrailPoint { position: pos, color, width, lifetime, elapsed: 0.0 }
    }

    fn is_expired(&self) -> bool {
        self.elapsed >= self.lifetime
    }

    fn normalized_age(&self) -> f32 {
        (self.elapsed / self.lifetime).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct TrailModule {
    pub enabled: bool,
    pub lifetime: f32,
    pub min_vertex_distance: f32,
    pub world_space: bool,
    pub die_with_particles: bool,
    pub texture_mode: u8,
    pub start_color: Color,
    pub end_color: Color,
    pub start_width: f32,
    pub end_width: f32,
    pub inherit_particle_color: bool,
    pub color_affects_mesh: bool,
    pub width_over_trail: bool,

    trails: Vec<Vec<TrailPoint>>,
}

impl TrailModule {
    pub fn new() -> Self {
        TrailModule {
            enabled: false,
            lifetime: 1.0,
            min_vertex_distance: 0.1,
            world_space: true,
            die_with_particles: true,
            texture_mode: 0,
            start_color: Color::WHITE,
            end_color: Color::new(255, 255, 255, 0),
            start_width: 0.1,
            end_width: 0.0,
            inherit_particle_color: true,
            color_affects_mesh: true,
            width_over_trail: true,
            trails: Vec::new(),
        }
    }

    pub fn add_particle_trail(&mut self) -> usize {
        self.trails.push(Vec::new());
        self.trails.len() - 1
    }

    pub fn remove_particle_trail(&mut self, idx: usize) {
        if idx < self.trails.len() {
            self.trails.remove(idx);
        }
    }

    pub fn emit_point(&mut self, trail_idx: usize, pos: Vec3, color: Color) {
        if !self.enabled || trail_idx >= self.trails.len() {
            return;
        }
        let trail = &mut self.trails[trail_idx];
        if let Some(last) = trail.last() {
            let dx = pos.x - last.position.x;
            let dy = pos.y - last.position.y;
            let dz = pos.z - last.position.z;
            let dist2 = dx * dx + dy * dy + dz * dz;
            if dist2 < self.min_vertex_distance * self.min_vertex_distance {
                return;
            }
        }
        trail.push(TrailPoint::new(pos, color, self.start_width, self.lifetime));
    }

    pub fn update(&mut self, dt: f32) {
        for trail in self.trails.iter_mut() {
            for point in trail.iter_mut() {
                point.elapsed += dt;
                let t = point.normalized_age();
                point.width = self.start_width + (self.end_width - self.start_width) * t;
            }
            trail.retain(|p| !p.is_expired());
        }
    }

    pub fn get_trail(&self, idx: usize) -> Option<&[TrailPoint]> {
        self.trails.get(idx).map(|v| v.as_slice())
    }

    pub fn get_trail_count(&self) -> usize {
        self.trails.len()
    }

    pub fn get_total_points(&self) -> usize {
        self.trails.iter().map(|t| t.len()).sum()
    }

    pub fn clear_all(&mut self) {
        for trail in self.trails.iter_mut() {
            trail.clear();
        }
    }
}

impl Default for TrailModule {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: f32, y: f32, z: f32) -> Vec3 { Vec3::new(x, y, z) }

    #[test]
    fn test_noise_module_disabled_returns_zero() {
        let nm = NoiseModule::new();
        assert!(!nm.enabled);
        let v = nm.evaluate(pos(1.0, 2.0, 3.0), 0.5);
        assert_eq!(v, Vec3::ZERO);
    }

    #[test]
    fn test_noise_module_enabled() {
        let mut nm = NoiseModule::new();
        nm.enabled = true;
        let _v = nm.evaluate(pos(1.0, 2.0, 3.0), 0.5);
    }

    #[test]
    fn test_noise_damping_reduces_at_end() {
        let mut nm = NoiseModule::new();
        nm.enabled = true;
        nm.damping = true;
        nm.strength = 10.0;
        let v_start = nm.evaluate(pos(1.0, 1.0, 1.0), 0.0);
        let v_end = nm.evaluate(pos(1.0, 1.0, 1.0), 1.0);
        let len_start = (v_start.x * v_start.x + v_start.y * v_start.y + v_start.z * v_start.z).sqrt();
        let len_end = (v_end.x * v_end.x + v_end.y * v_end.y + v_end.z * v_end.z).sqrt();
        assert!(len_start >= len_end);
    }

    #[test]
    fn test_noise_octaves() {
        let mut nm = NoiseModule::new();
        nm.enabled = true;
        nm.octaves = 3;
        let v = nm.evaluate(pos(5.0, 5.0, 5.0), 0.5);
        assert!(v.x.is_finite() && v.y.is_finite() && v.z.is_finite());
    }

    #[test]
    fn test_noise_scroll_changes_value() {
        let mut nm = NoiseModule::new();
        nm.enabled = true;
        nm.scroll_speed = 1.0;
        let v1 = nm.evaluate(pos(0.0, 0.0, 0.0), 0.5);
        nm.update_time(1.0);
        let v2 = nm.evaluate(pos(0.0, 0.0, 0.0), 0.5);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_trail_module_new() {
        let tm = TrailModule::new();
        assert!(!tm.enabled);
        assert_eq!(tm.get_trail_count(), 0);
        assert_eq!(tm.get_total_points(), 0);
    }

    #[test]
    fn test_trail_add_remove() {
        let mut tm = TrailModule::new();
        tm.enabled = true;
        let idx = tm.add_particle_trail();
        assert_eq!(tm.get_trail_count(), 1);
        tm.remove_particle_trail(idx);
        assert_eq!(tm.get_trail_count(), 0);
    }

    #[test]
    fn test_trail_emit_point() {
        let mut tm = TrailModule::new();
        tm.enabled = true;
        tm.min_vertex_distance = 0.01;
        let idx = tm.add_particle_trail();
        tm.emit_point(idx, pos(0.0, 0.0, 0.0), Color::WHITE);
        tm.emit_point(idx, pos(1.0, 0.0, 0.0), Color::WHITE);
        assert_eq!(tm.get_trail(idx).unwrap().len(), 2);
    }

    #[test]
    fn test_trail_min_distance_filter() {
        let mut tm = TrailModule::new();
        tm.enabled = true;
        tm.min_vertex_distance = 1.0;
        let idx = tm.add_particle_trail();
        tm.emit_point(idx, pos(0.0, 0.0, 0.0), Color::WHITE);
        tm.emit_point(idx, pos(0.1, 0.0, 0.0), Color::WHITE);
        assert_eq!(tm.get_trail(idx).unwrap().len(), 1);
    }

    #[test]
    fn test_trail_points_expire() {
        let mut tm = TrailModule::new();
        tm.enabled = true;
        tm.lifetime = 0.5;
        tm.min_vertex_distance = 0.0;
        let idx = tm.add_particle_trail();
        tm.emit_point(idx, pos(0.0, 0.0, 0.0), Color::WHITE);
        tm.update(0.6);
        assert_eq!(tm.get_trail(idx).unwrap().len(), 0);
    }

    #[test]
    fn test_trail_width_interpolates() {
        let mut tm = TrailModule::new();
        tm.enabled = true;
        tm.start_width = 1.0;
        tm.end_width = 0.0;
        tm.lifetime = 1.0;
        tm.min_vertex_distance = 0.0;
        let idx = tm.add_particle_trail();
        tm.emit_point(idx, pos(0.0, 0.0, 0.0), Color::WHITE);
        tm.update(0.5);
        let trail = tm.get_trail(idx).unwrap();
        assert!((trail[0].width - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_trail_disabled_no_emit() {
        let mut tm = TrailModule::new();
        let idx = tm.add_particle_trail();
        tm.emit_point(idx, pos(0.0, 0.0, 0.0), Color::WHITE);
        assert_eq!(tm.get_trail(idx).unwrap().len(), 0);
    }

    #[test]
    fn test_trail_clear_all() {
        let mut tm = TrailModule::new();
        tm.enabled = true;
        tm.min_vertex_distance = 0.0;
        let idx = tm.add_particle_trail();
        tm.emit_point(idx, pos(0.0, 0.0, 0.0), Color::WHITE);
        tm.emit_point(idx, pos(1.0, 0.0, 0.0), Color::WHITE);
        tm.clear_all();
        assert_eq!(tm.get_total_points(), 0);
    }

    #[test]
    fn test_trail_multiple_trails() {
        let mut tm = TrailModule::new();
        tm.enabled = true;
        tm.min_vertex_distance = 0.0;
        let idx0 = tm.add_particle_trail();
        let idx1 = tm.add_particle_trail();
        tm.emit_point(idx0, pos(0.0, 0.0, 0.0), Color::WHITE);
        tm.emit_point(idx1, pos(5.0, 0.0, 0.0), Color::WHITE);
        tm.emit_point(idx1, pos(6.0, 0.0, 0.0), Color::WHITE);
        assert_eq!(tm.get_total_points(), 3);
    }
}
