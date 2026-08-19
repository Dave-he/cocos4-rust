# cocos4-rust 接口与功能对齐审计

审计日期：2026-05-27

## 审计口径

本文件用于二次校验 `cocos4-rust` 与原版 `cocos4` 的接口和功能对齐状态。当前是可验证基线，不是最终 100% 兼容声明。

扫描命令：

```bash
# cocos4-rust
rg -n "^\s*pub\s+(struct|enum|trait|fn|type|const|mod)\b" src | wc -l
find src -type f -name '*.rs' | wc -l

# cocos4
rg -n "^\s*export\s+(abstract\s+)?(class|enum|interface|function|const|let|var|type)\b" cocos exports native/cocos -g '*.ts' -g '*.js' -g '*.h' -g '*.cpp' | wc -l
find tests -type f \( -name '*.test.ts' -o -name '*.test.js' -o -name '*.spec.ts' -o -name '*.spec.js' \) | wc -l
```

当前扫描结果：

| 项目 | 数量 |
|---|---:|
| cocos4-rust Rust 源文件 | 330 |
| cocos4-rust public symbol 粗扫描 | 6040 |
| cocos4-rust pub symbol map 扫描 | 6003 |
| cocos4 原版源文件粗扫描 | 3296 |
| cocos4 原版 exported symbol 粗扫描 | 3877 |
| cocos4 原版 exported symbol map 扫描 | 4489 |
| cocos4 原版测试文件 | 163 |

## 模块映射

| cocos4 原版模块 | cocos4-rust 模块 | 当前验证状态 | 说明 |
|---|---|---|---|
| `2d` | `src/2d`, `src/ui` | 部分通过 | Label/Sprite/UI 基础测试存在，渲染批处理仍需深挖 |
| `3d` | `src/3d`, `src/scene` | 部分通过 | Mesh/Model/Skinning/Camera/Scene 测试通过 |
| `animation` | `src/core/animation.rs`, `src/3d/skeletal_animation.rs` | 部分通过 | 基础动画测试存在，new-gen-anim 仍未逐文件闭环 |
| `asset` / `asset-manager` | `src/core/assets/*` | 部分通过 | AssetManager/loader/material/texture 等测试通过，pack/finalizer/depend-util 等需补 |
| `audio` | `src/audio/*` | 部分通过 | AudioClip/Source/Manager 测试通过，真实平台音频后端需继续验证 |
| `core` | `src/core/*`, `src/base/*`, `src/math/*` | 部分通过 | 数学、事件、scheduler、memop、scene graph 均有测试 |
| `dragon-bones` | `src/dragon_bones/*` | 部分通过 | 解析与骨骼对象测试存在，运行时驱动需对照原版 |
| `game` | `src/game/*`, `src/application/*` | 部分通过 | bootstrap mock/real runtime 测试存在，native JS runtime 仍有未实现提示 |
| `gfx` / `webgpu` | `src/renderer/gfx-*` | 部分通过 | empty/validator/agent/wgpu 骨架测试通过，真实 GPU 后端能力未完全证明 |
| `gi` | `src/gi/*` | 部分通过 | SH/light probe/delaunay 等存在，需场景级验证 |
| `input` | `src/input/*` | 部分通过 | keyboard/mouse/touch 基础分发测试存在 |
| `native-binding` | `src/bindings/*` | 部分通过 | wasm 宏/注册测试存在，JNI/ObjC 等 native 绑定未完整闭环 |
| `particle` | `src/particle/*` | 部分通过 | 粒子系统基础存在，CPU/GPU culling 和材质渲染仍需对照 |
| `particle-2d` | `src/particle_2d/*` | 部分通过 | 2D 粒子基础存在，plist/png/tiff 原版行为需补 |
| `physics` | `src/physics/*` | 部分通过 | 3D physics API 有测试，真实 PhysX 等价性未证明 |
| `physics-2d` | `src/physics_2d/*` | 部分通过 | builtin collision/world 测试通过，Box2D/Rapier 等价性未证明 |
| `primitive` | `src/primitive/*` | 部分通过 | 几何生成测试通过，仍需与原版参数边界逐项对照 |
| `profiler` | `src/profiler/*` | 部分通过 | Counter/PerfCounter/Profiler 测试存在 |
| `render-scene` / `rendering` | `src/scene/*`, `src/renderer/pipeline/*` | 部分通过 | forward/deferred/custom pipeline 测试存在，真实渲染帧需补 |
| `scene-graph` | `src/core/scene-graph.rs`, `src/scene/*` | 部分通过 | Node/Scene/Transform 测试存在 |
| `serialization` | `src/serialization/*` | 部分通过 | JSON/primitive/dictionary 等测试存在，shared/cases 仍未全覆盖 |
| `sorting` | `src/sorting/*` | 部分通过 | sorting/sorting-layers 测试通过 |
| `spine` | `src/spine/*` | 部分通过 | parser/skeleton/animation 测试存在 |
| `terrain` | `src/terrain/*` | 部分通过 | height field/LOD/asset/buffer 测试通过 |
| `tiledmap` | `src/tiled_map/*` | 部分通过 | TMX parser/layer/map/asset 测试通过，复杂 TMX 行为需补 |
| `tween` | `src/tween/*` | 部分通过 | easing/tween/action/system 测试通过 |
| `ui` | `src/ui/*`, `src/2d/*` | 部分通过 | Button/Layout/Scroll/Toggle/EditBox/Video/WebView 等基础测试通过 |
| `video` / `web-view` | `src/ui/video_player.rs`, `src/ui/web_view.rs` | 部分通过 | mock/state model 通过，平台真实组件未证明 |
| `xr` | `src/xr/*` | 部分通过 | XR session/config 测试通过 |

