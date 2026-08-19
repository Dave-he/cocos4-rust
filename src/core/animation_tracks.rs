use crate::math::{Quaternion, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackInterpolation {
    Step,
    #[default]
    Linear,
    Cubic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackType {
    Real,
    Quaternion,
    Vector3,
    Color,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct Keyframe<T: Clone + Default> {
    pub time: f32,
    pub value: T,
    pub interp: TrackInterpolation,
    pub in_tangent: Option<f32>,
    pub out_tangent: Option<f32>,
}

impl<T: Clone + Default> Keyframe<T> {
    pub fn new(time: f32, value: T) -> Self {
        Self { time, value, interp: TrackInterpolation::Linear, in_tangent: None, out_tangent: None }
    }

    pub fn step(time: f32, value: T) -> Self {
        Self { time, value, interp: TrackInterpolation::Step, in_tangent: None, out_tangent: None }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RealTrack {
    pub keyframes: Vec<Keyframe<f32>>,
}

impl RealTrack {
    pub fn new() -> Self { Self { keyframes: Vec::new() } }

    pub fn add_keyframe(&mut self, kf: Keyframe<f32>) {
        self.keyframes.push(kf);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn sample(&self, time: f32) -> f32 {
        if self.keyframes.is_empty() { return 0.0; }
        if self.keyframes.len() == 1 { return self.keyframes[0].value; }
        if time <= self.keyframes[0].time { return self.keyframes[0].value; }
        if time >= self.keyframes.last().unwrap().time { return self.keyframes.last().unwrap().value; }

        let mut prev = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= time { prev = i; }
            else { break; }
        }
        let next = prev + 1;
        let pkf = &self.keyframes[prev];
        let nkf = &self.keyframes[next];

        match pkf.interp {
            TrackInterpolation::Step => pkf.value,
            TrackInterpolation::Linear => {
                let alpha = (time - pkf.time) / (nkf.time - pkf.time);
                pkf.value * (1.0 - alpha) + nkf.value * alpha
            }
            TrackInterpolation::Cubic => {
                let alpha = (time - pkf.time) / (nkf.time - pkf.time);
                if let (Some(t0), Some(t1)) = (pkf.out_tangent, nkf.in_tangent) {
                    let dt = nkf.time - pkf.time;
                    cubic_hermite(alpha, pkf.value, t0 * dt, nkf.value, t1 * dt)
                } else {
                    let alpha2 = alpha * alpha;
                    pkf.value * (1.0 - alpha2) + nkf.value * alpha2
                }
            }
        }
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|kf| kf.time).unwrap_or(0.0)
    }
}

fn cubic_hermite(t: f32, p0: f32, m0: f32, p1: f32, m1: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0 + (t3 - 2.0 * t2 + t) * m0 + (-2.0 * t3 + 3.0 * t2) * p1 + (t3 - t2) * m1
}

#[derive(Debug, Clone, Default)]
pub struct QuaternionTrack {
    pub keyframes: Vec<Keyframe<Quaternion>>,
}

impl QuaternionTrack {
    pub fn new() -> Self { Self { keyframes: Vec::new() } }

    pub fn add_keyframe(&mut self, kf: Keyframe<Quaternion>) {
        self.keyframes.push(kf);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn sample(&self, time: f32) -> Quaternion {
        if self.keyframes.is_empty() { return Quaternion::IDENTITY; }
        if self.keyframes.len() == 1 { return self.keyframes[0].value; }
        if time <= self.keyframes[0].time { return self.keyframes[0].value; }
        if time >= self.keyframes.last().unwrap().time { return self.keyframes.last().unwrap().value; }

        let mut prev = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= time { prev = i; }
            else { break; }
        }
        let next = prev + 1;
        let pkf = &self.keyframes[prev];
        let nkf = &self.keyframes[next];

        match pkf.interp {
            TrackInterpolation::Step => pkf.value,
            TrackInterpolation::Linear | TrackInterpolation::Cubic => {
                let alpha = (time - pkf.time) / (nkf.time - pkf.time);
                Quaternion::slerp(&pkf.value, &nkf.value, alpha)
            }
        }
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|kf| kf.time).unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Vector3Track {
    pub keyframes: Vec<Keyframe<Vec3>>,
}

impl Vector3Track {
    pub fn new() -> Self { Self { keyframes: Vec::new() } }

    pub fn add_keyframe(&mut self, kf: Keyframe<Vec3>) {
        self.keyframes.push(kf);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn sample(&self, time: f32) -> Vec3 {
        if self.keyframes.is_empty() { return Vec3::ZERO; }
        if self.keyframes.len() == 1 { return self.keyframes[0].value; }
        if time <= self.keyframes[0].time { return self.keyframes[0].value; }
        if time >= self.keyframes.last().unwrap().time { return self.keyframes.last().unwrap().value; }

        let mut prev = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= time { prev = i; }
            else { break; }
        }
        let next = prev + 1;
        let pkf = &self.keyframes[prev];
        let nkf = &self.keyframes[next];

        match pkf.interp {
            TrackInterpolation::Step => pkf.value,
            TrackInterpolation::Linear => {
                let alpha = (time - pkf.time) / (nkf.time - pkf.time);
                let inv = 1.0 - alpha;
                Vec3::new(
                    pkf.value.x * inv + nkf.value.x * alpha,
                    pkf.value.y * inv + nkf.value.y * alpha,
                    pkf.value.z * inv + nkf.value.z * alpha,
                )
            }
            TrackInterpolation::Cubic => {
                let alpha = (time - pkf.time) / (nkf.time - pkf.time);
                Vec3::new(
                    cubic_hermite(alpha, pkf.value.x, 0.0, nkf.value.x, 0.0),
                    cubic_hermite(alpha, pkf.value.y, 0.0, nkf.value.y, 0.0),
                    cubic_hermite(alpha, pkf.value.z, 0.0, nkf.value.z, 0.0),
                )
            }
        }
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|kf| kf.time).unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ColorTrack {
    pub keyframes: Vec<Keyframe<[u8; 4]>>,
}

impl ColorTrack {
    pub fn new() -> Self { Self { keyframes: Vec::new() } }

    pub fn add_keyframe(&mut self, kf: Keyframe<[u8; 4]>) {
        self.keyframes.push(kf);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn sample(&self, time: f32) -> [u8; 4] {
        if self.keyframes.is_empty() { return [255, 255, 255, 255]; }
        if self.keyframes.len() == 1 { return self.keyframes[0].value; }
        if time <= self.keyframes[0].time { return self.keyframes[0].value; }
        if time >= self.keyframes.last().unwrap().time { return self.keyframes.last().unwrap().value; }

        let mut prev = 0;
        for (i, kf) in self.keyframes.iter().enumerate() {
            if kf.time <= time { prev = i; }
            else { break; }
        }
        let next = prev + 1;
        let pkf = &self.keyframes[prev];
        let nkf = &self.keyframes[next];

        match pkf.interp {
            TrackInterpolation::Step => pkf.value,
            TrackInterpolation::Linear | TrackInterpolation::Cubic => {
                let alpha = (time - pkf.time) / (nkf.time - pkf.time);
                let inv = 1.0 - alpha;
                [
                    (pkf.value[0] as f32 * inv + nkf.value[0] as f32 * alpha) as u8,
                    (pkf.value[1] as f32 * inv + nkf.value[1] as f32 * alpha) as u8,
                    (pkf.value[2] as f32 * inv + nkf.value[2] as f32 * alpha) as u8,
                    (pkf.value[3] as f32 * inv + nkf.value[3] as f32 * alpha) as u8,
                ]
            }
        }
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|kf| kf.time).unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct BooleanTrack {
    pub keyframes: Vec<Keyframe<bool>>,
}

impl BooleanTrack {
    pub fn new() -> Self { Self { keyframes: Vec::new() } }

    pub fn add_keyframe(&mut self, kf: Keyframe<bool>) {
        self.keyframes.push(kf);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    pub fn sample(&self, time: f32) -> bool {
        if self.keyframes.is_empty() { return false; }
        let mut result = self.keyframes[0].value;
        for kf in &self.keyframes {
            if kf.time <= time {
                result = kf.value;
            } else {
                break;
            }
        }
        result
    }

    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|kf| kf.time).unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PropertyPath {
    pub target_name: String,
    pub component_type: String,
    pub property_name: String,
    pub array_index: Option<usize>,
}

impl PropertyPath {
    pub fn new(target: &str, property: &str) -> Self {
        Self {
            target_name: target.to_string(),
            component_type: String::new(),
            property_name: property.to_string(),
            array_index: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TrackBinding {
    Real(RealTrack),
    Quaternion(QuaternionTrack),
    Vector3(Vector3Track),
    Color(ColorTrack),
    Boolean(BooleanTrack),
}

impl TrackBinding {
    pub fn duration(&self) -> f32 {
        match self {
            TrackBinding::Real(t) => t.duration(),
            TrackBinding::Quaternion(t) => t.duration(),
            TrackBinding::Vector3(t) => t.duration(),
            TrackBinding::Color(t) => t.duration(),
            TrackBinding::Boolean(t) => t.duration(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnimationTrackBinding {
    pub path: PropertyPath,
    pub track: Option<TrackBinding>,
}

impl AnimationTrackBinding {
    pub fn new(target: &str, property: &str) -> Self {
        Self {
            path: PropertyPath::new(target, property),
            track: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_track_linear() {
        let mut t = RealTrack::new();
        t.add_keyframe(Keyframe::new(0.0, 0.0));
        t.add_keyframe(Keyframe::new(1.0, 100.0));
        assert!((t.sample(0.5) - 50.0).abs() < 1e-4);
        assert!((t.sample(0.0) - 0.0).abs() < 1e-4);
        assert!((t.sample(1.0) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn test_real_track_step() {
        let mut t = RealTrack::new();
        t.add_keyframe(Keyframe::step(0.0, 0.0));
        t.add_keyframe(Keyframe::step(1.0, 100.0));
        assert_eq!(t.sample(0.5), 0.0);
        assert_eq!(t.sample(1.0), 100.0);
    }

    #[test]
    fn test_real_track_multiple_keyframes() {
        let mut t = RealTrack::new();
        t.add_keyframe(Keyframe::new(0.0, 0.0));
        t.add_keyframe(Keyframe::new(1.0, 50.0));
        t.add_keyframe(Keyframe::new(2.0, 200.0));
        assert!((t.sample(0.5) - 25.0).abs() < 1e-4);
        assert!((t.sample(1.5) - 125.0).abs() < 1e-4);
    }

    #[test]
    fn test_real_track_empty() {
        let t = RealTrack::new();
        assert_eq!(t.sample(1.0), 0.0);
        assert_eq!(t.duration(), 0.0);
    }

    #[test]
    fn test_real_track_cubic() {
        let mut t = RealTrack::new();
        let mut kf0 = Keyframe::new(0.0, 0.0);
        kf0.interp = TrackInterpolation::Cubic;
        kf0.out_tangent = Some(1.0);
        let mut kf1 = Keyframe::new(1.0, 100.0);
        kf1.in_tangent = Some(1.0);
        t.add_keyframe(kf0);
        t.add_keyframe(kf1);
        let v = t.sample(0.5);
        assert!(v > 0.0 && v < 100.0);
    }

    #[test]
    fn test_quaternion_track_slerp() {
        let mut t = QuaternionTrack::new();
        t.add_keyframe(Keyframe::new(0.0, Quaternion::IDENTITY));
        let q90 = Quaternion::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), 90.0_f32.to_radians());
        t.add_keyframe(Keyframe::new(1.0, q90));
        let q = t.sample(0.5);
        let (_, angle) = q.get_axis_angle(); let angle = angle.to_degrees();
        assert!((angle - 45.0).abs() < 2.0);
    }

    #[test]
    fn test_quaternion_track_empty() {
        let t = QuaternionTrack::new();
        assert_eq!(t.sample(1.0), Quaternion::IDENTITY);
    }

    #[test]
    fn test_quaternion_track_step() {
        let mut t = QuaternionTrack::new();
        t.add_keyframe(Keyframe::step(0.0, Quaternion::IDENTITY));
        let q90 = Quaternion::from_axis_angle(&Vec3::new(0.0, 1.0, 0.0), 90.0_f32.to_radians());
        t.add_keyframe(Keyframe::step(1.0, q90));
        assert_eq!(t.sample(0.5), Quaternion::IDENTITY);
    }

    #[test]
    fn test_vector3_track_linear() {
        let mut t = Vector3Track::new();
        t.add_keyframe(Keyframe::new(0.0, Vec3::new(0.0, 0.0, 0.0)));
        t.add_keyframe(Keyframe::new(1.0, Vec3::new(100.0, 200.0, 300.0)));
        let v = t.sample(0.5);
        assert!((v.x - 50.0).abs() < 1e-4);
        assert!((v.y - 100.0).abs() < 1e-4);
        assert!((v.z - 150.0).abs() < 1e-4);
    }

    #[test]
    fn test_vector3_track_step() {
        let mut t = Vector3Track::new();
        t.add_keyframe(Keyframe::step(0.0, Vec3::new(0.0, 0.0, 0.0)));
        t.add_keyframe(Keyframe::new(1.0, Vec3::new(100.0, 200.0, 300.0)));
        assert_eq!(t.sample(0.5), Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_vector3_track_empty() {
        let t = Vector3Track::new();
        assert_eq!(t.sample(1.0), Vec3::ZERO);
    }

    #[test]
    fn test_color_track_linear() {
        let mut t = ColorTrack::new();
        t.add_keyframe(Keyframe::new(0.0, [0, 0, 0, 255]));
        t.add_keyframe(Keyframe::new(1.0, [100, 200, 50, 255]));
        let c = t.sample(0.5);
        assert_eq!(c[0], 50);
        assert_eq!(c[1], 100);
        assert_eq!(c[2], 25);
    }

    #[test]
    fn test_color_track_empty() {
        let t = ColorTrack::new();
        assert_eq!(t.sample(1.0), [255, 255, 255, 255]);
    }

    #[test]
    fn test_boolean_track() {
        let mut t = BooleanTrack::new();
        t.add_keyframe(Keyframe::new(0.0, false));
        t.add_keyframe(Keyframe::new(1.0, true));
        t.add_keyframe(Keyframe::new(2.0, false));
        assert!(!t.sample(0.5));
        assert!(t.sample(1.5));
        assert!(!t.sample(2.5));
    }

    #[test]
    fn test_boolean_track_empty() {
        let t = BooleanTrack::new();
        assert!(!t.sample(1.0));
    }

    #[test]
    fn test_property_path() {
        let p = PropertyPath::new("node1", "position");
        assert_eq!(p.target_name, "node1");
        assert_eq!(p.property_name, "position");
    }

    #[test]
    fn test_track_binding_duration() {
        let mut rt = RealTrack::new();
        rt.add_keyframe(Keyframe::new(0.0, 1.0));
        rt.add_keyframe(Keyframe::new(2.5, 3.0));
        let binding = TrackBinding::Real(rt);
        assert!((binding.duration() - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_animation_track_binding() {
        let mut b = AnimationTrackBinding::new("root", "position");
        b.track = Some(TrackBinding::Vector3({
            let mut t = Vector3Track::new();
            t.add_keyframe(Keyframe::new(0.0, Vec3::ZERO));
            t.add_keyframe(Keyframe::new(1.0, Vec3::new(1.0, 2.0, 3.0)));
            t
        }));
        assert_eq!(b.path.target_name, "root");
        assert_eq!(b.path.property_name, "position");
        if let Some(TrackBinding::Vector3(t)) = &b.track {
            let v = t.sample(0.5);
            assert!((v.x - 0.5).abs() < 1e-4);
        }
    }
}
