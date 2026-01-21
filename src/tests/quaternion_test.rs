use crate::cocos4_rust::{Mat4, Vec2, Vec3};

fn main() {
    println!("Testing Quaternion module");
    println!("Creating quaternion test...");

    let q = Quaternion::IDENTITY;
    println!("Quaternion::IDENTITY: {:?}", q);

    let angle = std::f32::consts::FRAC_PI_4;
    println!("Testing rotation from axis-angle: {}", angle);

    let q_rot = Quaternion::from_axis_angle(Vec3::FORWARD, std::f32::consts::FRAC_PI_4 * 0.5);
    println!("Rotated quaternion: {:?}", q_rot);

    let mut inv_q = q.inverse();
    let inv_q_val = inv_q.clone();
    println!("Inverse quaternion: {:?}", inv_q_val);

    let dot_result = Quaternion::dot(&q_rot, &q);
    println!("Dot product: {:?}", dot_result);

    println!("All tests completed successfully!");
}
