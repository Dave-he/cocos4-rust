# Cocos4-Rust

Rust implementation of Cocos4 game engine.

## Progress

This is a work-in-progress conversion of Cocos4 engine from C++ to Rust.

### Completed Modules (10/48 - 21%)

- **Math module** (9/10 complete)
  - ✅ Vec2 - 2D vector with full operator support (~450 lines)
- ✅ Vec3 - 3D vector with cross/dot products (~480 lines)
- ✅ Vec4 - 4D vector for homogeneous coordinates (~330 lines)
- ✅ Color - RGBA color type with conversion utilities (~100 lines)
- ✅ Geometry - Rect and Size types (~190 lines)
- ✅ Mat3 - 3x3 matrix operations (~260 lines)
- ✅ Mat4 - 4x4 matrix with transformations (~180 lines)

- **Base module** (1/9 complete)
  - ✅ Types - Core type aliases and constants

### In Progress (10/48 - 21%)

- Math module
  - 🔄 Quaternion - Rotation representation (pending)
- Core module
  - Event system (pending)
  - Scene graph (pending)

### Architecture Notes

- Column-major matrix layout matching OpenGL/GL conventions
- Rust's Copy trait for value types (no manual clone needed)
- Operator overloading via std::ops traits
- Zero-based indexing with const arrays

## Usage

```rust
use cocos4_rust::{Vec2, Vec3, Vec4, Color, Mat4};

fn main() {
    let v2 = Vec2::new(1.0, 2.0);
    let v3 = Vec3::new(1.0, 0.0, 0.0);
    let color = Color::WHITE;
    let mat4 = Mat4::IDENTITY;
    let scaled = v2 * mat4.multiply_vec4(&Vec3::new(2.0, 0.0, 0.0));
    
    println!("Vec2: {:?}, Vec3: {:?}, Color: {:?}",
             v2, v3, color);
    println!("Vec4 identity: {:?}", mat4);
    println!("Scaled Vec3: {:?}", scaled);
    
    let rotated = Mat4::IDENTITY;
    rotated.rotate(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::FRAC_PI_4 * 0.5);
    println!("Rotated: {:?}", rotated);
}
```

## Building

```bash
cd cocos4-rust
cargo build --release
```

## Project Structure

```
cocos4-rust/
├── src/
│   ├── math/
│   │   ├── vec2.rs       ✅
│   │   ├── vec3.rs       ✅
│   │   ├── vec4.rs       ✅
│   │   ├── mat3.rs       ✅
│   │   ├── mat4.rs       ✅
│   │   ├── quaternion.rs  (empty - pending)
│   │   ├── color.rs       ✅
│   │   ├── geometry.rs   ✅
│   └── ...
│   ├── base/
│   ├── types.rs      ✅
│   └── core/
│   └── ...
└── Cargo.toml
```

## Key Design Decisions

1. **No ref counting**: Cocos4 uses manual reference counting, but Rust uses ownership/borrowing instead
2. **Matrix layout**: Column-major for OpenGL compatibility
3. **Type safety**: No `as any` or type coercion
4. **Error handling**: Result types and Option<> for operations that can fail
5. **SIMD**: Avoid complex SSE optimizations initially - use safe Rust intrinsics

## Next Steps

High Priority:
- Quaternion module (rotation math)
- Core event system
- Base utilities (RefCounted, Scheduler, Log, Data, Value)

Medium Priority:
- 2D renderer components
- Scene graph
- Core animation

Low Priority:
- Platform abstractions
- Physics SDK integration
- Audio system

- Base module
  - 🔄 RefCounted - Reference counting system
  - 🔄 Data - Binary data buffer
  - 🔄 Value - Variant type for dynamic values
  - 🔄 Scheduler - Timer and callback system
  - 🔄 Log - Logging framework
  - 🔄 Utils - Utility functions

### Planned

- Core module
  - Event system
  - Scene graph
  - Animation system
  - Geometry primitives

- Renderer
  - Graphics API abstraction (gfx-base)
  - Frame graph
  - Pipeline state management

- 2D renderer
  - Sprite system
  - Batcher2d
  - UI components

- 3D renderer
  - Model system
  - Skeletal animation
  - Asset management

- Physics
  - SDK integration
  - PhysX backend

## Usage

```rust
use cocos4_rust::{Vec2, Vec3, Color, Rect};

fn main() {
    let v = Vec2::new(1.0, 2.0);
    let color = Color::WHITE;
    let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
    
    println!("Vector: {:?}, Color: {:?}, Rect: {:?}", v, color, rect);
}
```

## Building

```bash
cd cocos4-rust
cargo build --release
```

## Testing

```bash
cargo test
```

## License

MIT License - see LICENSE file for details.
