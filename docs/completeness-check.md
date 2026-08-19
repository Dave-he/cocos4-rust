# cocos4-rust 项目完整性检查

检查日期：2026-05-27

## 已执行命令

| 命令 | 结果 |
|---|---|
| `(cd ../cocos4 && npm ci --ignore-scripts)` | 通过，恢复原版 Jest 依赖 |
| `(cd ../cocos4 && npm run build:debug-infos)` | 通过，生成 Jest 初始化需要的 DebugInfos |
| `node scripts/compare-cocos4-vec3-tests.mjs` | 通过，原版 Vec3 Jest 4 passed；Rust Vec3 38 passed |
| `cargo test -- --list` | 1812 tests, 0 benchmarks |
| `cargo test` | 1812 passed, 0 failed |
| `cargo test --features "gfx-wgpu storage-disk platform-real"` | 1812 passed, 0 failed |
| `cargo test --features js-runtime-real game` | 131 passed, 0 failed |
| `cargo clippy --all-targets -- -D warnings` | 通过 |
| `ITERATIONS=1000000 cargo run --release --example perf_math` | 通过，输出 Rust 数学热路径耗时 |
| `ITERATIONS=1000000 node scripts/bench-cocos4-math.mjs` | 通过，输出 JS 公式基线耗时 |
| `python3 scripts/generate-interface-symbol-map.py` | 通过，生成 9876 行接口 symbol 对照 CSV |
| `npx --yes tsx -e "import { Vec3 } from './cocos/core/math/vec3.ts'"` in `../cocos4` | 失败，缺少 `internal:constants` 模块别名 |

## 本轮代码完整性修复

| 文件 | 修复内容 |
|---|---|
| `src/game/game.rs` | 将 JS bootstrap prelude 的 raw string delimiter 从 `r#` 提升到 `r###`，避免 `"#GameCanvas"` 截断 Rust 字符串 |
| `src/base/value.rs` | 增加 `Value::Int(i64)` 兼容现有状态代码；将 `Value::Float` 调整为 `f64` 存储并保持 `as_float()` 返回 `f32` |
| `src/agi_minigame/atom.rs` | `AtomRegistry::register/create/get_metadata/has_atom` 兼容闭包注册和 `&str` 查询 |
| `src/agi_minigame/atoms/match3.rs` | 修复初始棋盘横向匹配检测、越界宝石索引、Option 单元访问 |
| `src/agi_minigame/atoms/parkour.rs` | 修复短帧时间下积分被整数截断为 0 的问题 |
| `src/agi_minigame/atoms/turn_combat.rs` | 修复 `1 + f32` 类型错误 |
| `src/agi_minigame/dimension.rs` | 修复无强制目标时维度在首帧自动完成，以及 runner 更新时的借用冲突 |
| `src/math/vec3.rs` | 增加原版 Vec3 Jest 子集覆盖到的 `slerp`、`scaleAndAdd`、`equals`、`generateOrthogonal` 等方法和镜像测试 |
| 多个低风险 lint 文件 | 清理 dead code 访问器、默认实现、测试断言、`Display` 实现、GFX device `Default` 实现等，使 all-targets clippy 严格门禁通过 |

## 完整性结论

当前状态：

- 可编译：是
- 全量 Rust 单测：通过
- 主要 feature 组合测试：通过
- `js-runtime-real` game 路径：通过过滤测试
- 性能微基准：已建立并有本机结果
- 文档目录：已补齐
- 接口 symbol 机器基线：已生成
- clippy 严格门禁：通过
- 原版 cocos4 Jest 对照：已执行 Vec3 子集；全量未执行
- 每个接口逐项签名/行为闭环：未完成

因此当前项目完整性从“编译阻塞/报告位置不符合要求”推进到“Rust 侧可测试、可 lint、可基准、可审计”，但不能标记为最终完成。

## 下一步完整性门禁

1. 在当前 `../cocos4` 依赖环境上继续补齐 `internal:*`、`external:*` 和 native asset 映射，执行原版 163 个测试文件。
2. 将 `docs/interface-symbol-map.csv` 中的 `same-name` / `same-name-candidate` / `renamed-candidate` 逐项人工校验为 `same` / `renamed-equivalent` / `partial` / `missing` / `rust-only`。
3. 将 `covered-by-equivalent` 测试逐步升级为一一对应的 Rust 测试。
4. 增加场景级 benchmark：启动、资源加载、动画采样、序列化、物理步进、UI 布局、渲染管线构建。