## 原版测试映射状态

原版 `cocos4/tests` 当前有 163 个测试文件。根目录旧报告曾给出首版状态：`covered=9`、`covered-by-equivalent=30`、`pending=124`。本轮修复后 Rust 全量测试通过，并已用 `node scripts/compare-cocos4-vec3-tests.mjs` 跑通原版 `tests/value-types/vec3.test.ts` 的 4 个 Jest 测试；Rust 侧对应 `cargo test vec3` 38 个测试通过。但还没有把 124 个 pending 逐项转成一一对应测试。

本目录已经补充机器生成的 `interface-symbol-map.csv` 和 `interface-symbol-map-summary.md`：

```text
status,cocos4_symbol,cocos4_kind,cocos4_module,cocos4_path,cocos4_line,rust_symbol,rust_kind,rust_module,rust_path,rust_line,notes,cocos4_signature,rust_signature
```

状态枚举建议：

- `same`: 名称、职责、参数/返回语义均已对齐；
- `renamed-equivalent`: Rust 命名不同但语义等价；
- `partial`: API 存在但行为覆盖不完整；
- `missing`: Rust 侧缺失；
- `rust-only`: Rust 新增能力，不属于 cocos4 原版兼容面。

当前机器基线使用更保守的中间状态：

- `same-name`: 规范化名称一致且模块在等价映射内，仍需人工核对签名和行为；
- `same-name-candidate`: 名称一致但模块不在当前等价映射内；
- `renamed-candidate`: 字符串相似候选，只能作为人工复核入口；
- `missing`: 当前扫描不到 Rust public symbol 候选；
- `rust-only`: Rust public symbol 未匹配原版导出。

最新分布见 [接口 symbol 对照基线摘要](interface-symbol-map-summary.md)。当前 `missing=3273`，说明逐接口闭环还有大量工作，不能据此宣称 100% 兼容。

## 高风险差异

- 原版 TS/JS API 很多依赖动态对象、装饰器、模块别名和运行时注入；Rust 侧必须用 trait/struct 显式化，不能只按名称判断兼容。
- `gfx-wgpu`、`network-real`、`platform-real` 等 feature 已存在，但真实后端接入程度需要场景测试证明。
- `game` 模块仍保留 native JS runtime 未实现提示，说明启动链路还不是完整原版运行时替代。
