use crate::math::Vec2;

pub fn point_in_aabb(point: &Vec2, min: &Vec2, max: &Vec2) -> bool {
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

pub fn aabb_overlap(min_a: &Vec2, max_a: &Vec2, min_b: &Vec2, max_b: &Vec2) -> bool {
    min_a.x <= max_b.x && max_a.x >= min_b.x && min_a.y <= max_b.y && max_a.y >= min_b.y
}

pub fn circle_circle_intersect(
    center_a: &Vec2, radius_a: f32, center_b: &Vec2, radius_b: f32,
) -> bool {
    let dx = center_a.x - center_b.x;
    let dy = center_a.y - center_b.y;
    let dist_sq = dx * dx + dy * dy;
    let radii = radius_a + radius_b;
    dist_sq <= radii * radii
}

pub fn aabb_circle_intersect(
    min: &Vec2, max: &Vec2, center: &Vec2, radius: f32,
) -> bool {
    let closest_x = center.x.clamp(min.x, max.x);
    let closest_y = center.y.clamp(min.y, max.y);
    let dx = center.x - closest_x;
    let dy = center.y - closest_y;
    dx * dx + dy * dy <= radius * radius
}

pub fn box_box_intersect(
    min_a: &Vec2, max_a: &Vec2, min_b: &Vec2, max_b: &Vec2,
) -> bool {
    aabb_overlap(min_a, max_a, min_b, max_b)
}

pub fn ray_segment_intersection(
    ray_origin: &Vec2, ray_dir: &Vec2, seg_start: &Vec2, seg_end: &Vec2,
) -> Option<(f32, Vec2)> {
    let seg = *seg_end - *seg_start;
    let denom = ray_dir.x * seg.y - ray_dir.y * seg.x;
    if denom.abs() < 1e-10 {
        return None;
    }
    let t = ((seg_start.x - ray_origin.x) * seg.y - (seg_start.y - ray_origin.y) * seg.x) / denom;
    let u = ((seg_start.x - ray_origin.x) * ray_dir.y - (seg_start.y - ray_origin.y) * ray_dir.x) / denom;
    if t < 0.0 || !(0.0..=1.0).contains(&u) {
        return None;
    }
    let point = *ray_origin + *ray_dir * t;
    Some((t, point))
}

pub fn closest_point_on_segment(point: &Vec2, seg_start: &Vec2, seg_end: &Vec2) -> (f32, Vec2) {
    let seg = *seg_end - *seg_start;
    let len_sq = seg.x * seg.x + seg.y * seg.y;
    if len_sq < 1e-10 {
        return (0.0, *seg_start);
    }
    let t = ((point.x - seg_start.x) * seg.x + (point.y - seg_start.y) * seg.y) / len_sq;
    let t_clamped = t.clamp(0.0, 1.0);
    let closest = *seg_start + seg * t_clamped;
    (t_clamped, closest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_aabb() {
        assert!(point_in_aabb(
            &Vec2::new(0.0, 0.0),
            &Vec2::new(-1.0, -1.0),
            &Vec2::new(1.0, 1.0),
        ));
        assert!(!point_in_aabb(
            &Vec2::new(2.0, 2.0),
            &Vec2::new(-1.0, -1.0),
            &Vec2::new(1.0, 1.0),
        ));
    }

    #[test]
    fn test_aabb_overlap() {
        assert!(aabb_overlap(
            &Vec2::new(0.0, 0.0), &Vec2::new(2.0, 2.0),
            &Vec2::new(1.0, 1.0), &Vec2::new(3.0, 3.0),
        ));
        assert!(!aabb_overlap(
            &Vec2::new(0.0, 0.0), &Vec2::new(1.0, 1.0),
            &Vec2::new(2.0, 2.0), &Vec2::new(3.0, 3.0),
        ));
    }

    #[test]
    fn test_circle_circle_intersect() {
        assert!(circle_circle_intersect(
            &Vec2::new(0.0, 0.0), 1.0, &Vec2::new(1.0, 0.0), 1.0,
        ));
        assert!(!circle_circle_intersect(
            &Vec2::new(0.0, 0.0), 1.0, &Vec2::new(3.0, 0.0), 1.0,
        ));
    }

    #[test]
    fn test_aabb_circle_intersect() {
        assert!(aabb_circle_intersect(
            &Vec2::new(-1.0, -1.0), &Vec2::new(1.0, 1.0),
            &Vec2::new(0.0, 0.0), 1.5,
        ));
    }

    #[test]
    fn test_ray_intersection() {
        let result = ray_segment_intersection(
            &Vec2::new(0.0, 0.0), &Vec2::new(1.0, 0.0),
            &Vec2::new(0.5, -1.0), &Vec2::new(0.5, 1.0),
        );
        assert!(result.is_some());
        let (_t, point) = result.unwrap();
        assert!((point.x - 0.5).abs() < 0.001);
        assert!(point.y.abs() < 0.001);
    }

    #[test]
    fn test_ray_no_intersection() {
        let result = ray_segment_intersection(
            &Vec2::new(0.0, 0.0), &Vec2::new(1.0, 0.0),
            &Vec2::new(0.5, 1.0), &Vec2::new(1.5, 2.0),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_closest_point() {
        let (_t, point) = closest_point_on_segment(
            &Vec2::new(0.5, 1.0),
            &Vec2::new(0.0, 0.0),
            &Vec2::new(1.0, 0.0),
        );
        assert!((point.x - 0.5).abs() < 0.01);
        assert!(point.y.abs() < 0.01);
    }
}
