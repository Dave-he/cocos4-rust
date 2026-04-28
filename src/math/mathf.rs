pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

pub fn clamp01(v: f32) -> f32 {
    clamp(v, 0.0, 1.0)
}

pub fn remap(v: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = (v - in_min) / (in_max - in_min);
    lerp(out_min, out_max, t)
}

pub fn smooth_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp01((x - edge0) / (edge1 - edge0));
    t * t * (3.0 - 2.0 * t)
}

pub fn smoother_step(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp01((x - edge0) / (edge1 - edge0));
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub fn ping_pong(t: f32, length: f32) -> f32 {
    if length == 0.0 {
        return 0.0;
    }
    let t = t % (length * 2.0);
    if t <= length {
        t
    } else {
        length * 2.0 - t
    }
}

pub fn repeat(t: f32, length: f32) -> f32 {
    if length == 0.0 {
        return 0.0;
    }
    t - (t / length).floor() * length
}

pub fn approximately(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-6
}

pub fn sign(v: f32) -> f32 {
    if v > 0.0 { 1.0 } else if v < 0.0 { -1.0 } else { 0.0 }
}

pub fn deg_to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / std::f32::consts::PI
}

pub fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let diff = target - current;
    if diff.abs() <= max_delta {
        target
    } else {
        current + sign(diff) * max_delta
    }
}

