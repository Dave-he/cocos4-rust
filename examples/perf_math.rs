use std::hint::black_box;
use std::time::Instant;

use cocos4_rust::{Mat4, Vec3};

fn iterations() -> usize {
    std::env::var("ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1_000_000)
}

fn bench_vec3(iterations: usize) -> (f64, f32) {
    let matrix = Mat4::new(
        1.25, 0.10, 0.05, 0.00, 0.20, 0.90, 0.15, 0.00, 0.05, 0.25, 1.10, 0.00, 4.0, 5.0, 6.0, 1.00,
    );
    let mut acc = 0.0f32;
    let start = Instant::now();

    for i in 0..iterations {
        let t = i as f32 * 0.001;
        let a = Vec3::new(t.sin() + 1.0, t.cos() + 2.0, t * 0.5 + 3.0);
        let b = Vec3::new(t * 0.25 + 4.0, t.sin() * 2.0 + 5.0, t.cos() * 3.0 + 6.0);
        let mut c = Vec3::cross_vecs(black_box(&a), black_box(&b));
        c.add(&Vec3::add_vecs(&a, &b));
        c.normalize();
        let transformed = c.transform_mat4(&matrix);
        acc += transformed.dot(&a) + Vec3::dot_vecs(&a, &b);
    }

    (start.elapsed().as_secs_f64() * 1000.0, black_box(acc))
}

fn bench_mat4(iterations: usize) -> (f64, f32) {
    let a = Mat4::new(
        1.0, 0.2, 0.3, 0.0, 0.1, 1.1, 0.4, 0.0, 0.2, 0.3, 0.9, 0.0, 3.0, 4.0, 5.0, 1.0,
    );
    let b = Mat4::new(
        0.9, 0.3, 0.1, 0.0, 0.4, 1.0, 0.2, 0.0, 0.1, 0.5, 1.2, 0.0, 6.0, 7.0, 8.0, 1.0,
    );
    let mut out = Mat4::ZERO;
    let mut acc = 0.0f32;
    let start = Instant::now();

    for _ in 0..iterations {
        Mat4::multiply(black_box(&a), black_box(&b), &mut out);
        let inv = out.get_inverted();
        acc += inv.m[0] + inv.m[5] + inv.m[10] + inv.m[15];
    }

    (start.elapsed().as_secs_f64() * 1000.0, black_box(acc))
}

fn main() {
    let iterations = iterations();
    let mat_iterations = (iterations / 10).max(1);
    let (vec3_ms, vec3_checksum) = bench_vec3(iterations);
    let (mat4_ms, mat4_checksum) = bench_mat4(mat_iterations);

    println!("engine=cocos4-rust");
    println!("iterations={}", iterations);
    println!("mat4_iterations={}", mat_iterations);
    println!("vec3_hot_path_ms={:.3}", vec3_ms);
    println!("mat4_hot_path_ms={:.3}", mat4_ms);
    println!("checksum={:.6}", vec3_checksum + mat4_checksum);
}
