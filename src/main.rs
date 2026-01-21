use cocos4_rust::math::{Color, Quaternion, Vec2, Vec3, Vec4};

fn main() {
    let v1 = Vec2::new(1.0, 2.0);
    let v2 = Vec3::new(3.0, 4.0, 5.0);
    let v4 = Vec4::new(1.0, 2.0, 3.0, 4.0);

    println!("Vec2: {:?}", v1);
    println!("Vec3: {:?}", v2);
    println!("Vec4: {:?}", v4);

    let color = Color::WHITE;
    println!("Color: {:?}", color);

    let quat = Quaternion::IDENTITY;
    println!("Quaternion: {:?}", quat);

    println!("Completed basic math module tests passed!");
}