pub fn pow_f32(base: f32, exp: f32) -> f32 {
    base.powf(exp)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Vec2 {
            x: lerp(a.x, b.x, t),
            y: lerp(a.y, b.y, t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezierCurve2D {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
}

impl BezierCurve2D {
    pub fn new(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Self {
        BezierCurve2D { p0, p1, p2, p3 }
    }

    pub fn quadratic(p0: Vec2, p1: Vec2, p2: Vec2) -> Self {
        BezierCurve2D { p0, p1, p2: p1, p3: p2 }
    }

    pub fn evaluate(&self, t: f32) -> Vec2 {
        let t = clamp01(t);
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let t2 = t * t;
        let t3 = t2 * t;
        Vec2 {
            x: mt3 * self.p0.x + 3.0 * mt2 * t * self.p1.x + 3.0 * mt * t2 * self.p2.x + t3 * self.p3.x,
            y: mt3 * self.p0.y + 3.0 * mt2 * t * self.p1.y + 3.0 * mt * t2 * self.p2.y + t3 * self.p3.y,
        }
    }

    pub fn derivative(&self, t: f32) -> Vec2 {
        let t = clamp01(t);
        let mt = 1.0 - t;
        Vec2 {
            x: 3.0 * (mt * mt * (self.p1.x - self.p0.x) + 2.0 * mt * t * (self.p2.x - self.p1.x) + t * t * (self.p3.x - self.p2.x)),
            y: 3.0 * (mt * mt * (self.p1.y - self.p0.y) + 2.0 * mt * t * (self.p2.y - self.p1.y) + t * t * (self.p3.y - self.p2.y)),
        }
    }

    pub fn arc_length(&self, segments: u32) -> f32 {
        let n = segments.max(1);
        let mut length = 0.0f32;
        let mut prev = self.evaluate(0.0);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            let curr = self.evaluate(t);
            let dx = curr.x - prev.x;
            let dy = curr.y - prev.y;
            length += (dx * dx + dy * dy).sqrt();
            prev = curr;
        }
        length
    }

    pub fn sample_uniform(&self, count: usize) -> Vec<Vec2> {
        (0..count).map(|i| self.evaluate(i as f32 / (count - 1).max(1) as f32)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct AnimationCurve {
    keyframes: Vec<(f32, f32)>,
}

impl AnimationCurve {
    pub fn new() -> Self {
        AnimationCurve { keyframes: Vec::new() }
    }

    pub fn linear(from: f32, to: f32) -> Self {
        let mut c = Self::new();
        c.add_key(0.0, from);
        c.add_key(1.0, to);
        c
    }

    pub fn add_key(&mut self, time: f32, value: f32) {
        let pos = self.keyframes.partition_point(|&(t, _)| t < time);
        self.keyframes.insert(pos, (time, value));
    }

    pub fn evaluate(&self, time: f32) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }
        if self.keyframes.len() == 1 {
            return self.keyframes[0].1;
        }
        if time <= self.keyframes[0].0 {
            return self.keyframes[0].1;
        }
        let last = self.keyframes.last().unwrap();
        if time >= last.0 {
            return last.1;
        }
        let idx = self.keyframes.partition_point(|&(t, _)| t <= time);
        let (t0, v0) = self.keyframes[idx - 1];
        let (t1, v1) = self.keyframes[idx];
        let t = (time - t0) / (t1 - t0);
        lerp(v0, v1, t)
    }

    pub fn key_count(&self) -> usize {
        self.keyframes.len()
    }
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::new()
    }
}

pub fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

pub fn hermite(p0: f32, m0: f32, p1: f32, m1: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 1e-6);
        assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < 1e-6);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-1.0, 0.0, 1.0), 0.0);
        assert_eq!(clamp(2.0, 0.0, 1.0), 1.0);
    }

    #[test]
    fn test_clamp01() {
        assert_eq!(clamp01(0.5), 0.5);
        assert_eq!(clamp01(-0.5), 0.0);
        assert_eq!(clamp01(1.5), 1.0);
    }

    #[test]
    fn test_remap() {
        let v = remap(5.0, 0.0, 10.0, 0.0, 100.0);
        assert!((v - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_smooth_step() {
        assert!(smooth_step(0.0, 1.0, 0.0).abs() < 1e-6);
        assert!((smooth_step(0.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
        let mid = smooth_step(0.0, 1.0, 0.5);
        assert!((mid - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_ping_pong() {
        assert!((ping_pong(0.0, 1.0) - 0.0).abs() < 1e-6);
        assert!((ping_pong(0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((ping_pong(1.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((ping_pong(1.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((ping_pong(2.0, 1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_repeat() {
        assert!((repeat(0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((repeat(1.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((repeat(2.5, 1.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_move_towards() {
        assert!((move_towards(0.0, 10.0, 3.0) - 3.0).abs() < 1e-6);
        assert!((move_towards(0.0, 10.0, 20.0) - 10.0).abs() < 1e-6);
        assert!((move_towards(5.0, 3.0, 1.0) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_deg_rad() {
        let pi = std::f32::consts::PI;
        assert!((deg_to_rad(180.0) - pi).abs() < 1e-5);
        assert!((rad_to_deg(pi) - 180.0).abs() < 1e-4);
    }

    #[test]
    fn test_bezier_endpoints() {
        let curve = BezierCurve2D::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(3.0, 0.0),
        );
        let p0 = curve.evaluate(0.0);
        let p1 = curve.evaluate(1.0);
        assert!((p0.x - 0.0).abs() < 1e-5 && (p0.y - 0.0).abs() < 1e-5);
        assert!((p1.x - 3.0).abs() < 1e-5 && (p1.y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_bezier_midpoint() {
        let curve = BezierCurve2D::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
        );
        let mid = curve.evaluate(0.5);
        assert!(mid.x >= 0.0 && mid.x <= 1.0);
        assert!(mid.y > 0.0);
    }

    #[test]
    fn test_bezier_arc_length() {
        let curve = BezierCurve2D::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(3.0, 0.0),
        );
        let len = curve.arc_length(100);
        assert!((len - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_bezier_sample_uniform() {
        let curve = BezierCurve2D::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(3.0, 0.0),
        );
        let pts = curve.sample_uniform(5);
        assert_eq!(pts.len(), 5);
        assert!((pts[0].x - 0.0).abs() < 1e-5);
        assert!((pts[4].x - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_animation_curve_linear() {
        let c = AnimationCurve::linear(0.0, 10.0);
        assert!((c.evaluate(0.0) - 0.0).abs() < 1e-5);
        assert!((c.evaluate(0.5) - 5.0).abs() < 1e-5);
        assert!((c.evaluate(1.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_animation_curve_clamp_edges() {
        let c = AnimationCurve::linear(5.0, 10.0);
        assert!((c.evaluate(-1.0) - 5.0).abs() < 1e-5);
        assert!((c.evaluate(2.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_animation_curve_multi_keys() {
        let mut c = AnimationCurve::new();
        c.add_key(0.0, 0.0);
        c.add_key(0.5, 10.0);
        c.add_key(1.0, 0.0);
        assert!((c.evaluate(0.25) - 5.0).abs() < 1e-5);
        assert!((c.evaluate(0.75) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_catmull_rom() {
        let v = catmull_rom(0.0, 0.0, 1.0, 1.0, 0.0);
        assert!((v - 0.0).abs() < 1e-5);
        let v = catmull_rom(0.0, 0.0, 1.0, 1.0, 1.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_hermite() {
        let v = hermite(0.0, 0.0, 1.0, 0.0, 0.0);
        assert!((v - 0.0).abs() < 1e-5);
        let v = hermite(0.0, 0.0, 1.0, 0.0, 1.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_approximately() {
        assert!(approximately(1.0, 1.0 + 1e-7));
        assert!(!approximately(1.0, 1.1));
    }

    #[test]
    fn test_sign() {
        assert!((sign(5.0) - 1.0).abs() < 1e-6);
        assert!((sign(-3.0) - (-1.0)).abs() < 1e-6);
        assert!((sign(0.0)).abs() < 1e-6);
    }
}
