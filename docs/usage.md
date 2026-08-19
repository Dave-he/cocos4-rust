# cocos4-rust 使用与帮助文档

## 环境要求

- Rust 2021 edition
- 当前验证版本：`rustc 1.95.0`、`cargo 1.95.0`
- 可选：Node.js，用于与原版 Cocos4 工具链或 JS 公式基准对照

## 构建

```bash
cd /mnt/ssd/codespace/cocos-engine/cocos4-rust
cargo build
cargo build --release
```

## 测试

```bash
cargo test
cargo test -- --list
cargo test --features "gfx-wgpu storage-disk platform-real"
cargo test --features js-runtime-real game
```

当前全量结果：

```text
test result: ok. 1812 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

严格 lint：

```bash
cargo clippy --all-targets -- -D warnings
```

当前结果：通过。

## Feature

`Cargo.toml` 当前声明的主要 feature：

```text
gfx-empty
gfx-validator
gfx-wgpu
gfx-agent
pipeline-custom
pipeline-deferred
physics-2d
network-real
platform-real
storage-disk
dragon-bones
spine
js-bindings
terrain
tiled-map
```

默认 feature 已包含上述多数模块。注意：feature 存在不等于模块语义已经与原版 Cocos4 完全等价，具体状态见 [接口与功能对齐审计](interface-parity-audit.md)。

## 基础 API 示例

```rust
use cocos4_rust::{Mat4, Vec3};

let a = Vec3::new(1.0, 2.0, 3.0);
let b = Vec3::new(4.0, 5.0, 6.0);
let c = Vec3::cross_vecs(&a, &b);

let transform = Mat4::from_translation(&Vec3::new(10.0, 0.0, 0.0));
let moved = c.transform_mat4(&transform);

println!("{:?}", moved);
```

## 性能基准

```bash
ITERATIONS=1000000 cargo run --release --example perf_math
ITERATIONS=1000000 node scripts/bench-cocos4-math.mjs
```

## 原版 Cocos4 子集对照

首次运行前恢复原版依赖：

```bash
cd /mnt/ssd/codespace/cocos-engine/cocos4
npm ci --ignore-scripts
```

在 `cocos4-rust` 目录运行：

```bash
node scripts/compare-cocos4-vec3-tests.mjs
```

当前结果：原版 `tests/value-types/vec3.test.ts` 4 个 Jest 测试通过；Rust 侧 `cargo test vec3` 38 个测试通过。

## Android / App 集成边界

`game-studio` 采用标准 JNI 桥接：

```text
Kotlin/Android UI -> C++ CMake JNI bridge -> Rust cdylib -> cocos4-rust
```

这个边界用于保持 Android 模板与 Cocos4 生态兼容，同时把高性能引擎逻辑留在 Rust 侧。应用层文档在 `../game-studio/docs/`。

## 排障

- 如果 `cargo test` 编译失败，先确认 `src/game/game.rs` 的 JS bootstrap raw string delimiter 没有被内嵌选择器截断。
- 如果原版 `cocos4` TS import 失败，检查 `node_modules` 和 `internal:*` 模块别名环境；Jest 子集当前通过脚本内的 `cc.decorator` mapper 运行。
- 如果 clippy 失败，优先确认是否新增了测试目标 warning，再处理模块命名和 API 形态类 lint。
