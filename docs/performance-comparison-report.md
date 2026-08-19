# cocos4-rust 性能对比测试报告

测试日期：2026-05-27

测试目录：`/mnt/ssd/codespace/cocos-engine/cocos4-rust`

Rust：`rustc 1.95.0` / `cargo 1.95.0`

Node：`v22.22.1`

## 测试范围

本轮完成的是可重复的数学热路径微基准，并补充了一个真实原版 Jest 正确性子集对照；不是完整引擎帧渲染、资源加载、物理或 JS 运行时性能对比。

原版 `cocos4` 的 Node 依赖已通过 `npm ci --ignore-scripts` 恢复，`npm run build:debug-infos` 能生成 Jest 初始化所需文件。直接通过 `tsx` import 原版 `cocos/core/math/vec3.ts` 仍会因模块别名失败：

```text
Error: Cannot find module 'internal:constants'
Require stack:
- cocos/core/data/class.ts
- cocos/core/math/vec3.ts
```

因此性能数据仍采用 `scripts/bench-cocos4-math.mjs` 中与 Cocos4 `Vec3` / `Mat4` 热路径一致的数据流和公式；Rust 侧使用真实 `cocos4-rust` crate 的 `Vec3` / `Mat4`。正确性对照另由原版 Jest Vec3 子集和 Rust Vec3 测试承担。

## 执行命令

```bash
ITERATIONS=1000000 node scripts/bench-cocos4-math.mjs
ITERATIONS=1000000 cargo run --release --example perf_math
node scripts/compare-cocos4-vec3-tests.mjs
cargo test
cargo test --features "gfx-wgpu storage-disk platform-real"
```

## 微基准结果

| 用例 | cocos4 JS 公式基线 | cocos4-rust release | 提升 |
|---|---:|---:|---:|
| Vec3 hot path, 1,000,000 次 | 141.995 ms | 77.940 ms | 1.82x，耗时降低 45.1% |
| Mat4 hot path, 100,000 次 | 29.130 ms | 5.472 ms | 5.32x，耗时降低 81.2% |
| 合计 | 171.125 ms | 83.412 ms | 2.05x，耗时降低 51.3% |

Rust 输出：

```text
engine=cocos4-rust
iterations=1000000
mat4_iterations=100000
vec3_hot_path_ms=77.940
mat4_hot_path_ms=5.472
checksum=3230206720.000000
```

JS 输出：

```text
engine=cocos4-js-formula
iterations=1000000
mat4_iterations=100000
vec3_hot_path_ms=141.995
mat4_hot_path_ms=29.130
checksum=3230138395.296386
```

## 真实正确性对照

执行命令：

```bash
node scripts/compare-cocos4-vec3-tests.mjs
```

结果：

| 目标 | 测试 | 结果 | 耗时 |
|---|---|---|---:|
| 原版 `cocos4` | `tests/value-types/vec3.test.ts` | 4 passed, 0 failed | 30216.398 ms |
| `cocos4-rust` | `cargo test vec3` | 38 passed, 0 failed | 92.513 ms |

原版 Jest 初始化期间仍会记录一个被捕获的 `external:emscripten/meshopt/meshopt_decoder.wasm.js` 缺失错误；该错误未导致 `tests/value-types/vec3.test.ts` 失败。这里的耗时是测试框架端到端耗时，不作为引擎性能对比依据。

## 结论

在本机单次微基准中，Rust 重构后的数学热点明显快于 JS 公式基线，尤其是 `Mat4 multiply + invert`。这个结果可以说明 Rust 在值类型数值计算上有明确优势。

不能据此推导整个引擎已经全面提速。完整性能结论还需要补齐：

- 原版 `cocos4` 全量测试和 native/external asset 映射；
- 原版真实模块导入或独立 benchmark 的稳定运行路径；
- 渲染、资源加载、物理、动画、序列化、UI、网络等模块级场景；
- 多次运行的均值、方差和机器负载记录。
